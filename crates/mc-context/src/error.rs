use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ContextError>;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("automatic compaction requires an llm client")]
    MissingLlmClient,
    #[error("memory compaction requires a memory store")]
    MissingMemoryStore,
    #[error("platform cache signature mismatch")]
    PlatformSignatureMismatch,
    #[error("rolling summary packet is incomplete")]
    InvalidRollingSummary,
    #[error("failed to serialize platform info: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("io error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl ContextError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
