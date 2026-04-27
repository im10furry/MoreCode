use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::error::{TaskPileError, TaskPileResult};

use super::super::crypto::{decrypt, encrypt};
use super::storage::{TaskPileState, TaskPileStorage};
use super::super::types::TaskPileTask;

#[derive(Debug)]
pub struct SqliteTaskPileStore {
    root_dir: PathBuf,
    db_path: PathBuf,
    conn: Arc<Mutex<Connection>>,
}

impl SqliteTaskPileStore {
    pub fn new(root_dir: PathBuf) -> Self {
        // Create directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&root_dir) {
            panic!("Failed to create storage directory: {e}");
        }
        
        let db_path = root_dir.join("taskpile.db");
        let conn = Arc::new(Mutex::new(Connection::open(&db_path).expect("Failed to open database")));
        let store = Self {
            root_dir,
            db_path,
            conn,
        };
        store.init_db().expect("Failed to initialize database");
        store
    }

    pub fn ensure_ready(&self) -> TaskPileResult<()> {
        fs::create_dir_all(&self.root_dir).map_err(|error| TaskPileError::CreateStorageDir {
            path: self.root_dir.clone(),
            reason: error.to_string(),
        })
    }

    fn init_db(&self) -> TaskPileResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                instruction TEXT NOT NULL,
                status TEXT NOT NULL,
                priority TEXT NOT NULL,
                schedule TEXT NOT NULL,
                execution TEXT NOT NULL,
                tags TEXT NOT NULL,
                metadata TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                next_run_at TEXT,
                last_claimed_at TEXT,
                lease_expires_at TEXT,
                attempts INTEGER NOT NULL,
                max_attempts INTEGER NOT NULL,
                last_error TEXT,
                result_summary TEXT,
                origin TEXT NOT NULL
            )
            "#,
            [],
        ).map_err(|e| TaskPileError::DbError(e.to_string()))?;

        // Add indexes to improve query performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)",
            [],
        ).map_err(|e| TaskPileError::DbError(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority)",
            [],
        ).map_err(|e| TaskPileError::DbError(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_next_run_at ON tasks(next_run_at)",
            [],
        ).map_err(|e| TaskPileError::DbError(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at)",
            [],
        ).map_err(|e| TaskPileError::DbError(e.to_string()))?;

        Ok(())
    }
}

impl TaskPileStorage for SqliteTaskPileStore {
    fn load(&self) -> TaskPileResult<TaskPileState> {
        self.ensure_ready()?;
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM tasks").map_err(|e| TaskPileError::DbError(e.to_string()))?;
        let task_iter = stmt.query_map([], |row| {
            let instruction = row.get::<_, String>(2)?;
            let decrypted_instruction = match decrypt(&instruction) {
                Ok(decrypted) => decrypted,
                Err(_) => instruction, // Fallback to original if decryption fails
            };
            
            let status = match row.get::<_, String>(3)?.as_str() {
                "Queued" => crate::taskpile::types::TaskPileStatus::Queued,
                "Running" => crate::taskpile::types::TaskPileStatus::Running,
                "Paused" => crate::taskpile::types::TaskPileStatus::Paused,
                "Completed" => crate::taskpile::types::TaskPileStatus::Completed,
                "Failed" => crate::taskpile::types::TaskPileStatus::Failed,
                "Cancelled" => crate::taskpile::types::TaskPileStatus::Cancelled,
                _ => crate::taskpile::types::TaskPileStatus::Queued,
            };
            
            let priority = match row.get::<_, String>(4)?.as_str() {
                "Low" => crate::taskpile::types::TaskPilePriority::Low,
                "Normal" => crate::taskpile::types::TaskPilePriority::Normal,
                "High" => crate::taskpile::types::TaskPilePriority::High,
                "Critical" => crate::taskpile::types::TaskPilePriority::Critical,
                _ => crate::taskpile::types::TaskPilePriority::Normal,
            };
            
            let schedule = match serde_json::from_str(&row.get::<_, String>(5)?) {
                Ok(s) => s,
                Err(_) => crate::taskpile::types::TaskPileSchedule::Manual,
            };
            
            let execution = match serde_json::from_str(&row.get::<_, String>(6)?) {
                Ok(e) => e,
                Err(_) => crate::taskpile::types::ExecutionOptions {
                    target: crate::taskpile::types::TaskTarget::Local,
                    model: None,
                    parallelism: 1,
                    approval: crate::taskpile::types::ApprovalMode::Auto,
                    isolation: crate::taskpile::types::IsolationProfile::WorkspaceWrite,
                    token_controls: crate::taskpile::types::TokenControls {
                        budget: 12000,
                        compression: crate::taskpile::types::CompressionMode::Balanced,
                        summary_depth: 2,
                        allow_cache_reuse: true,
                        cache_namespace: None,
                    },
                    cloud_endpoint: None,
                    cloud_project_id: None,
                },
            };
            
            let tags: Vec<String> = serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default();
            
            let metadata: std::collections::HashMap<String, String> = serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default();
            
            let created_at = match chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?) {
                Ok(t) => t.into(),
                Err(_) => chrono::Utc::now(),
            };
            
            let updated_at = match chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?) {
                Ok(t) => t.into(),
                Err(_) => chrono::Utc::now(),
            };
            
            let next_run_at = row.get::<_, Option<String>>(11)?.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|t| t.into()));
            let last_claimed_at = row.get::<_, Option<String>>(12)?.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|t| t.into()));
            let lease_expires_at = row.get::<_, Option<String>>(13)?.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|t| t.into()));
            
            Ok(TaskPileTask {
                id: row.get(0)?,
                title: row.get(1)?,
                instruction: decrypted_instruction,
                status,
                priority,
                schedule,
                execution,
                tags,
                metadata,
                created_at,
                updated_at,
                next_run_at,
                last_claimed_at,
                lease_expires_at,
                attempts: row.get(14)?,
                max_attempts: row.get(15)?,
                last_error: row.get(16)?,
                result_summary: row.get(17)?,
                origin: row.get(18)?,
            })
        }).map_err(|e| TaskPileError::DbError(e.to_string()))?;

        let tasks: Vec<TaskPileTask> = task_iter.collect::<Result<_, _>>().map_err(|e| TaskPileError::DbError(e.to_string()))?;
        Ok(TaskPileState {
            tasks,
            updated_at: Some(Utc::now()),
        })
    }

    fn save(&self, state: TaskPileState) -> TaskPileResult<()> {
        self.ensure_ready()?;
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(|e| TaskPileError::DbError(e.to_string()))?;

        tx.execute("DELETE FROM tasks", []).map_err(|e| TaskPileError::DbError(e.to_string()))?;

        for task in state.tasks {
            let encrypted_instruction = match encrypt(&task.instruction) {
                Ok(encrypted) => encrypted,
                Err(_) => task.instruction, // Fallback to original if encryption fails
            };
            
            tx.execute(
                r#"
                INSERT INTO tasks (
                    id, title, instruction, status, priority, schedule, execution, tags, metadata, 
                    created_at, updated_at, next_run_at, last_claimed_at, lease_expires_at, 
                    attempts, max_attempts, last_error, result_summary, origin
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    task.id,
                    task.title,
                    encrypted_instruction,
                    format!("{:?}", task.status),
                    format!("{:?}", task.priority),
                    serde_json::to_string(&task.schedule).unwrap(),
                    serde_json::to_string(&task.execution).unwrap(),
                    serde_json::to_string(&task.tags).unwrap(),
                    serde_json::to_string(&task.metadata).unwrap(),
                    task.created_at.to_rfc3339(),
                    task.updated_at.to_rfc3339(),
                    task.next_run_at.map(|t| t.to_rfc3339()),
                    task.last_claimed_at.map(|t| t.to_rfc3339()),
                    task.lease_expires_at.map(|t| t.to_rfc3339()),
                    task.attempts,
                    task.max_attempts,
                    task.last_error,
                    task.result_summary,
                    task.origin
                ],
            ).map_err(|e| TaskPileError::DbError(e.to_string()))?;
        }

        tx.commit().map_err(|e| TaskPileError::DbError(e.to_string()))?;
        Ok(())
    }

    fn state_path(&self) -> &Path {
        &self.db_path
    }
}
