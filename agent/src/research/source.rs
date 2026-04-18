use serde::{Deserialize, Serialize};

/// Structured research result returned by the Research agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchReport {
    pub topic: String,
    pub findings: Vec<ResearchFinding>,
    pub recommendations: Vec<String>,
    pub sources: Vec<ResearchSource>,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comparisons: Vec<TechnologyComparison>,
}

/// Individual finding extracted from the research process.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchFinding {
    pub topic: String,
    pub description: String,
    pub confidence: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_titles: Vec<String>,
}

/// Source record used for citation and traceability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchSource {
    pub title: String,
    pub url: String,
    pub relevance: f64,
    pub summary: String,
    pub kind: ResearchSourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchSourceKind {
    Search,
    Web,
    ApiDoc,
    Recursive,
}

/// Lightweight comparison matrix for technology selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechnologyComparison {
    pub candidate: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub strengths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub weaknesses: Vec<String>,
    pub recommendation: String,
}
