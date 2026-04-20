use std::collections::HashMap;
use std::time::{Duration, Instant};

use mc_core::AgentType;

use crate::constants::DEFAULT_PROGRESS_THROTTLE_MS;
use crate::StateMessage;

#[derive(Debug, Clone, PartialEq)]
pub enum ThrottleOutcome {
    Emit(StateMessage),
    Deferred,
}

#[derive(Debug, Clone)]
struct PendingProgress {
    message: StateMessage,
    last_emitted_at: Instant,
}

#[derive(Debug, Clone)]
pub struct StateThrottler {
    interval: Duration,
    pending_progress: HashMap<(String, AgentType), PendingProgress>,
}

impl StateThrottler {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            pending_progress: HashMap::new(),
        }
    }

    pub fn default_interval() -> Duration {
        Duration::from_millis(DEFAULT_PROGRESS_THROTTLE_MS)
    }

    pub fn push(&mut self, message: StateMessage) -> ThrottleOutcome {
        self.push_at(message, Instant::now())
    }

    pub fn push_at(&mut self, message: StateMessage, now: Instant) -> ThrottleOutcome {
        match key_for_progress(&message) {
            Some(key) => {
                let entry = self
                    .pending_progress
                    .entry(key)
                    .or_insert_with(|| PendingProgress {
                        message: message.clone(),
                        last_emitted_at: now.checked_sub(self.interval).unwrap_or(now),
                    });

                if now.duration_since(entry.last_emitted_at) >= self.interval {
                    entry.last_emitted_at = now;
                    entry.message = message.clone();
                    ThrottleOutcome::Emit(message)
                } else {
                    entry.message = message;
                    ThrottleOutcome::Deferred
                }
            }
            None => {
                clear_related_progress(&mut self.pending_progress, &message);
                ThrottleOutcome::Emit(message)
            }
        }
    }

    pub fn flush_ready(&mut self) -> Vec<StateMessage> {
        self.flush_ready_at(Instant::now())
    }

    pub fn flush_ready_at(&mut self, now: Instant) -> Vec<StateMessage> {
        let mut ready = Vec::new();
        for pending in self.pending_progress.values_mut() {
            if now.duration_since(pending.last_emitted_at) >= self.interval {
                pending.last_emitted_at = now;
                ready.push(pending.message.clone());
            }
        }
        ready
    }
}

impl Default for StateThrottler {
    fn default() -> Self {
        Self::new(Self::default_interval())
    }
}

fn key_for_progress(message: &StateMessage) -> Option<(String, AgentType)> {
    match message {
        StateMessage::Progress {
            task_id,
            agent_type,
            ..
        } => Some((task_id.clone(), *agent_type)),
        _ => None,
    }
}

fn clear_related_progress(
    pending_progress: &mut HashMap<(String, AgentType), PendingProgress>,
    message: &StateMessage,
) {
    match message {
        StateMessage::TaskCompleted {
            task_id,
            agent_type,
            ..
        }
        | StateMessage::TaskFailed {
            task_id,
            agent_type,
            ..
        } => {
            pending_progress.remove(&(task_id.clone(), *agent_type));
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use mc_core::AgentType;

    use crate::test_support::{sample_report, sample_task_result};
    use crate::StateMessage;

    use super::{StateThrottler, ThrottleOutcome};

    fn progress(percent: u8) -> StateMessage {
        StateMessage::Progress {
            task_id: "task-1".into(),
            agent_type: AgentType::Coder,
            phase: "coding".into(),
            progress_percent: percent,
            message: format!("progress-{percent}"),
        }
    }

    #[test]
    fn first_progress_is_emitted_immediately() {
        let mut throttler = StateThrottler::new(Duration::from_millis(200));
        let now = Instant::now();

        let outcome = throttler.push_at(progress(10), now);
        assert!(matches!(outcome, ThrottleOutcome::Emit(_)));
    }

    #[test]
    fn rapid_progress_updates_are_deferred_and_flushed_latest() {
        let mut throttler = StateThrottler::new(Duration::from_millis(200));
        let now = Instant::now();

        assert!(matches!(
            throttler.push_at(progress(10), now),
            ThrottleOutcome::Emit(_)
        ));
        assert!(matches!(
            throttler.push_at(progress(20), now + Duration::from_millis(50)),
            ThrottleOutcome::Deferred
        ));
        assert!(matches!(
            throttler.push_at(progress(30), now + Duration::from_millis(100)),
            ThrottleOutcome::Deferred
        ));

        let flushed = throttler.flush_ready_at(now + Duration::from_millis(250));
        assert_eq!(flushed.len(), 1);
        assert!(matches!(
            &flushed[0],
            StateMessage::Progress {
                progress_percent: 30,
                ..
            }
        ));
    }

    #[test]
    fn terminal_messages_bypass_throttle_and_clear_pending() {
        let mut throttler = StateThrottler::new(Duration::from_millis(200));
        let now = Instant::now();

        let _ = throttler.push_at(progress(10), now);
        let _ = throttler.push_at(progress(20), now + Duration::from_millis(50));

        let completed = StateMessage::TaskCompleted {
            task_id: "task-1".into(),
            agent_type: AgentType::Coder,
            result: sample_task_result(),
            handoff: sample_report(),
            token_used: 100,
        };
        let outcome = throttler.push_at(completed.clone(), now + Duration::from_millis(60));
        assert_eq!(outcome, ThrottleOutcome::Emit(completed));

        let flushed = throttler.flush_ready_at(now + Duration::from_millis(300));
        assert!(flushed.is_empty());
    }
}
