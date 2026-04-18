use chrono::{DateTime, Utc};
use mc_core::AgentType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionReport {
    pub execution_id: Uuid,
    pub agent_type: AgentType,
    pub status: ExecutionStatus,
    pub result: Option<serde_json::Value>,
    pub tool_calls: Vec<ToolCallRecord>,
    pub llm_stats: LlmCallStats,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub files_affected: Vec<FileChange>,
    pub sub_reports: Vec<AgentExecutionReport>,
    pub handoff_summary: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    PartialSuccess,
    Failed,
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmCallStats {
    pub total_calls: u32,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub total_duration_ms: u64,
    pub cache_hit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: FileChangeType,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

impl AgentExecutionReport {
    pub fn success(agent_type: AgentType, execution_id: Uuid, result: serde_json::Value) -> Self {
        Self {
            execution_id,
            agent_type,
            status: ExecutionStatus::Success,
            result: Some(result),
            tool_calls: Vec::new(),
            llm_stats: LlmCallStats::default(),
            duration_ms: 0,
            error: None,
            files_affected: Vec::new(),
            sub_reports: Vec::new(),
            handoff_summary: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn failed(agent_type: AgentType, execution_id: Uuid, error: impl Into<String>) -> Self {
        Self {
            execution_id,
            agent_type,
            status: ExecutionStatus::Failed,
            result: None,
            tool_calls: Vec::new(),
            llm_stats: LlmCallStats::default(),
            duration_ms: 0,
            error: Some(error.into()),
            files_affected: Vec::new(),
            sub_reports: Vec::new(),
            handoff_summary: HashMap::new(),
            timestamp: Utc::now(),
        }
    }
}
