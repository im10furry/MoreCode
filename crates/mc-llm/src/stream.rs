use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{LlmError, StreamEvent, TokenUsage};

pub trait EventBus: Send + Sync {
    fn publish(&self, event: StreamEvent) -> Result<(), LlmError>;

    fn subscribe(&self) -> Result<mpsc::Receiver<StreamEvent>, LlmError>;

    fn subscriber_count(&self) -> usize;
}

#[derive(Debug)]
pub struct InMemoryEventBus {
    buffer_size: usize,
    subscribers: Mutex<Vec<mpsc::Sender<StreamEvent>>>,
}

impl InMemoryEventBus {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer_size,
            subscribers: Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new(64)
    }
}

impl EventBus for InMemoryEventBus {
    fn publish(&self, event: StreamEvent) -> Result<(), LlmError> {
        let mut subscribers = self
            .subscribers
            .lock()
            .map_err(|_| LlmError::Internal("event bus subscribers lock poisoned".into()))?;

        let mut index = 0usize;
        while index < subscribers.len() {
            match subscribers[index].try_send(event.clone()) {
                Ok(()) => {
                    index += 1;
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    subscribers.remove(index);
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    return Err(LlmError::StreamError(
                        "event bus subscriber buffer is full".into(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn subscribe(&self) -> Result<mpsc::Receiver<StreamEvent>, LlmError> {
        let (sender, receiver) = mpsc::channel(self.buffer_size);
        self.subscribers
            .lock()
            .map_err(|_| LlmError::Internal("event bus subscribers lock poisoned".into()))?
            .push(sender);
        Ok(receiver)
    }

    fn subscriber_count(&self) -> usize {
        self.subscribers
            .lock()
            .map(|guard| guard.len())
            .unwrap_or(0)
    }
}

type StreamCallback = Arc<dyn Fn(&StreamEvent) -> bool + Send + Sync>;

pub struct StreamForwarder {
    receiver: mpsc::Receiver<StreamEvent>,
    event_bus: Arc<dyn EventBus>,
    cancel_token: CancellationToken,
    finished: AtomicBool,
    callbacks: Mutex<Vec<StreamCallback>>,
}

impl StreamForwarder {
    pub fn new(receiver: mpsc::Receiver<StreamEvent>, event_bus: Arc<dyn EventBus>) -> Self {
        Self {
            receiver,
            event_bus,
            cancel_token: CancellationToken::new(),
            finished: AtomicBool::new(false),
            callbacks: Mutex::new(Vec::new()),
        }
    }

    pub fn with_cancel_token(
        receiver: mpsc::Receiver<StreamEvent>,
        event_bus: Arc<dyn EventBus>,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            receiver,
            event_bus,
            cancel_token,
            finished: AtomicBool::new(false),
            callbacks: Mutex::new(Vec::new()),
        }
    }

    pub fn on_event<F>(&mut self, callback: F)
    where
        F: Fn(&StreamEvent) -> bool + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.callbacks.lock() {
            callbacks.push(Arc::new(callback));
        }
    }

    pub async fn forward(&mut self) -> Result<(), LlmError> {
        let _ = self.forward_inner(false).await?;
        Ok(())
    }

    pub async fn into_content(mut self) -> Result<(String, TokenUsage), LlmError> {
        match self.forward_inner(true).await? {
            Some(state) => Ok((state.content, state.usage)),
            None => Ok((String::new(), TokenUsage::default())),
        }
    }

    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Relaxed)
    }

    async fn forward_inner(&mut self, aggregate: bool) -> Result<Option<AggregateState>, LlmError> {
        let mut state = AggregateState::default();

        loop {
            tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    self.finished.store(true, Ordering::Release);
                    return Err(LlmError::Cancelled {
                        reason: "stream forwarding cancelled".into(),
                    });
                }
                maybe_event = self.receiver.recv() => {
                    let Some(event) = maybe_event else {
                        self.finished.store(true, Ordering::Release);
                        return Ok(aggregate.then_some(state));
                    };

                    self.event_bus.publish(event.clone())?;

                    if aggregate {
                        state.apply(&event);
                    }

                    if !self.dispatch_callbacks(&event)? {
                        self.finished.store(true, Ordering::Release);
                        return Err(LlmError::Cancelled {
                            reason: "stream forwarding stopped by callback".into(),
                        });
                    }

                    match event {
                        StreamEvent::Finish { reason: crate::FinishReason::Cancelled, .. } => {
                            self.finished.store(true, Ordering::Release);
                            return Err(LlmError::Cancelled {
                                reason: "provider stream cancelled".into(),
                            });
                        }
                        StreamEvent::Finish { .. } => {
                            self.finished.store(true, Ordering::Release);
                            return Ok(aggregate.then_some(state));
                        }
                        StreamEvent::Error(message) => {
                            self.finished.store(true, Ordering::Release);
                            return Err(LlmError::StreamError(message));
                        }
                        StreamEvent::Delta { .. } | StreamEvent::ToolCallDelta { .. } => {}
                    }
                }
            }
        }
    }

    fn dispatch_callbacks(&self, event: &StreamEvent) -> Result<bool, LlmError> {
        let callbacks = self
            .callbacks
            .lock()
            .map_err(|_| LlmError::Internal("stream callbacks lock poisoned".into()))?
            .clone();

        for callback in callbacks {
            if !(callback)(event) {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[derive(Debug, Default)]
struct AggregateState {
    content: String,
    usage: TokenUsage,
}

impl AggregateState {
    fn apply(&mut self, event: &StreamEvent) {
        match event {
            StreamEvent::Delta { content, .. } => {
                self.content.push_str(content);
            }
            StreamEvent::Finish { usage, .. } => {
                if let Some(usage) = usage {
                    self.usage = *usage;
                }
            }
            StreamEvent::ToolCallDelta { .. } | StreamEvent::Error(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::mpsc;

    use super::{EventBus, InMemoryEventBus, StreamForwarder};
    use crate::{FinishReason, StreamEvent, TokenUsage};

    #[tokio::test]
    async fn stream_forwarder_aggregates_content() {
        let (tx, rx) = mpsc::channel(8);
        let bus = Arc::new(InMemoryEventBus::default());

        let send_result = tx
            .send(StreamEvent::Delta {
                content: "hello".into(),
                cumulative_tokens: Some(1),
            })
            .await;
        assert!(send_result.is_ok(), "delta should send successfully");

        let finish_result = tx
            .send(StreamEvent::Finish {
                reason: FinishReason::Stop,
                usage: Some(TokenUsage {
                    prompt_tokens: 1,
                    completion_tokens: 1,
                    cached_tokens: 0,
                    total_tokens: 2,
                }),
                response_id: "resp-1".into(),
            })
            .await;
        assert!(finish_result.is_ok(), "finish should send successfully");

        drop(tx);

        let result = StreamForwarder::new(rx, bus).into_content().await;
        assert!(result.is_ok(), "stream should aggregate without errors");
        let (content, usage) = result.unwrap_or_default();
        assert_eq!(content, "hello");
        assert_eq!(usage.total_tokens, 2);
    }

    #[tokio::test]
    async fn event_bus_subscription_receives_events() {
        let bus = InMemoryEventBus::default();
        let mut rx = bus
            .subscribe()
            .unwrap_or_else(|_| panic!("subscribe should succeed"));
        let publish = bus.publish(StreamEvent::Error("boom".into()));
        assert!(publish.is_ok(), "publish should succeed");
        let event = rx.recv().await;
        assert!(matches!(event, Some(StreamEvent::Error(message)) if message == "boom"));
    }
}
