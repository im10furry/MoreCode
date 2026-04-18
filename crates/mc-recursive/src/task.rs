use chrono::{DateTime, Utc};
use mc_core::generate_id;
use serde::{Deserialize, Serialize};

use crate::sub_agent::SubAgentSpec;

/// Runtime representation of a recursive orchestration task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecursiveTask {
    pub id: String,
    pub description: String,
    pub current_depth: usize,
    pub status: RecursiveTaskStatus,
    pub sub_agent_specs: Vec<SubAgentSpec>,
    pub token_budget: u64,
    pub created_at: DateTime<Utc>,
    pub parent_task_id: Option<String>,
}

/// Lifecycle state for a recursive task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecursiveTaskStatus {
    SplittingDirections,
    LaunchingSubAgents,
    WaitingForSubAgents,
    Filtering,
    Aggregating,
    Completed,
    Failed(String),
    Cancelled,
}

impl RecursiveTask {
    /// Create a new recursive task with generated identifier and timestamp.
    pub fn new(
        description: impl Into<String>,
        current_depth: usize,
        token_budget: u64,
        parent_task_id: Option<String>,
    ) -> Self {
        Self {
            id: generate_id(),
            description: description.into(),
            current_depth,
            status: RecursiveTaskStatus::SplittingDirections,
            sub_agent_specs: Vec::new(),
            token_budget,
            created_at: Utc::now(),
            parent_task_id,
        }
    }

    /// Replace the child-agent plan attached to the task.
    pub fn set_sub_agent_specs(&mut self, sub_agent_specs: Vec<SubAgentSpec>) {
        self.sub_agent_specs = sub_agent_specs;
    }

    /// Transition the task into a new state.
    pub fn transition_to(&mut self, status: RecursiveTaskStatus) {
        self.status = status;
    }

    /// Mark the task as completed.
    pub fn mark_completed(&mut self) {
        self.status = RecursiveTaskStatus::Completed;
    }

    /// Mark the task as failed.
    pub fn mark_failed(&mut self, reason: impl Into<String>) {
        self.status = RecursiveTaskStatus::Failed(reason.into());
    }

    /// Mark the task as cancelled.
    pub fn mark_cancelled(&mut self) {
        self.status = RecursiveTaskStatus::Cancelled;
    }
}

#[cfg(test)]
mod tests {
    use super::{RecursiveTask, RecursiveTaskStatus};

    #[test]
    fn task_can_transition_through_expected_states() {
        let mut task = RecursiveTask::new("analyze auth", 0, 10_000, None);
        task.transition_to(RecursiveTaskStatus::LaunchingSubAgents);
        task.transition_to(RecursiveTaskStatus::WaitingForSubAgents);
        task.transition_to(RecursiveTaskStatus::Filtering);
        task.transition_to(RecursiveTaskStatus::Aggregating);
        task.mark_completed();

        assert_eq!(task.status, RecursiveTaskStatus::Completed);
    }
}
