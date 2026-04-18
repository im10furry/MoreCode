use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    #[serde(default = "default_model")]
    pub default_model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_output_tokens")]
    pub max_output_tokens: u32,
    #[serde(default = "default_true")]
    pub streaming: bool,
    #[serde(default = "default_tool_timeout_secs")]
    pub tool_timeout_secs: u64,
    #[serde(default = "default_true")]
    pub auto_retry: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            default_model: default_model(),
            temperature: default_temperature(),
            max_output_tokens: default_max_output_tokens(),
            streaming: default_true(),
            tool_timeout_secs: default_tool_timeout_secs(),
            auto_retry: default_true(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialAgentConfig {
    pub default_model: Option<String>,
    pub temperature: Option<f32>,
    pub max_output_tokens: Option<u32>,
    pub streaming: Option<bool>,
    pub tool_timeout_secs: Option<u64>,
    pub auto_retry: Option<bool>,
}

impl AgentConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialAgentConfig) {
        if let Some(value) = partial.default_model {
            self.default_model = value;
        }
        if let Some(value) = partial.temperature {
            self.temperature = value;
        }
        if let Some(value) = partial.max_output_tokens {
            self.max_output_tokens = value;
        }
        if let Some(value) = partial.streaming {
            self.streaming = value;
        }
        if let Some(value) = partial.tool_timeout_secs {
            self.tool_timeout_secs = value;
        }
        if let Some(value) = partial.auto_retry {
            self.auto_retry = value;
        }
    }
}

impl PartialAgentConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            default_model: other.default_model.or(self.default_model),
            temperature: other.temperature.or(self.temperature),
            max_output_tokens: other.max_output_tokens.or(self.max_output_tokens),
            streaming: other.streaming.or(self.streaming),
            tool_timeout_secs: other.tool_timeout_secs.or(self.tool_timeout_secs),
            auto_retry: other.auto_retry.or(self.auto_retry),
        }
    }
}

fn default_model() -> String {
    "gpt-4o".to_string()
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_output_tokens() -> u32 {
    4_096
}

fn default_true() -> bool {
    true
}

fn default_tool_timeout_secs() -> u64 {
    60
}
