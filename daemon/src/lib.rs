#![forbid(unsafe_code)]

mod auto_update;
mod backoff;
mod daemon;
mod error;
mod health;
mod lifecycle;
mod pid;
mod shutdown;

pub use auto_update::{AutoUpdateCheck, AutoUpdateStatus};
pub use backoff::ExponentialBackoff;
pub use daemon::{DaemonController, DaemonRuntime, DaemonState, DaemonStatusSnapshot};
pub use error::DaemonError;
pub use health::{ComponentHealth, DaemonHealth, HealthState};
pub use lifecycle::DaemonLifecycle;
pub use pid::PidFileGuard;
pub use shutdown::{ShutdownCoordinator, ShutdownOutcome};
