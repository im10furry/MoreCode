use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::model::ResponseFormat;
use crate::{CacheControlPoint, CacheControlType, ToolCall, ToolDefinition};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cache_control_points: Vec<CacheControlPoint>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra_headers: HashMap<String, String>,
}

fn default_temperature() -> f32 {
    1.0
}

impl Default for ChatRequest {
    fn default() -> Self {
        Self {
            request_id: None,
            messages: Vec::new(),
            model: None,
            temperature: default_temperature(),
            top_p: None,
            max_tokens: None,
            stop_sequences: Vec::new(),
            tools: Vec::new(),
            response_format: None,
            timeout: None,
            user_id: None,
            cache_control_points: Vec::new(),
            extra_headers: HashMap::new(),
        }
    }
}

impl ChatRequest {
    pub fn estimated_prompt_tokens(&self) -> usize {
        self.messages
            .iter()
            .map(|message| crate::estimate_text_tokens(&message.content.to_text()))
            .sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControlType>,
}

impl ChatMessage {
    pub fn text(role: MessageRole, text: impl Into<String>) -> Self {
        Self {
            role,
            content: MessageContent::Text(text.into()),
            name: None,
            tool_calls: Vec::new(),
            tool_call_id: None,
            cache_control: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    pub fn to_text(&self) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Parts(parts) => parts
                .iter()
                .filter_map(|part| match part {
                    ContentPart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text {
        text: String,
    },
    Image {
        url: String,
        #[serde(default = "default_image_detail")]
        detail: ImageDetail,
    },
    File {
        filename: String,
        mime_type: String,
        data: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Low,
    High,
    Auto,
}

fn default_image_detail() -> ImageDetail {
    ImageDetail::Auto
}
