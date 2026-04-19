use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TuiError {
    #[error("panel not available: {0}")]
    PanelUnavailable(String),
    #[error("invalid layout: {0}")]
    InvalidLayout(String),
    #[error("event handling failed: {0}")]
    EventHandling(String),
}
