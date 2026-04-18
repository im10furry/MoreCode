use std::collections::HashMap;

use mc_core::agent::AgentType;
use serde::{Deserialize, Serialize};

/// Model metadata used for cost estimation and output-ratio tuning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentModelConfig {
    pub agent_type: AgentType,
    pub model_name: String,
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
    pub max_context_tokens: u64,
    pub output_ratio: f64,
}

/// Allocates token budgets between parent and child agents.
#[derive(Debug, Clone)]
pub struct TokenBudgetAllocator {
    model_configs: HashMap<AgentType, AgentModelConfig>,
    default_output_ratio: f64,
    child_budget_ratio: f64,
    min_child_budget: u64,
}

impl TokenBudgetAllocator {
    pub fn new() -> Self {
        Self {
            model_configs: HashMap::new(),
            default_output_ratio: 0.3,
            child_budget_ratio: 0.8,
            min_child_budget: 2_000,
        }
    }

    /// Registers model-specific pricing and context metadata.
    pub fn register_agent_model(&mut self, config: AgentModelConfig) {
        self.model_configs.insert(config.agent_type, config);
    }

    /// Ratio of the parent budget that may be delegated to children.
    pub const fn child_budget_ratio(&self) -> f64 {
        self.child_budget_ratio
    }

    /// Minimum child budget enforced when the parent budget allows it.
    pub const fn min_child_budget(&self) -> u64 {
        self.min_child_budget
    }

    /// Calculate per-child budgets while keeping the total child budget under 80% of the parent.
    pub fn calculate_child_budget(
        &self,
        parent_budget: u64,
        child_count: usize,
        depth: usize,
    ) -> Vec<u64> {
        if child_count == 0 {
            return Vec::new();
        }

        let depth_factor = 0.7_f64.powi(depth as i32);
        let total_budget =
            (parent_budget as f64 * self.child_budget_ratio * depth_factor).floor() as u64;
        let minimum_total = self.min_child_budget.saturating_mul(child_count as u64);

        if total_budget >= minimum_total {
            let mut budgets = vec![self.min_child_budget; child_count];
            let remaining = total_budget - minimum_total;
            let per_child = remaining / child_count as u64;
            let remainder = remaining % child_count as u64;
            for (index, budget) in budgets.iter_mut().enumerate() {
                *budget += per_child;
                if (index as u64) < remainder {
                    *budget += 1;
                }
            }
            budgets
        } else {
            let base = total_budget / child_count as u64;
            let remainder = total_budget % child_count as u64;
            (0..child_count)
                .map(|index| base + u64::from((index as u64) < remainder))
                .collect()
        }
    }

    /// Calculate the token budget retained by the parent reduce step.
    pub fn calculate_parent_budget(
        &self,
        parent_budget: u64,
        child_count: usize,
        depth: usize,
    ) -> u64 {
        let delegated: u64 = self
            .calculate_child_budget(parent_budget, child_count, depth)
            .into_iter()
            .sum();
        parent_budget.saturating_sub(delegated)
    }

    /// Output-budget hint derived from model metadata.
    pub fn output_budget(&self, agent_type: AgentType, total_budget: u64) -> u64 {
        let ratio = self
            .model_configs
            .get(&agent_type)
            .map(|config| config.output_ratio)
            .unwrap_or(self.default_output_ratio);
        (total_budget as f64 * ratio).floor() as u64
    }

    /// Estimate the cost in USD for a token pair.
    pub fn estimate_cost(
        &self,
        agent_type: AgentType,
        input_tokens: u64,
        output_tokens: u64,
    ) -> f64 {
        if let Some(config) = self.model_configs.get(&agent_type) {
            let input_cost = input_tokens as f64 / 1_000.0 * config.input_price_per_1k;
            let output_cost = output_tokens as f64 / 1_000.0 * config.output_price_per_1k;
            input_cost + output_cost
        } else {
            input_tokens as f64 / 1_000.0 * 0.03 + output_tokens as f64 / 1_000.0 * 0.06
        }
    }
}

impl Default for TokenBudgetAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::TokenBudgetAllocator;

    #[test]
    fn child_budgets_respect_parent_cap_and_minimums() {
        let allocator = TokenBudgetAllocator::default();
        let budgets = allocator.calculate_child_budget(30_000, 3, 0);
        assert_eq!(budgets.iter().sum::<u64>(), 24_000);
        assert!(budgets
            .iter()
            .all(|budget| *budget >= allocator.min_child_budget()));
    }

    #[test]
    fn child_budgets_decay_with_depth() {
        let allocator = TokenBudgetAllocator::default();
        let depth_zero = allocator.calculate_child_budget(30_000, 3, 0);
        let depth_one = allocator.calculate_child_budget(30_000, 3, 1);
        assert!(depth_one.iter().sum::<u64>() < depth_zero.iter().sum::<u64>());
    }
}
