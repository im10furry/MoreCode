use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::MemoryError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentNoteRecord {
    pub agent: String,
    pub topic: String,
    pub path: PathBuf,
    pub content: String,
}

pub async fn write_agent_note(
    memory_dir: &Path,
    agent: &str,
    topic: &str,
    content: &str,
) -> Result<PathBuf, MemoryError> {
    let notes_dir = memory_dir.join("agent-notes").join(agent);
    fs::create_dir_all(&notes_dir).await?;
    let path = notes_dir.join(format!("{topic}.md"));
    fs::write(&path, content).await?;
    Ok(path)
}

pub async fn load_agent_note(
    memory_dir: &Path,
    agent: &str,
    topic: &str,
) -> Result<Option<AgentNoteRecord>, MemoryError> {
    let path = memory_dir
        .join("agent-notes")
        .join(agent)
        .join(format!("{topic}.md"));
    if !fs::try_exists(&path).await? {
        return Ok(None);
    }

    Ok(Some(AgentNoteRecord {
        agent: agent.to_string(),
        topic: topic.to_string(),
        content: fs::read_to_string(&path).await?,
        path,
    }))
}

pub async fn list_agent_notes(
    memory_dir: &Path,
    agent: &str,
) -> Result<Vec<AgentNoteRecord>, MemoryError> {
    let notes_dir = memory_dir.join("agent-notes").join(agent);
    if !fs::try_exists(&notes_dir).await? {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&notes_dir).await?;
    let mut notes = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if entry.file_type().await?.is_file() {
            let topic = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string();
            notes.push(AgentNoteRecord {
                agent: agent.to_string(),
                topic,
                content: fs::read_to_string(&path).await?,
                path,
            });
        }
    }
    notes.sort_by(|left, right| left.topic.cmp(&right.topic));
    Ok(notes)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{list_agent_notes, load_agent_note, write_agent_note};

    #[tokio::test]
    async fn agent_notes_roundtrip() {
        let temp = tempdir().unwrap();
        write_agent_note(temp.path(), "explorer", "scan", "hello")
            .await
            .unwrap();

        let note = load_agent_note(temp.path(), "explorer", "scan")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(note.content, "hello");

        let notes = list_agent_notes(temp.path(), "explorer").await.unwrap();
        assert_eq!(notes.len(), 1);
    }
}
