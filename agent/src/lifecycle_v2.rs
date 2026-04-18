use async_trait::async_trait;
use mc_core::AgentExecutionReport;

use crate::{AgentContext, AgentError};

#[async_trait]
pub trait AgentLifecycle: Send + Sync {
    async fn on_start(&self, _ctx: &AgentContext) -> Result<(), AgentError> {
        Ok(())
    }

    async fn on_success(
        &self,
        _ctx: &AgentContext,
        _report: &AgentExecutionReport,
    ) -> Result<(), AgentError> {
        Ok(())
    }

    async fn on_failure(
        &self,
        _ctx: &AgentContext,
        _error: &AgentError,
    ) -> Result<(), AgentError> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopLifecycle;

#[async_trait]
impl AgentLifecycle for NoopLifecycle {}
