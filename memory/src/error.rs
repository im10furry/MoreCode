use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("token 预算超限: 需要 {required} tokens, 可用 {available} tokens")]
    TokenBudgetExceeded { required: usize, available: usize },
    #[error("记忆块不存在: {0}")]
    BlockNotFound(String),
    #[error("缓存容量超限")]
    CacheCapacityExceeded,
    #[error("无效的规则正则 `{pattern}`: {source}")]
    InvalidRuleRegex {
        pattern: String,
        #[source]
        source: regex::Error,
    },
    #[error("文件内容不是有效 UTF-8: {0}")]
    InvalidUtf8(PathBuf),
    #[error("序列化失败: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("I/O 失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite 失败: {0}")]
    Sqlite(#[from] sqlx::Error),
    #[error("Tantivy 失败: {0}")]
    Tantivy(#[from] tantivy::TantivyError),
    #[error("Tantivy 查询解析失败: {0}")]
    QueryParser(#[from] tantivy::query::QueryParserError),
    #[error("后台任务失败: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("命令执行失败: {0}")]
    Command(String),
}
