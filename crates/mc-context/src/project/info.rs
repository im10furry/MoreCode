use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectInfo {
    pub name: String,
    pub root_dir: PathBuf,
    pub primary_language: Option<String>,
    pub repository_url: Option<String>,
    pub summary: Option<String>,
}
