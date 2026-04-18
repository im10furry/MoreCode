use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum LlmError {
    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("authentication failed for provider '{provider}': {reason}")]
    AuthenticationFailed { provider: String, reason: String },

    #[error("rate limited by {provider}: retry after {retry_after_ms:?}ms")]
    RateLimited {
        provider: String,
        retry_after_ms: Option<u64>,
    },

    #[error("request timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("context length exceeded: {prompt_tokens} tokens > {max_tokens} limit")]
    ContextLengthExceeded { prompt_tokens: u32, max_tokens: u32 },

    #[error("model '{model_id}' not available: {reason}")]
    ModelUnavailable { model_id: String, reason: String },

    #[error("request cancelled: {reason}")]
    Cancelled { reason: String },

    #[error("stream error: {0}")]
    StreamError(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl LlmError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. } | Self::Timeout { .. } | Self::ApiError(_)
        )
    }

    pub fn retry_delay_ms(&self) -> Option<u64> {
        match self {
            Self::RateLimited { retry_after_ms, .. } => Some(retry_after_ms.unwrap_or(1000)),
            Self::Timeout { timeout_ms } => Some(*timeout_ms / 2),
            Self::ApiError(_) => Some(1000),
            _ => None,
        }
    }
}
