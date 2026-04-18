use chrono::Utc;
use mc_core::{AgentExecutionReport, AgentType};
use serde::Serialize;
use serde_json::Value;

use crate::AgentError;

pub fn serialize_extra<T: Serialize>(value: &T) -> Result<Value, AgentError> {
    serde_json::to_value(value).map_err(AgentError::from)
}

pub fn build_report(
    agent_type: AgentType,
    title: impl Into<String>,
    key_findings: Vec<String>,
    relevant_files: Vec<String>,
    recommendations: Vec<String>,
    warnings: Vec<String>,
    token_used: u32,
    extra: Option<Value>,
) -> AgentExecutionReport {
    AgentExecutionReport {
        title: format!("{}: {}", agent_type.as_str(), title.into()),
        key_findings,
        relevant_files,
        recommendations,
        warnings,
        token_used,
        timestamp: Utc::now(),
        extra,
    }
}
