use std::sync::atomic::{fence, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use thiserror::Error;

pub struct BudgetNode {
    remaining: AtomicU64,
    initial: AtomicU64,
    used: AtomicU64,
    on_exhausted: RwLock<Option<Arc<dyn Fn() + Send + Sync>>>,
}

impl BudgetNode {
    pub fn new(initial_budget: u64) -> Self {
        Self {
            remaining: AtomicU64::new(initial_budget),
            initial: AtomicU64::new(initial_budget),
            used: AtomicU64::new(0),
            on_exhausted: RwLock::new(None),
        }
    }

    pub fn try_deduct(&self, amount: u64) -> Result<u64, BudgetError> {
        loop {
            let current = self.remaining.load(Ordering::Acquire);
            if current < amount {
                return Err(BudgetError::InsufficientBudget {
                    required: amount,
                    remaining: current,
                });
            }

            let new_remaining = current - amount;
            match self.remaining.compare_exchange(
                current,
                new_remaining,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.used.fetch_add(amount, Ordering::AcqRel);
                    if new_remaining == 0 {
                        let callback = self
                            .on_exhausted
                            .read()
                            .ok()
                            .and_then(|guard| guard.as_ref().map(Arc::clone));
                        if let Some(callback) = callback {
                            callback();
                        }
                    }
                    return Ok(new_remaining);
                }
                Err(_) => continue,
            }
        }
    }

    pub fn remaining(&self) -> u64 {
        self.remaining.load(Ordering::Acquire)
    }

    pub fn used(&self) -> u64 {
        self.used.load(Ordering::Acquire)
    }

    pub fn initial(&self) -> u64 {
        self.initial.load(Ordering::Acquire)
    }

    pub fn usage_rate(&self) -> f64 {
        let initial = self.initial();
        if initial == 0 {
            0.0
        } else {
            self.used() as f64 / initial as f64
        }
    }

    pub fn on_exhausted<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if let Ok(mut slot) = self.on_exhausted.write() {
            *slot = Some(Arc::new(callback));
        }
    }

    pub fn reset(&self, new_budget: u64) {
        self.initial.store(new_budget, Ordering::Release);
        self.remaining.store(new_budget, Ordering::Release);
        self.used.store(0, Ordering::Release);
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum BudgetError {
    #[error("insufficient budget: required {required}, remaining {remaining}")]
    InsufficientBudget { required: u64, remaining: u64 },
}

pub fn calibrate(estimated: f64, actual: f64, alpha: Option<f64>) -> f64 {
    let alpha = alpha.unwrap_or(0.1);
    fence(Ordering::AcqRel);
    estimated + alpha * (actual - estimated)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{BudgetError, BudgetNode};

    #[test]
    fn rejects_deduction_when_budget_is_insufficient() {
        let budget = BudgetNode::new(5);
        let result = budget.try_deduct(8);
        assert_eq!(
            result,
            Err(BudgetError::InsufficientBudget {
                required: 8,
                remaining: 5,
            })
        );
    }

    #[test]
    fn handles_concurrent_deduction_with_cas_loop() {
        let budget = Arc::new(BudgetNode::new(100));
        let mut handles = Vec::new();

        for _ in 0..8 {
            let budget = Arc::clone(&budget);
            handles.push(std::thread::spawn(move || {
                for _ in 0..10 {
                    let _ = budget.try_deduct(1);
                }
            }));
        }

        for handle in handles {
            let join_result = handle.join();
            assert!(
                join_result.is_ok(),
                "worker thread should join successfully"
            );
        }

        assert_eq!(budget.remaining(), 20);
        assert_eq!(budget.used(), 80);
    }
}
