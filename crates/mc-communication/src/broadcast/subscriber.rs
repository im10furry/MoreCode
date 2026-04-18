use crate::{BroadcastEvent, CommunicationError};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tracing::warn;

pub(crate) type SnapshotStore = Arc<RwLock<Option<BroadcastEvent>>>;

pub(crate) fn new_snapshot_store() -> SnapshotStore {
    Arc::new(RwLock::new(None))
}

#[derive(Debug)]
pub struct BroadcastSubscriber {
    receiver: broadcast::Receiver<BroadcastEvent>,
    latest_snapshot: SnapshotStore,
    subscriber_name: String,
}

impl BroadcastSubscriber {
    pub(crate) fn new(
        receiver: broadcast::Receiver<BroadcastEvent>,
        latest_snapshot: SnapshotStore,
        subscriber_name: String,
    ) -> Self {
        Self {
            receiver,
            latest_snapshot,
            subscriber_name,
        }
    }

    pub async fn recv(&mut self) -> Result<BroadcastEvent, CommunicationError> {
        loop {
            match self.receiver.recv().await {
                Ok(event) => return Ok(event),
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(CommunicationError::ChannelClosed {
                        channel: "broadcast".to_string(),
                    });
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!(
                        subscriber = %self.subscriber_name,
                        skipped,
                        "broadcast subscriber lagged; replaying latest snapshot",
                    );
                    self.receiver = self.receiver.resubscribe();

                    if let Some(snapshot) = self.latest_snapshot.read().ok().and_then(|g| g.clone())
                    {
                        return Ok(snapshot);
                    }

                    return Err(CommunicationError::BroadcastLagged { skipped });
                }
            }
        }
    }

    pub fn into_inner(self) -> broadcast::Receiver<BroadcastEvent> {
        self.receiver
    }
}
