use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CommandParseError {
    #[error("拒绝空命令")]
    EmptyCommand,
    #[error("命令包含 shell 控制符，必须使用结构化命令: {0}")]
    ShellControlOperator(String),
    #[error("命令引号或转义无效: {0}")]
    InvalidQuoting(String),
    #[error("命令缺少可执行文件")]
    MissingExecutable,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SandboxError {
    #[error(transparent)]
    CommandParse(#[from] CommandParseError),

    #[error("命令不被允许: {command} - {reason}")]
    CommandNotAllowed { command: String, reason: String },

    #[error("路径访问被拒绝: {} - {reason}", path.display())]
    PathAccessDenied { path: PathBuf, reason: String },

    #[error("权限不足: task={task_id} - {reason}")]
    PermissionDenied { task_id: String, reason: String },

    #[error("命令执行失败: {command} - {reason}")]
    CommandExecutionFailed { command: String, reason: String },

    #[error("命令执行超时: {command} after {timeout_ms}ms")]
    Timeout { command: String, timeout_ms: u64 },

    #[error("IO 错误: {0}")]
    Io(String),

    #[error("当前平台不支持该操作: {0}")]
    UnsupportedPlatform(String),

    #[error("仅模拟执行: {0}")]
    Simulated(String),
}

impl From<std::io::Error> for SandboxError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
