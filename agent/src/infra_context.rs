use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mc_core::AgentType;
use mc_core::task::{ExecutionPlan, ProjectContext, TaskDescription};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::{broadcast, RwLock as AsyncRwLock};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::error::AgentError;
use crate::handoff::AgentHandoff;
use crate::stream::StreamForwarder;
use crate::trait_def::AgentConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactReport {
    pub summary: String,
    pub affected_files: Vec<String>,
    pub risk_level: ImpactRiskLevel,
    pub breaking_change: bool,
    pub notes: Vec<String>,
}

impl Default for ImpactReport {
    fn default() -> Self {
        Self {
            summary: String::new(),
            affected_files: Vec::new(),
            risk_level: ImpactRiskLevel::Low,
            breaking_change: false,
            notes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub app_name: String,
    pub environment: String,
    pub metadata: HashMap<String, Value>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            app_name: "MoreCode".to_string(),
            environment: "test".to_string(),
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmRequest {
    pub prompt: String,
    pub model_id: Option<String>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model_id: String,
    pub metadata: HashMap<String, Value>,
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, AgentError>;
}

#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: RwLock<HashSet<String>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, tool_name: impl Into<String>) -> Result<(), AgentError> {
        let mut tools = self
            .tools
            .write()
            .map_err(|_| AgentError::lock_poisoned("tool_registry"))?;
        tools.insert(tool_name.into());
        Ok(())
    }

    pub fn contains(&self, tool_name: &str) -> Result<bool, AgentError> {
        let tools = self
            .tools
            .read()
            .map_err(|_| AgentError::lock_poisoned("tool_registry"))?;
        Ok(tools.contains(tool_name))
    }
}

#[derive(Debug, Default)]
pub struct MemoryManager {
    values: AsyncRwLock<HashMap<String, Value>>,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn put_json(&self, key: impl Into<String>, value: Value) {
        self.values.write().await.insert(key.into(), value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentEventKind {
    Started,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentEvent {
    pub kind: AgentEventKind,
    pub execution_id: Uuid,
    pub agent_type: AgentType,
    pub attempt: u32,
    pub timestamp: DateTime<Utc>,
    pub data: Value,
}

pub trait EventBus: Send + Sync {
    fn publish(&self, event: AgentEvent) -> Result<(), AgentError>;
    fn subscribe(&self) -> broadcast::Receiver<AgentEvent>;
}

#[derive(Debug)]
pub struct InMemoryEventBus {
    sender: broadcast::Sender<AgentEvent>,
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(128);
        Self { sender }
    }
}

impl EventBus for InMemoryEventBus {
    fn publish(&self, event: AgentEvent) -> Result<(), AgentError> {
        let _ = self.sender.send(event);
        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.sender.subscribe()
    }
}

pub trait Logger: Send + Sync {
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
}

#[derive(Debug, Default)]
pub struct TracingLogger;

impl Logger for TracingLogger {
    fn info(&self, message: &str) {
        tracing::info!("{message}");
    }

    fn warn(&self, message: &str) {
        tracing::warn!("{message}");
    }

    fn error(&self, message: &str) {
        tracing::error!("{message}");
    }
}

#[derive(Clone)]
pub struct SharedResources {
    pub llm_client: Arc<dyn LlmClient>,
    pub tool_registry: Arc<ToolRegistry>,
    pub memory_manager: Arc<MemoryManager>,
    pub event_bus: Arc<dyn EventBus>,
    pub config: Arc<GlobalConfig>,
}

impl SharedResources {
    pub fn new(
        llm_client: Arc<dyn LlmClient>,
        tool_registry: Arc<ToolRegistry>,
        memory_manager: Arc<MemoryManager>,
        event_bus: Arc<dyn EventBus>,
        config: Arc<GlobalConfig>,
    ) -> Self {
        Self {
            llm_client,
            tool_registry,
            memory_manager,
            event_bus,
            config,
        }
    }
}

#[derive(Clone)]
pub struct AgentContext {
    pub task: Arc<TaskDescription>,
    pub execution_id: Uuid,
    pub parent_execution_id: Option<Uuid>,
    pub project_ctx: Option<Arc<ProjectContext>>,
    pub impact_report: Option<Arc<ImpactReport>>,
    pub execution_plan: Option<Arc<ExecutionPlan>>,
    pub handoff: Arc<AgentHandoff>,
    pub llm_client: Arc<dyn LlmClient>,
    pub tool_registry: Arc<ToolRegistry>,
    pub memory_manager: Arc<MemoryManager>,
    pub event_bus: Arc<dyn EventBus>,
    pub cancel_token: CancellationToken,
    pub stream_forwarder: Option<Arc<dyn StreamForwarder>>,
    pub config: Arc<AgentConfig>,
    pub started_at: DateTime<Utc>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub extra_params: HashMap<String, Value>,
}

impl AgentContext {
    pub fn new(task: TaskDescription, shared: &SharedResources, config: AgentConfig) -> Self {
        let max_retries = config.max_retries;
        Self {
            task: Arc::new(task),
            execution_id: Uuid::new_v4(),
            parent_execution_id: None,
            project_ctx: None,
            impact_report: None,
            execution_plan: None,
            handoff: Arc::new(AgentHandoff::new()),
            llm_client: shared.llm_client.clone(),
            tool_registry: shared.tool_registry.clone(),
            memory_manager: shared.memory_manager.clone(),
            event_bus: shared.event_bus.clone(),
            cancel_token: CancellationToken::new(),
            stream_forwarder: None,
            config: Arc::new(config),
            started_at: Utc::now(),
            retry_count: 0,
            max_retries,
            extra_params: HashMap::new(),
        }
    }

    pub fn create_child_context(&self, sub_task: TaskDescription) -> Self {
        Self {
            task: Arc::new(sub_task),
            execution_id: Uuid::new_v4(),
            parent_execution_id: Some(self.execution_id),
            project_ctx: self.project_ctx.clone(),
            impact_report: self.impact_report.clone(),
            execution_plan: self.execution_plan.clone(),
            handoff: Arc::new(AgentHandoff::with_parent(self.handoff.clone())),
            llm_client: self.llm_client.clone(),
            tool_registry: self.tool_registry.clone(),
            memory_manager: self.memory_manager.clone(),
            event_bus: self.event_bus.clone(),
            cancel_token: self.cancel_token.child_token(),
            stream_forwarder: self.stream_forwarder.clone(),
            config: self.config.clone(),
            started_at: Utc::now(),
            retry_count: 0,
            max_retries: self.max_retries,
            extra_params: HashMap::new(),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    pub fn elapsed(&self) -> std::time::Duration {
        Utc::now()
            .signed_duration_since(self.started_at)
            .to_std()
            .unwrap_or_default()
    }
}
