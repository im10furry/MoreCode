use serde::{Deserialize, Serialize};

use crate::{compression::ChatMessage, session::RollingSummaryPacket};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionMemoryLayers {
    pub pinned_info: Vec<String>,
    pub rolling_summary: Option<RollingSummaryPacket>,
    pub archive_context: Vec<String>,
    pub current_round: Vec<ChatMessage>,
}

impl SessionMemoryLayers {
    pub fn to_context_string(&self) -> String {
        let mut sections = Vec::new();

        if !self.pinned_info.is_empty() {
            sections.push(format!(
                "### Pinned Info\n- {}",
                self.pinned_info.join("\n- ")
            ));
        }

        if let Some(summary) = &self.rolling_summary {
            sections.push(summary.to_context_string());
        }

        if !self.archive_context.is_empty() {
            sections.push(format!(
                "### Archive\n- {}",
                self.archive_context.join("\n- ")
            ));
        }

        if !self.current_round.is_empty() {
            sections.push(format!(
                "### Current Round\n- {}",
                self.current_round
                    .iter()
                    .map(|message| message.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n- ")
            ));
        }

        sections.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::SessionMemoryLayers;
    use crate::{
        compression::{ChatMessage, MessageRole},
        session::RollingSummaryPacket,
    };

    #[test]
    fn layers_are_rendered_in_expected_order() {
        let layers = SessionMemoryLayers {
            pinned_info: vec!["Constraint".into()],
            rolling_summary: Some(RollingSummaryPacket {
                round_id: 1,
                pinned_updates: vec![],
                summary_markdown: "Summary".into(),
                key_decisions: vec![],
                unresolved_items: vec![],
                archive_candidates: vec![],
                transcript_ref: None,
            }),
            archive_context: vec!["Older result".into()],
            current_round: vec![ChatMessage {
                role: MessageRole::User,
                content: "Latest ask".into(),
                turn: Some(1),
                tool_call_id: None,
            }],
        };

        let rendered = layers.to_context_string();
        let pinned = rendered.find("### Pinned Info").unwrap();
        let summary = rendered.find("### Rolling Summary").unwrap();
        let archive = rendered.find("### Archive").unwrap();
        let current = rendered.find("### Current Round").unwrap();

        assert!(pinned < summary && summary < archive && archive < current);
    }
}
