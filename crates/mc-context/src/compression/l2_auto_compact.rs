use std::sync::Arc;

use anyhow::Result;
use metrics::{counter, gauge};
use seahash::hash;
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    compression::{
        ChatMessage, CompactSummary, CompressionPromptBuilder, LlmClient, MessageRole, TokenCounter,
    },
    error::ContextError,
};

#[derive(Debug, Clone)]
pub struct AutoCompactOutcome {
    pub messages: Vec<ChatMessage>,
    pub summary: CompactSummary,
    pub usage_before: f64,
}

#[derive(Default, Debug)]
struct TokenUsageCache {
    entries: Vec<CachedTokenEntry>,
    cached_total_tokens: usize,
}

#[derive(Clone, Debug)]
struct CachedTokenEntry {
    fingerprint: u64,
    tokens: usize,
}

#[derive(Clone)]
pub struct AutoCompactor {
    trigger_threshold: f64,
    target_usage: f64,
    llm_client: Option<Arc<dyn LlmClient>>,
    prompt_builder: CompressionPromptBuilder,
    max_context_tokens: usize,
    usage_cache: Arc<Mutex<TokenUsageCache>>,
}

impl std::fmt::Debug for AutoCompactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoCompactor")
            .field("trigger_threshold", &self.trigger_threshold)
            .field("target_usage", &self.target_usage)
            .field("max_context_tokens", &self.max_context_tokens)
            .finish()
    }
}

impl Default for AutoCompactor {
    fn default() -> Self {
        Self {
            trigger_threshold: 0.85,
            target_usage: 0.50,
            llm_client: None,
            prompt_builder: CompressionPromptBuilder::default(),
            max_context_tokens: 128_000,
            usage_cache: Arc::new(Mutex::new(TokenUsageCache::default())),
        }
    }
}

impl AutoCompactor {
    pub fn with_llm_client(mut self, llm_client: Arc<dyn LlmClient>) -> Self {
        self.llm_client = Some(llm_client);
        self
    }

    pub fn with_prompt_builder(mut self, prompt_builder: CompressionPromptBuilder) -> Self {
        self.prompt_builder = prompt_builder;
        self
    }

    pub fn with_max_context_tokens(mut self, max_context_tokens: usize) -> Self {
        self.max_context_tokens = max_context_tokens;
        self
    }

    pub fn trigger_threshold(&self) -> f64 {
        self.trigger_threshold
    }

    pub async fn maybe_compact(
        &self,
        messages: &[ChatMessage],
        token_counter: &dyn TokenCounter,
        max_tokens: usize,
    ) -> Result<Option<AutoCompactOutcome>> {
        let usage = self
            .calculate_usage(messages, token_counter, max_tokens)
            .await;
        gauge!("context.window.usage").set(usage);

        if usage <= self.trigger_threshold {
            return Ok(None);
        }

        let llm_client = self
            .llm_client
            .as_ref()
            .ok_or(ContextError::MissingLlmClient)?;
        let prepared = self.trim_for_llm_input(messages, token_counter);
        let request = self.prompt_builder.build_request(&prepared);
        let response = llm_client
            .complete(request)
            .await
            .map_err(anyhow::Error::new)?;
        let summary: CompactSummary = serde_json::from_str(&response.content)?;
        let rebuilt = self.rebuild_messages(messages, summary.clone());

        counter!("context.compress.count", "strategy" => "L2").increment(1);
        info!(
            usage_before = usage,
            target_usage = self.target_usage,
            "L2 Auto-Compact executed"
        );

        Ok(Some(AutoCompactOutcome {
            messages: rebuilt,
            summary,
            usage_before: usage,
        }))
    }

    /// Maintain an incremental token cache so the common "append one message" path stays O(1)-ish.
    pub async fn calculate_usage(
        &self,
        messages: &[ChatMessage],
        token_counter: &dyn TokenCounter,
        max_tokens: usize,
    ) -> f64 {
        let mut cache = self.usage_cache.lock().await;
        let unchanged_prefix = messages
            .iter()
            .zip(cache.entries.iter())
            .take_while(|(message, cached)| cached.fingerprint == fingerprint_message(message))
            .count();

        let mut total_tokens = cache.entries[..unchanged_prefix]
            .iter()
            .map(|entry| entry.tokens)
            .sum::<usize>();
        let mut new_entries = cache.entries[..unchanged_prefix].to_vec();

        for message in &messages[unchanged_prefix..] {
            let tokens = token_counter.count(&message.content);
            total_tokens += tokens;
            new_entries.push(CachedTokenEntry {
                fingerprint: fingerprint_message(message),
                tokens,
            });
        }

        cache.entries = new_entries;
        cache.cached_total_tokens = total_tokens;
        total_tokens as f64 / max_tokens as f64
    }

    fn trim_for_llm_input(
        &self,
        messages: &[ChatMessage],
        token_counter: &dyn TokenCounter,
    ) -> Vec<ChatMessage> {
        let budget = (self.max_context_tokens as f64 * 0.7) as usize;
        let mut preserved_system = None;
        let mut non_system = Vec::new();
        let mut total_tokens = 0usize;

        for message in messages {
            total_tokens += token_counter.count(&message.content);
            if preserved_system.is_none() && message.role == MessageRole::System {
                preserved_system = Some(message.clone());
            } else {
                non_system.push(message.clone());
            }
        }

        while total_tokens > budget && non_system.len() > 2 {
            let removed = non_system.remove(0);
            total_tokens = total_tokens.saturating_sub(token_counter.count(&removed.content));
        }

        let mut prepared = Vec::new();
        if let Some(system) = preserved_system {
            prepared.push(system);
        }
        prepared.extend(non_system);
        prepared
    }

    fn rebuild_messages(
        &self,
        messages: &[ChatMessage],
        summary: CompactSummary,
    ) -> Vec<ChatMessage> {
        let mut rebuilt = Vec::new();
        if let Some(system) = messages
            .first()
            .filter(|message| message.role == MessageRole::System)
        {
            rebuilt.push(system.clone());
        }

        rebuilt.push(summary.to_context_message());

        let recent_non_system = messages
            .iter()
            .filter(|message| message.role != MessageRole::System)
            .cloned()
            .collect::<Vec<_>>();
        let keep_start = recent_non_system.len().saturating_sub(4);
        rebuilt.extend(recent_non_system.into_iter().skip(keep_start));
        rebuilt
    }
}

fn fingerprint_message(message: &ChatMessage) -> u64 {
    let role = match message.role {
        MessageRole::System => "system",
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::Tool => "tool",
    };
    hash(
        format!(
            "{}|{}|{}|{}",
            role,
            message.turn.unwrap_or_default(),
            message.tool_call_id.as_deref().unwrap_or(""),
            message.content
        )
        .as_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::AutoCompactor;
    use crate::compression::{
        ChatMessage, CompactSummary, LlmClient, LlmRequest, LlmResponse, MessageRole,
        SimpleTokenCounter,
    };

    #[derive(Debug)]
    struct MockLlm {
        response: String,
        seen_history_len: Arc<tokio::sync::Mutex<Option<usize>>>,
    }

    #[async_trait]
    impl LlmClient for MockLlm {
        async fn complete(
            &self,
            request: LlmRequest,
        ) -> std::result::Result<LlmResponse, crate::compression::LlmError> {
            *self.seen_history_len.lock().await = Some(request.messages[1].content.len());
            Ok(LlmResponse {
                content: self.response.clone(),
            })
        }
    }

    fn sample_summary() -> String {
        serde_json::to_string(&CompactSummary {
            user_request: "Implement".into(),
            completed_work: vec!["Added structs".into()],
            current_state: "Writing tests".into(),
            key_decisions: vec!["Use rule-first compaction".into()],
            error_info: vec![],
            continuation_context: "Run cargo test".into(),
        })
        .unwrap()
    }

    #[tokio::test]
    async fn l2_compacts_when_usage_crosses_threshold() {
        let seen = Arc::new(tokio::sync::Mutex::new(None));
        let llm = Arc::new(MockLlm {
            response: sample_summary(),
            seen_history_len: Arc::clone(&seen),
        });
        let compactor = AutoCompactor::default()
            .with_llm_client(llm)
            .with_max_context_tokens(1_000);
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: "system".into(),
                turn: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: "x".repeat(4_000),
                turn: Some(1),
                tool_call_id: None,
            },
        ];

        let outcome = compactor
            .maybe_compact(&messages, &SimpleTokenCounter, 1_000)
            .await
            .unwrap()
            .unwrap();

        assert!(outcome
            .messages
            .iter()
            .any(|message| message.content.contains("[Context Compacted]")));
        assert!(seen.lock().await.is_some());
    }

    #[tokio::test]
    async fn l2_trims_input_to_seventy_percent_of_window() {
        let seen = Arc::new(tokio::sync::Mutex::new(None));
        let llm = Arc::new(MockLlm {
            response: sample_summary(),
            seen_history_len: Arc::clone(&seen),
        });
        let compactor = AutoCompactor::default()
            .with_llm_client(llm)
            .with_max_context_tokens(100);
        let messages = (0..20)
            .map(|index| ChatMessage {
                role: MessageRole::User,
                content: format!("message-{index}-{}", "x".repeat(40)),
                turn: Some(index),
                tool_call_id: None,
            })
            .collect::<Vec<_>>();

        let _ = compactor
            .maybe_compact(&messages, &SimpleTokenCounter, 100)
            .await
            .unwrap();

        assert!(seen.lock().await.unwrap() < serde_json::to_string(&messages).unwrap().len());
    }

    #[tokio::test]
    async fn calculate_usage_updates_incrementally() {
        let compactor = AutoCompactor::default();
        let counter = SimpleTokenCounter;
        let messages = vec![ChatMessage {
            role: MessageRole::User,
            content: "abcd".repeat(10),
            turn: Some(1),
            tool_call_id: None,
        }];

        let first = compactor.calculate_usage(&messages, &counter, 100).await;
        let mut extended = messages.clone();
        extended.push(ChatMessage {
            role: MessageRole::Assistant,
            content: "more".repeat(10),
            turn: Some(2),
            tool_call_id: None,
        });
        let second = compactor.calculate_usage(&extended, &counter, 100).await;

        assert!(second > first);
        assert!(compactor.usage_cache.lock().await.cached_total_tokens > 0);
    }
}
