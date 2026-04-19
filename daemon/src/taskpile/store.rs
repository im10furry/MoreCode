use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{TaskPileError, TaskPileResult};

use super::types::TaskPileTask;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskPileState {
    pub tasks: Vec<TaskPileTask>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct TaskPileStore {
    root_dir: PathBuf,
    state_file: PathBuf,
}

impl TaskPileStore {
    pub fn new(root_dir: PathBuf) -> Self {
        let state_file = root_dir.join("state.json");
        Self {
            root_dir,
            state_file,
        }
    }

    pub fn ensure_ready(&self) -> TaskPileResult<()> {
        fs::create_dir_all(&self.root_dir).map_err(|error| TaskPileError::CreateStorageDir {
            path: self.root_dir.clone(),
            reason: error.to_string(),
        })
    }

    pub fn load(&self) -> TaskPileResult<TaskPileState> {
        self.ensure_ready()?;
        if !self.state_file.exists() {
            return Ok(TaskPileState::default());
        }
        let raw =
            fs::read_to_string(&self.state_file).map_err(|error| TaskPileError::ReadState {
                path: self.state_file.clone(),
                reason: error.to_string(),
            })?;
        serde_json::from_str(&raw).map_err(|error| TaskPileError::ParseState {
            path: self.state_file.clone(),
            reason: error.to_string(),
        })
    }

    pub fn save(&self, mut state: TaskPileState) -> TaskPileResult<()> {
        self.ensure_ready()?;
        state.updated_at = Some(Utc::now());
        let raw =
            serde_json::to_string_pretty(&state).map_err(|error| TaskPileError::WriteState {
                path: self.state_file.clone(),
                reason: error.to_string(),
            })?;
        fs::write(&self.state_file, raw).map_err(|error| TaskPileError::WriteState {
            path: self.state_file.clone(),
            reason: error.to_string(),
        })
    }

    pub fn state_path(&self) -> &Path {
        &self.state_file
    }
}
