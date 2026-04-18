use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Shared core error enum for the MoreCode workspace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum McError {
    /// Configuration file loading failed.
    #[error("配置加载失败: {path}: {reason}")]
    ConfigLoadFailed {
        /// Path of the configuration file.
        path: String,
        /// Failure reason.
        reason: String,
    },
    /// Configuration parsing failed.
    #[error("配置解析失败: {path}: {reason}")]
    ConfigParseFailed {
        /// Path of the configuration file.
        path: String,
        /// Failure reason.
        reason: String,
    },
    /// Configuration validation failed.
    #[error("配置验证失败: {field}: {reason}")]
    ConfigValidationFailed {
        /// Invalid field name.
        field: String,
        /// Failure reason.
        reason: String,
    },
    /// A communication channel was closed unexpectedly.
    #[error("通道已关闭: {channel}")]
    ChannelClosed {
        /// Closed channel name.
        channel: String,
    },
    /// Sending on a channel timed out.
    #[error("发送超时: {channel} (超时 {timeout_ms}ms)")]
    SendTimeout {
        /// Timed out channel name.
        channel: String,
        /// Timeout in milliseconds.
        timeout_ms: u64,
    },
    /// A broadcast subscriber lagged behind and dropped messages.
    #[error("广播订阅者落后: {subscriber} (丢失 {skipped} 条消息)")]
    BroadcastLagged {
        /// Subscriber identifier.
        subscriber: String,
        /// Number of skipped messages.
        skipped: u64,
    },
    /// Requested agent was not registered.
    #[error("Agent 未注册: {agent_type}")]
    AgentNotRegistered {
        /// Agent type name.
        agent_type: String,
    },
    /// Agent execution exceeded its timeout.
    #[error("Agent 执行超时: {agent_type} (超时 {timeout_secs}s)")]
    AgentTimeout {
        /// Agent type name.
        agent_type: String,
        /// Timeout in seconds.
        timeout_secs: u64,
    },
    /// Agent execution failed.
    #[error("Agent 执行失败: {agent_type}: {reason}")]
    AgentExecutionFailed {
        /// Agent type name.
        agent_type: String,
        /// Failure reason.
        reason: String,
    },
    /// Task could not be found.
    #[error("任务未找到: {task_id}")]
    TaskNotFound {
        /// Missing task identifier.
        task_id: String,
    },
    /// Token budget was exceeded.
    #[error("Token 预算超限: 已使用 {used}, 预算 {budget}")]
    TokenBudgetExceeded {
        /// Used token count.
        used: u32,
        /// Budget token count.
        budget: u32,
    },
    /// File system operation failed.
    #[error("文件操作失败: {path}: {reason}")]
    FileOperationFailed {
        /// File path involved in the failure.
        path: String,
        /// Failure reason.
        reason: String,
    },
    /// JSON serialization or deserialization failed.
    #[error("JSON 处理失败: {reason}")]
    SerializationFailed {
        /// Failure reason.
        reason: String,
    },
    /// LLM provider call failed.
    #[error("LLM 调用失败: {provider}: {reason}")]
    LlmError {
        /// Provider name.
        provider: String,
        /// Failure reason.
        reason: String,
    },
    /// Catch-all internal error.
    #[error("内部错误: {message}")]
    InternalError {
        /// Internal error message.
        message: String,
    },
}

/// Convenient result alias for `mc-core`.
pub type McResult<T> = Result<T, McError>;
