use async_trait::async_trait;
use mc_core::AgentType;
use mc_core::task::{ProjectContext, TaskDescription};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::infra_context::{AgentContext, SharedResources};
use crate::infra_error::AgentError;
use crate::infra_execution_report::AgentExecutionReport;
use crate::stream::StreamForwarder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCapability {
    Recursion,
    Parallel,
    Streaming,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub agent_type: AgentType,
    pub supports_recursion: bool,
    pub supports_parallel: bool,
    pub supports_streaming: bool,
}

impl AgentCapabilities {
    pub fn has_capability(&self, capability: AgentCapability) -> bool {
        match capability {
            AgentCapability::Recursion => self.supports_recursion,
            AgentCapability::Parallel => self.supports_parallel,
            AgentCapability::Streaming => self.supports_streaming,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmAgentConfig {
    pub model_id: String,
    pub temperature: f32,
    pub max_output_tokens: u32,
    pub system_prompt_template: String,
    pub function_calling_enabled: bool,
    pub max_function_rounds: u32,
}

impl Default for LlmAgentConfig {
    fn default() -> Self {
        Self {
            model_id: "default".to_string(),
            temperature: 0.3,
            max_output_tokens: 4_096,
            system_prompt_template: "default".to_string(),
            function_calling_enabled: true,
            max_function_rounds: 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub llm_config: LlmAgentConfig,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub streaming_enabled: bool,
    pub max_parallelism: u32,
    pub context_window_limit: u32,
    pub preflight_enabled: bool,
    pub custom_params: HashMap<String, Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_type: AgentType::Coder,
            llm_config: LlmAgentConfig::default(),
            timeout_ms: 300_000,
            max_retries: 3,
            streaming_enabled: false,
            max_parallelism: 0,
            context_window_limit: 128_000,
            preflight_enabled: true,
            custom_params: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AgentConfigPatch {
    pub llm_config: Option<LlmAgentConfig>,
    pub timeout_ms: Option<u64>,
    pub max_retries: Option<u32>,
    pub streaming_enabled: Option<bool>,
    pub max_parallelism: Option<u32>,
    pub context_window_limit: Option<u32>,
    pub preflight_enabled: Option<bool>,
    pub custom_params: HashMap<String, Value>,
}

impl AgentConfig {
    pub fn for_agent_type(agent_type: AgentType) -> Self {
        Self {
            agent_type,
            ..Self::default()
        }
    }

    pub fn from_toml_file(path: impl AsRef<Path>) -> Result<Self, AgentError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|error| {
            AgentError::invalid_config(format!("failed to read config {}: {error}", path.display()))
        })?;
        toml::from_str(&content).map_err(|error| {
            AgentError::invalid_config(format!("failed to parse config {}: {error}", path.display()))
        })
    }
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn agent_type(&self) -> AgentType;

    fn supports_recursion(&self) -> bool {
        false
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        false
    }

    fn capabilities(&self) -> AgentCapabilities {
        AgentCapabilities {
            agent_type: self.agent_type(),
            supports_recursion: self.supports_recursion(),
            supports_parallel: self.supports_parallel(),
            supports_streaming: self.supports_streaming(),
        }
    }

    async fn build_context(
        &self,
        task: &TaskDescription,
        project_ctx: Option<ProjectContext>,
        shared: &SharedResources,
    ) -> Result<AgentContext, AgentError>;

    async fn execute(&self, ctx: &AgentContext) -> Result<AgentExecutionReport, AgentError>;

    async fn execute_streaming(
        &self,
        ctx: &AgentContext,
        forwarder: &mut dyn StreamForwarder,
    ) -> Result<AgentExecutionReport, AgentError> {
        let report = self.execute(ctx).await?;
        forwarder.forward_final(&report).await?;
        Ok(report)
    }

    fn default_config(&self) -> AgentConfig;

    fn update_config(&mut self, config: AgentConfig);
}
