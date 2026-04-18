use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub turn: Option<usize>,
    pub tool_call_id: Option<String>,
}

impl Default for ChatMessage {
    fn default() -> Self {
        Self {
            role: MessageRole::User,
            content: String::new(),
            turn: None,
            tool_call_id: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactStats {
    pub small_cleared: usize,
    pub large_cleared: usize,
    pub chars_saved: usize,
    pub memory_writes: usize,
    pub dropped_messages: usize,
}

impl CompactStats {
    pub fn estimated_tokens_saved(&self) -> usize {
        self.chars_saved.div_ceil(4)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactSummary {
    pub user_request: String,
    pub completed_work: Vec<String>,
    pub current_state: String,
    pub key_decisions: Vec<String>,
    pub error_info: Vec<String>,
    pub continuation_context: String,
}

impl CompactSummary {
    pub fn to_context_message(&self) -> ChatMessage {
        ChatMessage {
            role: MessageRole::System,
            content: format!(
                "[Context Compacted]\n## User Request\n{}\n\n## Completed Work\n{}\n\n## Current State\n{}\n\n## Key Decisions\n{}\n\n## Error Info\n{}\n\n## Continuation Context\n{}",
                self.user_request,
                bullet_list(&self.completed_work),
                self.current_state,
                bullet_list(&self.key_decisions),
                bullet_list(&self.error_info),
                self.continuation_context,
            ),
            turn: None,
            tool_call_id: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryCompactResult {
    pub code_changes: usize,
    pub bug_records: usize,
    pub conventions: usize,
}

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<usize>,
    pub response_format: Option<ResponseFormat>,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    Json,
    Text,
}

#[derive(Debug, Clone, Error)]
pub enum LlmError {
    #[error("api error: {message}")]
    ApiError { message: String, status_code: u16 },
    #[error("context length exceeded")]
    ContextLengthExceeded,
    #[error("timeout")]
    Timeout,
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, request: LlmRequest) -> std::result::Result<LlmResponse, LlmError>;
}

#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn append(&self, path: &str, entry: &serde_json::Value) -> anyhow::Result<()>;
}

pub trait TokenCounter: Send + Sync {
    fn count(&self, text: &str) -> usize;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SimpleTokenCounter;

impl TokenCounter for SimpleTokenCounter {
    fn count(&self, text: &str) -> usize {
        text.chars().count().div_ceil(4)
    }
}

#[derive(Debug, Clone)]
pub struct CompressionPromptBuilder {
    template: String,
}

impl Default for CompressionPromptBuilder {
    fn default() -> Self {
        Self {
            template: Self::default_template(),
        }
    }
}

impl CompressionPromptBuilder {
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }

    pub fn default_template() -> String {
        "You are a context compaction assistant. Summarize the conversation as JSON with keys user_request, completed_work, current_state, key_decisions, error_info, continuation_context.".into()
    }

    pub fn build_request(&self, messages: &[ChatMessage]) -> LlmRequest {
        let history = serde_json::to_string_pretty(messages).unwrap_or_default();
        LlmRequest {
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: self.template.clone(),
                    turn: None,
                    tool_call_id: None,
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: history,
                    turn: None,
                    tool_call_id: None,
                },
            ],
            temperature: Some(0.0),
            max_tokens: Some(2048),
            response_format: Some(ResponseFormat::Json),
        }
    }
}

fn bullet_list(items: &[String]) -> String {
    if items.is_empty() {
        "- None".into()
    } else {
        items
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::{ChatMessage, CompactSummary, CompressionPromptBuilder, MessageRole};

    #[test]
    fn compact_summary_converts_to_system_message() {
        let summary = CompactSummary {
            user_request: "Implement context manager".into(),
            completed_work: vec!["Added structs".into()],
            current_state: "Writing tests".into(),
            key_decisions: vec!["Use tokio::fs".into()],
            error_info: vec![],
            continuation_context: "Run cargo test".into(),
        };

        let message = summary.to_context_message();
        assert_eq!(message.role, MessageRole::System);
        assert!(message.content.contains("Implement context manager"));
    }

    #[test]
    fn prompt_builder_wraps_history_as_json() {
        let request = CompressionPromptBuilder::default().build_request(&[ChatMessage {
            role: MessageRole::User,
            content: "Need summary".into(),
            turn: Some(1),
            tool_call_id: None,
        }]);

        assert_eq!(request.messages.len(), 2);
        assert!(request.messages[1].content.contains("Need summary"));
    }
}
