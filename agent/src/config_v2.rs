use std::collections::HashMap;

use mc_core::AgentType;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::AgentError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub timeout_ms: u64,
    pub max_parallel_tasks: usize,
    pub max_recursion_depth: u8,
    pub max_tool_calls: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_directory: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tool_overrides: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub template_overrides: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Value>,
}

impl AgentConfig {
    pub fn for_agent_type(agent_type: AgentType) -> Self {
        match agent_type {
            AgentType::Research => Self {
                agent_type,
                timeout_ms: 30_000,
                max_parallel_tasks: 4,
                max_recursion_depth: 3,
                max_tool_calls: 16,
                output_directory: None,
                tool_overrides: HashMap::new(),
                template_overrides: HashMap::new(),
                metadata: HashMap::new(),
            },
            AgentType::DocWriter => Self {
                agent_type,
                timeout_ms: 20_000,
                max_parallel_tasks: 4,
                max_recursion_depth: 1,
                max_tool_calls: 8,
                output_directory: Some("docs".to_string()),
                tool_overrides: HashMap::new(),
                template_overrides: HashMap::new(),
                metadata: HashMap::new(),
            },
            AgentType::Debugger => Self {
                agent_type,
                timeout_ms: 30_000,
                max_parallel_tasks: 6,
                max_recursion_depth: 3,
                max_tool_calls: 12,
                output_directory: None,
                tool_overrides: HashMap::new(),
                template_overrides: HashMap::new(),
                metadata: HashMap::new(),
            },
            _ => Self {
                agent_type,
                timeout_ms: 15_000,
                max_parallel_tasks: 2,
                max_recursion_depth: 1,
                max_tool_calls: 6,
                output_directory: None,
                tool_overrides: HashMap::new(),
                template_overrides: HashMap::new(),
                metadata: HashMap::new(),
            },
        }
    }

    pub fn with_output_directory(mut self, directory: impl Into<String>) -> Self {
        self.output_directory = Some(directory.into());
        self
    }

    pub fn with_tool_override(
        mut self,
        logical_name: impl Into<String>,
        actual_name: impl Into<String>,
    ) -> Self {
        self.tool_overrides
            .insert(logical_name.into(), actual_name.into());
        self
    }

    pub fn with_template_override(
        mut self,
        template_name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        self.template_overrides
            .insert(template_name.into(), content.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    pub fn from_toml_str(contents: &str) -> Result<Self, AgentError> {
        toml::from_str(contents).map_err(|error| AgentError::Config {
            message: error.to_string(),
        })
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self::for_agent_type(AgentType::Research)
    }
}
