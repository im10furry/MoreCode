use std::path::Path;

use chrono::DateTime;
use crate::error::{TaskPileError, TaskPileResult};

use super::super::types::TaskPileTask;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct TaskPileState {
    pub tasks: Vec<TaskPileTask>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
}

pub trait TaskPileStorage {
    fn load(&self) -> TaskPileResult<TaskPileState>;
    fn save(&self, state: TaskPileState) -> TaskPileResult<()>;
    fn state_path(&self) -> &Path;
}
