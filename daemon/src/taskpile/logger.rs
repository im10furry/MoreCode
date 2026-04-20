use std::sync::OnceLock;
use tracing::{info, warn, error, debug};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

static LOGGER_INITIALIZED: OnceLock<()> = OnceLock::new();

pub fn init_logger() {
    LOGGER_INITIALIZED.get_or_init(|| {
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true);

        tracing_subscriber::registry()
            .with(fmt_layer)
            .init();

        info!("TaskPile logger initialized");
    });
}

pub fn log_task_creation(task_id: &str, title: &str) {
    info!(task_id = task_id, title = title, "Task created");
}

pub fn log_task_claim(task_id: &str, title: &str) {
    info!(task_id = task_id, title = title, "Task claimed");
}

pub fn log_task_completion(task_id: &str, title: &str, summary: &str) {
    info!(task_id = task_id, title = title, summary = summary, "Task completed");
}

pub fn log_task_failure(task_id: &str, title: &str, reason: &str) {
    error!(task_id = task_id, title = title, reason = reason, "Task failed");
}

pub fn log_task_pause(task_id: &str, title: &str) {
    info!(task_id = task_id, title = title, "Task paused");
}

pub fn log_task_resume(task_id: &str, title: &str) {
    info!(task_id = task_id, title = title, "Task resumed");
}

pub fn log_task_cancel(task_id: &str, title: &str) {
    info!(task_id = task_id, title = title, "Task cancelled");
}

pub fn log_error(message: &str, error: &str) {
    error!(message = message, error = error, "Error occurred");
}

pub fn log_warning(message: &str) {
    warn!(message = message, "Warning");
}

pub fn log_debug(message: &str) {
    debug!(message = message, "Debug");
}
