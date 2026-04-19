use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::MemoryError;

use super::learning::{PreferenceCandidate, PreferenceProfile};
use super::rules::{RuleBundle, UserRule};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPreferences {
    pub version: u32,
    pub updated_at: DateTime<Utc>,
    pub preferences: Vec<PreferenceCandidate>,
    pub rules: Vec<UserRule>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            version: 1,
            updated_at: Utc::now(),
            preferences: Vec::new(),
            rules: Vec::new(),
        }
    }
}

impl UserPreferences {
    pub fn path(project_root: &Path) -> PathBuf {
        project_root
            .join(".assistant-memory")
            .join("user-preferences.json")
    }

    pub async fn load(project_root: &Path) -> Result<Self, MemoryError> {
        let path = Self::path(project_root);
        if !fs::try_exists(&path).await? {
            return Ok(Self::default());
        }

        Ok(serde_json::from_str(&fs::read_to_string(path).await?)?)
    }

    pub async fn save(&self, project_root: &Path) -> Result<(), MemoryError> {
        let path = Self::path(project_root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut next = self.clone();
        next.updated_at = Utc::now();
        fs::write(path, serde_json::to_vec_pretty(&next)?).await?;
        Ok(())
    }

    pub fn preference_profile(&self, promote_threshold: f32) -> PreferenceProfile {
        PreferenceProfile {
            preferences: self
                .preferences
                .iter()
                .filter(|candidate| candidate.confidence >= promote_threshold)
                .cloned()
                .collect(),
        }
    }

    pub fn rule_bundle(&self) -> RuleBundle {
        RuleBundle {
            rules: self.rules.clone(),
        }
    }

    pub fn upsert_candidate(&mut self, candidate: PreferenceCandidate) {
        if let Some(existing) = self
            .preferences
            .iter_mut()
            .find(|item| item.key == candidate.key && item.value == candidate.value)
        {
            *existing = candidate;
        } else {
            self.preferences.push(candidate);
        }
    }

    pub fn upsert_rule(&mut self, rule: UserRule) {
        if let Some(existing) = self.rules.iter_mut().find(|item| item.id == rule.id) {
            *existing = rule;
        } else {
            self.rules.push(rule);
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use tempfile::tempdir;

    use crate::preference::{PreferenceCandidate, RuleScope, RuleSource, RuleType, UserRule};

    use super::UserPreferences;

    #[tokio::test]
    async fn user_preferences_roundtrip() {
        let temp = tempdir().unwrap();
        let mut prefs = UserPreferences::default();
        prefs.upsert_candidate(PreferenceCandidate {
            key: "output_format".into(),
            value: "markdown".into(),
            confidence: 0.9,
            occurrences: 3,
            evidence: vec!["user picked markdown".into()],
            updated_at: Utc::now(),
        });
        prefs.upsert_rule(UserRule {
            id: "rule-1".into(),
            description: "no unwrap".into(),
            rule_type: RuleType::CodeConstraint {
                forbidden_patterns: vec!["\\.unwrap\\(\\)".into()],
                required_patterns: Vec::new(),
            },
            scope: RuleScope::Project,
            created_at: Utc::now(),
            source: RuleSource::Manual,
            enabled: true,
        });

        prefs.save(temp.path()).await.unwrap();
        let loaded = UserPreferences::load(temp.path()).await.unwrap();
        assert_eq!(loaded.preferences.len(), 1);
        assert_eq!(loaded.rules.len(), 1);
        assert_eq!(loaded.preference_profile(0.7).preferences.len(), 1);
    }
}
