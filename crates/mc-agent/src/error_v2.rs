use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum AgentError {
    #[error("缺少工具: {tool}")]
    MissingTool { tool: String },

    #[error("工具调用失败: {tool}: {reason}")]
    ToolExecutionFailed { tool: String, reason: String },

    #[error("工具返回无法解析: {tool}: {reason}")]
    ToolPayloadInvalid { tool: String, reason: String },

    #[error("执行已取消: {reason}")]
    Cancelled { reason: String },

    #[error("递归深度超限: 当前 {current}, 最大 {max}")]
    RecursionDepthExceeded { current: u8, max: u8 },

    #[error("模板渲染失败: {message}")]
    TemplateError { message: String },

    #[error("序列化失败: {message}")]
    Serialization { message: String },

    #[error("配置错误: {message}")]
    Config { message: String },

    #[error("校验失败: {message}")]
    Validation { message: String },

    #[error("IO 失败: {path}: {reason}")]
    Io { path: String, reason: String },

    #[error("内部错误: {message}")]
    Internal { message: String },
}

impl From<serde_json::Error> for AgentError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization {
            message: error.to_string(),
        }
    }
}

impl From<toml::de::Error> for AgentError {
    fn from(error: toml::de::Error) -> Self {
        Self::Config {
            message: error.to_string(),
        }
    }
}
