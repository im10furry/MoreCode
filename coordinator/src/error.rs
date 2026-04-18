use std::path::PathBuf;

use mc_agent::trait_def_min::AgentError;
use mc_llm::LlmError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoordinatorError {
    #[error("intent parse failed for {schema}: first={first_error}; repair={repair_error}")]
    IntentParseFailed {
        schema: String,
        first_error: String,
        repair_error: String,
    },
    #[error("config load failed: {0}: {1}")]
    ConfigLoadFailed(PathBuf, String),
    #[error("config parse failed: {0}: {1}")]
    ConfigParseFailed(PathBuf, String),
    #[error("io failure at {path}: {reason}")]
    Io { path: PathBuf, reason: String },
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("daemon task requires preloaded project context")]
    DaemonContextUnavailable,
    #[error("background task join failed: {0}")]
    Join(String),
    #[error(transparent)]
    Agent(#[from] AgentError),
    #[error(transparent)]
    Llm(#[from] LlmError),
}

impl CoordinatorError {
    pub fn io(path: impl Into<PathBuf>, reason: impl ToString) -> Self {
        Self::Io {
            path: path.into(),
            reason: reason.to_string(),
        }
    }
}

impl From<serde_json::Error> for CoordinatorError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value.to_string())
    }
}

impl From<tokio::task::JoinError> for CoordinatorError {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::Join(value.to_string())
    }
}
