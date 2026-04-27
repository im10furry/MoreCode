use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc, Datelike, Timelike};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskPileStatus {
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl TaskPileStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPilePriority {
    Low,
    Normal,
    High,
    Critical,
}

impl TaskPilePriority {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "low" => Some(Self::Low),
            "normal" | "medium" => Some(Self::Normal),
            "high" => Some(Self::High),
            "critical" | "urgent" => Some(Self::Critical),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskTarget {
    Local,
    Cloud,
}

impl TaskTarget {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "local" => Some(Self::Local),
            "cloud" => Some(Self::Cloud),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IsolationProfile {
    WorkspaceWrite,
    ReadOnly,
    FullAccess,
    CloudWorker,
    Custom(String),
}

impl IsolationProfile {
    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "workspace-write" => Self::WorkspaceWrite,
            "read-only" | "readonly" => Self::ReadOnly,
            "full-access" => Self::FullAccess,
            "cloud-worker" => Self::CloudWorker,
            other => Self::Custom(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::WorkspaceWrite => "workspace-write",
            Self::ReadOnly => "read-only",
            Self::FullAccess => "full-access",
            Self::CloudWorker => "cloud-worker",
            Self::Custom(value) => value.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressionMode {
    Aggressive,
    Balanced,
    Off,
}

impl CompressionMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "aggressive" => Some(Self::Aggressive),
            "balanced" => Some(Self::Balanced),
            "off" | "none" => Some(Self::Off),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalMode {
    Auto,
    Always,
    Never,
}

impl ApprovalMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "always" => Some(Self::Always),
            "never" => Some(Self::Never),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskPileSchedule {
    Manual,
    At(DateTime<Utc>),
    IntervalSeconds(u64),
    Cron(String),
    WorkdayOnly {
        hour: u32,
        minute: u32,
    },
    WeekendOnly {
        hour: u32,
        minute: u32,
    },
}

impl TaskPileSchedule {
    pub fn next_run_at(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Self::Manual => Some(now),
            Self::At(at) => Some(*at),
            Self::IntervalSeconds(seconds) => Some(now + Duration::seconds(*seconds as i64)),
            Self::Cron(_cron) => {
                // In a real implementation, we would parse the cron expression
                // and calculate the next run time. For now, we'll return the next hour.
                Some(now + Duration::hours(1))
            },
            Self::WorkdayOnly { hour, minute } => {
                let mut next = now;
                // Limit to 7 days to avoid infinite loop
                for _ in 0..7 {
                    next += Duration::days(1);
                    let weekday = next.weekday();
                    if weekday != chrono::Weekday::Sat && weekday != chrono::Weekday::Sun {
                        return Some(next.with_hour(*hour).unwrap().with_minute(*minute).unwrap());
                    }
                }
                // Fallback if no workday found within 7 days (should never happen)
                Some(now + Duration::days(1))
            },
            Self::WeekendOnly { hour, minute } => {
                let mut next = now;
                // Limit to 7 days to avoid infinite loop
                for _ in 0..7 {
                    next += Duration::days(1);
                    let weekday = next.weekday();
                    if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
                        return Some(next.with_hour(*hour).unwrap().with_minute(*minute).unwrap());
                    }
                }
                // Fallback if no weekend found within 7 days (should never happen)
                Some(now + Duration::days(1))
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenControls {
    pub budget: u32,
    pub compression: CompressionMode,
    pub summary_depth: u8,
    pub allow_cache_reuse: bool,
    pub cache_namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionOptions {
    pub target: TaskTarget,
    pub model: Option<String>,
    pub parallelism: u8,
    pub approval: ApprovalMode,
    pub isolation: IsolationProfile,
    pub token_controls: TokenControls,
    pub cloud_endpoint: Option<String>,
    pub cloud_project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskPileTask {
    pub id: String,
    pub title: String,
    pub instruction: String,
    pub status: TaskPileStatus,
    pub priority: TaskPilePriority,
    pub schedule: TaskPileSchedule,
    pub execution: ExecutionOptions,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_claimed_at: Option<DateTime<Utc>>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub attempts: u32,
    pub max_attempts: u32,
    pub last_error: Option<String>,
    pub result_summary: Option<String>,
    pub origin: String,
}

impl TaskPileTask {
    pub fn due_at(&self, now: DateTime<Utc>) -> bool {
        matches!(self.status, TaskPileStatus::Queued)
            && self
                .next_run_at
                .as_ref()
                .map(|at| at <= &now)
                .unwrap_or(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewTaskRequest {
    pub title: Option<String>,
    pub instruction: String,
    pub priority: TaskPilePriority,
    pub schedule: TaskPileSchedule,
    pub target: TaskTarget,
    pub isolation: IsolationProfile,
    pub token_budget: u32,
    pub compression: CompressionMode,
    pub parallelism: u8,
    pub approval: ApprovalMode,
    pub max_attempts: u32,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub model: Option<String>,
    pub cloud_endpoint: Option<String>,
    pub cloud_project_id: Option<String>,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskPileStats {
    pub total: usize,
    pub queued: usize,
    pub running: usize,
    pub paused: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub next_due_at: Option<DateTime<Utc>>,
    pub storage_path: String,
    pub cloud_ready: bool,
}
