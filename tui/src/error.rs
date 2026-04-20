use thiserror::Error;

/// Errors produced by the terminal UI runtime and state model.
#[derive(Debug, Error)]
pub enum TuiError {
    #[error("terminal io failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid layout: {0}")]
    InvalidLayout(String),
    #[error("event handling failed: {0}")]
    EventHandling(String),
    #[error("terminal update channel is closed")]
    UpdateChannelClosed,
}
