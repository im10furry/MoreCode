use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use mc_config::DaemonConfig;
use tokio::sync::RwLock;

use crate::{
    AutoUpdateCheck, AutoUpdateStatus, ComponentHealth, DaemonError, DaemonHealth, DaemonLifecycle,
    HealthState, PidFileGuard, ShutdownCoordinator, ShutdownOutcome,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonState {
    Idle,
    Running,
    Paused,
    ShuttingDown,
}

#[derive(Debug, Clone)]
pub struct DaemonStatusSnapshot {
    pub state: DaemonState,
    pub started_at: Option<DateTime<Utc>>,
    pub pid_file: String,
    pub health: DaemonHealth,
    pub update: AutoUpdateCheck,
}

pub struct DaemonRuntime {
    config: DaemonConfig,
    state: Arc<RwLock<DaemonState>>,
    started_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    shutdown: ShutdownCoordinator,
    pid_guard: Arc<RwLock<Option<PidFileGuard>>>,
}

impl DaemonRuntime {
    pub fn new(config: DaemonConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(DaemonState::Idle)),
            started_at: Arc::new(RwLock::new(None)),
            shutdown: ShutdownCoordinator::new(),
            pid_guard: Arc::new(RwLock::new(None)),
        }
    }

    pub fn config(&self) -> &DaemonConfig {
        &self.config
    }

    pub fn shutdown_coordinator(&self) -> ShutdownCoordinator {
        self.shutdown.clone()
    }

    pub async fn state(&self) -> DaemonState {
        *self.state.read().await
    }

    pub async fn status_snapshot(&self) -> DaemonStatusSnapshot {
        let state = *self.state.read().await;
        let health = self.health_check().await;
        DaemonStatusSnapshot {
            state,
            started_at: *self.started_at.read().await,
            pid_file: self.config.pid_file.clone(),
            health,
            update: AutoUpdateCheck::new("0.1.0", AutoUpdateStatus::Unknown),
        }
    }

    pub async fn health_check(&self) -> DaemonHealth {
        let pid_state = if PidFileGuard::read_pid(&self.config.pid_file)
            .ok()
            .flatten()
            .is_some()
        {
            HealthState::Healthy
        } else if matches!(*self.state.read().await, DaemonState::Idle) {
            HealthState::Degraded
        } else {
            HealthState::Unhealthy
        };

        DaemonHealth::new(vec![
            ComponentHealth {
                name: "runtime".into(),
                state: match *self.state.read().await {
                    DaemonState::Running | DaemonState::Paused => HealthState::Healthy,
                    DaemonState::Idle => HealthState::Degraded,
                    DaemonState::ShuttingDown => HealthState::Degraded,
                },
                detail: format!("{:?}", *self.state.read().await),
            },
            ComponentHealth {
                name: "pid".into(),
                state: pid_state,
                detail: self.config.pid_file.clone(),
            },
        ])
    }
}

pub struct DaemonController {
    runtime: Arc<DaemonRuntime>,
}

impl DaemonController {
    pub fn new(config: DaemonConfig) -> Self {
        Self {
            runtime: Arc::new(DaemonRuntime::new(config)),
        }
    }

    pub fn runtime(&self) -> Arc<DaemonRuntime> {
        Arc::clone(&self.runtime)
    }

    pub async fn start(&self) -> Result<(), DaemonError> {
        let mut state = self.runtime.state.write().await;
        DaemonLifecycle::ensure_transition(*state, DaemonState::Running)?;
        let pid_guard = PidFileGuard::acquire(&self.runtime.config.pid_file)?;
        *self.runtime.pid_guard.write().await = Some(pid_guard);
        *self.runtime.started_at.write().await = Some(Utc::now());
        *state = DaemonState::Running;
        Ok(())
    }

    pub async fn pause(&self) -> Result<(), DaemonError> {
        let mut state = self.runtime.state.write().await;
        DaemonLifecycle::ensure_transition(*state, DaemonState::Paused)?;
        *state = DaemonState::Paused;
        Ok(())
    }

    pub async fn resume(&self) -> Result<(), DaemonError> {
        let mut state = self.runtime.state.write().await;
        DaemonLifecycle::ensure_transition(*state, DaemonState::Running)?;
        *state = DaemonState::Running;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<ShutdownOutcome, DaemonError> {
        {
            let mut state = self.runtime.state.write().await;
            DaemonLifecycle::ensure_transition(*state, DaemonState::ShuttingDown)?;
            *state = DaemonState::ShuttingDown;
        }

        self.runtime.shutdown.request_shutdown();
        let outcome = self
            .runtime
            .shutdown
            .wait_for_shutdown(Duration::from_millis(10))
            .await?;
        *self.runtime.pid_guard.write().await = None;
        *self.runtime.started_at.write().await = None;
        *self.runtime.state.write().await = DaemonState::Idle;
        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{DaemonController, DaemonState};
    use crate::ShutdownOutcome;

    #[tokio::test]
    async fn daemon_controller_transitions_and_cleans_pid() {
        let temp = tempdir().unwrap();
        let mut config = mc_config::DaemonConfig::default();
        config.pid_file = temp.path().join("daemon.pid").to_string_lossy().to_string();
        let controller = DaemonController::new(config.clone());

        controller.start().await.unwrap();
        assert_eq!(controller.runtime().state().await, DaemonState::Running);
        assert!(std::path::Path::new(&config.pid_file).exists());

        controller.pause().await.unwrap();
        assert_eq!(controller.runtime().state().await, DaemonState::Paused);
        controller.resume().await.unwrap();
        assert_eq!(controller.runtime().state().await, DaemonState::Running);

        let outcome = controller.shutdown().await.unwrap();
        assert_eq!(outcome, ShutdownOutcome::Completed);
        assert_eq!(controller.runtime().state().await, DaemonState::Idle);
        assert!(!std::path::Path::new(&config.pid_file).exists());
    }
}
