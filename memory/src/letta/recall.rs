use std::{path::Path, str::FromStr};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    QueryBuilder, Row, Sqlite,
};
use tokio::fs;

use crate::error::MemoryError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationEntry {
    pub id: String,
    pub session_id: String,
    pub role: ConversationRole,
    pub content: String,
    pub tokens_used: u32,
    pub timestamp: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConversationRole {
    User,
    Agent(String),
    System,
    Tool,
}

impl ConversationRole {
    fn as_db_value(&self) -> String {
        match self {
            Self::User => "user".to_string(),
            Self::Agent(name) => format!("agent:{name}"),
            Self::System => "system".to_string(),
            Self::Tool => "tool".to_string(),
        }
    }

    fn from_db_value(value: &str) -> Self {
        match value {
            "user" => Self::User,
            "system" => Self::System,
            "tool" => Self::Tool,
            value if value.starts_with("agent:") => {
                Self::Agent(value.trim_start_matches("agent:").to_string())
            }
            _ => Self::Agent(value.to_string()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub keyword: Option<String>,
    pub session_id: Option<String>,
    pub role: Option<ConversationRole>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: usize,
    pub offset: usize,
}

#[async_trait]
pub trait ConversationStore: Send + Sync {
    async fn append(&self, entry: ConversationEntry) -> Result<(), MemoryError>;
    async fn get_session(
        &self,
        session_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ConversationEntry>, MemoryError>;
    async fn search(&self, query: &SearchQuery) -> Result<Vec<ConversationEntry>, MemoryError>;
    async fn get_session_summary(&self, session_id: &str) -> Result<String, MemoryError>;
}

#[derive(Debug, Clone)]
pub struct SqliteConversationStore {
    pool: sqlx::SqlitePool,
}

impl SqliteConversationStore {
    pub async fn new(db_path: &Path) -> Result<Self, MemoryError> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .expect("hard-coded SQLite URL is valid")
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id          TEXT PRIMARY KEY,
                session_id  TEXT NOT NULL,
                role        TEXT NOT NULL,
                content     TEXT NOT NULL,
                tokens_used INTEGER NOT NULL DEFAULT 0,
                timestamp   TEXT NOT NULL,
                tags        TEXT NOT NULL DEFAULT '[]'
            );
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_conversations_session_time
            ON conversations(session_id, timestamp);
            "#,
        )
        .execute(&pool)
        .await?;

        // FTS5 is required here; LIKE fallback is intentionally not used.
        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS conversations_fts
            USING fts5(
                id UNINDEXED,
                session_id UNINDEXED,
                role UNINDEXED,
                content,
                tags
            );
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &sqlx::SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl ConversationStore for SqliteConversationStore {
    async fn append(&self, entry: ConversationEntry) -> Result<(), MemoryError> {
        let tags = serde_json::to_string(&entry.tags)?;
        let role = entry.role.as_db_value();
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO conversations (id, session_id, role, content, tokens_used, timestamp, tags)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.session_id)
        .bind(&role)
        .bind(&entry.content)
        .bind(entry.tokens_used as i64)
        .bind(entry.timestamp.to_rfc3339())
        .bind(&tags)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO conversations_fts (id, session_id, role, content, tags)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.session_id)
        .bind(&role)
        .bind(&entry.content)
        .bind(tags)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn get_session(
        &self,
        session_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ConversationEntry>, MemoryError> {
        let rows = if let Some(limit) = limit {
            sqlx::query(
                r#"
                SELECT id, session_id, role, content, tokens_used, timestamp, tags
                FROM conversations
                WHERE session_id = ?1
                ORDER BY timestamp ASC
                LIMIT ?2
                "#,
            )
            .bind(session_id)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, session_id, role, content, tokens_used, timestamp, tags
                FROM conversations
                WHERE session_id = ?1
                ORDER BY timestamp ASC
                "#,
            )
            .bind(session_id)
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter().map(row_to_conversation).collect()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<ConversationEntry>, MemoryError> {
        let mut builder = QueryBuilder::<Sqlite>::new(
            "SELECT c.id, c.session_id, c.role, c.content, c.tokens_used, c.timestamp, c.tags \
             FROM conversations c",
        );

        if query.keyword.is_some() {
            builder.push(" JOIN conversations_fts ON conversations_fts.id = c.id");
        }

        builder.push(" WHERE 1 = 1");

        if let Some(keyword) = &query.keyword {
            builder.push(" AND conversations_fts MATCH ");
            builder.push_bind(keyword);
        }

        if let Some(session_id) = &query.session_id {
            builder.push(" AND c.session_id = ");
            builder.push_bind(session_id);
        }

        if let Some(role) = &query.role {
            builder.push(" AND c.role = ");
            builder.push_bind(role.as_db_value());
        }

        if let Some(start_time) = query.start_time {
            builder.push(" AND c.timestamp >= ");
            builder.push_bind(start_time.to_rfc3339());
        }

        if let Some(end_time) = query.end_time {
            builder.push(" AND c.timestamp <= ");
            builder.push_bind(end_time.to_rfc3339());
        }

        builder.push(" ORDER BY c.timestamp DESC LIMIT ");
        builder.push_bind(query.limit.max(1) as i64);
        builder.push(" OFFSET ");
        builder.push_bind(query.offset as i64);

        let rows = builder.build().fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_conversation).collect()
    }

    async fn get_session_summary(&self, session_id: &str) -> Result<String, MemoryError> {
        let entries = self.get_session(session_id, Some(6)).await?;
        if entries.is_empty() {
            return Ok(String::new());
        }

        let mut lines = Vec::new();
        for entry in entries {
            lines.push(format!(
                "[{}] {}",
                entry.role.as_db_value(),
                entry.content.chars().take(120).collect::<String>()
            ));
        }
        Ok(lines.join("\n"))
    }
}

fn row_to_conversation(row: sqlx::sqlite::SqliteRow) -> Result<ConversationEntry, MemoryError> {
    let timestamp: String = row.try_get("timestamp")?;
    let parsed = DateTime::parse_from_rfc3339(&timestamp)
        .map_err(|error| MemoryError::Command(error.to_string()))?
        .with_timezone(&Utc);
    let tags: String = row.try_get("tags")?;
    Ok(ConversationEntry {
        id: row.try_get("id")?,
        session_id: row.try_get("session_id")?,
        role: ConversationRole::from_db_value(&row.try_get::<String, _>("role")?),
        content: row.try_get("content")?,
        tokens_used: row.try_get::<i64, _>("tokens_used")? as u32,
        timestamp: parsed,
        tags: serde_json::from_str(&tags)?,
    })
}
