use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, timeout, Duration};

use crate::context::{AgentContext, Logger};
use crate::error::AgentError;
use crate::execution_report::{AgentExecutionReport, ExecutionStatus};
use crate::trait_def::Agent;
use crate::stream::StreamForwarder;

pub type LifecycleHandler = Arc<dyn AgentLifecycle>;

#[async_trait]
pub trait AgentLifecycle: Send + Sync {
    async fn on_start(&self, ctx: &AgentContext) -> Result<(), AgentError>;
    async fn on_complete(
        &self,
        ctx: &AgentContext,
        report: &AgentExecutionReport,
    ) -> Result<(), AgentError>;
    async fn on_error(&self, ctx: &AgentContext, error: &AgentError) -> Result<(), AgentError>;
    async fn on_cancel(&self, ctx: &AgentContext) -> Result<(), AgentError>;
}

#[derive(Debug, Default)]
pub struct NoopLifecycle;

#[async_trait]
impl AgentLifecycle for NoopLifecycle {
    async fn on_start(&self, _ctx: &AgentContext) -> Result<(), AgentError> { Ok(()) }
    async fn on_complete(&self, _ctx: &AgentContext, _report: &AgentExecutionReport) -> Result<(), AgentError> { Ok(()) }
    async fn on_error(&self, _ctx: &AgentContext, _error: &AgentError) -> Result<(), AgentError> { Ok(()) }
    async fn on_cancel(&self, _ctx: &AgentContext) -> Result<(), AgentError> { Ok(()) }
}

pub struct LoggingLifecycle {
    logger: Arc<dyn Logger>,
}

impl LoggingLifecycle {
    pub fn new(logger: Arc<dyn Logger>) -> Self {
        Self { logger }
    }
}

pub async fn execute_with_context(
    agent: &dyn Agent,
    ctx: &AgentContext,
    lifecycle: Option<LifecycleHandler>,
) -> Result<AgentExecutionReport, AgentError> {
    let _ = ctx.event_bus.publish(crate::context::AgentEvent {
        kind: crate::context::AgentEventKind::Started,
        execution_id: ctx.execution_id,
        agent_type: agent.agent_type(),
        attempt: 1,
        timestamp: chrono::Utc::now(),
        data: json!({}),
    });
    if let Some(handler) = lifecycle.as_deref() {
        let _ = handler.on_start(ctx).await;
    }

    let timeout_duration = if ctx.config.timeout_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(ctx.config.timeout_ms))
    };

    let result = execute_with_retry(agent, ctx, timeout_duration).await;

    match &result {
        Ok(report) if report.status == ExecutionStatus::Cancelled => {
            let _ = ctx.event_bus.publish(crate::context::AgentEvent {
                kind: crate::context::AgentEventKind::Cancelled,
                execution_id: ctx.execution_id,
                agent_type: agent.agent_type(),
                attempt: 0,
                timestamp: chrono::Utc::now(),
                data: json!({}),
            });
            if let Some(handler) = lifecycle.as_deref() {
                let _ = handler.on_cancel(ctx).await;
            }
        }
        Ok(report) => {
            let _ = ctx.event_bus.publish(crate::context::AgentEvent {
                kind: crate::context::AgentEventKind::Completed,
                execution_id: ctx.execution_id,
                agent_type: agent.agent_type(),
                attempt: 0,
                timestamp: chrono::Utc::now(),
                data: json!({ "status": format!("{:?}", report.status) }),
            });
            if let Some(handler) = lifecycle.as_deref() {
                let _ = handler.on_complete(ctx, report).await;
            }
        }
        Err(AgentError::ExecutionCancelled { .. }) => {
            let _ = ctx.event_bus.publish(crate::context::AgentEvent {
                kind: crate::context::AgentEventKind::Cancelled,
                execution_id: ctx.execution_id,
                agent_type: agent.agent_type(),
                attempt: 0,
                timestamp: chrono::Utc::now(),
                data: json!({}),
            });
            if let Some(handler) = lifecycle.as_deref() {
                let _ = handler.on_cancel(ctx).await;
            }
        }
        Err(error) => {
            let _ = ctx.event_bus.publish(crate::context::AgentEvent {
                kind: crate::context::AgentEventKind::Failed,
                execution_id: ctx.execution_id,
                agent_type: agent.agent_type(),
                attempt: 0,
                timestamp: chrono::Utc::now(),
                data: json!({ "error": error.to_string() }),
            });
            if let Some(handler) = lifecycle.as_deref() {
                let _ = handler.on_error(ctx, error).await;
            }
        }
    }

    result
}

pub async fn execute_streaming_with_context(
    agent: &dyn Agent,
    ctx: &AgentContext,
    forwarder: &mut dyn StreamForwarder,
    lifecycle: Option<LifecycleHandler>,
) -> Result<AgentExecutionReport, AgentError> {
    let _ = ctx.event_bus.publish(crate::context::AgentEvent {
        kind: crate::context::AgentEventKind::Started,
        execution_id: ctx.execution_id,
        agent_type: agent.agent_type(),
        attempt: 1,
        timestamp: chrono::Utc::now(),
        data: json!({ "streaming": true }),
    });
    if let Some(handler) = lifecycle.as_deref() {
        let _ = handler.on_start(ctx).await;
    }

    let timeout_duration = if ctx.config.timeout_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(ctx.config.timeout_ms))
    };

    for attempt in 0..=ctx.max_retries {
        if ctx.is_cancelled() {
            return Err(AgentError::ExecutionCancelled {
                agent_type: agent.agent_type(),
            });
        }

        let result = match timeout_duration {
            Some(duration) => {
                let future = agent.execute_streaming(ctx, forwarder);
                tokio::select! {
                    _ = ctx.cancel_token.cancelled() => Err(AgentError::ExecutionCancelled { agent_type: agent.agent_type() }),
                    timed = timeout(duration, future) => match timed {
                        Ok(result) => result,
                        Err(_) => Err(AgentError::ExecutionTimeout { agent_type: agent.agent_type(), timeout_ms: ctx.config.timeout_ms }),
                    }
                }
            }
            None => agent.execute_streaming(ctx, forwarder).await,
        };

        match result {
            Ok(report) => {
                forwarder.flush().await?;
                let _ = ctx.event_bus.publish(crate::context::AgentEvent {
                    kind: crate::context::AgentEventKind::Completed,
                    execution_id: ctx.execution_id,
                    agent_type: agent.agent_type(),
                    attempt: 0,
                    timestamp: chrono::Utc::now(),
                    data: json!({ "status": format!("{:?}", report.status), "streaming": true }),
                });
                if let Some(handler) = lifecycle.as_deref() {
                    let _ = handler.on_complete(ctx, &report).await;
                }
                return Ok(report);
            }
            Err(error) if error.is_retryable() && attempt < ctx.max_retries => {
                let _ = ctx.event_bus.publish(crate::context::AgentEvent {
                    kind: crate::context::AgentEventKind::Retrying,
                    execution_id: ctx.execution_id,
                    agent_type: agent.agent_type(),
                    attempt: attempt + 1,
                    timestamp: chrono::Utc::now(),
                    data: json!({ "error": error.to_string(), "streaming": true }),
                });
                sleep(Duration::from_millis(error.retry_delay_ms())).await;
            }
            Err(error) => {
                let _ = ctx.event_bus.publish(crate::context::AgentEvent {
                    kind: crate::context::AgentEventKind::Failed,
                    execution_id: ctx.execution_id,
                    agent_type: agent.agent_type(),
                    attempt: 0,
                    timestamp: chrono::Utc::now(),
                    data: json!({ "error": error.to_string(), "streaming": true }),
                });
                if let Some(handler) = lifecycle.as_deref() {
                    let _ = handler.on_error(ctx, &error).await;
                }
                return Err(error);
            }
        }
    }

    Err(AgentError::internal("streaming retry loop exhausted unexpectedly"))
}

async fn execute_with_retry(
    agent: &dyn Agent,
    ctx: &AgentContext,
    timeout_duration: Option<Duration>,
) -> Result<AgentExecutionReport, AgentError> {
    for attempt in 0..=ctx.max_retries {
        if ctx.is_cancelled() {
            return Err(AgentError::ExecutionCancelled {
                agent_type: agent.agent_type(),
            });
        }

        let result = match timeout_duration {
            Some(duration) => {
                let future = agent.execute(ctx);
                tokio::select! {
                    _ = ctx.cancel_token.cancelled() => Err(AgentError::ExecutionCancelled { agent_type: agent.agent_type() }),
                    timed = timeout(duration, future) => match timed {
                        Ok(result) => result,
                        Err(_) => Err(AgentError::ExecutionTimeout { agent_type: agent.agent_type(), timeout_ms: ctx.config.timeout_ms }),
                    }
                }
            }
            None => agent.execute(ctx).await,
        };

        match result {
            Ok(report) => return Ok(report),
            Err(error) if error.is_retryable() && attempt < ctx.max_retries => {
                sleep(Duration::from_millis(error.retry_delay_ms())).await;
            }
            Err(error) => return Err(error),
        }
    }

    Err(AgentError::internal("retry loop exhausted unexpectedly"))
}

#[async_trait]
impl AgentLifecycle for LoggingLifecycle {
    async fn on_start(&self, ctx: &AgentContext) -> Result<(), AgentError> {
        self.logger.info(&format!("{} started", ctx.config.agent_type.identifier()));
        Ok(())
    }

    async fn on_complete(&self, _ctx: &AgentContext, report: &AgentExecutionReport) -> Result<(), AgentError> {
        self.logger.info(&format!("completed with {:?}", report.status));
        Ok(())
    }

    async fn on_error(&self, _ctx: &AgentContext, error: &AgentError) -> Result<(), AgentError> {
        self.logger.error(&error.to_string());
        Ok(())
    }

    async fn on_cancel(&self, _ctx: &AgentContext) -> Result<(), AgentError> {
        self.logger.warn("cancelled");
        Ok(())
    }
}
