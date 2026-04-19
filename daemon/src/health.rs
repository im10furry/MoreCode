use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentHealth {
    pub name: String,
    pub state: HealthState,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonHealth {
    pub state: HealthState,
    pub checked_at: DateTime<Utc>,
    pub components: Vec<ComponentHealth>,
}

impl DaemonHealth {
    pub fn new(components: Vec<ComponentHealth>) -> Self {
        let state = components
            .iter()
            .fold(HealthState::Healthy, |acc, component| match (acc, component.state) {
                (HealthState::Unhealthy, _) | (_, HealthState::Unhealthy) => HealthState::Unhealthy,
                (HealthState::Degraded, _) | (_, HealthState::Degraded) => HealthState::Degraded,
                _ => HealthState::Healthy,
            });

        Self {
            state,
            checked_at: Utc::now(),
            components,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentHealth, DaemonHealth, HealthState};

    #[test]
    fn daemon_health_rolls_up_component_states() {
        let degraded = DaemonHealth::new(vec![
            ComponentHealth {
                name: "queue".into(),
                state: HealthState::Healthy,
                detail: "ok".into(),
            },
            ComponentHealth {
                name: "llm".into(),
                state: HealthState::Degraded,
                detail: "slow".into(),
            },
        ]);
        assert_eq!(degraded.state, HealthState::Degraded);
    }
}
