use mc_core::AgentType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BroadcastEvent {
    ProgressSnapshot {
        task_id: String,
        agent_type: AgentType,
        progress_percent: u8,
        summary: String,
    },
    SystemNotification {
        level: String,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::BroadcastEvent;
    use mc_core::AgentType;

    #[test]
    fn broadcast_event_roundtrip() {
        let events = vec![
            BroadcastEvent::ProgressSnapshot {
                task_id: "task-1".to_string(),
                agent_type: AgentType::Coder,
                progress_percent: 75,
                summary: "Waiting for review".to_string(),
            },
            BroadcastEvent::SystemNotification {
                level: "warn".to_string(),
                message: "Queue depth high".to_string(),
            },
        ];

        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            let decoded: BroadcastEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, event);
        }
    }
}
