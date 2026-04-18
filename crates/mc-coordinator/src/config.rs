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
    #[serde(default = "default_llm_weight")]
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
            llm_weight_multiplier: default_llm_weight(),
        }
    }
}

fn default_max_token_budget() -> u32 {
    24_000
}

fn default_max_recursion_depth() -> u8 {
    2
}

fn default_agent_timeout_secs() -> u64 {
    90
}

fn default_max_retries() -> u8 {
    2
}

fn default_true() -> bool {
    true
}

fn default_memory_stale_days() -> i64 {
    7
}

fn default_llm_weight() -> f32 {
    1.0
}
