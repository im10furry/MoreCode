use std::sync::{Arc, OnceLock};

use anyhow::Result;
use metrics::counter;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};

use crate::{
    compression::{CompactSummary, MemoryCompactResult, MemoryStore},
    error::ContextError,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExtractorType {
    DependencyChange,
    Convention,
    RiskArea,
    CodeChange,
    BugRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtractionRule {
    pub name: String,
    pub pattern: String,
    pub target_file: String,
    pub extractor_type: ExtractorType,
}

#[derive(Clone)]
pub struct MemoryCompactor {
    trigger_threshold: f64,
    memory_store: Option<Arc<dyn MemoryStore>>,
    pub extraction_rules: Vec<ExtractionRule>,
}

impl std::fmt::Debug for MemoryCompactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryCompactor")
            .field("trigger_threshold", &self.trigger_threshold)
            .field("extraction_rules", &self.extraction_rules)
            .finish()
    }
}

impl Default for MemoryCompactor {
    fn default() -> Self {
        Self {
            trigger_threshold: 0.90,
            memory_store: None,
            extraction_rules: vec![
                ExtractionRule {
                    name: "cargo_dependency".into(),
                    pattern: r"Cargo\.toml.*(added|changed|removed)".into(),
                    target_file: "tech-stack.json".into(),
                    extractor_type: ExtractorType::DependencyChange,
                },
                ExtractionRule {
                    name: "code_convention".into(),
                    pattern: r"(convention|pattern|使用|统一|约定|规范)".into(),
                    target_file: "conventions.md".into(),
                    extractor_type: ExtractorType::Convention,
                },
                ExtractionRule {
                    name: "bug_found".into(),
                    pattern: r"(error|bug|panic|fail)".into(),
                    target_file: "agent-notes/debugger/bug-history.json".into(),
                    extractor_type: ExtractorType::BugRecord,
                },
            ],
        }
    }
}

impl MemoryCompactor {
    pub fn with_memory_store(mut self, memory_store: Arc<dyn MemoryStore>) -> Self {
        self.memory_store = Some(memory_store);
        self
    }

    pub fn trigger_threshold(&self) -> f64 {
        self.trigger_threshold
    }

    pub async fn extract_and_persist(
        &self,
        summary: &CompactSummary,
        session_id: &str,
    ) -> Result<MemoryCompactResult> {
        let memory_store = self
            .memory_store
            .as_ref()
            .ok_or(ContextError::MissingMemoryStore)?;
        let mut result = MemoryCompactResult::default();

        for work_item in &summary.completed_work {
            if let Some(entry) = self.extract_code_change(work_item, session_id) {
                memory_store
                    .append("agent-notes/coder/recent-changes.json", &entry)
                    .await?;
                result.code_changes += 1;
            }
        }

        for error in &summary.error_info {
            if let Some(entry) = self.extract_bug_record(error, session_id) {
                memory_store
                    .append("agent-notes/debugger/bug-history.json", &entry)
                    .await?;
                result.bug_records += 1;
            }
        }

        for decision in &summary.key_decisions {
            if self.matches_convention_pattern(decision) {
                memory_store
                    .append(
                        "conventions.md",
                        &json!({
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "session_id": session_id,
                            "line": decision,
                        }),
                    )
                    .await?;
                result.conventions += 1;
            }
        }

        let total_writes = result.code_changes + result.bug_records + result.conventions;
        if total_writes > 0 {
            counter!("context.compress.count", "strategy" => "L3").increment(1);
            info!(
                code_changes = result.code_changes,
                bug_records = result.bug_records,
                conventions = result.conventions,
                "L3 Memory-Compact executed"
            );
        } else {
            warn!("L3 Memory-Compact found no persistent entries");
        }

        Ok(result)
    }

    fn extract_code_change(&self, work_item: &str, session_id: &str) -> Option<serde_json::Value> {
        let captures = file_pattern()
            .captures_iter(work_item)
            .map(|capture| capture[1].to_string())
            .collect::<Vec<_>>();
        if captures.is_empty() {
            return None;
        }

        Some(json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "session_id": session_id,
            "files": captures,
            "description": work_item,
        }))
    }

    fn extract_bug_record(&self, error: &str, session_id: &str) -> Option<serde_json::Value> {
        if error.len() < 10 {
            return None;
        }

        Some(json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "session_id": session_id,
            "error": error,
            "status": "recorded",
        }))
    }

    fn matches_convention_pattern(&self, text: &str) -> bool {
        self.extraction_rules.iter().any(|rule| {
            rule.extractor_type == ExtractorType::Convention
                && Regex::new(&rule.pattern)
                    .map(|regex| regex.is_match(text))
                    .unwrap_or(false)
        })
    }
}

fn file_pattern() -> &'static Regex {
    static FILE_PATTERN: OnceLock<Regex> = OnceLock::new();
    FILE_PATTERN.get_or_init(|| Regex::new(r"([\w./-]+\.\w+)").expect("valid regex"))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use async_trait::async_trait;
    use regex::Regex;
    use tokio::sync::Mutex;

    use super::{file_pattern, MemoryCompactor};
    use crate::compression::{CompactSummary, MemoryStore};

    #[derive(Default)]
    struct MockStore {
        writes: Mutex<HashMap<String, Vec<serde_json::Value>>>,
    }

    #[async_trait]
    impl MemoryStore for MockStore {
        async fn append(&self, path: &str, entry: &serde_json::Value) -> anyhow::Result<()> {
            self.writes
                .lock()
                .await
                .entry(path.to_string())
                .or_default()
                .push(entry.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn l3_extracts_code_change_bug_and_convention() {
        let store = Arc::new(MockStore::default());
        let compactor = MemoryCompactor::default().with_memory_store(store.clone());
        let summary = CompactSummary {
            user_request: "Implement context".into(),
            completed_work: vec!["Updated src/lib.rs and Cargo.toml".into()],
            current_state: "done".into(),
            key_decisions: vec!["统一使用 tokio::fs 处理 I/O".into()],
            error_info: vec!["compile error: missing import".into()],
            continuation_context: "run tests".into(),
        };

        let result = compactor
            .extract_and_persist(&summary, "session-1")
            .await
            .unwrap();
        assert_eq!(result.code_changes, 1);
        assert_eq!(result.bug_records, 1);
        assert_eq!(result.conventions, 1);
    }

    #[test]
    fn file_pattern_is_cached_with_oncelock() {
        let first = file_pattern() as *const Regex;
        let second = file_pattern() as *const Regex;
        assert_eq!(first, second);
    }
}
