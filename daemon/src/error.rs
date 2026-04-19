use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("daemon is already running")]
    AlreadyRunning,
    #[error("daemon is not running")]
    NotRunning,
    #[error("invalid daemon state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },
    #[error("pid file error at {path}: {reason}")]
    PidFile { path: PathBuf, reason: String },
    #[error("health check failed: {0}")]
    HealthCheck(String),
    #[error("auto update check failed: {0}")]
    AutoUpdate(String),
    #[error("shutdown failed: {0}")]
    Shutdown(String),
    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),
}
