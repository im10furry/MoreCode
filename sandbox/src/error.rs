use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CommandParseError {
    #[error("empty command is not allowed")]
    EmptyCommand,
    #[error("commands containing shell control operators must be structured: {0}")]
    ShellControlOperator(String),
    #[error("invalid command quoting or escaping: {0}")]
    InvalidQuoting(String),
    #[error("missing executable in command")]
    MissingExecutable,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SandboxError {
    #[error(transparent)]
    CommandParse(#[from] CommandParseError),

    #[error(transparent)]
    Wasm(#[from] WasmSandboxError),

    #[error("command is not allowed: {command} - {reason}")]
    CommandNotAllowed { command: String, reason: String },

    #[error("path access denied: {} - {reason}", path.display())]
    PathAccessDenied { path: PathBuf, reason: String },

    #[error("permission denied: task={task_id} - {reason}")]
    PermissionDenied { task_id: String, reason: String },

    #[error("command execution failed: {command} - {reason}")]
    CommandExecutionFailed { command: String, reason: String },

    #[error("command timed out: {command} after {timeout_ms}ms")]
    Timeout { command: String, timeout_ms: u64 },

    #[error("io error: {0}")]
    Io(String),

    #[error("landlock operation failed: {operation} - {reason}")]
    Landlock { operation: String, reason: String },

    #[error("operation is not supported on this platform: {0}")]
    UnsupportedPlatform(String),

    #[error("sandbox configuration invalid: {layer} - {reason}")]
    InvalidSandboxConfig { layer: String, reason: String },

    #[error("sandbox backend failure: {layer} - {reason}")]
    SandboxBackend { layer: String, reason: String },

    #[error("simulated execution: {0}")]
    Simulated(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WasmSandboxError {
    #[error("the wasm sandbox backend is disabled; enable the `wasm` feature")]
    FeatureDisabled,

    #[error("wasm sandbox limits must be non-zero")]
    InvalidLimits,

    #[error("wasm execution requests must include a module_path for Guardian integration")]
    MissingModulePath,

    #[error("WASI preopened directory does not exist: {0}")]
    MissingPreopenedDir(PathBuf),

    #[error("invalid WASI guest path: {0}")]
    InvalidGuestPath(String),

    #[error("invalid network allowlist pattern `{pattern}`: {reason}")]
    InvalidNetworkPattern { pattern: String, reason: String },

    #[error("environment variable `{0}` is not allowed by the WASI access plan")]
    EnvNotAllowed(String),

    #[error("failed to configure the wasm sandbox: {0}")]
    Setup(String),

    #[error("wasm validation failed: {0}")]
    Validation(String),

    #[error("failed to load wasm module: {0}")]
    Load(String),

    #[error("failed to instantiate wasm module: {0}")]
    Instantiate(String),

    #[error("wasm module does not export entrypoint `{0}`")]
    MissingEntrypoint(String),

    #[error("failed to execute wasm entrypoint `{entrypoint}`: {reason}")]
    Execute { entrypoint: String, reason: String },

    #[error("failed to join the wasm execution task: {0}")]
    Join(String),
}

impl From<std::io::Error> for SandboxError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
