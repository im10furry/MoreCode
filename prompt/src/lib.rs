#![forbid(unsafe_code)]

pub mod cache;
pub mod cache_adapter;
pub mod error;
pub mod layer;
pub mod manager;
pub mod template;
pub mod version;
pub mod warmup;
pub mod watcher;

pub use cache::{
    calculate_cache_savings, plan_cacheable_layers, should_set_cache_breakpoint,
    CacheBreakpointPlan,
};
pub use error::PromptCacheError;
pub use layer::{PromptLayer, PromptLayerContent, PromptLayers, TurnMessage};
pub use manager::{CacheInvalidationEvent, InvalidationReason, PromptLayerManager};
pub use template::{
    LockMismatch, PromptTemplate, PromptsLock, TemplateLockEntry, TemplateManager,
    TemplateRenderer, TemplateVariable,
};
pub use version::{CacheVersionTracker, VersionedLayer};
pub use warmup::{CacheWarmupStrategy, WarmupPlan, WarmupResult, WarmupTarget};
