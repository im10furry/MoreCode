use mc_core::{AgentLayer, AgentType};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::context::SharedResources;
use crate::error::AgentError;
use crate::trait_def::{Agent, AgentCapabilities, AgentConfig};

pub type AgentFactory =
    Arc<dyn Fn(&SharedResources, &AgentConfig) -> Box<dyn Agent> + Send + Sync + 'static>;
pub type SharedAgentHandle = Arc<RwLock<Box<dyn Agent>>>;

pub struct AgentRegistry {
    factories: RwLock<HashMap<AgentType, AgentFactory>>,
    capabilities: RwLock<HashMap<AgentType, AgentCapabilities>>,
    instances: RwLock<HashMap<AgentType, SharedAgentHandle>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            factories: RwLock::new(HashMap::new()),
            capabilities: RwLock::new(HashMap::new()),
            instances: RwLock::new(HashMap::new()),
        }
    }

    pub fn register<F>(&self, agent_type: AgentType, factory: F) -> Result<(), AgentError>
    where
        F: Fn(&SharedResources, &AgentConfig) -> Box<dyn Agent> + Send + Sync + 'static,
    {
        let mut factories = self
            .factories
            .write()
            .map_err(|_| AgentError::lock_poisoned("agent_factories"))?;

        if factories.contains_key(&agent_type) {
            return Err(AgentError::DuplicateRegistration {
                agent_type,
                message: format!("agent {} is already registered", agent_type.identifier()),
            });
        }

        factories.insert(agent_type, Arc::new(factory));
        Ok(())
    }

    pub fn create_agent(
        &self,
        agent_type: &AgentType,
        shared: &SharedResources,
        config: &AgentConfig,
    ) -> Result<Box<dyn Agent>, AgentError> {
        let factory = self
            .factories
            .read()
            .map_err(|_| AgentError::lock_poisoned("agent_factories"))?
            .get(agent_type)
            .cloned()
            .ok_or_else(|| AgentError::AgentNotFound {
                agent_type: *agent_type,
                message: format!("no factory registered for {}", agent_type.identifier()),
            })?;

        let agent = factory(shared, config);
        self.capabilities
            .write()
            .map_err(|_| AgentError::lock_poisoned("agent_capabilities"))?
            .insert(*agent_type, agent.capabilities());
        Ok(agent)
    }

    pub fn get(
        &self,
        agent_type: &AgentType,
        shared: &SharedResources,
        config: &AgentConfig,
    ) -> Result<SharedAgentHandle, AgentError> {
        if let Some(instance) = self
            .instances
            .read()
            .map_err(|_| AgentError::lock_poisoned("agent_instances"))?
            .get(agent_type)
            .cloned()
        {
            return Ok(instance);
        }

        let agent = self.create_agent(agent_type, shared, config)?;
        let handle = Arc::new(RwLock::new(agent));
        self.instances
            .write()
            .map_err(|_| AgentError::lock_poisoned("agent_instances"))?
            .insert(*agent_type, handle.clone());
        Ok(handle)
    }

    pub fn list_all(&self) -> Vec<AgentType> {
        let mut items = self
            .factories
            .read()
            .map(|factories| factories.keys().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        items.sort_by_key(|agent_type| agent_type.identifier());
        items
    }

    pub fn list_by_layer(&self, layer: AgentLayer) -> Vec<AgentType> {
        self.list_all()
            .into_iter()
            .filter(|agent_type| agent_type.layer() == layer)
            .collect()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
