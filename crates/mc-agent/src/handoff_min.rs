use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AgentHandoff {
    pub changed_files: Vec<String>,
    pub new_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured: Option<serde_json::Value>,
}
