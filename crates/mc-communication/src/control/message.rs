use mc_core::{AgentExecutionReport, AgentType, SubTask};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlMessage {
    TaskAssigned {
        task_id: String,
        agent_type: AgentType,
        task: SubTask,
        context: AgentExecutionReport,
        token_budget: u64,
    },
    Cancel {
        task_id: String,
        reason: String,
    },
    ApprovalRequired {
        task_id: String,
        agent_type: AgentType,
        reason: String,
        options: Vec<String>,
        recommendation: Option<String>,
    },
    CollaborationRequest {
        from_agent: AgentType,
        to_agent: AgentType,
        request_type: String,
        payload: serde_json::Value,
    },
}

#[cfg(test)]
mod tests {
    use super::ControlMessage;
    use crate::test_support::{sample_report, sample_subtask};
    use mc_core::AgentType;

    #[test]
    fn control_message_roundtrip() {
        let messages = vec![
            ControlMessage::TaskAssigned {
                task_id: "task-1".to_string(),
                agent_type: AgentType::Coder,
                task: sample_subtask(AgentType::Coder),
                context: sample_report(),
                token_budget: 4_096,
            },
            ControlMessage::Cancel {
                task_id: "task-2".to_string(),
                reason: "Superseded".to_string(),
            },
            ControlMessage::ApprovalRequired {
                task_id: "task-3".to_string(),
                agent_type: AgentType::Reviewer,
                reason: "Needs destructive write".to_string(),
                options: vec!["approve".to_string(), "reject".to_string()],
                recommendation: Some("approve".to_string()),
            },
            ControlMessage::CollaborationRequest {
                from_agent: AgentType::Explorer,
                to_agent: AgentType::Coder,
                request_type: "analyze-module".to_string(),
                payload: serde_json::json!({ "module": "auth" }),
            },
        ];

        for message in messages {
            let json = serde_json::to_string(&message).unwrap();
            let decoded: ControlMessage = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, message);
        }
    }
}
