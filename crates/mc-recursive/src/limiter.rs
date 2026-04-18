use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

/// Global resource guard that prevents recursive fan-out from running away.
#[derive(Debug, Clone)]
pub struct ResourceLimiter {
    pub max_sub_agents: usize,
    pub max_depth: usize,
    pub max_total: usize,
    current_total: Arc<AtomicUsize>,
}

impl ResourceLimiter {
    pub fn new(max_sub_agents: usize, max_depth: usize, max_total: usize) -> Self {
        Self {
            max_sub_agents,
            max_depth,
            max_total,
            current_total: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Check whether a new batch of child agents can be spawned.
    pub fn can_spawn(&self, current_depth: usize, sub_agent_count: usize) -> bool {
        if current_depth >= self.max_depth {
            return false;
        }
        if sub_agent_count > self.max_sub_agents {
            return false;
        }
        self.current_count().saturating_add(sub_agent_count) <= self.max_total
    }

    /// Acquire agent slots.
    pub fn acquire(&self, count: usize) -> bool {
        loop {
            let current = self.current_total.load(Ordering::SeqCst);
            if current.saturating_add(count) > self.max_total {
                return false;
            }
            if self
                .current_total
                .compare_exchange_weak(current, current + count, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return true;
            }
        }
    }

    /// Acquire a scoped permit that will be released automatically on drop.
    pub fn try_acquire(&self, count: usize) -> Option<ResourcePermit> {
        self.acquire(count).then(|| ResourcePermit {
            limiter: self.clone(),
            count,
        })
    }

    /// Release agent slots.
    pub fn release(&self, count: usize) {
        self.current_total.fetch_sub(count, Ordering::SeqCst);
    }

    /// Number of currently acquired agent slots.
    pub fn current_count(&self) -> usize {
        self.current_total.load(Ordering::SeqCst)
    }
}

impl Default for ResourceLimiter {
    fn default() -> Self {
        Self::new(5, 2, 20)
    }
}

/// RAII guard returned by [`ResourceLimiter::try_acquire`].
#[derive(Debug)]
pub struct ResourcePermit {
    limiter: ResourceLimiter,
    count: usize,
}

impl Drop for ResourcePermit {
    fn drop(&mut self) {
        self.limiter.release(self.count);
    }
}

#[cfg(test)]
mod tests {
    use super::ResourceLimiter;

    #[test]
    fn enforces_depth_and_count_limits() {
        let limiter = ResourceLimiter::default();
        assert!(limiter.can_spawn(0, 3));
        assert!(!limiter.can_spawn(2, 1));
        assert!(!limiter.can_spawn(0, 6));
    }

    #[test]
    fn enforces_global_total_limit() {
        let limiter = ResourceLimiter::new(5, 2, 4);
        let _permit = limiter.try_acquire(3).expect("first permit");
        assert!(!limiter.acquire(2));
        assert_eq!(limiter.current_count(), 3);
    }
}
