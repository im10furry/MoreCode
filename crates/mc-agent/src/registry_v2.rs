use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use mc_core::AgentType;

use crate::{Agent, AgentCapabilities};

pub type AgentFactory = Arc<dyn Fn() -> Box<dyn Agent> + Send + Sync>;

pub struct AgentRegistry {
    factories: RwLock<HashMap<AgentType, AgentFactory>>,
    capabilities: RwLock<HashMap<AgentType, AgentCapabilities>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            factories: RwLock::new(HashMap::new()),
            capabilities: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_factory(
        &self,
        agent_type: AgentType,
        capabilities: AgentCapabilities,
        factory: AgentFactory,
    ) {
        self.factories
            .write()
            .expect("agent factory lock poisoned")
            .insert(agent_type, factory);
        self.capabilities
            .write()
            .expect("agent capabilities lock poisoned")
            .insert(agent_type, capabilities);
    }

    pub fn create(&self, agent_type: AgentType) -> Option<Box<dyn Agent>> {
        let factory = self
            .factories
            .read()
            .expect("agent factory lock poisoned")
            .get(&agent_type)
            .cloned()?;
        Some(factory())
    }

    pub fn capabilities(&self, agent_type: AgentType) -> Option<AgentCapabilities> {
        self.capabilities
            .read()
            .expect("agent capabilities lock poisoned")
            .get(&agent_type)
            .cloned()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
