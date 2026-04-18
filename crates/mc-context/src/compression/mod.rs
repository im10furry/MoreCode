pub mod compressor_agent;
pub mod l1_global;
pub mod l2_auto_compact;
pub mod l3_persist;
pub mod l4_truncate;
pub mod strategy;

pub use compressor_agent::{
    ChatMessage, CompactStats, CompactSummary, CompressionPromptBuilder, LlmClient, LlmError,
    LlmRequest, LlmResponse, MemoryCompactResult, MemoryStore, MessageRole, ResponseFormat,
    SimpleTokenCounter, TokenCounter,
};
pub use l1_global::MicroCompactor;
pub use l2_auto_compact::{AutoCompactOutcome, AutoCompactor};
pub use l3_persist::{ExtractionRule, ExtractorType, MemoryCompactor};
pub use l4_truncate::ReactiveTruncator;
pub use strategy::{
    CompressionCoordinator, CompressionResult, ContextCompressor, L0RetentionPolicy,
};
