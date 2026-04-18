use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoordinatorConfig {
    #[serde(default = "default_max_token_budget")]
    pub max_token_budget: u32,
    #[serde(default = "default_max_recursion_depth")]
    pub max_recursion_depth: u8,
    #[serde(default = "default_agent_timeout_secs")]
    pub agent_timeout_secs: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u8,
    #[serde(default = "default_true")]
    pub memory_aware_routing: bool,
    #[serde(default = "default_true")]
    pub recursive_orchestration: bool,
    #[serde(default = "default_memory_stale_days")]
    pub memory_stale_threshold_days: i64,
    #[serde(default = "default_true")]
    pub preflight_check: bool,
    #[serde(default = "default_llm_weight_multiplier")]
    pub llm_weight_multiplier: f32,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_token_budget: default_max_token_budget(),
            max_recursion_depth: default_max_recursion_depth(),
            agent_timeout_secs: default_agent_timeout_secs(),
            max_retries: default_max_retries(),
            memory_aware_routing: default_true(),
            recursive_orchestration: default_true(),
            memory_stale_threshold_days: default_memory_stale_days(),
            preflight_check: default_true(),
            llm_weight_multiplier: default_llm_weight_multiplier(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialCoordinatorConfig {
    pub max_token_budget: Option<u32>,
    pub max_recursion_depth: Option<u8>,
    pub agent_timeout_secs: Option<u64>,
    pub max_retries: Option<u8>,
    pub memory_aware_routing: Option<bool>,
    pub recursive_orchestration: Option<bool>,
    pub memory_stale_threshold_days: Option<i64>,
    pub preflight_check: Option<bool>,
    pub llm_weight_multiplier: Option<f32>,
}

impl CoordinatorConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialCoordinatorConfig) {
        if let Some(value) = partial.max_token_budget {
            self.max_token_budget = value;
        }
        if let Some(value) = partial.max_recursion_depth {
            self.max_recursion_depth = value;
        }
        if let Some(value) = partial.agent_timeout_secs {
            self.agent_timeout_secs = value;
        }
        if let Some(value) = partial.max_retries {
            self.max_retries = value;
        }
        if let Some(value) = partial.memory_aware_routing {
            self.memory_aware_routing = value;
        }
        if let Some(value) = partial.recursive_orchestration {
            self.recursive_orchestration = value;
        }
        if let Some(value) = partial.memory_stale_threshold_days {
            self.memory_stale_threshold_days = value;
        }
        if let Some(value) = partial.preflight_check {
            self.preflight_check = value;
        }
        if let Some(value) = partial.llm_weight_multiplier {
            self.llm_weight_multiplier = value;
        }
    }
}

impl PartialCoordinatorConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            max_token_budget: other.max_token_budget.or(self.max_token_budget),
            max_recursion_depth: other.max_recursion_depth.or(self.max_recursion_depth),
            agent_timeout_secs: other.agent_timeout_secs.or(self.agent_timeout_secs),
            max_retries: other.max_retries.or(self.max_retries),
            memory_aware_routing: other.memory_aware_routing.or(self.memory_aware_routing),
            recursive_orchestration: other
                .recursive_orchestration
                .or(self.recursive_orchestration),
            memory_stale_threshold_days: other
                .memory_stale_threshold_days
                .or(self.memory_stale_threshold_days),
            preflight_check: other.preflight_check.or(self.preflight_check),
            llm_weight_multiplier: other.llm_weight_multiplier.or(self.llm_weight_multiplier),
        }
    }
}

fn default_max_token_budget() -> u32 {
    200_000
}

fn default_max_recursion_depth() -> u8 {
    2
}

fn default_agent_timeout_secs() -> u64 {
    300
}

fn default_max_retries() -> u8 {
    3
}

fn default_true() -> bool {
    true
}

fn default_memory_stale_days() -> i64 {
    7
}

fn default_llm_weight_multiplier() -> f32 {
    1.0
}
