use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mc_core::AgentType;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use uuid::Uuid;

use crate::error::AgentError;
use crate::execution_report::AgentExecutionReport;

/// Backend abstraction for agent streaming output.
#[async_trait]
pub trait StreamForwarder: Send + Sync {
    async fn forward_chunk(&mut self, chunk: &str) -> Result<(), AgentError>;
    async fn forward_event(&mut self, event: StreamEvent) -> Result<(), AgentError>;
    async fn forward_final(&mut self, report: &AgentExecutionReport) -> Result<(), AgentError>;
    async fn flush(&mut self) -> Result<(), AgentError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub event_type: StreamEventType,
    pub agent_type: AgentType,
    pub execution_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamEventType {
    Thinking,
    ToolCallStart,
    ToolCallComplete,
    FileChange,
    Progress,
    Warning,
    Error,
    FinalResult,
}

/// Stdout forwarder for CLI-oriented streaming flows.
#[derive(Debug, Default)]
pub struct StdoutStreamForwarder {
    buffer: String,
}

impl StdoutStreamForwarder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }
}

#[async_trait]
impl StreamForwarder for StdoutStreamForwarder {
    async fn forward_chunk(&mut self, chunk: &str) -> Result<(), AgentError> {
        self.buffer.push_str(chunk);
        print!("{chunk}");
        io::stdout()
            .flush()
            .map_err(|error| AgentError::stream(format!("failed to flush stdout: {error}")))?;
        Ok(())
    }

    async fn forward_event(&mut self, event: StreamEvent) -> Result<(), AgentError> {
        let payload = serde_json::to_string(&event)
            .map_err(|error| AgentError::stream(format!("failed to serialize stream event: {error}")))?;
        println!("[STREAM] {payload}");
        Ok(())
    }

    async fn forward_final(&mut self, report: &AgentExecutionReport) -> Result<(), AgentError> {
        let payload = serde_json::to_string_pretty(report).map_err(|error| {
            AgentError::stream(format!("failed to serialize final stream report: {error}"))
        })?;
        println!("[FINAL] {payload}");
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), AgentError> {
        io::stdout()
            .flush()
            .map_err(|error| AgentError::stream(format!("failed to flush stdout: {error}")))?;
        Ok(())
    }
}
