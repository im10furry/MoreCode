use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticCacheNamespace {
    pub name: String,
    pub model_id: String,
    pub project_id: Option<String>,
}
