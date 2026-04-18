use mc_core::AgentType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmConfig {
    pub model_id: String,
    pub temperature: f32,
    pub max_output_tokens: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model_id: "mock-cognitive-model".to_string(),
            temperature: 0.2,
            max_output_tokens: 2_048,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplorerConfig {
    pub cache_ttl_secs: u64,
    pub incremental_change_threshold: f32,
    pub max_tree_depth: usize,
    pub max_file_size_bytes: u64,
    pub max_files: usize,
}

impl Default for ExplorerConfig {
    fn default() -> Self {
        Self {
            cache_ttl_secs: 24 * 60 * 60,
            incremental_change_threshold: 0.2,
            max_tree_depth: 3,
            max_file_size_bytes: 512 * 1024,
            max_files: 20_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlannerConfig {
    pub max_parallel_groups: usize,
    pub max_group_token_budget: u32,
    pub max_total_token_budget: u32,
    pub context_window_limit: usize,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            max_parallel_groups: 8,
            max_group_token_budget: 12_000,
            max_total_token_budget: 32_000,
            context_window_limit: 16_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub llm_config: LlmConfig,
    pub explorer: ExplorerConfig,
    pub planner: PlannerConfig,
}

impl AgentConfig {
    pub fn for_agent_type(agent_type: AgentType) -> Self {
        Self {
            agent_type,
            llm_config: LlmConfig::default(),
            explorer: ExplorerConfig::default(),
            planner: PlannerConfig::default(),
        }
    }
}
