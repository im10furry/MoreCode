use serde::{Deserialize, Serialize};

use crate::{ChatMessage, TokenUsage};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatResponse {
    pub id: String,
    pub model: String,
    pub message: ChatMessage,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
    pub latency_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_response: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    Delta {
        content: String,
        cumulative_tokens: Option<u32>,
    },
    ToolCallDelta {
        index: u32,
        id: Option<String>,
        name: Option<String>,
        arguments_delta: String,
    },
    Finish {
        reason: FinishReason,
        usage: Option<TokenUsage>,
        response_id: String,
    },
    Error(String),
}

impl StreamEvent {
    pub fn is_finish(&self) -> bool {
        matches!(self, Self::Finish { .. })
    }

    pub fn as_delta_content(&self) -> Option<&str> {
        match self {
            Self::Delta { content, .. } => Some(content),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Cancelled,
}
