use chrono::{DateTime, Utc};

/// Return the current UTC timestamp.
#[inline]
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Format milliseconds into a compact human-readable duration.
pub fn format_duration(duration_ms: u64) -> String {
    if duration_ms < 1_000 {
        format!("{duration_ms}ms")
    } else if duration_ms < 60_000 {
        format!("{:.1}s", duration_ms as f64 / 1_000.0)
    } else if duration_ms < 3_600_000 {
        let minutes = duration_ms / 60_000;
        let seconds = (duration_ms % 60_000) / 1_000;
        format!("{minutes}m {seconds}s")
    } else {
        let hours = duration_ms / 3_600_000;
        let minutes = (duration_ms % 3_600_000) / 60_000;
        let seconds = (duration_ms % 60_000) / 1_000;
        format!("{hours}h {minutes}m {seconds}s")
    }
}
