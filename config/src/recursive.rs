use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecursiveConfig {
    #[serde(default = "default_max_sub_agents")]
    pub max_sub_agents: usize,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default = "default_max_total_sub_agents")]
    pub max_total_sub_agents: usize,
    #[serde(default = "default_sub_agent_timeout_secs")]
    pub sub_agent_timeout_secs: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for RecursiveConfig {
    fn default() -> Self {
        Self {
            max_sub_agents: default_max_sub_agents(),
            max_depth: default_max_depth(),
            max_total_sub_agents: default_max_total_sub_agents(),
            sub_agent_timeout_secs: default_sub_agent_timeout_secs(),
            enabled: default_true(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialRecursiveConfig {
    pub max_sub_agents: Option<usize>,
    pub max_depth: Option<usize>,
    pub max_total_sub_agents: Option<usize>,
    pub sub_agent_timeout_secs: Option<u64>,
    pub enabled: Option<bool>,
}

impl RecursiveConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialRecursiveConfig) {
        if let Some(value) = partial.max_sub_agents {
            self.max_sub_agents = value;
        }
        if let Some(value) = partial.max_depth {
            self.max_depth = value;
        }
        if let Some(value) = partial.max_total_sub_agents {
            self.max_total_sub_agents = value;
        }
        if let Some(value) = partial.sub_agent_timeout_secs {
            self.sub_agent_timeout_secs = value;
        }
        if let Some(value) = partial.enabled {
            self.enabled = value;
        }
    }
}

impl PartialRecursiveConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            max_sub_agents: other.max_sub_agents.or(self.max_sub_agents),
            max_depth: other.max_depth.or(self.max_depth),
            max_total_sub_agents: other.max_total_sub_agents.or(self.max_total_sub_agents),
            sub_agent_timeout_secs: other.sub_agent_timeout_secs.or(self.sub_agent_timeout_secs),
            enabled: other.enabled.or(self.enabled),
        }
    }
}

fn default_max_sub_agents() -> usize {
    5
}

fn default_max_depth() -> usize {
    2
}

fn default_max_total_sub_agents() -> usize {
    20
}

fn default_sub_agent_timeout_secs() -> u64 {
    120
}

fn default_true() -> bool {
    true
}
