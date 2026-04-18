use serde::{Deserialize, Serialize};

/// Token usage accounting information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Number of prompt tokens.
    pub prompt_tokens: u32,
    /// Number of completion tokens.
    pub completion_tokens: u32,
    /// Total token count reported by the provider.
    pub total_tokens: u32,
    /// Number of cached prompt tokens.
    pub cached_tokens: u32,
    /// Estimated cost in USD.
    pub estimated_cost_usd: f64,
}

impl TokenUsage {
    /// Compute the sum of prompt and completion tokens.
    pub fn total(&self) -> u32 {
        self.prompt_tokens + self.completion_tokens
    }
}

/// Static capability and pricing information for a model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Provider-specific model identifier.
    pub model_id: String,
    /// User-facing display name.
    pub display_name: String,
    /// Provider name.
    pub provider_name: String,
    /// Maximum context window size.
    pub max_context_tokens: u32,
    /// Maximum output token count.
    pub max_output_tokens: u32,
    /// Input price per million tokens.
    pub input_price_per_million: f64,
    /// Output price per million tokens.
    pub output_price_per_million: f64,
    /// Whether streaming is supported.
    pub supports_streaming: bool,
    /// Whether function or tool calling is supported.
    pub supports_function_calling: bool,
    /// Whether JSON mode is supported.
    pub supports_json_mode: bool,
    /// Whether prompt caching is supported.
    pub supports_prompt_caching: bool,
}

/// Function-calling tool definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON schema describing the parameters.
    pub parameters: serde_json::Value,
    /// Whether the tool is required.
    pub required: bool,
}
