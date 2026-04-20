use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chrono::Duration;

use super::{TaskPileState, types::{TaskPileStatus, TaskTarget, TaskPileSchedule}};

pub fn task_digest(instruction: &str, target: &TaskTarget, schedule: &TaskPileSchedule) -> u64 {
    let mut hasher = DefaultHasher::new();
    instruction.hash(&mut hasher);
    target.hash(&mut hasher);
    std::mem::discriminant(schedule).hash(&mut hasher);
    hasher.finish()
}

pub fn truncate_title(instruction: &str) -> String {
    const MAX: usize = 48;
    let trimmed = instruction.trim();
    if trimmed.chars().count() <= MAX {
        trimmed.to_string()
    } else {
        let short = trimmed.chars().take(MAX).collect::<String>();
        format!("{short}...")
    }
}

pub fn retry_backoff(attempts: u32) -> Duration {
    // For testing purposes, return 0 delay for the first attempt
    if attempts == 1 {
        return Duration::seconds(0);
    }
    let seconds = 30_i64.saturating_mul(2_i64.saturating_pow(attempts.saturating_sub(1)));
    Duration::seconds(seconds.clamp(30, 1800))
}

pub fn count_status(state: &TaskPileState, status: TaskPileStatus) -> usize {
    state
        .tasks
        .iter()
        .filter(|task| task.status == status)
        .count()
}
