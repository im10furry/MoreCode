use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Agent role used by the coordinator and execution pipeline.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AgentType {
    /// Coordinator that orchestrates the full task lifecycle.
    Coordinator,
    /// Explorer that scans the codebase and gathers context.
    Explorer,
    /// Impact analyzer that evaluates change scope and risk.
    ImpactAnalyzer,
    /// Planner that decomposes work into executable subtasks.
    Planner,
    /// Coder that performs implementation work.
    Coder,
    /// Reviewer that checks correctness and quality risks.
    Reviewer,
    /// Tester that verifies behavior through tests.
    Tester,
    /// Debugger that investigates failures and root causes.
    Debugger,
    /// Research agent that gathers external information.
    Research,
    /// Documentation writer that produces or updates docs.
    DocWriter,
}

impl AgentType {
    /// All supported agent variants.
    pub const ALL: [Self; 10] = [
        Self::Coordinator,
        Self::Explorer,
        Self::ImpactAnalyzer,
        Self::Planner,
        Self::Coder,
        Self::Reviewer,
        Self::Tester,
        Self::Debugger,
        Self::Research,
        Self::DocWriter,
    ];

    /// Stable display name for the agent type.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Coordinator => "Coordinator",
            Self::Explorer => "Explorer",
            Self::ImpactAnalyzer => "ImpactAnalyzer",
            Self::Planner => "Planner",
            Self::Coder => "Coder",
            Self::Reviewer => "Reviewer",
            Self::Tester => "Tester",
            Self::Debugger => "Debugger",
            Self::Research => "Research",
            Self::DocWriter => "DocWriter",
        }
    }
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Runtime execution status for an agent instance.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum AgentExecutionStatus {
    /// Agent was created but has not started execution yet.
    Pending,
    /// Agent is actively processing its assigned work.
    Running,
    /// Agent finished successfully.
    Completed,
    /// Agent failed while executing the task.
    Failed,
    /// Agent was cancelled before completion.
    Cancelled,
}

/// Coordinator-facing snapshot of an agent execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentStatus {
    /// Type of the tracked agent.
    pub agent_type: AgentType,
    /// Current execution status.
    pub status: AgentExecutionStatus,
    /// Assigned task identifier.
    pub task_id: String,
    /// Consumed token count.
    pub token_used: u32,
    /// Agent start timestamp in UTC.
    pub started_at: DateTime<Utc>,
    /// Current recursion depth where `0` means top-level execution.
    pub recursion_depth: u8,
}

/// UI event status for tool invocation reporting.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ToolCallStatus {
    /// Tool execution has started.
    Started,
    /// Tool execution completed successfully.
    Completed,
    /// Tool execution failed.
    Failed,
}
