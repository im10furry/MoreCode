use async_trait::async_trait;
use mc_core::{AgentExecutionReport, AgentType, ProjectContext, TaskDescription};
use serde::{Deserialize, Serialize};

use crate::{AgentConfig, AgentContext, AgentError, SharedResources};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub agent_type: AgentType,
    pub supports_recursion: bool,
    pub supports_parallel: bool,
    pub supports_streaming: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentStreamEvent {
    Started {
        agent_type: AgentType,
        execution_id: String,
        message: String,
    },
    Progress {
        agent_type: AgentType,
        message: String,
    },
    Completed {
        agent_type: AgentType,
        execution_id: String,
        summary: String,
    },
    Failed {
        agent_type: AgentType,
        execution_id: String,
        message: String,
    },
}

#[async_trait]
pub trait AgentEventSink: Send + Sync {
    async fn publish(&self, event: AgentStreamEvent) -> Result<(), AgentError>;
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
        sink: &dyn AgentEventSink,
    ) -> Result<AgentExecutionReport, AgentError> {
        sink.publish(AgentStreamEvent::Started {
            agent_type: self.agent_type(),
            execution_id: ctx.execution_id.to_string(),
            message: format!("{} started", self.agent_type()),
        })
        .await?;

        let result = self.execute(ctx).await;

        match &result {
            Ok(report) => {
                sink.publish(AgentStreamEvent::Completed {
                    agent_type: self.agent_type(),
                    execution_id: ctx.execution_id.to_string(),
                    summary: report.title.clone(),
                })
                .await?;
            }
            Err(error) => {
                let _ = sink
                    .publish(AgentStreamEvent::Failed {
                        agent_type: self.agent_type(),
                        execution_id: ctx.execution_id.to_string(),
                        message: error.to_string(),
                    })
                    .await;
            }
        }

        result
    }
}
