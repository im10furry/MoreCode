use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{LlmError, ModelInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiProviderConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: ModelInfo,
    #[serde(default)]
    pub default_headers: HashMap<String, String>,
    #[serde(default = "default_request_timeout")]
    pub request_timeout: Duration,
    #[serde(default = "default_stream_buffer_size")]
    pub stream_buffer_size: usize,
    #[serde(default = "default_true")]
    pub supports_structured_output: bool,
}

fn default_request_timeout() -> Duration {
    Duration::from_secs(120)
}

fn default_stream_buffer_size() -> usize {
    64
}

fn default_true() -> bool {
    true
}

impl OpenAiProviderConfig {
    pub fn validate(&self) -> Result<(), LlmError> {
        if self.base_url.trim().is_empty() {
            return Err(LlmError::Internal(
                "openai compatible provider base_url cannot be empty".into(),
            ));
        }
        if self.api_key.trim().is_empty() {
            return Err(LlmError::AuthenticationFailed {
                provider: self.model.provider_id.clone(),
                reason: "API key cannot be empty".into(),
            });
        }
        if self.stream_buffer_size == 0 {
            return Err(LlmError::Internal(
                "openai compatible stream_buffer_size must be greater than zero".into(),
            ));
        }
        Ok(())
    }
}
