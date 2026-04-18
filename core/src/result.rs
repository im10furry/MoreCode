use crate::task::ResultType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Generic task result returned by an agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskResult {
    /// Category of the produced result.
    pub result_type: ResultType,
    /// Whether execution succeeded.
    pub success: bool,
    /// Structured result payload.
    pub data: serde_json::Value,
    /// Files changed by the execution.
    pub changed_files: Vec<String>,
    /// Generated textual content, if any.
    pub generated_content: Option<String>,
    /// Error message for failed executions.
    pub error_message: Option<String>,
}

/// Summary handoff passed between agents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentExecutionReport {
    /// Report title.
    pub title: String,
    /// Key findings from the previous agent.
    pub key_findings: Vec<String>,
    /// Files the next agent should focus on.
    pub relevant_files: Vec<String>,
    /// Recommendations for the next agent.
    pub recommendations: Vec<String>,
    /// Warnings or unresolved risks.
    pub warnings: Vec<String>,
    /// Tokens consumed by the previous agent.
    pub token_used: u32,
    /// Report creation timestamp.
    pub timestamp: DateTime<Utc>,
    /// Extra structured payload.
    pub extra: Option<serde_json::Value>,
}
