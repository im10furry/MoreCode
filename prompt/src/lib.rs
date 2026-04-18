#![forbid(unsafe_code)]

pub mod cache;
pub mod error;
pub mod layer;
pub mod manager;
pub mod template;
pub mod watcher;

pub use cache::{calculate_cache_savings, should_set_cache_breakpoint};
pub use error::PromptCacheError;
pub use layer::{PromptLayer, PromptLayerContent, PromptLayers, TurnMessage};
pub use manager::{CacheInvalidationEvent, InvalidationReason, PromptLayerManager};
pub use template::{PromptTemplate, TemplateManager, TemplateRenderer, TemplateVariable};
