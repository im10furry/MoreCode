use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;

use crate::error::MemoryError;
use crate::store::{MemoryManager as StoreMemoryManager, MemoryUpdate, MemoryWriteRequest};

pub struct MemoryWriteQueue {
    sender: mpsc::Sender<MemoryWriteRequest>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl MemoryWriteQueue {
    pub fn new(
        sender: mpsc::Sender<MemoryWriteRequest>,
        receiver: mpsc::Receiver<MemoryWriteRequest>,
        manager: Arc<StoreMemoryManager>,
    ) -> Self {
        let worker = tokio::spawn(async move {
            manager.run_write_loop(receiver).await;
        });

        Self {
            sender,
            worker: Mutex::new(Some(worker)),
        }
    }

    pub fn sender(&self) -> mpsc::Sender<MemoryWriteRequest> {
        self.sender.clone()
    }

    pub async fn submit(&self, update: MemoryUpdate) -> Result<(), MemoryError> {
        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(MemoryWriteRequest {
                update,
                ack: ack_tx,
            })
            .await
            .map_err(|_| MemoryError::Internal("memory write queue is closed".into()))?;

        ack_rx
            .await
            .map_err(|_| MemoryError::Internal("memory write queue worker dropped".into()))?
            .map_err(|error| MemoryError::Internal(error.to_string()))
    }

    pub async fn shutdown(&self) {
        if let Some(worker) = self.worker.lock().await.take() {
            worker.abort();
        }
    }
}

impl Drop for MemoryWriteQueue {
    fn drop(&mut self) {
        if let Ok(mut worker) = self.worker.try_lock() {
            if let Some(handle) = worker.take() {
                handle.abort();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tempfile::TempDir;
    use tokio::fs;

    use crate::store::{MemoryManager as StoreMemoryManager, MemoryUpdate};

    use super::MemoryWriteQueue;

    #[tokio::test]
    async fn queued_updates_are_persisted() {
        let temp = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp.path().join("src")).await.unwrap();
        fs::write(temp.path().join("src/lib.rs"), "pub fn demo() {}\n")
            .await
            .unwrap();

        let (manager, receiver) = StoreMemoryManager::with_channel(temp.path(), 8);
        let manager = Arc::new(manager);
        let queue = MemoryWriteQueue::new(manager.write_sender(), receiver, Arc::clone(&manager));

        queue
            .submit(MemoryUpdate::RiskDiscovered {
                area: "src/lib.rs:1".into(),
                r#type: "consistency".into(),
                description: "test".into(),
                severity: "medium".into(),
            })
            .await
            .unwrap();

        let risks_path = temp
            .path()
            .join(".assistant-memory")
            .join("risk-areas.json");
        let contents = fs::read_to_string(risks_path).await.unwrap();
        assert!(contents.contains("src/lib.rs:1"));

        queue.shutdown().await;
    }
}
