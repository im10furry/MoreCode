use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{LlmError, ModelInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleProviderConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: ModelInfo,
    #[serde(default)]
    pub default_headers: HashMap<String, String>,
    #[serde(default = "default_request_timeout")]
    pub request_timeout: Duration,
    #[serde(default = "default_stream_buffer_size")]
    pub stream_buffer_size: usize,
    #[serde(default = "default_max_output_tokens")]
    pub default_max_output_tokens: u32,
}

fn default_request_timeout() -> Duration {
    Duration::from_secs(120)
}

fn default_stream_buffer_size() -> usize {
    64
}

fn default_max_output_tokens() -> u32 {
    8_192
}

impl GoogleProviderConfig {
    pub fn validate(&self) -> Result<(), LlmError> {
        if self.base_url.trim().is_empty() {
            return Err(LlmError::Internal(
                "google provider base_url cannot be empty".into(),
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
                "google stream_buffer_size must be greater than zero".into(),
            ));
        }
        if self.default_max_output_tokens == 0 {
            return Err(LlmError::Internal(
                "google default_max_output_tokens must be greater than zero".into(),
            ));
        }
        Ok(())
    }
}
