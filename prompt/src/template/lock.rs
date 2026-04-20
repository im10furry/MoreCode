use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::PromptCacheError;

use super::manager::TemplateManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsLock {
    pub lock_version: u32,
    pub generated_at: DateTime<Utc>,
    pub templates: HashMap<String, TemplateLockEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateLockEntry {
    pub version: u64,
    pub content_hash: u64,
    pub file_path: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockMismatch {
    pub template_id: String,
    pub expected_hash: Option<u64>,
    pub actual_hash: Option<u64>,
    pub expected_version: Option<u64>,
    pub actual_version: Option<u64>,
}

impl PromptsLock {
    pub async fn generate(manager: &TemplateManager) -> Self {
        let templates = manager.list_templates().await;
        let mut entries = HashMap::new();

        for template in templates {
            entries.insert(
                template.id.clone(),
                TemplateLockEntry {
                    version: template.version,
                    content_hash: template.content_hash,
                    file_path: template.file_path,
                    updated_at: template.updated_at,
                },
            );
        }

        Self {
            lock_version: 1,
            generated_at: Utc::now(),
            templates: entries,
        }
    }

    pub async fn verify(&self, manager: &TemplateManager) -> Vec<LockMismatch> {
        let templates = manager.list_templates().await;
        let current = templates
            .into_iter()
            .map(|template| (template.id.clone(), template))
            .collect::<HashMap<_, _>>();
        let mut mismatches = Vec::new();

        for (id, locked) in &self.templates {
            match current.get(id) {
                Some(template) => {
                    if template.content_hash != locked.content_hash
                        || template.version != locked.version
                    {
                        mismatches.push(LockMismatch {
                            template_id: id.clone(),
                            expected_hash: Some(locked.content_hash),
                            actual_hash: Some(template.content_hash),
                            expected_version: Some(locked.version),
                            actual_version: Some(template.version),
                        });
                    }
                }
                None => mismatches.push(LockMismatch {
                    template_id: id.clone(),
                    expected_hash: Some(locked.content_hash),
                    actual_hash: None,
                    expected_version: Some(locked.version),
                    actual_version: None,
                }),
            }
        }

        for (id, template) in current {
            if !self.templates.contains_key(&id) {
                mismatches.push(LockMismatch {
                    template_id: id,
                    expected_hash: None,
                    actual_hash: Some(template.content_hash),
                    expected_version: None,
                    actual_version: Some(template.version),
                });
            }
        }

        mismatches.sort_by(|left, right| left.template_id.cmp(&right.template_id));
        mismatches
    }

    pub fn to_toml_string(&self) -> Result<String, PromptCacheError> {
        toml::to_string_pretty(self)
            .map_err(|error| PromptCacheError::LockFormatError(error.to_string()))
    }

    pub fn from_toml_str(input: &str) -> Result<Self, PromptCacheError> {
        toml::from_str(input).map_err(|error| PromptCacheError::LockFormatError(error.to_string()))
    }

    pub async fn write_to_path(&self, path: &Path) -> Result<(), PromptCacheError> {
        let content = self.to_toml_string()?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    pub async fn load_from_path(path: &Path) -> Result<Self, PromptCacheError> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_toml_str(&content)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::template::TemplateManager;

    use super::PromptsLock;

    #[tokio::test]
    async fn generates_and_verifies_lockfile() {
        let temp = tempdir().unwrap();
        let prompts_root = temp.path().join("prompts");
        tokio::fs::create_dir_all(prompts_root.join("system"))
            .await
            .unwrap();
        tokio::fs::write(
            prompts_root.join("system").join("coder.md"),
            "hello {{name}}",
        )
        .await
        .unwrap();

        let manager = TemplateManager::new(&prompts_root);
        manager.load_all().await.unwrap();

        let lock = PromptsLock::generate(&manager).await;
        assert!(lock.verify(&manager).await.is_empty());
    }

    #[tokio::test]
    async fn lockfile_roundtrip_and_detects_mismatch() {
        let temp = tempdir().unwrap();
        let prompts_root = temp.path().join("prompts");
        tokio::fs::create_dir_all(prompts_root.join("system"))
            .await
            .unwrap();
        let template_path = prompts_root.join("system").join("planner.md");
        tokio::fs::write(&template_path, "plan {{task}}")
            .await
            .unwrap();

        let manager = TemplateManager::new(&prompts_root);
        manager.load_all().await.unwrap();

        let lock = PromptsLock::generate(&manager).await;
        let lock_path = prompts_root.join("prompts.lock");
        lock.write_to_path(&lock_path).await.unwrap();
        let loaded = PromptsLock::load_from_path(&lock_path).await.unwrap();
        assert!(loaded.verify(&manager).await.is_empty());

        tokio::fs::write(&template_path, "changed {{task}}")
            .await
            .unwrap();
        manager.load_all().await.unwrap();

        let mismatches = loaded.verify(&manager).await;
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].template_id, "system/planner");
    }
}
