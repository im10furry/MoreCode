use mc_core::AgentType;
use serde::Serialize;
use std::error::Error as StdError;
use thiserror::Error;

pub type BoxError = Box<dyn StdError + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ErrorCategory {
    Registration,
    Execution,
    Context,
    Llm,
    Tool,
    Config,
    Lifecycle,
    Stream,
    Internal,
}

#[derive(Debug, Error, Serialize)]
pub enum AgentError {
    #[error("Agent already registered: {agent_type} - {message}")]
    DuplicateRegistration {
        agent_type: AgentType,
        message: String,
    },

    #[error("Agent not found: {agent_type} - {message}")]
    AgentNotFound {
        agent_type: AgentType,
        message: String,
    },

    #[error("Agent execution failed: {agent_type} - {message}")]
    ExecutionFailed {
        agent_type: AgentType,
        message: String,
        #[serde(skip_serializing)]
        #[source]
        source: Option<BoxError>,
    },

    #[error("Agent execution timed out after {timeout_ms}ms: {agent_type}")]
    ExecutionTimeout {
        agent_type: AgentType,
        timeout_ms: u64,
    },

    #[error("Agent execution cancelled: {agent_type}")]
    ExecutionCancelled { agent_type: AgentType },

    #[error("Context build failed: {message}")]
    ContextBuildFailed {
        message: String,
        #[serde(skip_serializing)]
        #[source]
        source: Option<BoxError>,
    },

    #[error("Missing context data: {data_type}")]
    MissingContextData { data_type: String },

    #[error("LLM request failed: {message}")]
    LlmError {
        message: String,
        #[serde(skip_serializing)]
        #[source]
        source: Option<BoxError>,
    },

    #[error("LLM response parse failed: {message}")]
    LlmParseError { message: String },

    #[error("Tool execution failed: {tool_name} - {message}")]
    ToolError {
        tool_name: String,
        message: String,
    },

    #[error("Tool not found: {tool_name}")]
    ToolNotFound { tool_name: String },

    #[error("Invalid agent config: {message}")]
    InvalidConfig { message: String },

    #[error("Lifecycle error at stage {stage}: {message}")]
    LifecycleError { stage: String, message: String },

    #[error("Stream forwarder error: {message}")]
    StreamError { message: String },

    #[error("Agent internal error: {message}")]
    Internal {
        message: String,
        #[serde(skip_serializing)]
        #[source]
        source: Option<BoxError>,
    },
}

impl AgentError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::DuplicateRegistration { .. } | Self::AgentNotFound { .. } => {
                ErrorCategory::Registration
            }
            Self::ExecutionFailed { .. }
            | Self::ExecutionTimeout { .. }
            | Self::ExecutionCancelled { .. } => ErrorCategory::Execution,
            Self::ContextBuildFailed { .. } | Self::MissingContextData { .. } => {
                ErrorCategory::Context
            }
            Self::LlmError { .. } | Self::LlmParseError { .. } => ErrorCategory::Llm,
            Self::ToolError { .. } | Self::ToolNotFound { .. } => ErrorCategory::Tool,
            Self::InvalidConfig { .. } => ErrorCategory::Config,
            Self::LifecycleError { .. } => ErrorCategory::Lifecycle,
            Self::StreamError { .. } => ErrorCategory::Stream,
            Self::Internal { .. } => ErrorCategory::Internal,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::LlmError { .. } | Self::ExecutionTimeout { .. } | Self::ToolError { .. }
        )
    }

    pub fn retry_delay_ms(&self) -> u64 {
        match self {
            Self::LlmError { .. } => 100,
            Self::ExecutionTimeout { .. } => 200,
            Self::ToolError { .. } => 50,
            _ => 100,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    pub fn stream(message: impl Into<String>) -> Self {
        Self::StreamError {
            message: message.into(),
        }
    }

    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfig {
            message: message.into(),
        }
    }

    pub fn lock_poisoned(lock_name: &'static str) -> Self {
        Self::internal(format!("lock poisoned: {lock_name}"))
    }
}
