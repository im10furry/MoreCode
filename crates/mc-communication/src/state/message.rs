use mc_core::{AgentExecutionReport, AgentType, TaskResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StateMessage {
    Progress {
        task_id: String,
        agent_type: AgentType,
        phase: String,
        progress_percent: u8,
        message: String,
    },
    TaskCompleted {
        task_id: String,
        agent_type: AgentType,
        result: TaskResult,
        handoff: AgentExecutionReport,
        token_used: u64,
    },
    TaskFailed {
        task_id: String,
        agent_type: AgentType,
        error: String,
        retry_count: u8,
        can_retry: bool,
    },
    Handoff {
        task_id: String,
        from_agent: AgentType,
        to_agent: AgentType,
        handoff: AgentExecutionReport,
    },
    PartialResult {
        task_id: String,
        from_agent: AgentType,
        to_agent: AgentType,
        payload: serde_json::Value,
    },
    StreamChunk {
        task_id: String,
        from_agent: AgentType,
        to_agent: AgentType,
        sequence: u32,
        payload: String,
        is_last: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::StateMessage;
    use crate::test_support::{sample_report, sample_task_result};
    use mc_core::AgentType;

    #[test]
    fn state_message_roundtrip() {
        let messages = vec![
            StateMessage::Progress {
                task_id: "task-1".to_string(),
                agent_type: AgentType::Coder,
                phase: "editing".to_string(),
                progress_percent: 50,
                message: "Editing files".to_string(),
            },
            StateMessage::TaskCompleted {
                task_id: "task-2".to_string(),
                agent_type: AgentType::Reviewer,
                result: sample_task_result(),
                handoff: sample_report(),
                token_used: 2_048,
            },
            StateMessage::TaskFailed {
                task_id: "task-3".to_string(),
                agent_type: AgentType::Tester,
                error: "Unit test failed".to_string(),
                retry_count: 2,
                can_retry: true,
            },
            StateMessage::Handoff {
                task_id: "task-4".to_string(),
                from_agent: AgentType::Explorer,
                to_agent: AgentType::ImpactAnalyzer,
                handoff: sample_report(),
            },
            StateMessage::PartialResult {
                task_id: "task-5".to_string(),
                from_agent: AgentType::Research,
                to_agent: AgentType::Planner,
                payload: serde_json::json!({ "match": 3 }),
            },
            StateMessage::StreamChunk {
                task_id: "task-6".to_string(),
                from_agent: AgentType::Coder,
                to_agent: AgentType::Reviewer,
                sequence: 7,
                payload: "chunk".to_string(),
                is_last: false,
            },
        ];

        for message in messages {
            let json = serde_json::to_string(&message).unwrap();
            let decoded: StateMessage = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, message);
        }
    }
}
