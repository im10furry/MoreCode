use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalResponse {
    pub request_id: String,
    pub choice: String,
    pub approved: bool,
    pub comment: Option<String>,
    pub responded_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::ApprovalResponse;
    use chrono::Utc;

    #[test]
    fn approval_response_roundtrip() {
        let response = ApprovalResponse {
            request_id: "approval-1".to_string(),
            choice: "approve".to_string(),
            approved: true,
            comment: Some("Safe to proceed".to_string()),
            responded_at: Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let decoded: ApprovalResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, response);
    }
}
