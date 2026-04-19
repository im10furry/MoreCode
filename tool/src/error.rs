use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ToolError {
    #[error("tool not found: {tool_name}")]
    NotFound { tool_name: String },
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("tool execution failed: {0}")]
    ExecutionFailed(String),
    #[error("tool registry is temporarily unavailable: {0}")]
    RegistryUnavailable(String),
}

impl From<serde_json::Error> for ToolError {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidParams(value.to_string())
    }
}
