use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;
use uuid::Uuid;

use crate::error::ContextError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecoveryMode {
    SummaryOnly,
    TranscriptOnDemand,
    HardResume,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RollingSummaryPacket {
    pub round_id: u64,
    pub pinned_updates: Vec<String>,
    pub summary_markdown: String,
    pub key_decisions: Vec<String>,
    pub unresolved_items: Vec<String>,
    pub archive_candidates: Vec<String>,
    pub transcript_ref: Option<PathBuf>,
}

impl RollingSummaryPacket {
    pub fn validate(&self) -> bool {
        !self.summary_markdown.trim().is_empty() && self.round_id > 0
    }

    pub fn to_context_string(&self) -> String {
        let mut output = String::new();

        if !self.pinned_updates.is_empty() {
            output.push_str("### Pinned Updates\n");
            for update in &self.pinned_updates {
                output.push_str(&format!("- {}\n", update));
            }
            output.push('\n');
        }

        output.push_str("### Rolling Summary\n");
        output.push_str(&self.summary_markdown);
        output.push('\n');

        if !self.key_decisions.is_empty() {
            output.push_str("\n### Key Decisions\n");
            for decision in &self.key_decisions {
                output.push_str(&format!("- {}\n", decision));
            }
        }

        if !self.unresolved_items.is_empty() {
            output.push_str("\n### Unresolved Items\n");
            for item in &self.unresolved_items {
                output.push_str(&format!("- {}\n", item));
            }
        }

        output
    }

    pub async fn persist_transcript(
        &mut self,
        base_dir: impl AsRef<Path>,
        transcript: &str,
    ) -> Result<PathBuf> {
        let dir = base_dir
            .as_ref()
            .join(".assistant-memory")
            .join("subagents");
        fs::create_dir_all(&dir).await?;

        let path = dir.join(format!("{}.md", Uuid::new_v4()));
        fs::write(&path, transcript).await?;
        self.transcript_ref = Some(path.clone());

        Ok(path)
    }

    pub async fn load_transcript(&self, mode: RecoveryMode) -> Result<Option<String>> {
        match mode {
            RecoveryMode::SummaryOnly => Ok(None),
            RecoveryMode::TranscriptOnDemand | RecoveryMode::HardResume => {
                let Some(path) = &self.transcript_ref else {
                    return Ok(None);
                };

                let content = fs::read_to_string(path).await?;
                Ok(Some(content))
            }
        }
    }

    pub async fn cleanup_transcripts(base_dir: impl AsRef<Path>, ttl_days: i64) -> Result<usize> {
        let dir = base_dir
            .as_ref()
            .join(".assistant-memory")
            .join("subagents");
        let mut removed = 0usize;

        let mut entries = match fs::read_dir(&dir).await {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(err) => return Err(err.into()),
        };

        let cutoff = Utc::now() - Duration::days(ttl_days);

        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            let modified = metadata.modified()?;
            let modified: DateTime<Utc> = modified.into();

            if modified < cutoff {
                fs::remove_file(entry.path())
                    .await
                    .map_err(|source| ContextError::io(entry.path(), source))?;
                removed += 1;
            }
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::{RecoveryMode, RollingSummaryPacket};
    use tempfile::tempdir;
    use uuid::Uuid;

    fn packet() -> RollingSummaryPacket {
        RollingSummaryPacket {
            round_id: 1,
            pinned_updates: vec!["Use tokio::fs".into()],
            summary_markdown: "Implemented context pool.".into(),
            key_decisions: vec!["Prefer rule-based compaction first.".into()],
            unresolved_items: vec!["Wire coordinator bootstrap.".into()],
            archive_candidates: vec![],
            transcript_ref: None,
        }
    }

    #[tokio::test]
    async fn persist_transcript_uses_uuid_filename() {
        let dir = tempdir().unwrap();
        let mut packet = packet();
        let path = packet
            .persist_transcript(dir.path(), "full transcript")
            .await
            .unwrap();
        let stem = path.file_stem().unwrap().to_string_lossy();

        assert!(Uuid::parse_str(&stem).is_ok());
    }

    #[tokio::test]
    async fn summary_only_does_not_load_transcript() {
        let packet = packet();
        assert_eq!(
            packet
                .load_transcript(RecoveryMode::SummaryOnly)
                .await
                .unwrap(),
            None
        );
    }

    #[test]
    fn validate_checks_required_fields() {
        let valid = packet();
        assert!(valid.validate());

        let invalid = RollingSummaryPacket {
            summary_markdown: String::new(),
            ..packet()
        };
        assert!(!invalid.validate());
    }
}
