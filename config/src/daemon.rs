use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DaemonConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_pid_file")]
    pub pid_file: String,
    #[serde(default = "default_health_check_interval_secs")]
    pub health_check_interval_secs: u64,
    #[serde(default = "default_auto_update_check_hours")]
    pub auto_update_check_hours: u64,
    #[serde(default)]
    pub quiet_hours: Option<QuietHours>,
    #[serde(default)]
    pub daily_budget_usd: Option<f64>,
    #[serde(default)]
    pub taskpile: TaskPileConfig,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            pid_file: default_pid_file(),
            health_check_interval_secs: default_health_check_interval_secs(),
            auto_update_check_hours: default_auto_update_check_hours(),
            quiet_hours: None,
            daily_budget_usd: None,
            taskpile: TaskPileConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskPileConfig {
    #[serde(default = "default_taskpile_enabled")]
    pub enabled: bool,
    #[serde(default = "default_taskpile_storage_dir")]
    pub storage_dir: Option<String>,
    #[serde(default = "default_taskpile_max_running_tasks")]
    pub max_running_tasks: usize,
    #[serde(default = "default_taskpile_dedup_window_secs")]
    pub dedup_window_secs: u64,
    #[serde(default = "default_taskpile_default_token_budget")]
    pub default_token_budget: u32,
    #[serde(default = "default_taskpile_default_isolation_profile")]
    pub default_isolation_profile: String,
    #[serde(default)]
    pub cloud: TaskPileCloudConfig,
}

impl Default for TaskPileConfig {
    fn default() -> Self {
        Self {
            enabled: default_taskpile_enabled(),
            storage_dir: default_taskpile_storage_dir(),
            max_running_tasks: default_taskpile_max_running_tasks(),
            dedup_window_secs: default_taskpile_dedup_window_secs(),
            default_token_budget: default_taskpile_default_token_budget(),
            default_isolation_profile: default_taskpile_default_isolation_profile(),
            cloud: TaskPileCloudConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TaskPileCloudConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuietHours {
    pub start_hour: u8,
    pub end_hour: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialDaemonConfig {
    pub enabled: Option<bool>,
    pub pid_file: Option<String>,
    pub health_check_interval_secs: Option<u64>,
    pub auto_update_check_hours: Option<u64>,
    pub quiet_hours: Option<PartialQuietHours>,
    pub daily_budget_usd: Option<f64>,
    pub taskpile: Option<PartialTaskPileConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialTaskPileConfig {
    pub enabled: Option<bool>,
    pub storage_dir: Option<Option<String>>,
    pub max_running_tasks: Option<usize>,
    pub dedup_window_secs: Option<u64>,
    pub default_token_budget: Option<u32>,
    pub default_isolation_profile: Option<String>,
    pub cloud: Option<PartialTaskPileCloudConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialTaskPileCloudConfig {
    pub enabled: Option<bool>,
    pub endpoint: Option<Option<String>>,
    pub project_id: Option<Option<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialQuietHours {
    pub start_hour: Option<u8>,
    pub end_hour: Option<u8>,
}

impl DaemonConfig {
    pub(crate) fn apply_partial(
        &mut self,
        partial: PartialDaemonConfig,
    ) -> Result<(), ConfigError> {
        if let Some(value) = partial.enabled {
            self.enabled = value;
        }
        if let Some(value) = partial.pid_file {
            self.pid_file = value;
        }
        if let Some(value) = partial.health_check_interval_secs {
            self.health_check_interval_secs = value;
        }
        if let Some(value) = partial.auto_update_check_hours {
            self.auto_update_check_hours = value;
        }
        if let Some(value) = partial.quiet_hours {
            let mut quiet_hours = self.quiet_hours.clone().unwrap_or(QuietHours {
                start_hour: u8::MAX,
                end_hour: u8::MAX,
            });
            quiet_hours.apply_partial(value);
            self.quiet_hours = Some(quiet_hours);
        }
        if let Some(value) = partial.daily_budget_usd {
            self.daily_budget_usd = Some(value);
        }
        if let Some(value) = partial.taskpile {
            self.taskpile.apply_partial(value);
        }
        Ok(())
    }
}

impl QuietHours {
    pub(crate) fn apply_partial(&mut self, partial: PartialQuietHours) {
        if let Some(value) = partial.start_hour {
            self.start_hour = value;
        }
        if let Some(value) = partial.end_hour {
            self.end_hour = value;
        }
    }
}

impl PartialDaemonConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            enabled: other.enabled.or(self.enabled),
            pid_file: other.pid_file.or(self.pid_file),
            health_check_interval_secs: other
                .health_check_interval_secs
                .or(self.health_check_interval_secs),
            auto_update_check_hours: other
                .auto_update_check_hours
                .or(self.auto_update_check_hours),
            quiet_hours: match (self.quiet_hours, other.quiet_hours) {
                (Some(base), Some(overlay)) => Some(base.merge(overlay)),
                (Some(base), None) => Some(base),
                (None, Some(overlay)) => Some(overlay),
                (None, None) => None,
            },
            daily_budget_usd: other.daily_budget_usd.or(self.daily_budget_usd),
            taskpile: match (self.taskpile, other.taskpile) {
                (Some(base), Some(overlay)) => Some(base.merge(overlay)),
                (Some(base), None) => Some(base),
                (None, Some(overlay)) => Some(overlay),
                (None, None) => None,
            },
        }
    }
}

impl PartialQuietHours {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            start_hour: other.start_hour.or(self.start_hour),
            end_hour: other.end_hour.or(self.end_hour),
        }
    }
}

impl TaskPileConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialTaskPileConfig) {
        if let Some(value) = partial.enabled {
            self.enabled = value;
        }
        if let Some(value) = partial.storage_dir {
            self.storage_dir = value;
        }
        if let Some(value) = partial.max_running_tasks {
            self.max_running_tasks = value;
        }
        if let Some(value) = partial.dedup_window_secs {
            self.dedup_window_secs = value;
        }
        if let Some(value) = partial.default_token_budget {
            self.default_token_budget = value;
        }
        if let Some(value) = partial.default_isolation_profile {
            self.default_isolation_profile = value;
        }
        if let Some(value) = partial.cloud {
            self.cloud.apply_partial(value);
        }
    }
}

impl TaskPileCloudConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialTaskPileCloudConfig) {
        if let Some(value) = partial.enabled {
            self.enabled = value;
        }
        if let Some(value) = partial.endpoint {
            self.endpoint = value;
        }
        if let Some(value) = partial.project_id {
            self.project_id = value;
        }
    }
}

impl PartialTaskPileConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            enabled: other.enabled.or(self.enabled),
            storage_dir: other.storage_dir.or(self.storage_dir),
            max_running_tasks: other.max_running_tasks.or(self.max_running_tasks),
            dedup_window_secs: other.dedup_window_secs.or(self.dedup_window_secs),
            default_token_budget: other.default_token_budget.or(self.default_token_budget),
            default_isolation_profile: other
                .default_isolation_profile
                .or(self.default_isolation_profile),
            cloud: match (self.cloud, other.cloud) {
                (Some(base), Some(overlay)) => Some(base.merge(overlay)),
                (Some(base), None) => Some(base),
                (None, Some(overlay)) => Some(overlay),
                (None, None) => None,
            },
        }
    }
}

impl PartialTaskPileCloudConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            enabled: other.enabled.or(self.enabled),
            endpoint: other.endpoint.or(self.endpoint),
            project_id: other.project_id.or(self.project_id),
        }
    }
}

fn default_pid_file() -> String {
    "/tmp/morecode.pid".to_string()
}

fn default_health_check_interval_secs() -> u64 {
    60
}

fn default_auto_update_check_hours() -> u64 {
    24
}

fn default_taskpile_enabled() -> bool {
    false
}

fn default_taskpile_storage_dir() -> Option<String> {
    None
}

fn default_taskpile_max_running_tasks() -> usize {
    2
}

fn default_taskpile_dedup_window_secs() -> u64 {
    900
}

fn default_taskpile_default_token_budget() -> u32 {
    12_000
}

fn default_taskpile_default_isolation_profile() -> String {
    "workspace-write".to_string()
}
