use std::path::PathBuf;

use anyhow::Result;
use metrics::counter;
use seahash::hash;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::info;

use crate::compression::{ChatMessage, CompactStats, MessageRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroCompactor {
    min_turn_gap: usize,
    small_result_threshold: usize,
    large_result_threshold: usize,
    temp_dir: PathBuf,
}

impl Default for MicroCompactor {
    fn default() -> Self {
        Self {
            min_turn_gap: 3,
            small_result_threshold: 10_000,
            large_result_threshold: 50_000,
            temp_dir: PathBuf::from(".assistant-memory/tmp"),
        }
    }
}

impl MicroCompactor {
    pub fn with_temp_dir(mut self, temp_dir: PathBuf) -> Self {
        self.temp_dir = temp_dir;
        self
    }

    pub async fn compact(
        &self,
        messages: &[ChatMessage],
        current_turn: usize,
    ) -> Result<(Vec<ChatMessage>, CompactStats)> {
        let mut cloned = messages.to_vec();
        let stats = self.compact_in_place(&mut cloned, current_turn).await?;
        Ok((cloned, stats))
    }

    pub async fn compact_in_place(
        &self,
        messages: &mut [ChatMessage],
        current_turn: usize,
    ) -> Result<CompactStats> {
        let mut stats = CompactStats::default();

        for message in messages.iter_mut() {
            if message.role != MessageRole::Tool {
                continue;
            }

            let turn = message.turn.unwrap_or_default();
            if turn >= current_turn.saturating_sub(self.min_turn_gap) {
                continue;
            }

            let content_len = message.content.chars().count();
            if content_len <= self.small_result_threshold {
                continue;
            }

            if content_len > self.large_result_threshold {
                match self.persist_to_disk(&message.content, turn).await {
                    Ok(path) => {
                        message.content = format!(
                            "[Old tool result saved to {} (was {} chars)]",
                            path.display(),
                            content_len
                        );
                        stats.large_cleared += 1;
                    }
                    Err(error) => {
                        message.content = format!(
                            "[Old tool result content cleared (was {} chars, persist failed: {})]",
                            content_len, error
                        );
                        stats.small_cleared += 1;
                    }
                }
            } else {
                message.content = format!(
                    "[Old tool result content cleared (was {} chars)]",
                    content_len
                );
                stats.small_cleared += 1;
            }

            stats.chars_saved += content_len;
        }

        if stats.small_cleared > 0 || stats.large_cleared > 0 {
            counter!("context.compress.count", "strategy" => "L1").increment(1);
            info!(
                small = stats.small_cleared,
                large = stats.large_cleared,
                chars_saved = stats.chars_saved,
                "L1 Micro-Compact executed"
            );
        }

        Ok(stats)
    }

    async fn persist_to_disk(&self, content: &str, turn: usize) -> Result<PathBuf> {
        fs::create_dir_all(&self.temp_dir).await?;
        let file_name = format!(
            "tool-result-turn{}-{:016x}.txt",
            turn,
            hash(content.as_bytes())
        );
        let path = self.temp_dir.join(file_name);
        fs::write(&path, content).await?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::MicroCompactor;
    use crate::compression::{ChatMessage, MessageRole};

    #[tokio::test]
    async fn l1_clears_old_large_tool_results() {
        let compactor = MicroCompactor::default();
        let messages = vec![ChatMessage {
            role: MessageRole::Tool,
            content: "x".repeat(11_000),
            turn: Some(1),
            tool_call_id: Some("call".into()),
        }];

        let (messages, stats) = compactor.compact(&messages, 6).await.unwrap();
        assert!(messages[0].content.contains("cleared"));
        assert_eq!(stats.small_cleared, 1);
    }

    #[tokio::test]
    async fn l1_persists_very_large_tool_results() {
        let dir = tempdir().unwrap();
        let compactor = MicroCompactor::default().with_temp_dir(dir.path().join("tmp"));
        let messages = vec![ChatMessage {
            role: MessageRole::Tool,
            content: "x".repeat(60_000),
            turn: Some(1),
            tool_call_id: Some("call".into()),
        }];

        let (messages, stats) = compactor.compact(&messages, 6).await.unwrap();
        assert!(messages[0].content.contains("saved to"));
        assert_eq!(stats.large_cleared, 1);
    }

    #[tokio::test]
    async fn l1_degrades_to_placeholder_when_persist_fails() {
        let dir = tempdir().unwrap();
        let bad_path = dir.path().join("not-a-dir");
        tokio::fs::write(&bad_path, b"file").await.unwrap();
        let compactor = MicroCompactor::default().with_temp_dir(bad_path);
        let messages = vec![ChatMessage {
            role: MessageRole::Tool,
            content: "x".repeat(60_000),
            turn: Some(1),
            tool_call_id: Some("call".into()),
        }];

        let (messages, stats) = compactor.compact(&messages, 6).await.unwrap();
        assert!(messages[0].content.contains("persist failed"));
        assert_eq!(stats.small_cleared, 1);
    }
}
