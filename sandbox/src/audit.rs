use std::cmp::Reverse;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::RwLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub caller: String,
    pub tool_name: String,
    pub parameters: Value,
    pub decision_result: String,
    pub decision_detail: String,
}

#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub caller: Option<String>,
    pub tool_name: Option<String>,
    pub decision_result: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct AuditLogger {
    entries: RwLock<Vec<AuditEntry>>,
    max_entries: usize,
    log_file: Option<PathBuf>,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            max_entries: 100_000,
            log_file: None,
        }
    }

    pub fn with_log_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.log_file = Some(path.into());
        self
    }

    pub fn log(&self, entry: AuditEntry) {
        if let Ok(mut entries) = self.entries.write() {
            entries.push(entry.clone());
            if entries.len() > self.max_entries {
                let overflow = entries.len().saturating_sub(self.max_entries);
                entries.drain(0..overflow);
            }
        }

        if let Some(log_file) = &self.log_file {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_file) {
                let _ = writeln!(
                    file,
                    "{}",
                    serde_json::to_string(&entry).unwrap_or_else(|_| {
                        format!(
                            "{{\"timestamp\":\"{}\",\"caller\":\"{}\",\"tool_name\":\"{}\",\"decision_result\":\"{}\"}}",
                            entry.timestamp.to_rfc3339(),
                            entry.caller,
                            entry.tool_name,
                            entry.decision_result,
                        )
                    })
                );
            }
        }
    }

    pub fn entries(&self) -> Vec<AuditEntry> {
        self.entries
            .read()
            .map(|entries| entries.clone())
            .unwrap_or_default()
    }

    pub fn query(&self, filter: AuditFilter) -> Vec<AuditEntry> {
        let mut results = self
            .entries()
            .into_iter()
            .filter(|entry| {
                if let Some(caller) = &filter.caller {
                    if entry.caller != *caller {
                        return false;
                    }
                }
                if let Some(tool_name) = &filter.tool_name {
                    if entry.tool_name != *tool_name {
                        return false;
                    }
                }
                if let Some(decision_result) = &filter.decision_result {
                    if entry.decision_result != *decision_result {
                        return false;
                    }
                }
                if let Some(since) = filter.since {
                    if entry.timestamp < since {
                        return false;
                    }
                }
                if let Some(until) = filter.until {
                    if entry.timestamp > until {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<_>>();

        results.sort_by_key(|entry| Reverse(entry.timestamp));
        results
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}
