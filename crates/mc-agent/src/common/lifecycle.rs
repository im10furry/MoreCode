use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, timeout, Duration};

use crate::context::{AgentContext, AgentEvent, AgentEventKind, Logger};
use crate::error::AgentError;
use crate::execution_report::{AgentExecutionReport, ExecutionStatus};
use crate::stream::StreamForwarder;
use crate::trait_def::Agent;

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
    async fn on_start(&self, _ctx: &AgentContext) -> Result<(), AgentError> {
        Ok(())
    }

    async fn on_complete(
        &self,
        _ctx: &AgentContext,
        _report: &AgentExecutionReport,
    ) -> Result<(), AgentError> {
        Ok(())
    }

    async fn on_error(&self, _ctx: &AgentContext, _error: &AgentError) -> Result<(), AgentError> {
        Ok(())
    }

    async fn on_cancel(&self, _ctx: &AgentContext) -> Result<(), AgentError> {
        Ok(())
    }
}

pub struct LoggingLifecycle {
    logger: Arc<dyn Logger>,
}

impl LoggingLifecycle {
    pub fn new(logger: Arc<dyn Logger>) -> Self {
        Self { logger }
    }
}

#[async_trait]
impl AgentLifecycle for LoggingLifecycle {
    async fn on_start(&self, ctx: &AgentContext) -> Result<(), AgentError> {
        self.logger.info(&format!(
            "[Lifecycle] agent={} execution_id={} started",
            ctx.config.agent_type.identifier(),
            ctx.execution_id
        ));
        Ok(())
    }

    async fn on_complete(
        &self,
        ctx: &AgentContext,
        report: &AgentExecutionReport,
    ) -> Result<(), AgentError> {
        self.logger.info(&format!(
            "[Lifecycle] agent={} completed in {:?} status={:?}",
            ctx.config.agent_type.identifier(),
            ctx.elapsed(),
            report.status
        ));
        Ok(())
    }

    async fn on_error(&self, ctx: &AgentContext, error: &AgentError) -> Result<(), AgentError> {
        self.logger.error(&format!(
            "[Lifecycle] agent={} failed: {}",
            ctx.config.agent_type.identifier(),
            error
        ));
        Ok(())
    }

    async fn on_cancel(&self, ctx: &AgentContext) -> Result<(), AgentError> {
        self.logger.warn(&format!(
            "[Lifecycle] agent={} cancelled after {:?}",
            ctx.config.agent_type.identifier(),
            ctx.elapsed()
        ));
        Ok(())
    }
}

pub async fn execute_with_context(
    agent: &dyn Agent,
    ctx: &AgentContext,
    lifecycle: Option<LifecycleHandler>,
) -> Result<AgentExecutionReport, AgentError> {
    emit_event(ctx, agent, AgentEventKind::Started, 1, json!({}));
    invoke_on_start(lifecycle.as_deref(), ctx).await;

    let result = execute_with_timeout_and_retry(agent, ctx).await;
    handle_terminal_state(agent, ctx, lifecycle.as_deref(), &result).await;
    result
}

pub async fn execute_streaming_with_context(
    agent: &dyn Agent,
    ctx: &AgentContext,
    forwarder: &mut dyn StreamForwarder,
    lifecycle: Option<LifecycleHandler>,
) -> Result<AgentExecutionReport, AgentError> {
    emit_event(ctx, agent, AgentEventKind::Started, 1, json!({ "streaming": true }));
    invoke_on_start(lifecycle.as_deref(), ctx).await;

    let result = execute_streaming_with_timeout_and_retry(agent, ctx, forwarder).await;
    handle_terminal_state(agent, ctx, lifecycle.as_deref(), &result).await;
    result
}

async fn handle_terminal_state(
    agent: &dyn Agent,
    ctx: &AgentContext,
    lifecycle: Option<&dyn AgentLifecycle>,
    result: &Result<AgentExecutionReport, AgentError>,
) {
    match result {
        Ok(report) if report.status == ExecutionStatus::Cancelled => {
            emit_event(ctx, agent, AgentEventKind::Cancelled, 0, json!({}));
            invoke_on_cancel(lifecycle, ctx).await;
        }
        Ok(report) => {
            emit_event(
                ctx,
                agent,
                AgentEventKind::Completed,
                0,
                json!({
                    "status": format!("{:?}", report.status),
                    "duration_ms": report.duration_ms
                }),
            );
            invoke_on_complete(lifecycle, ctx, report).await;
        }
        Err(AgentError::ExecutionCancelled { .. }) => {
            emit_event(ctx, agent, AgentEventKind::Cancelled, 0, json!({}));
            invoke_on_cancel(lifecycle, ctx).await;
        }
        Err(error) => {
            emit_event(
                ctx,
                agent,
                AgentEventKind::Failed,
                0,
                json!({ "error": error.to_string() }),
            );
            invoke_on_error(lifecycle, ctx, error).await;
        }
    }
}

async fn execute_with_timeout_and_retry(
    agent: &dyn Agent,
    ctx: &AgentContext,
) -> Result<AgentExecutionReport, AgentError> {
    let timeout_duration = configured_timeout(ctx);

    for attempt in 0..=ctx.max_retries {
        let attempt_number = attempt + 1;
        let result = run_non_streaming_attempt(agent, ctx, timeout_duration).await;

        match result {
            Ok(report) => return Ok(finalize_report(agent, ctx, report)),
            Err(error) if error.is_retryable() && attempt < ctx.max_retries => {
                emit_event(
                    ctx,
                    agent,
                    AgentEventKind::Retrying,
                    attempt_number,
                    json!({ "error": error.to_string() }),
                );
                wait_retry_backoff(ctx, &error, attempt_number).await?;
            }
            Err(error) => return Err(error),
        }
    }

    Err(AgentError::internal("retry loop exhausted unexpectedly"))
}

async fn execute_streaming_with_timeout_and_retry(
    agent: &dyn Agent,
    ctx: &AgentContext,
    forwarder: &mut dyn StreamForwarder,
) -> Result<AgentExecutionReport, AgentError> {
    let timeout_duration = configured_timeout(ctx);

    for attempt in 0..=ctx.max_retries {
        let attempt_number = attempt + 1;
        let result = run_streaming_attempt(agent, ctx, timeout_duration, forwarder).await;

        match result {
            Ok(report) => {
                forwarder.flush().await?;
                return Ok(finalize_report(agent, ctx, report));
            }
            Err(error) if error.is_retryable() && attempt < ctx.max_retries => {
                emit_event(
                    ctx,
                    agent,
                    AgentEventKind::Retrying,
                    attempt_number,
                    json!({ "error": error.to_string(), "streaming": true }),
                );
                wait_retry_backoff(ctx, &error, attempt_number).await?;
            }
            Err(error) => return Err(error),
        }
    }

    Err(AgentError::internal("streaming retry loop exhausted unexpectedly"))
}

async fn run_non_streaming_attempt(
    agent: &dyn Agent,
    ctx: &AgentContext,
    timeout_duration: Option<Duration>,
) -> Result<AgentExecutionReport, AgentError> {
    if ctx.is_cancelled() {
        return Err(AgentError::ExecutionCancelled {
            agent_type: agent.agent_type(),
        });
    }

    match timeout_duration {
        Some(duration) => {
            let exec_future = agent.execute(ctx);
            tokio::select! {
                _ = ctx.cancel_token.cancelled() => Err(AgentError::ExecutionCancelled {
                    agent_type: agent.agent_type(),
                }),
                timed = timeout(duration, exec_future) => match timed {
                    Ok(result) => result,
                    Err(_) => Err(AgentError::ExecutionTimeout {
                        agent_type: agent.agent_type(),
                        timeout_ms: ctx.config.timeout_ms,
                    }),
                },
            }
        }
        None => {
            let exec_future = agent.execute(ctx);
            tokio::select! {
                _ = ctx.cancel_token.cancelled() => Err(AgentError::ExecutionCancelled {
                    agent_type: agent.agent_type(),
                }),
                result = exec_future => result,
            }
        }
    }
}

async fn run_streaming_attempt(
    agent: &dyn Agent,
    ctx: &AgentContext,
    timeout_duration: Option<Duration>,
    forwarder: &mut dyn StreamForwarder,
) -> Result<AgentExecutionReport, AgentError> {
    if ctx.is_cancelled() {
        return Err(AgentError::ExecutionCancelled {
            agent_type: agent.agent_type(),
        });
    }

    match timeout_duration {
        Some(duration) => {
            let exec_future = agent.execute_streaming(ctx, forwarder);
            tokio::select! {
                _ = ctx.cancel_token.cancelled() => Err(AgentError::ExecutionCancelled {
                    agent_type: agent.agent_type(),
                }),
                timed = timeout(duration, exec_future) => match timed {
                    Ok(result) => result,
                    Err(_) => Err(AgentError::ExecutionTimeout {
                        agent_type: agent.agent_type(),
                        timeout_ms: ctx.config.timeout_ms,
                    }),
                },
            }
        }
        None => {
            let exec_future = agent.execute_streaming(ctx, forwarder);
            tokio::select! {
                _ = ctx.cancel_token.cancelled() => Err(AgentError::ExecutionCancelled {
                    agent_type: agent.agent_type(),
                }),
                result = exec_future => result,
            }
        }
    }
}

async fn wait_retry_backoff(
    ctx: &AgentContext,
    error: &AgentError,
    attempt_number: u32,
) -> Result<(), AgentError> {
    let base = error.retry_delay_ms();
    let exponent = attempt_number.saturating_sub(1);
    let multiplier = 1_u64.checked_shl(exponent).unwrap_or(u64::MAX);
    let delay_ms = base.saturating_mul(multiplier);

    tokio::select! {
        _ = ctx.cancel_token.cancelled() => Err(AgentError::ExecutionCancelled {
            agent_type: ctx.config.agent_type,
        }),
        _ = sleep(Duration::from_millis(delay_ms)) => Ok(()),
    }
}

fn configured_timeout(ctx: &AgentContext) -> Option<Duration> {
    if ctx.config.timeout_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(ctx.config.timeout_ms))
    }
}

fn finalize_report(
    agent: &dyn Agent,
    ctx: &AgentContext,
    mut report: AgentExecutionReport,
) -> AgentExecutionReport {
    report.execution_id = ctx.execution_id;
    report.agent_type = agent.agent_type();
    report.duration_ms = ctx.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    report.timestamp = chrono::Utc::now();
    report
}

fn emit_event(
    ctx: &AgentContext,
    agent: &dyn Agent,
    kind: AgentEventKind,
    attempt: u32,
    data: serde_json::Value,
) {
    let event = AgentEvent {
        kind,
        execution_id: ctx.execution_id,
        agent_type: agent.agent_type(),
        attempt,
        timestamp: chrono::Utc::now(),
        data,
    };

    if let Err(error) = ctx.event_bus.publish(event) {
        tracing::warn!("failed to publish agent event: {error}");
    }
}

async fn invoke_on_start(lifecycle: Option<&dyn AgentLifecycle>, ctx: &AgentContext) {
    if let Some(handler) = lifecycle {
        if let Err(error) = handler.on_start(ctx).await {
            tracing::warn!("lifecycle on_start failed: {error}");
        }
    }
}

async fn invoke_on_complete(
    lifecycle: Option<&dyn AgentLifecycle>,
    ctx: &AgentContext,
    report: &AgentExecutionReport,
) {
    if let Some(handler) = lifecycle {
        if let Err(error) = handler.on_complete(ctx, report).await {
            tracing::warn!("lifecycle on_complete failed: {error}");
        }
    }
}

async fn invoke_on_error(
    lifecycle: Option<&dyn AgentLifecycle>,
    ctx: &AgentContext,
    error: &AgentError,
) {
    if let Some(handler) = lifecycle {
        if let Err(lifecycle_error) = handler.on_error(ctx, error).await {
            tracing::warn!("lifecycle on_error failed: {lifecycle_error}");
        }
    }
}

async fn invoke_on_cancel(lifecycle: Option<&dyn AgentLifecycle>, ctx: &AgentContext) {
    if let Some(handler) = lifecycle {
        if let Err(error) = handler.on_cancel(ctx).await {
            tracing::warn!("lifecycle on_cancel failed: {error}");
        }
    }
}
