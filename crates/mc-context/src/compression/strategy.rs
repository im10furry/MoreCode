use async_trait::async_trait;
use tracing::warn;

use crate::compression::{
    AutoCompactor, ChatMessage, CompactStats, LlmError, MemoryCompactor, MicroCompactor,
    ReactiveTruncator, TokenCounter,
};

#[derive(Debug, Clone, Copy)]
pub struct L0RetentionPolicy {
    pub keep_recent_rounds: usize,
}

impl Default for L0RetentionPolicy {
    fn default() -> Self {
        Self {
            keep_recent_rounds: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub messages: Vec<ChatMessage>,
    pub stats: CompactStats,
    pub level: u8,
}

#[async_trait]
pub trait ContextCompressor: Send + Sync {
    async fn compress(
        &self,
        messages: &[ChatMessage],
        current_turn: usize,
        token_counter: &dyn TokenCounter,
        max_tokens: usize,
    ) -> anyhow::Result<CompressionResult>;
}

#[derive(Debug, Clone, Default)]
pub struct CompressionCoordinator {
    pub l0: L0RetentionPolicy,
    pub l1: MicroCompactor,
    pub l2: AutoCompactor,
    pub l3: MemoryCompactor,
    pub l4: ReactiveTruncator,
}

impl CompressionCoordinator {
    pub fn with_l2(mut self, l2: AutoCompactor) -> Self {
        self.l2 = l2;
        self
    }

    pub fn with_l3(mut self, l3: MemoryCompactor) -> Self {
        self.l3 = l3;
        self
    }

    pub fn handle_context_error(
        &self,
        messages: &[ChatMessage],
        error: &LlmError,
    ) -> Option<CompressionResult> {
        if !ReactiveTruncator::is_context_length_error(error) {
            return None;
        }

        let (messages, dropped) = self.l4.truncate(messages);
        let mut stats = CompactStats::default();
        stats.dropped_messages = dropped;

        Some(CompressionResult {
            messages,
            stats,
            level: 4,
        })
    }
}

#[async_trait]
impl ContextCompressor for CompressionCoordinator {
    async fn compress(
        &self,
        messages: &[ChatMessage],
        current_turn: usize,
        token_counter: &dyn TokenCounter,
        max_tokens: usize,
    ) -> anyhow::Result<CompressionResult> {
        let (l1_messages, mut stats) = self.l1.compact(messages, current_turn).await?;
        let mut level = if stats.small_cleared > 0 || stats.large_cleared > 0 {
            1
        } else {
            0
        };
        let mut final_messages = l1_messages;

        match self
            .l2
            .maybe_compact(&final_messages, token_counter, max_tokens)
            .await
        {
            Ok(Some(outcome)) => {
                level = 2;
                final_messages = outcome.messages;

                if outcome.usage_before >= self.l3.trigger_threshold() {
                    match self
                        .l3
                        .extract_and_persist(&outcome.summary, "session")
                        .await
                    {
                        Ok(memory_result) => {
                            stats.memory_writes += memory_result.code_changes
                                + memory_result.bug_records
                                + memory_result.conventions;
                            if stats.memory_writes > 0 {
                                level = 3;
                            }
                        }
                        Err(error) => {
                            warn!(error = %error, "L3 Memory-Compact failed; keeping L2 result");
                        }
                    }
                }
            }
            Ok(None) => {}
            Err(error) => {
                warn!(error = %error, "L2 Auto-Compact failed; falling back to rule-based result");
            }
        }

        Ok(CompressionResult {
            messages: final_messages,
            stats,
            level,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::{CompressionCoordinator, ContextCompressor};
    use crate::compression::{
        ChatMessage, CompactSummary, LlmClient, LlmError, LlmRequest, LlmResponse, MessageRole,
        SimpleTokenCounter,
    };

    #[derive(Debug)]
    struct MockLlm;

    #[async_trait]
    impl LlmClient for MockLlm {
        async fn complete(
            &self,
            _request: LlmRequest,
        ) -> std::result::Result<LlmResponse, LlmError> {
            Ok(LlmResponse {
                content: serde_json::to_string(&CompactSummary {
                    user_request: "Implement context".into(),
                    completed_work: vec!["Updated src/lib.rs".into()],
                    current_state: "done".into(),
                    key_decisions: vec!["Use rules first".into()],
                    error_info: vec![],
                    continuation_context: "run tests".into(),
                })
                .unwrap(),
            })
        }
    }

    #[tokio::test]
    async fn coordinator_runs_l1_then_l2() {
        let coordinator = CompressionCoordinator::default().with_l2(
            crate::compression::AutoCompactor::default().with_llm_client(Arc::new(MockLlm)),
        );
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: "system".into(),
                turn: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::Tool,
                content: "x".repeat(12_000),
                turn: Some(1),
                tool_call_id: Some("call".into()),
            },
            ChatMessage {
                role: MessageRole::User,
                content: "x".repeat(4_000),
                turn: Some(2),
                tool_call_id: None,
            },
        ];

        let result = coordinator
            .compress(&messages, 8, &SimpleTokenCounter, 1_000)
            .await
            .unwrap();

        assert!(result.level >= 1);
        assert!(result
            .messages
            .iter()
            .any(|message| message.content.contains("[Context Compacted]")));
    }

    #[test]
    fn coordinator_handles_context_length_errors_via_l4() {
        let coordinator = CompressionCoordinator::default();
        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "x".repeat(100),
                turn: Some(1),
                tool_call_id: None,
            };
            8
        ];

        let result = coordinator.handle_context_error(
            &messages,
            &LlmError::ApiError {
                message: "maximum context length".into(),
                status_code: 400,
            },
        );

        assert!(result.is_some());
        assert_eq!(result.unwrap().level, 4);
    }
}
