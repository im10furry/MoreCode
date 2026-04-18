use mc_agent::TestReport;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseType {
    Completed,
    ClarificationNeeded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoordinatorResponse {
    pub response_type: ResponseType,
    pub content: String,
    pub changed_files: Vec<String>,
    pub review_issues: Vec<String>,
    pub test_results: Vec<TestReport>,
    pub total_tokens_used: usize,
    pub total_duration_ms: u64,
}

impl Default for CoordinatorResponse {
    fn default() -> Self {
        Self {
            response_type: ResponseType::Completed,
            content: String::new(),
            changed_files: Vec::new(),
            review_issues: Vec::new(),
            test_results: Vec::new(),
            total_tokens_used: 0,
            total_duration_ms: 0,
        }
    }
}
