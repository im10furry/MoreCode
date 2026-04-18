use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use mc_core::{agent::AgentType, token::TokenUsage};
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

/// Specification used to spin up a child agent with minimal, isolated context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubAgentSpec {
    pub id: String,
    pub agent_type: AgentType,
    pub focus: String,
    pub context_files: Vec<PathBuf>,
    pub token_budget: usize,
    pub timeout_secs: u64,
    pub allowed_tools: Vec<String>,
}

impl SubAgentSpec {
    /// Construct a new child-agent specification.
    pub fn new(
        id: impl Into<String>,
        agent_type: AgentType,
        focus: impl Into<String>,
        context_files: Vec<PathBuf>,
        token_budget: usize,
        timeout_secs: u64,
        allowed_tools: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            agent_type,
            focus: focus.into(),
            context_files,
            token_budget,
            timeout_secs,
            allowed_tools,
        }
    }

    /// Build the isolated context view a child agent should receive.
    pub fn isolated_context(&self, parent_task: &str, depth: usize) -> SubAgentContext {
        SubAgentContext::new(parent_task, self, depth)
    }
}

/// Child-agent execution state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubAgentStatus {
    Pending,
    Running,
    Completed,
    Timeout,
    Failed(String),
    Cancelled,
}

impl SubAgentStatus {
    /// Whether the state is terminal.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Timeout | Self::Failed(_) | Self::Cancelled
        )
    }
}

/// Child-agent execution result with raw and filtered output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubAgentResult {
    pub sub_agent_id: String,
    pub raw_output: String,
    pub filtered_output: Option<String>,
    pub token_usage: TokenUsage,
    pub duration: Duration,
    pub spec: SubAgentSpec,
    pub status: SubAgentStatus,
}

impl SubAgentResult {
    /// Create a pending result wrapper for stateful transitions.
    pub fn pending(spec: SubAgentSpec) -> Self {
        Self {
            sub_agent_id: spec.id.clone(),
            raw_output: String::new(),
            filtered_output: None,
            token_usage: TokenUsage::default(),
            duration: Duration::ZERO,
            spec,
            status: SubAgentStatus::Pending,
        }
    }

    /// Transition into the running state.
    pub fn mark_running(&mut self) {
        self.status = SubAgentStatus::Running;
    }

    /// Mark the child-agent as completed.
    pub fn mark_completed(
        &mut self,
        raw_output: String,
        token_usage: TokenUsage,
        duration: Duration,
    ) {
        self.raw_output = raw_output;
        self.token_usage = token_usage;
        self.duration = duration;
        self.status = SubAgentStatus::Completed;
    }

    /// Mark the child-agent as failed.
    pub fn mark_failed(
        &mut self,
        reason: impl Into<String>,
        token_usage: TokenUsage,
        duration: Duration,
    ) {
        let reason = reason.into();
        self.raw_output = reason.clone();
        self.token_usage = token_usage;
        self.duration = duration;
        self.status = SubAgentStatus::Failed(reason);
    }

    /// Mark the child-agent as timed out.
    pub fn mark_timeout(&mut self, token_usage: TokenUsage, duration: Duration) {
        self.token_usage = token_usage;
        self.duration = duration;
        self.status = SubAgentStatus::Timeout;
    }

    /// Mark the child-agent as cancelled.
    pub fn mark_cancelled(&mut self, token_usage: TokenUsage, duration: Duration) {
        self.token_usage = token_usage;
        self.duration = duration;
        self.status = SubAgentStatus::Cancelled;
    }

    /// Convenience constructor for a completed result.
    pub fn completed(
        spec: SubAgentSpec,
        raw_output: String,
        token_usage: TokenUsage,
        duration: Duration,
    ) -> Self {
        let mut result = Self::pending(spec);
        result.mark_completed(raw_output, token_usage, duration);
        result
    }

    /// Convenience constructor for a failed result.
    pub fn failed(
        spec: SubAgentSpec,
        token_usage: TokenUsage,
        duration: Duration,
        reason: impl Into<String>,
    ) -> Self {
        let mut result = Self::pending(spec);
        result.mark_failed(reason, token_usage, duration);
        result
    }

    /// Convenience constructor for a timed-out result.
    pub fn timed_out(spec: SubAgentSpec, token_usage: TokenUsage, duration: Duration) -> Self {
        let mut result = Self::pending(spec);
        result.mark_timeout(token_usage, duration);
        result
    }

    /// Convenience constructor for a cancelled result.
    pub fn cancelled(spec: SubAgentSpec, token_usage: TokenUsage, duration: Duration) -> Self {
        let mut result = Self::pending(spec);
        result.mark_cancelled(token_usage, duration);
        result
    }
}

/// Minimal context view injected into a child agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubAgentContext {
    pub task_focus: String,
    pub relevant_files: Vec<FileSlice>,
    pub tool_permissions: Vec<String>,
    pub depth: usize,
    pub token_budget: usize,
}

impl SubAgentContext {
    pub fn new(parent_task: &str, spec: &SubAgentSpec, depth: usize) -> Self {
        Self {
            task_focus: format!(
                "[递归深度: {}] 父任务: {}\n你的焦点: {}",
                depth, parent_task, spec.focus
            ),
            relevant_files: spec
                .context_files
                .iter()
                .cloned()
                .map(|path| FileSlice {
                    path,
                    start_line: 0,
                    end_line: 0,
                })
                .collect(),
            tool_permissions: spec.allowed_tools.clone(),
            depth,
            token_budget: spec.token_budget,
        }
    }
}

/// File slice to inject into a child agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileSlice {
    pub path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
}

/// Factory used by the orchestrator to create concrete child agents.
#[async_trait]
pub trait AgentFactory: Send + Sync {
    async fn create_agent(
        &self,
        agent_type: AgentType,
        spec: SubAgentSpec,
    ) -> Result<Box<dyn SubAgentExecutor>>;
}

/// Execution interface implemented by concrete child agents.
#[async_trait]
pub trait SubAgentExecutor: Send + Sync {
    async fn execute(&mut self, cancel: CancellationToken) -> Result<String>;
    fn token_usage(&self) -> TokenUsage;
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use mc_core::agent::AgentType;

    use super::{SubAgentResult, SubAgentSpec, SubAgentStatus};

    #[test]
    fn sub_agent_result_transitions() {
        let spec = SubAgentSpec::new(
            "sub-1",
            AgentType::Explorer,
            "scan auth",
            vec![PathBuf::from("src/auth.rs")],
            4_000,
            30,
            vec!["read_file".to_string()],
        );

        let mut result = SubAgentResult::pending(spec);
        result.mark_running();
        result.mark_completed(
            "发现: issue".to_string(),
            mc_core::token::TokenUsage::default(),
            std::time::Duration::from_secs(1),
        );

        assert_eq!(result.status, SubAgentStatus::Completed);
        assert_eq!(result.raw_output, "发现: issue");
    }
}
