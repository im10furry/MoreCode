use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::layer::PromptLayer;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub layer: PromptLayer,
    pub file_path: String,
    pub raw_content: String,
    pub variables: Vec<TemplateVariable>,
    pub version: u64,
    pub content_hash: u64,
    pub updated_at: DateTime<Utc>,
}

impl PromptTemplate {
    pub fn new(
        id: impl Into<String>,
        layer: PromptLayer,
        file_path: impl Into<String>,
        raw_content: impl Into<String>,
        variables: Vec<TemplateVariable>,
    ) -> Self {
        let raw_content = raw_content.into();
        Self {
            id: id.into(),
            layer,
            file_path: file_path.into(),
            version: 1,
            content_hash: seahash::hash(raw_content.as_bytes()),
            updated_at: Utc::now(),
            raw_content,
            variables,
        }
    }
}
