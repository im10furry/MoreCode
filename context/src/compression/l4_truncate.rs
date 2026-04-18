use metrics::counter;
use tracing::warn;

use crate::compression::{ChatMessage, LlmError, MessageRole};

#[derive(Debug, Clone)]
pub struct ReactiveTruncator {
    retention_ratio: f64,
    keep_system_prompt: bool,
    min_messages: usize,
}

impl Default for ReactiveTruncator {
    fn default() -> Self {
        Self {
            retention_ratio: 0.80,
            keep_system_prompt: true,
            min_messages: 4,
        }
    }
}

impl ReactiveTruncator {
    pub fn truncate(&self, messages: &[ChatMessage]) -> (Vec<ChatMessage>, usize) {
        if messages.len() <= self.min_messages {
            return (messages.to_vec(), 0);
        }

        let mut result = Vec::new();
        if self.keep_system_prompt {
            if let Some(system) = messages
                .first()
                .filter(|message| message.role == MessageRole::System)
            {
                result.push(system.clone());
            }
        }

        let non_system = messages
            .iter()
            .filter(|message| message.role != MessageRole::System)
            .cloned()
            .collect::<Vec<_>>();
        let keep_count = ((non_system.len() as f64) * self.retention_ratio).ceil() as usize;
        let keep_count = keep_count.max(self.min_messages);
        let start = non_system.len().saturating_sub(keep_count);
        let dropped = start;
        result.extend(non_system[start..].iter().cloned());

        if dropped > 0 {
            counter!("context.compress.count", "strategy" => "L4").increment(1);
            warn!(
                total = messages.len(),
                kept = result.len(),
                dropped = dropped,
                "L4 Reactive-Truncate executed"
            );
        }

        (result, dropped)
    }

    pub fn is_context_length_error(error: &LlmError) -> bool {
        match error {
            LlmError::ApiError { message, .. } => {
                let lower = message.to_ascii_lowercase();
                lower.contains("prompt_too_long")
                    || lower.contains("maximum context length")
                    || lower.contains("context_length_exceeded")
                    || lower.contains("token limit")
            }
            LlmError::ContextLengthExceeded => true,
            LlmError::Timeout => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReactiveTruncator;
    use crate::compression::{ChatMessage, LlmError, MessageRole};

    #[test]
    fn truncate_keeps_system_prompt_and_recent_messages() {
        let truncator = ReactiveTruncator::default();
        let mut messages = vec![ChatMessage {
            role: MessageRole::System,
            content: "system".into(),
            turn: None,
            tool_call_id: None,
        }];
        messages.extend((0..10).map(|index| ChatMessage {
            role: MessageRole::User,
            content: format!("m{index}"),
            turn: Some(index),
            tool_call_id: None,
        }));

        let (truncated, dropped) = truncator.truncate(&messages);
        assert_eq!(truncated[0].role, MessageRole::System);
        assert!(dropped > 0);
        assert!(truncated.iter().any(|message| message.content == "m9"));
    }

    #[test]
    fn context_length_error_detection_matches_common_messages() {
        assert!(ReactiveTruncator::is_context_length_error(
            &LlmError::ApiError {
                message: "prompt_too_long".into(),
                status_code: 400,
            }
        ));
        assert!(ReactiveTruncator::is_context_length_error(
            &LlmError::ContextLengthExceeded
        ));
        assert!(!ReactiveTruncator::is_context_length_error(
            &LlmError::Timeout
        ));
    }
}
