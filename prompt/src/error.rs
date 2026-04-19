use crate::layer::PromptLayer;

#[derive(Debug, thiserror::Error)]
pub enum PromptCacheError {
    #[error("layer '{0}' is not initialized")]
    LayerNotInitialized(PromptLayer),

    #[error("turn layer must be updated through PromptLayerManager turn APIs")]
    TurnLayerNotUpdatable,

    #[error("stale invalidation event for layer '{layer}': event version {event_version}, current version {current_version}")]
    StaleEvent {
        layer: PromptLayer,
        event_version: u64,
        current_version: u64,
    },

    #[error("template '{0}' not found")]
    TemplateNotFound(String),

    #[error("invalid variable name '{0}'")]
    InvalidVariableName(String),

    #[error("template render error: {0}")]
    TemplateRenderError(String),

    #[error("template lock mismatch: {0}")]
    LockMismatch(String),

    #[error("template lock format error: {0}")]
    LockFormatError(String),

    #[error("file watcher error: {0}")]
    FileWatcherError(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
