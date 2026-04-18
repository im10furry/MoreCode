#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Row,
};
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Field, SchemaBuilder, Value, STORED, STRING, TEXT},
    Index, IndexReader, IndexWriter, TantivyDocument, Term,
};
use tokio::{fs, task};

use crate::error::MemoryError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub source: KnowledgeSource,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub embedding: Option<Vec<f32>>,
    pub entities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KnowledgeSource {
    ConversationExtraction {
        session_id: String,
        entry_id: String,
    },
    CoreMemoryDemotion {
        block_id: String,
    },
    Manual,
    CodeComment {
        file_path: String,
    },
}

#[async_trait]
pub trait KnowledgeStore: Send + Sync {
    async fn store(&self, entry: KnowledgeEntry) -> Result<String, MemoryError>;
    async fn retrieve(&self, id: &str) -> Result<Option<KnowledgeEntry>, MemoryError>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeEntry>, MemoryError>;
    async fn delete(&self, id: &str) -> Result<bool, MemoryError>;
}

#[derive(Debug, Clone, Copy)]
struct TantivyFields {
    title: Field,
    content: Field,
    category: Field,
    tags: Field,
    entry_id: Field,
}

struct TantivyState {
    index: Index,
    reader: IndexReader,
    writer: IndexWriter,
    fields: TantivyFields,
}

#[derive(Clone)]
pub struct TantivySearchEngine {
    state: Arc<Mutex<TantivyState>>,
    #[cfg(test)]
    commit_counter: Arc<AtomicUsize>,
}

impl TantivySearchEngine {
    pub async fn new(index_dir: &Path) -> Result<Self, MemoryError> {
        fs::create_dir_all(index_dir).await?;
        let index_dir = index_dir.to_path_buf();
        let state = task::spawn_blocking(move || create_tantivy_state(&index_dir)).await??;

        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            #[cfg(test)]
            commit_counter: Arc::new(AtomicUsize::new(0)),
        })
    }

    pub async fn index_entry(&self, entry: &KnowledgeEntry) -> Result<(), MemoryError> {
        let entry = entry.clone();
        let state = Arc::clone(&self.state);
        #[cfg(test)]
        let commit_counter = Arc::clone(&self.commit_counter);

        task::spawn_blocking(move || {
            let mut state = state.lock().expect("tantivy mutex poisoned");
            let document = build_document(&state.fields, &entry);
            state
                .writer
                .delete_term(Term::from_field_text(state.fields.entry_id, &entry.id));
            state.writer.add_document(document)?;
            state.writer.commit()?;
            state.reader.reload()?;
            #[cfg(test)]
            commit_counter.fetch_add(1, Ordering::SeqCst);
            Ok::<_, MemoryError>(())
        })
        .await??;

        Ok(())
    }

    pub async fn index_entries(&self, entries: &[KnowledgeEntry]) -> Result<(), MemoryError> {
        let entries = entries.to_vec();
        let state = Arc::clone(&self.state);
        #[cfg(test)]
        let commit_counter = Arc::clone(&self.commit_counter);

        task::spawn_blocking(move || {
            let mut state = state.lock().expect("tantivy mutex poisoned");
            for entry in &entries {
                state
                    .writer
                    .delete_term(Term::from_field_text(state.fields.entry_id, &entry.id));
                state
                    .writer
                    .add_document(build_document(&state.fields, entry))?;
            }
            state.writer.commit()?;
            state.reader.reload()?;
            #[cfg(test)]
            commit_counter.fetch_add(1, Ordering::SeqCst);
            Ok::<_, MemoryError>(())
        })
        .await??;

        Ok(())
    }

    pub async fn delete_entry(&self, id: &str) -> Result<(), MemoryError> {
        let id = id.to_string();
        let state = Arc::clone(&self.state);
        #[cfg(test)]
        let commit_counter = Arc::clone(&self.commit_counter);

        task::spawn_blocking(move || {
            let mut state = state.lock().expect("tantivy mutex poisoned");
            state
                .writer
                .delete_term(Term::from_field_text(state.fields.entry_id, &id));
            state.writer.commit()?;
            state.reader.reload()?;
            #[cfg(test)]
            commit_counter.fetch_add(1, Ordering::SeqCst);
            Ok::<_, MemoryError>(())
        })
        .await??;

        Ok(())
    }

    pub async fn search_ids(&self, query: &str, limit: usize) -> Result<Vec<String>, MemoryError> {
        let query = query.to_string();
        let state = Arc::clone(&self.state);
        task::spawn_blocking(move || {
            let state = state.lock().expect("tantivy mutex poisoned");
            let searcher = state.reader.searcher();
            let parser = QueryParser::for_index(
                &state.index,
                vec![state.fields.title, state.fields.content, state.fields.tags],
            );
            let parsed = parser.parse_query(&query)?;
            let top_docs = searcher.search(&parsed, &TopDocs::with_limit(limit))?;

            let mut ids = Vec::with_capacity(top_docs.len());
            for (_score, address) in top_docs {
                let document: TantivyDocument = searcher.doc(address)?;
                if let Some(id) = document
                    .get_first(state.fields.entry_id)
                    .and_then(|value| value.as_str())
                {
                    ids.push(id.to_string());
                }
            }

            Ok::<_, MemoryError>(ids)
        })
        .await?
    }

    #[cfg(test)]
    pub fn commit_count(&self) -> usize {
        self.commit_counter.load(Ordering::SeqCst)
    }
}

#[derive(Clone)]
pub struct SqliteKnowledgeStore {
    pool: sqlx::SqlitePool,
    search_engine: TantivySearchEngine,
}

impl SqliteKnowledgeStore {
    pub async fn new(base_dir: &Path) -> Result<Self, MemoryError> {
        fs::create_dir_all(base_dir).await?;
        let db_path = base_dir.join("archival.db");
        let index_dir = base_dir.join("archival-index");

        let options = SqliteConnectOptions::new()
            .filename(&db_path)
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
            CREATE TABLE IF NOT EXISTS knowledge_entries (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                category TEXT NOT NULL,
                source TEXT NOT NULL,
                tags TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                embedding TEXT,
                entities TEXT NOT NULL
            );
            "#,
        )
        .execute(&pool)
        .await?;

        let search_engine = TantivySearchEngine::new(&index_dir).await?;
        Ok(Self {
            pool,
            search_engine,
        })
    }

    pub fn search_engine(&self) -> &TantivySearchEngine {
        &self.search_engine
    }
}

#[async_trait]
impl KnowledgeStore for SqliteKnowledgeStore {
    async fn store(&self, entry: KnowledgeEntry) -> Result<String, MemoryError> {
        sqlx::query(
            r#"
            INSERT INTO knowledge_entries (
                id, title, content, category, source, tags, created_at, updated_at, embedding, entities
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                category = excluded.category,
                source = excluded.source,
                tags = excluded.tags,
                updated_at = excluded.updated_at,
                embedding = excluded.embedding,
                entities = excluded.entities
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.title)
        .bind(&entry.content)
        .bind(&entry.category)
        .bind(serde_json::to_string(&entry.source)?)
        .bind(serde_json::to_string(&entry.tags)?)
        .bind(entry.created_at.to_rfc3339())
        .bind(entry.updated_at.to_rfc3339())
        .bind(entry.embedding.as_ref().map(serde_json::to_string).transpose()?)
        .bind(serde_json::to_string(&entry.entities)?)
        .execute(&self.pool)
        .await?;

        self.search_engine.index_entry(&entry).await?;
        Ok(entry.id)
    }

    async fn retrieve(&self, id: &str) -> Result<Option<KnowledgeEntry>, MemoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, title, content, category, source, tags, created_at, updated_at, embedding, entities
            FROM knowledge_entries
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_knowledge).transpose()
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeEntry>, MemoryError> {
        let ids = self.search_engine.search_ids(query, limit).await?;
        let mut entries = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(entry) = self.retrieve(&id).await? {
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    async fn delete(&self, id: &str) -> Result<bool, MemoryError> {
        let result = sqlx::query("DELETE FROM knowledge_entries WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        let deleted = result.rows_affected() > 0;
        if deleted {
            self.search_engine.delete_entry(id).await?;
        }
        Ok(deleted)
    }
}

fn create_tantivy_state(index_dir: &Path) -> Result<TantivyState, MemoryError> {
    let mut schema_builder = SchemaBuilder::default();
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let content = schema_builder.add_text_field("content", TEXT | STORED);
    let category = schema_builder.add_text_field("category", STRING | STORED);
    let tags = schema_builder.add_text_field("tags", TEXT);
    let entry_id = schema_builder.add_text_field("entry_id", STRING | STORED);
    let schema = schema_builder.build();

    let index =
        Index::create_in_dir(index_dir, schema).or_else(|_| Index::open_in_dir(index_dir))?;
    let reader = index.reader()?;
    let writer = index.writer(50_000_000)?;

    Ok(TantivyState {
        index,
        reader,
        writer,
        fields: TantivyFields {
            title,
            content,
            category,
            tags,
            entry_id,
        },
    })
}

fn build_document(fields: &TantivyFields, entry: &KnowledgeEntry) -> TantivyDocument {
    doc!(
        fields.title => entry.title.clone(),
        fields.content => entry.content.clone(),
        fields.category => entry.category.clone(),
        fields.tags => entry.tags.join(" "),
        fields.entry_id => entry.id.clone(),
    )
}

fn row_to_knowledge(row: sqlx::sqlite::SqliteRow) -> Result<KnowledgeEntry, MemoryError> {
    let created_at = DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
        .map_err(|error| MemoryError::Command(error.to_string()))?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
        .map_err(|error| MemoryError::Command(error.to_string()))?
        .with_timezone(&Utc);

    let source: String = row.try_get("source")?;
    let tags: String = row.try_get("tags")?;
    let entities: String = row.try_get("entities")?;
    let embedding: Option<String> = row.try_get("embedding")?;

    Ok(KnowledgeEntry {
        id: row.try_get("id")?,
        title: row.try_get("title")?,
        content: row.try_get("content")?,
        category: row.try_get("category")?,
        source: serde_json::from_str(&source)?,
        tags: serde_json::from_str(&tags)?,
        created_at,
        updated_at,
        embedding: embedding.as_deref().map(serde_json::from_str).transpose()?,
        entities: serde_json::from_str(&entities)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn batch_index_commits_once() {
        let temp = TempDir::new().expect("temp dir");
        let engine = TantivySearchEngine::new(temp.path())
            .await
            .expect("search engine");
        let entries = vec![
            KnowledgeEntry {
                id: "one".into(),
                title: "One".into(),
                content: "alpha beta".into(),
                category: "note".into(),
                source: KnowledgeSource::Manual,
                tags: vec!["alpha".into()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                embedding: None,
                entities: Vec::new(),
            },
            KnowledgeEntry {
                id: "two".into(),
                title: "Two".into(),
                content: "beta gamma".into(),
                category: "note".into(),
                source: KnowledgeSource::Manual,
                tags: vec!["beta".into()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                embedding: None,
                entities: Vec::new(),
            },
        ];

        engine.index_entries(&entries).await.expect("batch index");
        assert_eq!(engine.commit_count(), 1);
    }
}
