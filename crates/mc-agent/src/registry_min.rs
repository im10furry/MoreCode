use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use mc_core::AgentType;

use crate::trait_def_min::{Agent, AgentError, DefaultAgent};

type Factory = dyn Fn() -> Arc<dyn Agent> + Send + Sync;

#[derive(Default)]
pub struct AgentRegistry {
    factories: RwLock<HashMap<AgentType, Arc<Factory>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<F>(&self, agent_type: AgentType, factory: F) -> Result<(), AgentError>
    where
        F: Fn() -> Arc<dyn Agent> + Send + Sync + 'static,
    {
        let mut factories = self
            .factories
            .write()
            .map_err(|_| AgentError::Internal("agent registry lock poisoned".into()))?;

        if factories.contains_key(&agent_type) {
            return Err(AgentError::Internal(format!(
                "agent already registered: {}",
                agent_type.as_str()
            )));
        }

        factories.insert(agent_type, Arc::new(factory));
        Ok(())
    }

    pub fn register_defaults(&self) {
        for agent_type in AgentType::ALL {
            if agent_type == AgentType::Coordinator || self.is_registered(&agent_type) {
                continue;
            }

            let _ = self.register(agent_type, move || Arc::new(DefaultAgent::new(agent_type)));
        }
    }

    pub fn create_agent(&self, agent_type: AgentType) -> Result<Arc<dyn Agent>, AgentError> {
        let factories = self
            .factories
            .read()
            .map_err(|_| AgentError::Internal("agent registry lock poisoned".into()))?;

        factories
            .get(&agent_type)
            .map(|factory| factory())
            .ok_or(AgentError::AgentNotFound(agent_type))
    }

    pub fn list_types(&self) -> Vec<AgentType> {
        let mut types = self
            .factories
            .read()
            .map(|guard| guard.keys().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        types.sort_by_key(|agent_type| agent_type.as_str());
        types
    }

    pub fn is_registered(&self, agent_type: &AgentType) -> bool {
        self.factories
            .read()
            .map(|guard| guard.contains_key(agent_type))
            .unwrap_or(false)
    }
}
