use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::store::MemoryUpdate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryUpdateKind {
    FileModified,
    FileAdded,
    FileDeleted,
    ApiAdded,
    ApiRemoved,
    DataModelChanged,
    RiskDiscovered,
    RiskResolved,
    AgentNote,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryHistoryEntry {
    pub kind: MemoryUpdateKind,
    pub actor: String,
    pub summary: String,
    pub recorded_at: DateTime<Utc>,
    pub payload: MemoryUpdate,
}

impl MemoryHistoryEntry {
    pub fn new(payload: MemoryUpdate, actor: impl Into<String>) -> Self {
        let actor = actor.into();
        let summary = summarize_update(&payload);
        Self {
            kind: kind_of(&payload),
            actor,
            summary,
            recorded_at: Utc::now(),
            payload,
        }
    }
}

fn kind_of(update: &MemoryUpdate) -> MemoryUpdateKind {
    match update {
        MemoryUpdate::FileModified { .. } => MemoryUpdateKind::FileModified,
        MemoryUpdate::FileAdded { .. } => MemoryUpdateKind::FileAdded,
        MemoryUpdate::FileDeleted { .. } => MemoryUpdateKind::FileDeleted,
        MemoryUpdate::ApiAdded { .. } => MemoryUpdateKind::ApiAdded,
        MemoryUpdate::ApiRemoved { .. } => MemoryUpdateKind::ApiRemoved,
        MemoryUpdate::DataModelChanged { .. } => MemoryUpdateKind::DataModelChanged,
        MemoryUpdate::RiskDiscovered { .. } => MemoryUpdateKind::RiskDiscovered,
        MemoryUpdate::RiskResolved { .. } => MemoryUpdateKind::RiskResolved,
        MemoryUpdate::AgentNote { .. } => MemoryUpdateKind::AgentNote,
    }
}

fn summarize_update(update: &MemoryUpdate) -> String {
    match update {
        MemoryUpdate::FileModified { path, summary } => format!("modified {path}: {summary}"),
        MemoryUpdate::FileAdded { path, module_name } => {
            format!(
                "added {path} ({})",
                module_name
                    .clone()
                    .unwrap_or_else(|| "unknown module".into())
            )
        }
        MemoryUpdate::FileDeleted { path, module_name } => {
            format!(
                "deleted {path} ({})",
                module_name
                    .clone()
                    .unwrap_or_else(|| "unknown module".into())
            )
        }
        MemoryUpdate::ApiAdded { endpoint } => {
            format!("added API {} {}", endpoint.method, endpoint.path)
        }
        MemoryUpdate::ApiRemoved { method, path } => format!("removed API {method} {path}"),
        MemoryUpdate::DataModelChanged { model, change_type } => {
            format!("data model {model} changed: {change_type}")
        }
        MemoryUpdate::RiskDiscovered { area, severity, .. } => {
            format!("risk discovered at {area} ({severity})")
        }
        MemoryUpdate::RiskResolved { area } => format!("risk resolved at {area}"),
        MemoryUpdate::AgentNote { agent, topic, .. } => {
            format!("agent note recorded for {agent}/{topic}")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::store::{ApiEndpoint, MemoryUpdate};

    use super::{MemoryHistoryEntry, MemoryUpdateKind};

    #[test]
    fn history_entry_tracks_kind_and_summary() {
        let entry = MemoryHistoryEntry::new(
            MemoryUpdate::ApiAdded {
                endpoint: ApiEndpoint {
                    method: "POST".into(),
                    path: "/api/users".into(),
                    handler: "UserHandler::create".into(),
                    request_type: Some("CreateUser".into()),
                    response_type: Some("User".into()),
                    auth_required: true,
                },
            },
            "coder",
        );

        assert_eq!(entry.kind, MemoryUpdateKind::ApiAdded);
        assert!(entry.summary.contains("/api/users"));
        assert_eq!(entry.actor, "coder");
    }
}
