#![forbid(unsafe_code)]

#[cfg(feature = "anthropic")]
pub mod anthropic;
mod cache_capability;
mod error;
#[cfg(feature = "google")]
pub mod google;
#[cfg(feature = "mock")]
pub mod mock;
mod model;
#[cfg(feature = "openai-compat")]
pub mod openai_compat;
mod provider;
mod request;
mod response;
mod semantic_cache;
mod stream;
mod token;

#[cfg(feature = "anthropic")]
pub use anthropic::{AnthropicProvider, AnthropicProviderConfig};
pub use cache_capability::{
    CacheCapability, CacheControlPoint, CacheControlType, CacheStrategy, OpenAiCacheStrategy,
};
pub use error::LlmError;
#[cfg(feature = "google")]
pub use google::{GoogleProvider, GoogleProviderConfig};
#[cfg(feature = "mock")]
pub use mock::{MockProvider, MockResponse, MockStreamChunk};
pub use model::{ModelInfo, ResponseFormat, TokenUsage, ToolCall, ToolDefinition};
#[cfg(feature = "openai-compat")]
pub use openai_compat::{
    OpenAiCompatiblePreset, OpenAiCompatibleProviderPreset, OpenAiProvider, OpenAiProviderConfig,
};
pub use provider::LlmProvider;
pub use request::{
    ChatMessage, ChatRequest, ContentPart, ImageDetail, MessageContent, MessageRole,
};
pub use response::{ChatResponse, FinishReason, StreamEvent};
pub use semantic_cache::{
    AgentType, CacheStats, InMemorySemanticCacheStore, SemanticCacheConfig, SemanticCacheEntry,
    SemanticCacheMiddleware, SemanticCacheNamespace, SemanticCacheStore,
};
pub use stream::{EventBus, InMemoryEventBus, StreamForwarder};
pub use token::{
    calibrate, estimate_content_tokens, estimate_message_tokens, estimate_part_tokens,
    estimate_text_tokens, BudgetError, BudgetNode, CostTracker, ModelCostRecord,
    MultimodalEstimateBreakdown, Pricing,
};
