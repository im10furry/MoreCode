use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Readme,
    Api,
    Changelog,
    Contributing,
}

impl DocumentType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Readme => "README",
            Self::Api => "API",
            Self::Changelog => "CHANGELOG",
            Self::Contributing => "CONTRIBUTING",
        }
    }

    pub fn template_name(self) -> &'static str {
        match self {
            Self::Readme => "readme",
            Self::Api => "api",
            Self::Changelog => "changelog",
            Self::Contributing => "contributing",
        }
    }

    pub fn default_file_name(self) -> &'static str {
        match self {
            Self::Readme => "README.md",
            Self::Api => "API.md",
            Self::Changelog => "CHANGELOG.md",
            Self::Contributing => "CONTRIBUTING.md",
        }
    }

    pub fn from_hint(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "readme" => Some(Self::Readme),
            "api" | "api_doc" | "api-doc" | "api_docs" => Some(Self::Api),
            "changelog" => Some(Self::Changelog),
            "contributing" | "contrib" | "contribution" => Some(Self::Contributing),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratedDocument {
    pub doc_type: DocumentType,
    pub path: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Documentation {
    pub files_created: Vec<String>,
    pub content_summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generated_documents: Vec<GeneratedDocument>,
}
