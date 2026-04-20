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

pub type TaskPileResult<T> = Result<T, TaskPileError>;

#[derive(Debug, Error)]
pub enum TaskPileError {
    #[error("failed to create taskpile storage directory at {path}: {reason}")]
    CreateStorageDir { path: PathBuf, reason: String },
    #[error("failed to read taskpile state from {path}: {reason}")]
    ReadState { path: PathBuf, reason: String },
    #[error("failed to parse taskpile state from {path}: {reason}")]
    ParseState { path: PathBuf, reason: String },
    #[error("failed to write taskpile state to {path}: {reason}")]
    WriteState { path: PathBuf, reason: String },
    #[error("task already exists: {existing_id}")]
    DuplicateTask { existing_id: String },
    #[error("task not found: {task_id}")]
    TaskNotFound { task_id: String },
    #[error("task is not due: {task_id}")]
    TaskNotDue { task_id: String },
    #[error("invalid task status for {task_id}: {status}")]
    InvalidStatus { task_id: String, status: String },
    #[error("running task limit reached: {current}/{limit}")]
    RunningLimitReached { current: usize, limit: usize },
    #[error("cloud adapter is unavailable")]
    CloudAdapterUnavailable,
    #[error("invalid taskpile option: {0}")]
    InvalidOption(String),
    #[error("invalid taskpile schedule: {0}")]
    InvalidSchedule(String),
}
