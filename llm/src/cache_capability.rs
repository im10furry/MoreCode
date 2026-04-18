use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheCapability {
    pub supports_prompt_caching: bool,
    pub max_cache_ttl_secs: Option<u64>,
    pub min_cacheable_tokens: u32,
    pub supported_control_types: Vec<CacheControlType>,
    pub strategy: CacheStrategy,
}

impl Default for CacheCapability {
    fn default() -> Self {
        Self {
            supports_prompt_caching: false,
            max_cache_ttl_secs: None,
            min_cacheable_tokens: 0,
            supported_control_types: Vec::new(),
            strategy: CacheStrategy::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheStrategy {
    OpenAi(OpenAiCacheStrategy),
    Anthropic { breakpoint_marker: String },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheControlPoint {
    pub message_index: usize,
    pub control_type: CacheControlType,
    pub ttl_secs: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheControlType {
    CacheBreakpoint,
    Ephemeral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiCacheStrategy {
    pub auto_detect: bool,
    pub prefix_match_min_length: usize,
    pub warmup_threshold: u32,
}

impl Default for OpenAiCacheStrategy {
    fn default() -> Self {
        Self {
            auto_detect: true,
            prefix_match_min_length: 1024,
            warmup_threshold: 2,
        }
    }
}
