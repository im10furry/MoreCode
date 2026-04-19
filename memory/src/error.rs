use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("token budget exceeded: required {required}, available {available}")]
    TokenBudgetExceeded { required: usize, available: usize },
    #[error("memory block not found: {0}")]
    BlockNotFound(String),
    #[error("cache capacity exceeded")]
    CacheCapacityExceeded,
    #[error("invalid rule regex `{pattern}`: {source}")]
    InvalidRuleRegex {
        pattern: String,
        #[source]
        source: regex::Error,
    },
    #[error("file is not valid UTF-8: {0}")]
    InvalidUtf8(PathBuf),
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite failed: {0}")]
    Sqlite(#[from] sqlx::Error),
    #[error("Tantivy failed: {0}")]
    Tantivy(#[from] tantivy::TantivyError),
    #[error("Tantivy query parsing failed: {0}")]
    QueryParser(#[from] tantivy::query::QueryParserError),
    #[error("background task failed: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("command execution failed: {0}")]
    Command(String),
    #[error("internal memory error: {0}")]
    Internal(String),
}
