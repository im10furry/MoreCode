#![forbid(unsafe_code)]

pub mod auto_update;
pub mod backoff;
pub mod daemon;
pub mod error;
pub mod health;
pub mod lifecycle;
pub mod pid;
pub mod shutdown;
pub mod taskpile;

pub use auto_update::{AutoUpdateCheck, AutoUpdateStatus};
pub use backoff::ExponentialBackoff;
pub use daemon::{DaemonController, DaemonRuntime, DaemonState, DaemonStatusSnapshot};
pub use error::{DaemonError, TaskPileError, TaskPileResult};
pub use health::{ComponentHealth, DaemonHealth, HealthState};
pub use lifecycle::DaemonLifecycle;
pub use pid::PidFileGuard;
pub use shutdown::{ShutdownCoordinator, ShutdownOutcome};
pub use taskpile::{
    ApprovalMode, CloudAdapterStatus, CloudPayload, CompressionMode, ExecutionOptions,
    IsolationProfile, NewTaskRequest, NoopCloudAdapter, TaskPilePriority, TaskPileSchedule,
    TaskPileService, TaskPileStats, TaskPileStatus, TaskPileTask, TaskTarget, TokenControls,
};
