use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration knobs for the recursive orchestration engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecursiveConfig {
    /// Maximum number of child agents allowed on the same level.
    #[serde(default = "default_max_sub_agents")]
    pub max_sub_agents: usize,
    /// Maximum recursion depth allowed for the entire execution tree.
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    /// Maximum number of agents across the full recursion tree.
    #[serde(default = "default_max_total")]
    pub max_total: usize,
    /// Minimum child budget to allocate when the parent budget allows it.
    #[serde(default = "default_min_child_budget")]
    pub min_child_budget: u64,
    /// Default timeout applied to a child agent when a spec omits it.
    #[serde(default = "default_sub_agent_timeout_secs")]
    pub sub_agent_timeout_secs: u64,
    /// Upper bound for aggregate tokens passed into the reduce phase.
    #[serde(default = "default_max_aggregate_tokens")]
    pub max_aggregate_tokens: usize,
}

const fn default_max_sub_agents() -> usize {
    5
}

const fn default_max_depth() -> usize {
    2
}

const fn default_max_total() -> usize {
    20
}

const fn default_min_child_budget() -> u64 {
    2_000
}

const fn default_sub_agent_timeout_secs() -> u64 {
    120
}

const fn default_max_aggregate_tokens() -> usize {
    50_000
}

impl Default for RecursiveConfig {
    fn default() -> Self {
        Self {
            max_sub_agents: default_max_sub_agents(),
            max_depth: default_max_depth(),
            max_total: default_max_total(),
            min_child_budget: default_min_child_budget(),
            sub_agent_timeout_secs: default_sub_agent_timeout_secs(),
            max_aggregate_tokens: default_max_aggregate_tokens(),
        }
    }
}

impl RecursiveConfig {
    /// Validate that the config is internally consistent.
    pub fn validate(&self) -> Result<()> {
        if self.max_sub_agents == 0 {
            bail!("max_sub_agents 不能为 0");
        }
        if self.max_depth == 0 {
            bail!("max_depth 不能为 0");
        }
        if self.max_depth > 3 {
            bail!("max_depth 不能超过 3，当前值: {}", self.max_depth);
        }
        if self.max_total == 0 {
            bail!("max_total 不能为 0");
        }
        if self.max_total < self.max_sub_agents {
            bail!(
                "max_total 不能小于 max_sub_agents，当前值: {} < {}",
                self.max_total,
                self.max_sub_agents
            );
        }
        if self.min_child_budget == 0 {
            bail!("min_child_budget 不能为 0");
        }
        if self.sub_agent_timeout_secs == 0 {
            bail!("sub_agent_timeout_secs 不能为 0");
        }
        if self.max_aggregate_tokens == 0 {
            bail!("max_aggregate_tokens 不能为 0");
        }
        Ok(())
    }

    /// Default timeout for child-agent execution.
    pub fn sub_agent_timeout(&self) -> Duration {
        Duration::from_secs(self.sub_agent_timeout_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::RecursiveConfig;

    #[test]
    fn validate_rejects_invalid_depths() {
        let mut config = RecursiveConfig::default();
        config.max_depth = 0;
        assert!(config.validate().is_err());

        config.max_depth = 4;
        assert!(config.validate().is_err());
    }
}
