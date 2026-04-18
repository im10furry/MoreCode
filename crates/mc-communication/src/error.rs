use mc_core::McError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommunicationError {
    #[error("agent not registered: {agent_type}")]
    AgentNotRegistered { agent_type: String },
    #[error("channel closed: {channel}")]
    ChannelClosed { channel: String },
    #[error("send timeout: {channel} ({timeout_ms}ms)")]
    SendTimeout { channel: String, timeout_ms: u64 },
    #[error("data link not found: {from} -> {to}")]
    DataLinkNotFound { from: String, to: String },
    #[error("channel group not found: {group_id}")]
    GroupNotFound { group_id: String },
    #[error("broadcast subscriber lagged and lost {skipped} messages")]
    BroadcastLagged { skipped: u64 },
    #[error("backpressure alert: {channel} queue depth {depth}/{capacity}")]
    BackpressureAlert {
        channel: String,
        depth: usize,
        capacity: usize,
    },
    #[error("mc-core error: {0}")]
    Core(String),
}

impl From<McError> for CommunicationError {
    fn from(value: McError) -> Self {
        Self::Core(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::CommunicationError;
    use mc_core::McError;

    #[test]
    fn displays_human_readable_errors() {
        let error = CommunicationError::BackpressureAlert {
            channel: "control/Coder".to_string(),
            depth: 26,
            capacity: 32,
        };
        assert!(error.to_string().contains("26/32"));
    }

    #[test]
    fn converts_mc_core_errors() {
        let error = CommunicationError::from(McError::ChannelClosed {
            channel: "broken".to_string(),
        });
        assert!(error.to_string().contains("broken"));
    }
}
