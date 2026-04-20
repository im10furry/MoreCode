use chrono::{DateTime, Utc};
use mc_core::AgentType;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentExecutionMetrics {
    pub duration_ms: u64,
    pub tokens_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentExecutionReport {
    pub agent_type: AgentType,
    pub execution_id: String,
    pub success: bool,
    pub summary: String,
    pub result: Option<Value>,
    pub warnings: Vec<String>,
    pub metrics: AgentExecutionMetrics,
    pub created_at: DateTime<Utc>,
}

impl AgentExecutionReport {
    pub fn success(
        agent_type: AgentType,
        execution_id: impl Into<String>,
        summary: impl Into<String>,
        result: Value,
        duration_ms: u64,
        tokens_used: u32,
    ) -> Self {
        Self {
            agent_type,
            execution_id: execution_id.into(),
            success: true,
            summary: summary.into(),
            result: Some(result),
            warnings: Vec::new(),
            metrics: AgentExecutionMetrics {
                duration_ms,
                tokens_used,
            },
            created_at: Utc::now(),
        }
    }
}
