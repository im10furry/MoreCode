use serde::{Deserialize, Serialize};

/// Role of a message in an LLM conversation.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    /// System prompt content.
    System,
    /// End-user content.
    User,
    /// Assistant model response.
    Assistant,
    /// Tool call or tool result content.
    Tool,
}

impl MessageRole {
    /// All supported role variants.
    pub const ALL: [Self; 4] = [Self::System, Self::User, Self::Assistant, Self::Tool];
}

/// Reason why an LLM response finished.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum FinishReason {
    /// Generation completed normally.
    Stop,
    /// Generation stopped because of the max token limit.
    Length,
    /// Generation was stopped by a content filter.
    ContentFilter,
    /// Generation ended because a tool call was emitted.
    ToolCalls,
    /// Generation ended for an unknown provider-specific reason.
    Unknown(String),
}
