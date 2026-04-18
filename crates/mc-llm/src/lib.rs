#![forbid(unsafe_code)]

mod cache_capability;
mod error;
mod model;
#[cfg(feature = "openai-compat")]
pub mod openai_compat;
mod provider;
mod request;
mod response;
mod semantic_cache;
mod stream;
mod token;

pub use cache_capability::{
    CacheCapability, CacheControlPoint, CacheControlType, CacheStrategy, OpenAiCacheStrategy,
};
pub use error::LlmError;
pub use model::{ModelInfo, ResponseFormat, TokenUsage, ToolCall, ToolDefinition};
#[cfg(feature = "openai-compat")]
pub use openai_compat::{OpenAiProvider, OpenAiProviderConfig};
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
    calibrate, estimate_text_tokens, BudgetError, BudgetNode, CostTracker, ModelCostRecord, Pricing,
};
