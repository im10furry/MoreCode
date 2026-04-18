use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncWriteExt};

use crate::{error::MemoryError, preference::rules::UserRule};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreferenceObservation {
    pub key: String,
    pub value: String,
    pub confidence_delta: f32,
    pub evidence: String,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreferenceCandidate {
    pub key: String,
    pub value: String,
    pub confidence: f32,
    pub occurrences: u32,
    pub evidence: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PreferenceProfile {
    pub preferences: Vec<PreferenceCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PreferenceFile {
    version: u32,
    preferences: Vec<PreferenceCandidate>,
    rules: Vec<UserRule>,
}

impl Default for PreferenceFile {
    fn default() -> Self {
        Self {
            version: 1,
            preferences: Vec::new(),
            rules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreferenceManager {
    memory_dir: PathBuf,
    promote_threshold: f32,
}

impl PreferenceManager {
    pub fn new(project_root: &Path) -> Self {
        Self {
            memory_dir: project_root.join(".assistant-memory"),
            promote_threshold: 0.7,
        }
    }

    pub async fn load_profile(&self) -> Result<PreferenceProfile, MemoryError> {
        let file = self.load_file().await?;
        Ok(PreferenceProfile {
            preferences: file
                .preferences
                .into_iter()
                .filter(|candidate| candidate.confidence >= self.promote_threshold)
                .collect(),
        })
    }

    pub async fn record_sideband_observations(
        &self,
        observations: &[PreferenceObservation],
    ) -> Result<PreferenceProfile, MemoryError> {
        fs::create_dir_all(&self.memory_dir).await?;

        let mut file = self.load_file().await?;
        for observation in observations {
            if let Some(candidate) = file.preferences.iter_mut().find(|candidate| {
                candidate.key == observation.key && candidate.value == observation.value
            }) {
                candidate.occurrences += 1;
                candidate.confidence =
                    (candidate.confidence + observation.confidence_delta).clamp(0.0, 1.0);
                candidate.updated_at = observation.observed_at;
                if !candidate
                    .evidence
                    .iter()
                    .any(|evidence| evidence == &observation.evidence)
                {
                    candidate.evidence.push(observation.evidence.clone());
                }
            } else {
                file.preferences.push(PreferenceCandidate {
                    key: observation.key.clone(),
                    value: observation.value.clone(),
                    confidence: observation.confidence_delta.clamp(0.0, 1.0),
                    occurrences: 1,
                    evidence: vec![observation.evidence.clone()],
                    updated_at: observation.observed_at,
                });
            }
        }

        file.preferences.sort_by(|left, right| {
            right
                .confidence
                .partial_cmp(&left.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.write_file(&file).await?;
        self.append_candidates(observations).await?;
        self.load_profile().await
    }

    pub async fn render_prompt_block(&self) -> Result<String, MemoryError> {
        let profile = self.load_profile().await?;
        if profile.preferences.is_empty() {
            return Ok(String::new());
        }

        let mut lines = vec!["=== 用户偏好（从既有对话旁路提取，无额外 API 调用） ===".to_string()];
        for preference in &profile.preferences {
            lines.push(format!(
                "- {} = {} (confidence {:.2})",
                preference.key, preference.value, preference.confidence
            ));
        }
        Ok(lines.join("\n"))
    }

    async fn load_file(&self) -> Result<PreferenceFile, MemoryError> {
        let path = self.memory_dir.join("user-preferences.json");
        if !fs::try_exists(&path).await? {
            return Ok(PreferenceFile::default());
        }
        let contents = fs::read_to_string(path).await?;
        Ok(serde_json::from_str(&contents)?)
    }

    async fn write_file(&self, file: &PreferenceFile) -> Result<(), MemoryError> {
        let path = self.memory_dir.join("user-preferences.json");
        let contents = serde_json::to_vec_pretty(file)?;
        fs::write(path, contents).await?;
        Ok(())
    }

    async fn append_candidates(
        &self,
        observations: &[PreferenceObservation],
    ) -> Result<(), MemoryError> {
        let path = self.memory_dir.join("preference-candidates.jsonl");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;

        for observation in observations {
            let line = serde_json::to_string(observation)? + "\n";
            file.write_all(line.as_bytes()).await?;
        }
        file.flush().await?;
        Ok(())
    }
}
