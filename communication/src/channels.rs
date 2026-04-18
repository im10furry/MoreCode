use crate::approval::{ApprovalRequest, ApprovalResponse};
use crate::broadcast::{new_snapshot_store, BroadcastEvent, BroadcastSubscriber, SnapshotStore};
use crate::channel_group::ChannelGroup;
use crate::{CommunicationError, ControlMessage, StateMessage};
use mc_core::{
    AgentType, APPROVAL_CHANNEL_CAPACITY, BROADCAST_CHANNEL_CAPACITY, CONTROL_CHANNEL_CAPACITY,
    QUEUE_DEPTH_ALERT_PERCENT, SEND_TIMEOUT_MS, STATE_CHANNEL_CAPACITY,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tracing::warn;

pub struct CommunicationChannels {
    control_senders: HashMap<AgentType, mpsc::Sender<ControlMessage>>,
    state_senders: HashMap<AgentType, mpsc::Sender<StateMessage>>,
    state_receivers: HashMap<AgentType, Option<mpsc::Receiver<StateMessage>>>,
    broadcast_sender: broadcast::Sender<BroadcastEvent>,
    latest_snapshot: SnapshotStore,
    approval_sender: mpsc::Sender<ApprovalRequest>,
    approval_request_receiver: Option<mpsc::Receiver<ApprovalRequest>>,
    approval_response_sender: mpsc::Sender<ApprovalResponse>,
    approval_response_receiver: Option<mpsc::Receiver<ApprovalResponse>>,
    channel_groups: HashMap<String, ChannelGroup>,
}

impl CommunicationChannels {
    pub fn new() -> Self {
        let (broadcast_sender, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        let (approval_sender, approval_request_receiver) = mpsc::channel(APPROVAL_CHANNEL_CAPACITY);
        let (approval_response_sender, approval_response_receiver) =
            mpsc::channel(APPROVAL_CHANNEL_CAPACITY);

        Self {
            control_senders: HashMap::new(),
            state_senders: HashMap::new(),
            state_receivers: HashMap::new(),
            broadcast_sender,
            latest_snapshot: new_snapshot_store(),
            approval_sender,
            approval_request_receiver: Some(approval_request_receiver),
            approval_response_sender,
            approval_response_receiver: Some(approval_response_receiver),
            channel_groups: HashMap::new(),
        }
    }

    pub fn register_agent(
        &mut self,
        agent_type: AgentType,
    ) -> (mpsc::Receiver<ControlMessage>, mpsc::Sender<StateMessage>) {
        let (control_sender, control_receiver) = mpsc::channel(CONTROL_CHANNEL_CAPACITY);
        let (state_sender, state_receiver) = mpsc::channel(STATE_CHANNEL_CAPACITY);

        self.control_senders.insert(agent_type, control_sender);
        self.state_senders.insert(agent_type, state_sender.clone());
        self.state_receivers
            .insert(agent_type, Some(state_receiver));

        (control_receiver, state_sender)
    }

    pub async fn send_control(
        &self,
        agent_type: &AgentType,
        message: ControlMessage,
    ) -> Result<(), CommunicationError> {
        let sender = self.control_senders.get(agent_type).ok_or_else(|| {
            CommunicationError::AgentNotRegistered {
                agent_type: agent_type.to_string(),
            }
        })?;

        send_with_timeout(
            sender,
            message,
            format!("control/{agent_type}"),
            Some(&self.broadcast_sender),
        )
        .await
    }

    pub fn state_sender(&self, agent_type: &AgentType) -> Option<mpsc::Sender<StateMessage>> {
        self.state_senders.get(agent_type).cloned()
    }

    pub fn take_state_receiver(
        &mut self,
        agent_type: &AgentType,
    ) -> Option<mpsc::Receiver<StateMessage>> {
        self.state_receivers
            .get_mut(agent_type)
            .and_then(Option::take)
    }

    pub fn broadcast(&self, event: BroadcastEvent) {
        if matches!(event, BroadcastEvent::ProgressSnapshot { .. }) {
            if let Ok(mut snapshot) = self.latest_snapshot.write() {
                *snapshot = Some(event.clone());
            }
        }

        let _ = self.broadcast_sender.send(event);
    }

    pub fn subscribe_broadcast(&self) -> broadcast::Receiver<BroadcastEvent> {
        self.broadcast_sender.subscribe()
    }

    pub fn subscribe_broadcast_with_recovery(
        &self,
        subscriber_name: impl Into<String>,
    ) -> BroadcastSubscriber {
        BroadcastSubscriber::new(
            self.broadcast_sender.subscribe(),
            self.latest_snapshot.clone(),
            subscriber_name.into(),
        )
    }

    pub fn create_channel_group(&mut self, group_id: &str, edges: &[(AgentType, AgentType)]) {
        let group = ChannelGroup::new(group_id, edges, self.broadcast_sender.clone());
        self.channel_groups.insert(group_id.to_string(), group);
    }

    pub fn destroy_channel_group(&mut self, group_id: &str) {
        self.channel_groups.remove(group_id);
    }

    pub fn channel_group(&self, group_id: &str) -> Option<&ChannelGroup> {
        self.channel_groups.get(group_id)
    }

    pub fn channel_group_mut(&mut self, group_id: &str) -> Option<&mut ChannelGroup> {
        self.channel_groups.get_mut(group_id)
    }

    pub async fn send_group_data(
        &self,
        group_id: &str,
        from: AgentType,
        to: AgentType,
        message: StateMessage,
    ) -> Result<(), CommunicationError> {
        let group =
            self.channel_groups
                .get(group_id)
                .ok_or_else(|| CommunicationError::GroupNotFound {
                    group_id: group_id.to_string(),
                })?;

        group.send_data(from, to, message).await
    }

    pub fn take_approval_request_receiver(&mut self) -> Option<mpsc::Receiver<ApprovalRequest>> {
        self.approval_request_receiver.take()
    }

    pub fn take_approval_response_receiver(&mut self) -> Option<mpsc::Receiver<ApprovalResponse>> {
        self.approval_response_receiver.take()
    }

    pub fn approval_sender(&self) -> &mpsc::Sender<ApprovalRequest> {
        &self.approval_sender
    }

    pub fn approval_response_sender(&self) -> &mpsc::Sender<ApprovalResponse> {
        &self.approval_response_sender
    }

    pub async fn send_approval_request(
        &self,
        request: ApprovalRequest,
    ) -> Result<(), CommunicationError> {
        send_with_timeout(
            &self.approval_sender,
            request,
            "approval/request",
            Some(&self.broadcast_sender),
        )
        .await
    }

    pub async fn send_approval_response(
        &self,
        response: ApprovalResponse,
    ) -> Result<(), CommunicationError> {
        send_with_timeout(
            &self.approval_response_sender,
            response,
            "approval/response",
            Some(&self.broadcast_sender),
        )
        .await
    }
}

impl Default for CommunicationChannels {
    fn default() -> Self {
        Self::new()
    }
}

fn queue_depth<T>(sender: &mpsc::Sender<T>) -> (usize, usize) {
    let capacity = sender.max_capacity();
    let available = sender.capacity();
    let depth = capacity.saturating_sub(available);
    (depth, capacity)
}

fn should_warn(depth: usize, capacity: usize) -> bool {
    capacity > 0 && depth.saturating_mul(100) >= capacity.saturating_mul(QUEUE_DEPTH_ALERT_PERCENT)
}

fn maybe_emit_backpressure_warning(
    channel: &str,
    depth: usize,
    capacity: usize,
    broadcast_sender: Option<&broadcast::Sender<BroadcastEvent>>,
) {
    if !should_warn(depth, capacity) {
        return;
    }

    let warning = CommunicationError::BackpressureAlert {
        channel: channel.to_string(),
        depth,
        capacity,
    };

    warn!(channel = channel, depth, capacity, "{warning}");

    if let Some(sender) = broadcast_sender {
        let _ = sender.send(BroadcastEvent::SystemNotification {
            level: "warn".to_string(),
            message: warning.to_string(),
        });
    }
}

pub(crate) async fn send_with_timeout<T>(
    sender: &mpsc::Sender<T>,
    message: T,
    channel: impl Into<String>,
    broadcast_sender: Option<&broadcast::Sender<BroadcastEvent>>,
) -> Result<(), CommunicationError>
where
    T: Send,
{
    let channel = channel.into();
    let (depth_before, capacity) = queue_depth(sender);
    let projected_depth = depth_before.saturating_add(1).min(capacity);
    maybe_emit_backpressure_warning(&channel, projected_depth, capacity, broadcast_sender);

    let timeout = Duration::from_millis(SEND_TIMEOUT_MS);
    match sender.send_timeout(message, timeout).await {
        Ok(()) => {
            let (depth_after, capacity) = queue_depth(sender);
            maybe_emit_backpressure_warning(&channel, depth_after, capacity, broadcast_sender);
            Ok(())
        }
        Err(mpsc::error::SendTimeoutError::Closed(_)) => {
            Err(CommunicationError::ChannelClosed { channel })
        }
        Err(mpsc::error::SendTimeoutError::Timeout(_)) => Err(CommunicationError::SendTimeout {
            channel,
            timeout_ms: SEND_TIMEOUT_MS,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{should_warn, CommunicationChannels};
    use crate::approval::{ApprovalRequest, ApprovalResponse};
    use crate::test_support::{sample_report, sample_task_result};
    use crate::{BroadcastEvent, CommunicationError, ControlMessage, StateMessage};
    use chrono::Utc;
    use mc_core::{AgentType, BROADCAST_CHANNEL_CAPACITY, CONTROL_CHANNEL_CAPACITY};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::{advance, timeout};

    #[test]
    fn creates_and_destroys_channels_without_panicking() {
        let mut channels = CommunicationChannels::new();
        channels.register_agent(AgentType::Explorer);
        channels.create_channel_group("group-1", &[(AgentType::Explorer, AgentType::Coder)]);
        assert!(channels.channel_group("group-1").is_some());
        channels.destroy_channel_group("group-1");
        assert!(channels.channel_group("group-1").is_none());
    }

    #[tokio::test]
    async fn register_agent_returns_live_receiver_and_sender() {
        let mut channels = CommunicationChannels::new();
        let (mut control_receiver, state_sender) = channels.register_agent(AgentType::Coder);
        let mut state_receiver = channels.take_state_receiver(&AgentType::Coder).unwrap();

        let control_message = ControlMessage::Cancel {
            task_id: "task-1".to_string(),
            reason: "stop".to_string(),
        };
        channels
            .send_control(&AgentType::Coder, control_message.clone())
            .await
            .unwrap();
        assert_eq!(control_receiver.recv().await.unwrap(), control_message);

        let state_message = StateMessage::Progress {
            task_id: "task-1".to_string(),
            agent_type: AgentType::Coder,
            phase: "coding".to_string(),
            progress_percent: 60,
            message: "Editing".to_string(),
        };
        state_sender.send(state_message.clone()).await.unwrap();
        assert_eq!(state_receiver.recv().await.unwrap(), state_message);
    }

    #[tokio::test]
    async fn send_blocks_when_control_queue_is_full() {
        let mut channels = CommunicationChannels::new();
        let (mut control_receiver, _) = channels.register_agent(AgentType::Reviewer);
        let channels = Arc::new(channels);

        for index in 0..CONTROL_CHANNEL_CAPACITY {
            channels
                .send_control(
                    &AgentType::Reviewer,
                    ControlMessage::Cancel {
                        task_id: format!("task-{index}"),
                        reason: "fill".to_string(),
                    },
                )
                .await
                .unwrap();
        }

        let channels_for_send = Arc::clone(&channels);
        let mut pending = tokio::spawn(async move {
            channels_for_send
                .send_control(
                    &AgentType::Reviewer,
                    ControlMessage::Cancel {
                        task_id: "blocked".to_string(),
                        reason: "blocked".to_string(),
                    },
                )
                .await
        });

        assert!(timeout(Duration::from_millis(100), &mut pending)
            .await
            .is_err());

        let _ = control_receiver.recv().await.unwrap();

        let pending_result = pending.await.unwrap();
        assert!(pending_result.is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn send_times_out_after_thirty_seconds() {
        let mut channels = CommunicationChannels::new();
        let (_control_receiver, _) = channels.register_agent(AgentType::Tester);

        for index in 0..CONTROL_CHANNEL_CAPACITY {
            channels
                .send_control(
                    &AgentType::Tester,
                    ControlMessage::Cancel {
                        task_id: format!("task-{index}"),
                        reason: "fill".to_string(),
                    },
                )
                .await
                .unwrap();
        }

        let handle = tokio::spawn(async move {
            channels
                .send_control(
                    &AgentType::Tester,
                    ControlMessage::Cancel {
                        task_id: "timeout".to_string(),
                        reason: "timeout".to_string(),
                    },
                )
                .await
        });

        tokio::task::yield_now().await;
        advance(Duration::from_millis(30_001)).await;

        let result = handle.await.unwrap();
        assert!(matches!(
            result,
            Err(CommunicationError::SendTimeout { .. })
        ));
    }

    #[tokio::test]
    async fn broadcast_reaches_all_receivers() {
        let channels = CommunicationChannels::new();
        let mut rx_a = channels.subscribe_broadcast();
        let mut rx_b = channels.subscribe_broadcast();
        let event = BroadcastEvent::SystemNotification {
            level: "info".to_string(),
            message: "hello".to_string(),
        };

        channels.broadcast(event.clone());

        assert_eq!(rx_a.recv().await.unwrap(), event);
        assert_eq!(rx_b.recv().await.unwrap(), event);
    }

    #[tokio::test]
    async fn lagged_broadcast_subscriber_reloads_latest_snapshot() {
        let channels = CommunicationChannels::new();
        let mut subscriber = channels.subscribe_broadcast_with_recovery("ui");
        let last_index = BROADCAST_CHANNEL_CAPACITY * 3;

        for index in 0..last_index {
            channels.broadcast(BroadcastEvent::ProgressSnapshot {
                task_id: "task-1".to_string(),
                agent_type: AgentType::Coder,
                progress_percent: (index % 100) as u8,
                summary: format!("snapshot-{index}"),
            });
        }

        let event = subscriber.recv().await.unwrap();
        assert_eq!(
            event,
            BroadcastEvent::ProgressSnapshot {
                task_id: "task-1".to_string(),
                agent_type: AgentType::Coder,
                progress_percent: ((last_index - 1) % 100) as u8,
                summary: format!("snapshot-{}", last_index - 1),
            }
        );
    }

    #[tokio::test]
    async fn approval_channels_work_end_to_end() {
        let mut channels = CommunicationChannels::new();
        let mut request_receiver = channels.take_approval_request_receiver().unwrap();
        let mut response_receiver = channels.take_approval_response_receiver().unwrap();

        let request = ApprovalRequest {
            request_id: "approval-1".to_string(),
            task_id: "task-1".to_string(),
            agent_type: "Coder".to_string(),
            reason: "Need approval".to_string(),
            options: vec!["approve".to_string(), "reject".to_string()],
            recommendation: Some("approve".to_string()),
            created_at: Utc::now(),
            timeout_secs: 30,
        };

        let response = ApprovalResponse {
            request_id: request.request_id.clone(),
            choice: "approve".to_string(),
            approved: true,
            comment: Some("Proceed".to_string()),
            responded_at: Utc::now(),
        };

        channels
            .send_approval_request(request.clone())
            .await
            .unwrap();
        assert_eq!(request_receiver.recv().await.unwrap(), request);

        channels
            .send_approval_response(response.clone())
            .await
            .unwrap();
        assert_eq!(response_receiver.recv().await.unwrap(), response);
    }

    #[tokio::test]
    async fn channel_group_send_and_receive_works() {
        let mut channels = CommunicationChannels::new();
        channels.create_channel_group(
            "group-1",
            &[(AgentType::Explorer, AgentType::ImpactAnalyzer)],
        );
        let mut receiver = channels
            .channel_group_mut("group-1")
            .unwrap()
            .take_data_receiver(AgentType::Explorer, AgentType::ImpactAnalyzer)
            .unwrap();

        let message = StateMessage::TaskCompleted {
            task_id: "task-1".to_string(),
            agent_type: AgentType::Explorer,
            result: sample_task_result(),
            handoff: sample_report(),
            token_used: 123,
        };

        channels
            .send_group_data(
                "group-1",
                AgentType::Explorer,
                AgentType::ImpactAnalyzer,
                message.clone(),
            )
            .await
            .unwrap();

        assert_eq!(receiver.recv().await.unwrap(), message);
    }

    #[test]
    fn warns_when_queue_depth_reaches_eighty_percent() {
        assert!(!should_warn(25, 32));
        assert!(should_warn(26, 32));
    }
}
