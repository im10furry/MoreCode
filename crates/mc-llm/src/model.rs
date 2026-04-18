use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub cached_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    pub fn cache_hit_rate(&self) -> f64 {
        if self.prompt_tokens == 0 {
            0.0
        } else {
            self.cached_tokens as f64 / self.prompt_tokens as f64
        }
    }

    pub fn estimate_cost(&self, input_price_per_1k: f64, output_price_per_1k: f64) -> f64 {
        let uncached_prompt = self.prompt_tokens.saturating_sub(self.cached_tokens);
        let input_cost = (uncached_prompt as f64 / 1000.0) * input_price_per_1k;
        let output_cost = (self.completion_tokens as f64 / 1000.0) * output_price_per_1k;
        input_cost + output_cost
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub provider_id: String,
    pub max_context_tokens: u32,
    pub max_output_tokens: u32,
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
    pub cache_read_price_per_1k: f64,
    pub cache_write_price_per_1k: f64,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_json_mode: bool,
}

impl ModelInfo {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        provider_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            provider_id: provider_id.into(),
            max_context_tokens: 0,
            max_output_tokens: 0,
            input_price_per_1k: 0.0,
            output_price_per_1k: 0.0,
            cache_read_price_per_1k: 0.0,
            cache_write_price_per_1k: 0.0,
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
            supports_json_mode: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    #[serde(default)]
    pub strict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    Text,
    JsonObject,
    JsonSchema {
        schema: serde_json::Value,
        name: String,
        #[serde(default)]
        strict: bool,
    },
}
