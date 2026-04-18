use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub request_id: String,
    pub task_id: String,
    pub agent_type: String,
    pub reason: String,
    pub options: Vec<String>,
    pub recommendation: Option<String>,
    pub created_at: DateTime<Utc>,
    pub timeout_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::ApprovalRequest;
    use chrono::Utc;

    #[test]
    fn approval_request_roundtrip() {
        let request = ApprovalRequest {
            request_id: "approval-1".to_string(),
            task_id: "task-1".to_string(),
            agent_type: "Coder".to_string(),
            reason: "Needs user confirmation".to_string(),
            options: vec!["approve".to_string(), "reject".to_string()],
            recommendation: Some("approve".to_string()),
            created_at: Utc::now(),
            timeout_secs: 30,
        };

        let json = serde_json::to_string(&request).unwrap();
        let decoded: ApprovalRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, request);
    }
}
