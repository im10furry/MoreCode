use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use mc_core::ProjectContext;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanMode {
    Full,
    Incremental { changed_files: Vec<String> },
    Cached,
}

impl ScanMode {
    pub fn label(&self) -> String {
        match self {
            Self::Full => "full".to_string(),
            Self::Incremental { .. } => "incremental".to_string(),
            Self::Cached => "cached".to_string(),
        }
    }

    pub fn changed_files(&self) -> Vec<String> {
        match self {
            Self::Incremental { changed_files } => changed_files.clone(),
            Self::Full | Self::Cached => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedFileRecord {
    pub relative_path: String,
    pub hash: u64,
    pub size_bytes: u64,
    pub line_count: usize,
    pub language: String,
    pub module: String,
    pub is_entry: bool,
    pub is_config: bool,
    pub dependencies: Vec<String>,
    pub exports: Vec<String>,
    pub risk_markers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCache {
    pub version: u32,
    pub saved_at: DateTime<Utc>,
    pub root_path: String,
    pub records: BTreeMap<String, CachedFileRecord>,
    pub project_context: ProjectContext,
}
