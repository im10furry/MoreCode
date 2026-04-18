use async_trait::async_trait;
use mc_core::{AgentType, ProjectContext, TaskDescription};
use mc_llm::StreamForwarder;

use crate::{AgentConfig, AgentContext, AgentError, AgentExecutionReport, SharedResources};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentCapabilities {
    pub agent_type: AgentType,
    pub supports_parallel: bool,
    pub supports_streaming: bool,
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn agent_type(&self) -> AgentType;

    fn supports_parallel(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        false
    }

    fn capabilities(&self) -> AgentCapabilities {
        AgentCapabilities {
            agent_type: self.agent_type(),
            supports_parallel: self.supports_parallel(),
            supports_streaming: self.supports_streaming(),
        }
    }

    fn default_config(&self) -> AgentConfig;

    fn update_config(&mut self, config: AgentConfig);

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
        _forwarder: &mut StreamForwarder,
    ) -> Result<AgentExecutionReport, AgentError> {
        self.execute(ctx).await
    }
}
