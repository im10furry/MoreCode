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
        }
    }
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

fn default_pid_file() -> String {
    "/tmp/morecode.pid".to_string()
}

fn default_health_check_interval_secs() -> u64 {
    60
}

fn default_auto_update_check_hours() -> u64 {
    24
}
