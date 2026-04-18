use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{BudgetError, BudgetNode, ModelInfo, TokenUsage};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Pricing {
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
    pub cache_read_price_per_1k: f64,
    pub cache_write_price_per_1k: f64,
}

impl From<&ModelInfo> for Pricing {
    fn from(model: &ModelInfo) -> Self {
        Self {
            input_price_per_1k: model.input_price_per_1k,
            output_price_per_1k: model.output_price_per_1k,
            cache_read_price_per_1k: model.cache_read_price_per_1k,
            cache_write_price_per_1k: model.cache_write_price_per_1k,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelCostRecord {
    pub total_requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cached_tokens: u64,
    pub total_cost_cents: u64,
}

pub struct CostTracker {
    budget: BudgetNode,
    model_costs: RwLock<HashMap<String, ModelCostRecord>>,
}

impl CostTracker {
    pub fn new(budget_cents: u64) -> Self {
        Self {
            budget: BudgetNode::new(budget_cents),
            model_costs: RwLock::new(HashMap::new()),
        }
    }

    pub async fn record_usage(
        &self,
        model_id: &str,
        model_info: &ModelInfo,
        usage: &TokenUsage,
    ) -> Result<(), BudgetError> {
        let uncached_prompt = usage.prompt_tokens.saturating_sub(usage.cached_tokens);
        let input_cost = (uncached_prompt as f64 / 1000.0) * model_info.input_price_per_1k;
        let output_cost =
            (usage.completion_tokens as f64 / 1000.0) * model_info.output_price_per_1k;
        let cache_cost = (usage.cached_tokens as f64 / 1000.0) * model_info.cache_read_price_per_1k;
        let total_cost_cents = ((input_cost + output_cost + cache_cost) * 100.0).ceil() as u64;

        self.budget.try_deduct(total_cost_cents)?;

        let mut costs = self.model_costs.write().await;
        let record = costs
            .entry(model_id.to_string())
            .or_insert_with(ModelCostRecord::default);
        record.total_requests += 1;
        record.total_input_tokens += usage.prompt_tokens as u64;
        record.total_output_tokens += usage.completion_tokens as u64;
        record.total_cached_tokens += usage.cached_tokens as u64;
        record.total_cost_cents += total_cost_cents;

        Ok(())
    }

    pub async fn model_cost(&self, model_id: &str) -> Option<ModelCostRecord> {
        self.model_costs.read().await.get(model_id).cloned()
    }

    pub fn budget_usage_rate(&self) -> f64 {
        self.budget.usage_rate()
    }

    pub fn remaining_budget(&self) -> u64 {
        self.budget.remaining()
    }

    pub fn budget(&self) -> &BudgetNode {
        &self.budget
    }
}
