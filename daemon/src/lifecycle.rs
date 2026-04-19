use crate::{DaemonError, DaemonState};

#[derive(Debug, Clone, Default)]
pub struct DaemonLifecycle;

impl DaemonLifecycle {
    pub fn can_transition(from: DaemonState, to: DaemonState) -> bool {
        matches!(
            (from, to),
            (DaemonState::Idle, DaemonState::Running)
                | (DaemonState::Running, DaemonState::Paused)
                | (DaemonState::Paused, DaemonState::Running)
                | (DaemonState::Running, DaemonState::ShuttingDown)
                | (DaemonState::Paused, DaemonState::ShuttingDown)
                | (DaemonState::ShuttingDown, DaemonState::Idle)
        )
    }

    pub fn ensure_transition(from: DaemonState, to: DaemonState) -> Result<(), DaemonError> {
        if Self::can_transition(from, to) {
            Ok(())
        } else {
            Err(DaemonError::InvalidStateTransition {
                from: format!("{from:?}"),
                to: format!("{to:?}"),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DaemonState;

    use super::DaemonLifecycle;

    #[test]
    fn lifecycle_allows_expected_transitions() {
        assert!(DaemonLifecycle::can_transition(
            DaemonState::Idle,
            DaemonState::Running
        ));
        assert!(!DaemonLifecycle::can_transition(
            DaemonState::Idle,
            DaemonState::Paused
        ));
    }
}
