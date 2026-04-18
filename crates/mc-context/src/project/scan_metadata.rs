use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanMetadata {
    pub scanned_at: DateTime<Utc>,
    pub scan_duration_ms: u64,
    pub scanner_version: Option<String>,
}

impl Default for ScanMetadata {
    fn default() -> Self {
        Self {
            scanned_at: Utc::now(),
            scan_duration_ms: 0,
            scanner_version: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ScanMetadata;

    #[test]
    fn scan_duration_is_non_negative() {
        let metadata = ScanMetadata::default();
        assert_eq!(metadata.scan_duration_ms, 0);
    }
}
