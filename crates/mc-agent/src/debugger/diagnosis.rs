use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    SyntaxError,
    TypeError,
    LogicError,
    RuntimeError,
    ConfigurationError,
    DependencyError,
    ConcurrencyError,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuggestedChange {
    pub file: String,
    pub change_type: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_diff: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixSuggestion {
    pub description: String,
    pub changes: Vec<SuggestedChange>,
    pub steps: Vec<String>,
    pub prevention: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixReport {
    pub root_cause: String,
    pub error_type: ErrorType,
    pub affected_files: Vec<String>,
    pub fix_suggestion: FixSuggestion,
    pub verified: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackFrame {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    pub raw: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ParsedStackTrace {
    pub frames: Vec<StackFrame>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub causes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogPattern {
    pub pattern: String,
    pub severity: String,
    pub frequency: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorAnalysis {
    pub root_cause: String,
    pub error_type: ErrorType,
    pub affected_files: Vec<String>,
    pub confidence: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}
