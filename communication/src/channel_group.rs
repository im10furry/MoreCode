use crate::broadcast::BroadcastEvent;
use crate::channels::send_with_timeout;
use crate::{CommunicationError, StateMessage};
use mc_core::{AgentType, DATA_LINK_CHANNEL_CAPACITY};
use std::collections::{BTreeSet, HashMap};
use tokio::sync::{broadcast, mpsc};

pub struct ChannelGroup {
    pub group_id: String,
    pub members: Vec<AgentType>,
    pub(crate) data_links: HashMap<(AgentType, AgentType), mpsc::Sender<StateMessage>>,
    data_receivers: HashMap<(AgentType, AgentType), mpsc::Receiver<StateMessage>>,
    broadcast_sender: broadcast::Sender<BroadcastEvent>,
}

impl ChannelGroup {
    pub(crate) fn new(
        group_id: impl Into<String>,
        edges: &[(AgentType, AgentType)],
        broadcast_sender: broadcast::Sender<BroadcastEvent>,
    ) -> Self {
        let mut members = BTreeSet::new();
        let mut data_links = HashMap::new();
        let mut data_receivers = HashMap::new();

        for &(from, to) in edges {
            if data_links.contains_key(&(from, to)) {
                continue;
            }

            let (sender, receiver) = mpsc::channel(DATA_LINK_CHANNEL_CAPACITY);
            data_links.insert((from, to), sender);
            data_receivers.insert((from, to), receiver);
            members.insert(from);
            members.insert(to);
        }

        Self {
            group_id: group_id.into(),
            members: members.into_iter().collect(),
            data_links,
            data_receivers,
            broadcast_sender,
        }
    }

    pub fn has_link(&self, from: AgentType, to: AgentType) -> bool {
        self.data_links.contains_key(&(from, to))
    }

    pub fn take_data_receiver(
        &mut self,
        from: AgentType,
        to: AgentType,
    ) -> Option<mpsc::Receiver<StateMessage>> {
        self.data_receivers.remove(&(from, to))
    }

    pub async fn send_data(
        &self,
        from: AgentType,
        to: AgentType,
        message: StateMessage,
    ) -> Result<(), CommunicationError> {
        let sender = self.data_links.get(&(from, to)).ok_or_else(|| {
            CommunicationError::DataLinkNotFound {
                from: from.to_string(),
                to: to.to_string(),
            }
        })?;

        send_with_timeout(
            sender,
            message,
            format!("channel_group/{}/{}->{}", self.group_id, from, to),
            Some(&self.broadcast_sender),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::ChannelGroup;
    use crate::test_support::sample_report;
    use crate::StateMessage;
    use mc_core::AgentType;
    use tokio::sync::broadcast;

    #[test]
    fn creates_links_only_for_declared_edges() {
        let (broadcast_sender, _) = broadcast::channel(4);
        let group = ChannelGroup::new(
            "group-1",
            &[
                (AgentType::Explorer, AgentType::Coder),
                (AgentType::Coder, AgentType::Reviewer),
            ],
            broadcast_sender,
        );

        assert!(group.has_link(AgentType::Explorer, AgentType::Coder));
        assert!(group.has_link(AgentType::Coder, AgentType::Reviewer));
        assert!(!group.has_link(AgentType::Explorer, AgentType::Reviewer));
        assert!(!group.has_link(AgentType::Coder, AgentType::Explorer));
    }

    #[tokio::test]
    async fn direct_data_plane_send_and_receive() {
        let (broadcast_sender, _) = broadcast::channel(4);
        let mut group = ChannelGroup::new(
            "group-1",
            &[(AgentType::Explorer, AgentType::Coder)],
            broadcast_sender,
        );
        let mut receiver = group
            .take_data_receiver(AgentType::Explorer, AgentType::Coder)
            .unwrap();

        let message = StateMessage::Handoff {
            task_id: "task-1".to_string(),
            from_agent: AgentType::Explorer,
            to_agent: AgentType::Coder,
            handoff: sample_report(),
        };

        group
            .send_data(AgentType::Explorer, AgentType::Coder, message.clone())
            .await
            .unwrap();

        assert_eq!(receiver.recv().await.unwrap(), message);
    }

    #[tokio::test]
    async fn rejects_undeclared_edges() {
        let (broadcast_sender, _) = broadcast::channel(4);
        let group = ChannelGroup::new(
            "group-1",
            &[(AgentType::Explorer, AgentType::Coder)],
            broadcast_sender,
        );

        let error = group
            .send_data(
                AgentType::Explorer,
                AgentType::Reviewer,
                StateMessage::PartialResult {
                    task_id: "task-1".to_string(),
                    from_agent: AgentType::Explorer,
                    to_agent: AgentType::Reviewer,
                    payload: serde_json::json!({ "missing": true }),
                },
            )
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            crate::CommunicationError::DataLinkNotFound { .. }
        ));
    }
}
