use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::LlmError;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Explorer,
    Research,
    Coder,
    Coordinator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCacheConfig {
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f64,
    #[serde(default = "default_cache_ttl")]
    pub max_ttl: Duration,
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
    #[serde(default = "default_stats_window")]
    pub stats_window_size: usize,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub enabled_agents: Vec<AgentType>,
    #[serde(default = "default_max_response_chars")]
    pub max_response_chars: usize,
    #[serde(default = "default_pollution_miss_threshold")]
    pub pollution_miss_threshold: u32,
    #[serde(default = "default_min_hit_rate")]
    pub min_hit_rate: f32,
    #[serde(default = "default_true")]
    pub disable_for_code_generation: bool,
}

fn default_similarity_threshold() -> f64 {
    0.95
}

fn default_cache_ttl() -> Duration {
    Duration::from_secs(300)
}

fn default_max_entries() -> usize {
    1000
}

fn default_stats_window() -> usize {
    100
}

fn default_true() -> bool {
    true
}

fn default_max_response_chars() -> usize {
    4096
}

fn default_pollution_miss_threshold() -> u32 {
    3
}

fn default_min_hit_rate() -> f32 {
    0.30
}

impl Default for SemanticCacheConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: default_similarity_threshold(),
            max_ttl: default_cache_ttl(),
            max_entries: default_max_entries(),
            stats_window_size: default_stats_window(),
            enabled: default_true(),
            enabled_agents: Vec::new(),
            max_response_chars: default_max_response_chars(),
            pollution_miss_threshold: default_pollution_miss_threshold(),
            min_hit_rate: default_min_hit_rate(),
            disable_for_code_generation: default_true(),
        }
    }
}

impl SemanticCacheConfig {
    pub fn validate(&self) -> Result<(), LlmError> {
        if !(0.0..=1.0).contains(&self.similarity_threshold) {
            return Err(LlmError::Internal(
                "semantic cache similarity_threshold must be between 0.0 and 1.0".into(),
            ));
        }
        if self.max_entries == 0 {
            return Err(LlmError::Internal(
                "semantic cache max_entries must be greater than zero".into(),
            ));
        }
        if self.max_response_chars == 0 {
            return Err(LlmError::Internal(
                "semantic cache max_response_chars must be greater than zero".into(),
            ));
        }
        Ok(())
    }

    pub fn is_agent_enabled(&self, agent_type: &AgentType) -> bool {
        self.enabled_agents.is_empty() || self.enabled_agents.iter().any(|item| item == agent_type)
    }
}

#[cfg(test)]
mod tests {
    use super::SemanticCacheConfig;

    #[test]
    fn default_threshold_matches_requirement() {
        let config = SemanticCacheConfig::default();
        assert_eq!(config.similarity_threshold, 0.95);
    }
}
