use mc_core::AgentType;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Error)]
pub enum AgentError {
    #[error("context build failed: {message}")]
    ContextBuildFailed { message: String },
    #[error("missing context data: {data_type}")]
    MissingContextData { data_type: String },
    #[error("LLM request failed: {message}")]
    LlmError { message: String },
    #[error("failed to parse structured output: {message}")]
    LlmParseError { message: String },
    #[error("execution failed for {agent_type}: {message}")]
    ExecutionFailed {
        agent_type: AgentType,
        message: String,
    },
    #[error("resource constraint violated: {message}")]
    ResourceConstraint { message: String },
    #[error("validation failed: {message}")]
    Validation { message: String },
    #[error("io failure at {path}: {message}")]
    Io { path: String, message: String },
    #[error("serialization failed: {message}")]
    Serialization { message: String },
    #[error("agent execution cancelled")]
    Cancelled,
}

impl AgentError {
    pub fn io(path: impl Into<String>, err: impl ToString) -> Self {
        Self::Io {
            path: path.into(),
            message: err.to_string(),
        }
    }

    pub fn serialization(err: impl ToString) -> Self {
        Self::Serialization {
            message: err.to_string(),
        }
    }
}
