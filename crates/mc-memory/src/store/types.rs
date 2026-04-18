use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TechStack {
    pub language: String,
    pub edition: String,
    pub framework: BTreeMap<String, String>,
    pub dev_tools: BTreeMap<String, String>,
    pub key_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ModuleMap {
    pub modules: Vec<ModuleInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ModuleInfo {
    pub name: String,
    pub path: String,
    pub responsibility: String,
    pub key_files: Vec<String>,
    pub public_api: Vec<String>,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ApiEndpoints {
    pub endpoints: Vec<ApiEndpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ApiEndpoint {
    pub method: String,
    pub path: String,
    pub handler: String,
    pub request_type: Option<String>,
    pub response_type: Option<String>,
    pub auth_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DataModels {
    pub models: Vec<DataModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DataModel {
    pub name: String,
    pub file: String,
    pub fields: Vec<FieldInfo>,
    pub table: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FieldInfo {
    pub name: String,
    pub r#type: String,
    pub db_column: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RiskAreas {
    pub risks: Vec<RiskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RiskInfo {
    pub area: String,
    pub r#type: String,
    pub description: String,
    pub severity: String,
    pub discovered_at: DateTime<Utc>,
    pub discovered_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DependencyGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<DependencyEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetaJson {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub project_hash: String,
    pub git_branch: String,
    pub git_commit: String,
    pub total_files: usize,
    pub total_lines: usize,
    pub memory_status: String,
    pub stale_threshold_days: i64,
}

impl Default for MetaJson {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            version: "1.0".to_string(),
            created_at: now,
            last_updated: now,
            project_hash: String::new(),
            git_branch: String::new(),
            git_commit: String::new(),
            total_files: 0,
            total_lines: 0,
            memory_status: "empty".to_string(),
            stale_threshold_days: 7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProjectMemorySnapshot {
    pub meta: MetaJson,
    pub overview: String,
    pub tech_stack: TechStack,
    pub module_map: ModuleMap,
    pub api_endpoints: ApiEndpoints,
    pub data_models: DataModels,
    pub conventions: String,
    pub risk_areas: RiskAreas,
    pub dependency_graph: DependencyGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectMemory {
    Empty,
    Stale(MetaJson),
    Valid(Box<ProjectMemorySnapshot>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryUpdate {
    FileModified {
        path: String,
        summary: String,
    },
    FileAdded {
        path: String,
        module_name: Option<String>,
    },
    FileDeleted {
        path: String,
        module_name: Option<String>,
    },
    ApiAdded {
        endpoint: ApiEndpoint,
    },
    ApiRemoved {
        method: String,
        path: String,
    },
    DataModelChanged {
        model: String,
        change_type: String,
    },
    RiskDiscovered {
        area: String,
        r#type: String,
        description: String,
        severity: String,
    },
    RiskResolved {
        area: String,
    },
    AgentNote {
        agent: String,
        topic: String,
        content: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChange {
    Added { path: String },
    Modified { path: String },
    Deleted { path: String },
    Renamed { old_path: String, new_path: String },
}

#[derive(Debug)]
pub struct MemoryWriteRequest {
    pub update: MemoryUpdate,
    pub ack: oneshot::Sender<anyhow::Result<()>>,
}
