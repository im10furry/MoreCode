use crate::{ChatMessage, ContentPart, ImageDetail, MessageContent};

const LOW_DETAIL_IMAGE_TOKENS: usize = 85;
const HIGH_DETAIL_IMAGE_TOKENS: usize = 255;
const AUTO_DETAIL_IMAGE_TOKENS: usize = 170;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MultimodalEstimateBreakdown {
    pub text_tokens: usize,
    pub image_tokens: usize,
    pub file_tokens: usize,
    pub tool_tokens: usize,
    pub overhead_tokens: usize,
}

impl MultimodalEstimateBreakdown {
    pub fn total(&self) -> usize {
        self.text_tokens
            + self.image_tokens
            + self.file_tokens
            + self.tool_tokens
            + self.overhead_tokens
    }
}

pub fn estimate_message_tokens(message: &ChatMessage) -> usize {
    estimate_message_breakdown(message).total()
}

pub fn estimate_content_tokens(content: &MessageContent) -> usize {
    match content {
        MessageContent::Text(text) => crate::estimate_text_tokens(text),
        MessageContent::Parts(parts) => parts.iter().map(estimate_part_tokens).sum(),
    }
}

pub fn estimate_part_tokens(part: &ContentPart) -> usize {
    match part {
        ContentPart::Text { text } => crate::estimate_text_tokens(text),
        ContentPart::Image { detail, .. } => match detail {
            ImageDetail::Low => LOW_DETAIL_IMAGE_TOKENS,
            ImageDetail::High => HIGH_DETAIL_IMAGE_TOKENS,
            ImageDetail::Auto => AUTO_DETAIL_IMAGE_TOKENS,
        },
        ContentPart::File {
            mime_type, data, ..
        } => estimate_file_tokens(mime_type, data),
    }
}

fn estimate_message_breakdown(message: &ChatMessage) -> MultimodalEstimateBreakdown {
    let mut breakdown = MultimodalEstimateBreakdown {
        overhead_tokens: 4,
        ..Default::default()
    };

    if let Some(name) = &message.name {
        breakdown.overhead_tokens += crate::estimate_text_tokens(name);
    }
    if let Some(tool_call_id) = &message.tool_call_id {
        breakdown.overhead_tokens += crate::estimate_text_tokens(tool_call_id);
    }

    match &message.content {
        MessageContent::Text(text) => {
            breakdown.text_tokens += crate::estimate_text_tokens(text);
        }
        MessageContent::Parts(parts) => {
            for part in parts {
                match part {
                    ContentPart::Text { text } => {
                        breakdown.text_tokens += crate::estimate_text_tokens(text);
                    }
                    ContentPart::Image { .. } => {
                        breakdown.image_tokens += estimate_part_tokens(part);
                    }
                    ContentPart::File { .. } => {
                        breakdown.file_tokens += estimate_part_tokens(part);
                    }
                }
            }
        }
    }

    for tool_call in &message.tool_calls {
        breakdown.tool_tokens += crate::estimate_text_tokens(&tool_call.id);
        breakdown.tool_tokens += crate::estimate_text_tokens(&tool_call.name);
        breakdown.tool_tokens += crate::estimate_text_tokens(&tool_call.arguments);
        breakdown.overhead_tokens += 2;
    }

    breakdown
}

fn estimate_file_tokens(mime_type: &str, data: &str) -> usize {
    if mime_type.starts_with("text/")
        || mime_type == "application/json"
        || mime_type.ends_with("+json")
        || mime_type.ends_with("+xml")
        || mime_type == "application/xml"
    {
        return crate::estimate_text_tokens(data);
    }

    let payload = strip_data_url_payload(data);
    let estimated_bytes = payload.len().saturating_mul(3) / 4;
    estimated_bytes.div_ceil(4).max(32)
}

fn strip_data_url_payload(data: &str) -> &str {
    data.split_once(',')
        .map(|(_, payload)| payload)
        .unwrap_or(data)
}

#[cfg(test)]
mod tests {
    use crate::{ChatMessage, ContentPart, ImageDetail, MessageRole, ToolCall};

    use super::{estimate_content_tokens, estimate_message_tokens, estimate_part_tokens};

    #[test]
    fn estimates_text_parts_with_base_counter() {
        let tokens = estimate_part_tokens(&ContentPart::Text {
            text: "hello world".into(),
        });
        assert!(tokens > 0);
    }

    #[test]
    fn estimates_image_parts_by_detail_level() {
        let low = estimate_part_tokens(&ContentPart::Image {
            url: "data:image/png;base64,AAAA".into(),
            detail: ImageDetail::Low,
        });
        let high = estimate_part_tokens(&ContentPart::Image {
            url: "data:image/png;base64,AAAA".into(),
            detail: ImageDetail::High,
        });

        assert!(high > low);
    }

    #[test]
    fn estimates_binary_files_from_payload_size() {
        let tokens = estimate_part_tokens(&ContentPart::File {
            filename: "image.png".into(),
            mime_type: "image/png".into(),
            data: "QUJDREVGR0g=".into(),
        });
        assert!(tokens >= 32);
    }

    #[test]
    fn estimates_full_message_with_tool_calls() {
        let message = ChatMessage {
            role: MessageRole::Assistant,
            content: crate::MessageContent::Parts(vec![
                ContentPart::Text {
                    text: "hello".into(),
                },
                ContentPart::Image {
                    url: "data:image/png;base64,AAAA".into(),
                    detail: ImageDetail::Auto,
                },
            ]),
            name: Some("assistant".into()),
            tool_calls: vec![ToolCall {
                id: "call_1".into(),
                name: "search".into(),
                arguments: r#"{"query":"rust"}"#.into(),
            }],
            tool_call_id: None,
            cache_control: None,
        };

        let tokens = estimate_message_tokens(&message);
        assert!(tokens > 100);
    }

    #[test]
    fn estimates_content_with_mixed_parts() {
        let tokens = estimate_content_tokens(&crate::MessageContent::Parts(vec![
            ContentPart::Text {
                text: "hello".into(),
            },
            ContentPart::File {
                filename: "note.txt".into(),
                mime_type: "text/plain".into(),
                data: "sample text".into(),
            },
        ]));

        assert!(tokens > 0);
    }
}
