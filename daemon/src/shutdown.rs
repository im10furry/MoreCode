use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::DaemonError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShutdownOutcome {
    Completed,
    TimedOut,
}

#[derive(Debug, Clone)]
pub struct ShutdownCoordinator {
    token: CancellationToken,
}

impl ShutdownCoordinator {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
        }
    }

    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    pub fn request_shutdown(&self) {
        self.token.cancel();
    }

    pub async fn wait_for_shutdown(
        &self,
        timeout: Duration,
    ) -> Result<ShutdownOutcome, DaemonError> {
        if self.token.is_cancelled() {
            return Ok(ShutdownOutcome::Completed);
        }

        match tokio::time::timeout(timeout, self.token.cancelled()).await {
            Ok(_) => Ok(ShutdownOutcome::Completed),
            Err(_) => Ok(ShutdownOutcome::TimedOut),
        }
    }
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{ShutdownCoordinator, ShutdownOutcome};

    #[tokio::test]
    async fn shutdown_coordinator_reports_completion() {
        let coordinator = ShutdownCoordinator::new();
        coordinator.request_shutdown();
        let outcome = coordinator
            .wait_for_shutdown(Duration::from_millis(10))
            .await
            .unwrap();
        assert_eq!(outcome, ShutdownOutcome::Completed);
    }
}
