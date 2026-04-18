use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostBudgetConfig {
    #[serde(default)]
    pub daily_budget_usd: Option<f64>,
    #[serde(default)]
    pub weekly_budget_usd: Option<f64>,
    #[serde(default)]
    pub monthly_budget_usd: Option<f64>,
    #[serde(default)]
    pub per_task_budget_usd: Option<f64>,
    #[serde(default = "default_over_budget_action")]
    pub over_budget_action: String,
    #[serde(default = "default_cost_log_path")]
    pub cost_log_path: String,
}

impl Default for CostBudgetConfig {
    fn default() -> Self {
        Self {
            daily_budget_usd: None,
            weekly_budget_usd: None,
            monthly_budget_usd: None,
            per_task_budget_usd: None,
            over_budget_action: default_over_budget_action(),
            cost_log_path: default_cost_log_path(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialCostBudgetConfig {
    pub daily_budget_usd: Option<f64>,
    pub weekly_budget_usd: Option<f64>,
    pub monthly_budget_usd: Option<f64>,
    pub per_task_budget_usd: Option<f64>,
    pub over_budget_action: Option<String>,
    pub cost_log_path: Option<String>,
}

impl CostBudgetConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialCostBudgetConfig) {
        if let Some(value) = partial.daily_budget_usd {
            self.daily_budget_usd = Some(value);
        }
        if let Some(value) = partial.weekly_budget_usd {
            self.weekly_budget_usd = Some(value);
        }
        if let Some(value) = partial.monthly_budget_usd {
            self.monthly_budget_usd = Some(value);
        }
        if let Some(value) = partial.per_task_budget_usd {
            self.per_task_budget_usd = Some(value);
        }
        if let Some(value) = partial.over_budget_action {
            self.over_budget_action = value;
        }
        if let Some(value) = partial.cost_log_path {
            self.cost_log_path = value;
        }
    }
}

impl PartialCostBudgetConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            daily_budget_usd: other.daily_budget_usd.or(self.daily_budget_usd),
            weekly_budget_usd: other.weekly_budget_usd.or(self.weekly_budget_usd),
            monthly_budget_usd: other.monthly_budget_usd.or(self.monthly_budget_usd),
            per_task_budget_usd: other.per_task_budget_usd.or(self.per_task_budget_usd),
            over_budget_action: other.over_budget_action.or(self.over_budget_action),
            cost_log_path: other.cost_log_path.or(self.cost_log_path),
        }
    }
}

fn default_over_budget_action() -> String {
    "warn".to_string()
}

fn default_cost_log_path() -> String {
    ".assistant-memory/cost-log.json".to_string()
}
