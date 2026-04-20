use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::MemoryError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProceduralRule {
    pub id: String,
    pub title: String,
    pub instruction: String,
    pub trigger: Option<String>,
    pub enabled: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProceduralMemory {
    pub rules: Vec<ProceduralRule>,
}

impl ProceduralMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert_rule(&mut self, mut rule: ProceduralRule) {
        rule.updated_at = Utc::now();
        if let Some(existing) = self.rules.iter_mut().find(|item| item.id == rule.id) {
            *existing = rule;
        } else {
            self.rules.push(rule);
        }
        self.rules.sort_by(|left, right| left.id.cmp(&right.id));
    }

    pub fn remove_rule(&mut self, id: &str) -> Option<ProceduralRule> {
        let index = self.rules.iter().position(|rule| rule.id == id)?;
        Some(self.rules.remove(index))
    }

    pub fn enabled_rules(&self) -> Vec<&ProceduralRule> {
        self.rules.iter().filter(|rule| rule.enabled).collect()
    }

    pub fn render_prompt_block(&self) -> String {
        let enabled = self.enabled_rules();
        if enabled.is_empty() {
            return String::new();
        }

        let mut lines = vec!["## Procedural Memory".to_string()];
        for rule in enabled {
            lines.push(format!("- {}: {}", rule.title, rule.instruction));
        }
        lines.join("\n")
    }

    pub async fn load_from_path(path: &Path) -> Result<Self, MemoryError> {
        if !fs::try_exists(path).await? {
            return Ok(Self::default());
        }
        Ok(serde_json::from_str(&fs::read_to_string(path).await?)?)
    }

    pub async fn save_to_path(&self, path: &Path) -> Result<(), MemoryError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, serde_json::to_vec_pretty(self)?).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use tempfile::tempdir;

    use super::{ProceduralMemory, ProceduralRule};

    #[tokio::test]
    async fn procedural_memory_roundtrip() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("procedural.json");

        let mut memory = ProceduralMemory::new();
        memory.upsert_rule(ProceduralRule {
            id: "rule-1".into(),
            title: "No unwrap".into(),
            instruction: "Use ? instead of unwrap".into(),
            trigger: Some("rust".into()),
            enabled: true,
            updated_at: Utc::now(),
        });
        memory.save_to_path(&path).await.unwrap();

        let loaded = ProceduralMemory::load_from_path(&path).await.unwrap();
        assert_eq!(loaded.rules.len(), 1);
        assert!(loaded.render_prompt_block().contains("No unwrap"));
    }
}
