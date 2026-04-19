use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoUpdateStatus {
    UpToDate,
    UpdateAvailable { latest_version: String },
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoUpdateCheck {
    pub checked_at: DateTime<Utc>,
    pub current_version: String,
    pub status: AutoUpdateStatus,
}

impl AutoUpdateCheck {
    pub fn new(current_version: impl Into<String>, status: AutoUpdateStatus) -> Self {
        Self {
            checked_at: Utc::now(),
            current_version: current_version.into(),
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AutoUpdateCheck, AutoUpdateStatus};

    #[test]
    fn auto_update_check_captures_status() {
        let check = AutoUpdateCheck::new(
            "0.1.0",
            AutoUpdateStatus::UpdateAvailable {
                latest_version: "0.2.0".into(),
            },
        );
        assert_eq!(check.current_version, "0.1.0");
        assert!(matches!(check.status, AutoUpdateStatus::UpdateAvailable { .. }));
    }
}
