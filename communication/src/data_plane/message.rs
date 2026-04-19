use mc_core::{AgentExecutionReport, AgentType};
use serde::{Deserialize, Serialize};

use crate::{CommunicationError, StateMessage};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataPlaneMessage {
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

impl From<DataPlaneMessage> for StateMessage {
    fn from(value: DataPlaneMessage) -> Self {
        match value {
            DataPlaneMessage::Handoff {
                task_id,
                from_agent,
                to_agent,
                handoff,
            } => StateMessage::Handoff {
                task_id,
                from_agent,
                to_agent,
                handoff,
            },
            DataPlaneMessage::PartialResult {
                task_id,
                from_agent,
                to_agent,
                payload,
            } => StateMessage::PartialResult {
                task_id,
                from_agent,
                to_agent,
                payload,
            },
            DataPlaneMessage::StreamChunk {
                task_id,
                from_agent,
                to_agent,
                sequence,
                payload,
                is_last,
            } => StateMessage::StreamChunk {
                task_id,
                from_agent,
                to_agent,
                sequence,
                payload,
                is_last,
            },
        }
    }
}

impl TryFrom<StateMessage> for DataPlaneMessage {
    type Error = CommunicationError;

    fn try_from(value: StateMessage) -> Result<Self, Self::Error> {
        match value {
            StateMessage::Handoff {
                task_id,
                from_agent,
                to_agent,
                handoff,
            } => Ok(Self::Handoff {
                task_id,
                from_agent,
                to_agent,
                handoff,
            }),
            StateMessage::PartialResult {
                task_id,
                from_agent,
                to_agent,
                payload,
            } => Ok(Self::PartialResult {
                task_id,
                from_agent,
                to_agent,
                payload,
            }),
            StateMessage::StreamChunk {
                task_id,
                from_agent,
                to_agent,
                sequence,
                payload,
                is_last,
            } => Ok(Self::StreamChunk {
                task_id,
                from_agent,
                to_agent,
                sequence,
                payload,
                is_last,
            }),
            other => Err(CommunicationError::Core(format!(
                "state message is not a data-plane message: {other:?}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::sample_report;
    use mc_core::AgentType;

    use super::DataPlaneMessage;

    #[test]
    fn data_plane_roundtrip_via_state_message() {
        let messages = vec![
            DataPlaneMessage::Handoff {
                task_id: "task-1".into(),
                from_agent: AgentType::Explorer,
                to_agent: AgentType::Coder,
                handoff: sample_report(),
            },
            DataPlaneMessage::PartialResult {
                task_id: "task-2".into(),
                from_agent: AgentType::Research,
                to_agent: AgentType::Planner,
                payload: serde_json::json!({ "match": 3 }),
            },
            DataPlaneMessage::StreamChunk {
                task_id: "task-3".into(),
                from_agent: AgentType::Coder,
                to_agent: AgentType::Reviewer,
                sequence: 7,
                payload: "chunk".into(),
                is_last: false,
            },
        ];

        for message in messages {
            let state: crate::StateMessage = message.clone().into();
            let decoded = DataPlaneMessage::try_from(state).unwrap();
            assert_eq!(decoded, message);
        }
    }
}
