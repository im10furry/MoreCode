use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("invalid HTTP header `{name}`: {message}")]
    InvalidHeader { name: String, message: String },
    #[error("tool `{tool_name}` has a non-object input schema")]
    ToolSchemaNotObject { tool_name: String },
    #[error("tool name conflict: `{name}` is already registered")]
    ToolNameConflict { name: String },
    #[error("failed to initialize MCP client `{name}`: {message}")]
    ClientInitialization { name: String, message: String },
    #[error("failed to initialize MCP server: {0}")]
    ServerInitialization(String),
    #[error("MCP request failed: {0}")]
    RequestFailed(String),
    #[error("remote tool `{server}:{tool}` timed out after {timeout_ms}ms")]
    ToolTimeout {
        server: String,
        tool: String,
        timeout_ms: u64,
    },
    #[error("failed to bind HTTP listener on `{address}`: {message}")]
    HttpBind { address: String, message: String },
    #[error("failed to bind unix socket `{path}`: {message}")]
    UnixBind { path: PathBuf, message: String },
    #[error("failed to serve HTTP connection: {0}")]
    HttpServe(String),
    #[error("background server task failed: {0}")]
    BackgroundTask(String),
    #[error("failed to close MCP connection: {0}")]
    ConnectionClose(String),
    #[error("unix socket transport is not supported on this platform")]
    UnixSocketUnsupported,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
