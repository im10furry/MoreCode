use std::collections::HashMap;

use mc_core::agent::AgentType;
use serde::{Deserialize, Serialize};

/// Rolling statistics collected across recursive orchestration executions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RecursiveStats {
    pub total_tasks: usize,
    pub successful_tasks: usize,
    pub failed_tasks: usize,
    pub avg_depth: f64,
    pub max_depth_reached: usize,
    pub total_sub_agents: usize,
    pub total_tokens_used: u64,
    pub total_duration_ms: u64,
    pub agent_call_counts: HashMap<AgentType, usize>,
}

impl RecursiveStats {
    /// Record a child-agent execution into the rolling stats.
    pub fn record_agent(
        &mut self,
        agent_type: AgentType,
        depth: usize,
        tokens_used: u64,
        duration_ms: u64,
        success: bool,
    ) {
        self.total_tasks += 1;
        self.total_sub_agents += 1;
        self.total_tokens_used += tokens_used;
        self.total_duration_ms += duration_ms;

        if success {
            self.successful_tasks += 1;
        } else {
            self.failed_tasks += 1;
        }

        if depth > self.max_depth_reached {
            self.max_depth_reached = depth;
        }

        let previous_total = self.total_tasks - 1;
        self.avg_depth = if previous_total == 0 {
            depth as f64
        } else {
            ((self.avg_depth * previous_total as f64) + depth as f64) / self.total_tasks as f64
        };

        *self.agent_call_counts.entry(agent_type).or_insert(0) += 1;
    }
}

#[cfg(test)]
mod tests {
    use mc_core::agent::AgentType;

    use super::RecursiveStats;

    #[test]
    fn record_agent_updates_rollups() {
        let mut stats = RecursiveStats::default();
        stats.record_agent(AgentType::Coder, 1, 1_000, 50, true);
        stats.record_agent(AgentType::Coder, 2, 2_000, 70, false);

        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.successful_tasks, 1);
        assert_eq!(stats.failed_tasks, 1);
        assert_eq!(stats.total_tokens_used, 3_000);
        assert_eq!(stats.max_depth_reached, 2);
        assert_eq!(stats.agent_call_counts.get(&AgentType::Coder), Some(&2));
    }
}
