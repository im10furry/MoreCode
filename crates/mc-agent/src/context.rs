use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use mc_core::{ExecutionPlan, ProjectContext, TaskDescription};
use mc_llm::LlmProvider;
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{AgentConfig, AgentHandoff, ImpactReport};

#[derive(Clone)]
pub struct SharedResources {
    pub project_root: PathBuf,
    pub llm_provider: Arc<dyn LlmProvider>,
}

impl SharedResources {
    pub fn new(project_root: impl Into<PathBuf>, llm_provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            project_root: project_root.into(),
            llm_provider,
        }
    }
}

#[derive(Clone)]
pub struct AgentContext {
    pub task: Arc<TaskDescription>,
    pub execution_id: String,
    pub parent_execution_id: Option<String>,
    pub project_ctx: Option<Arc<ProjectContext>>,
    pub impact_report: Option<Arc<ImpactReport>>,
    pub execution_plan: Option<Arc<ExecutionPlan>>,
    pub handoff: Arc<AgentHandoff>,
    pub llm_provider: Arc<dyn LlmProvider>,
    pub shared: Arc<SharedResources>,
    pub cancel_token: CancellationToken,
    pub config: Arc<AgentConfig>,
    pub started_at: DateTime<Utc>,
    pub extra_params: HashMap<String, Value>,
}

impl AgentContext {
    pub fn new(task: TaskDescription, shared: &SharedResources, config: AgentConfig) -> Self {
        Self {
            task: Arc::new(task),
            execution_id: Uuid::new_v4().to_string(),
            parent_execution_id: None,
            project_ctx: None,
            impact_report: None,
            execution_plan: None,
            handoff: Arc::new(AgentHandoff::new()),
            llm_provider: Arc::clone(&shared.llm_provider),
            shared: Arc::new(shared.clone()),
            cancel_token: CancellationToken::new(),
            config: Arc::new(config),
            started_at: Utc::now(),
            extra_params: HashMap::new(),
        }
    }

    pub fn with_handoff(mut self, handoff: Arc<AgentHandoff>) -> Self {
        self.handoff = handoff;
        self
    }

    pub fn with_project_ctx(mut self, project_ctx: ProjectContext) -> Self {
        self.project_ctx = Some(Arc::new(project_ctx));
        self
    }

    pub fn with_impact_report(mut self, impact_report: ImpactReport) -> Self {
        self.impact_report = Some(Arc::new(impact_report));
        self
    }

    pub fn with_execution_plan(mut self, execution_plan: ExecutionPlan) -> Self {
        self.execution_plan = Some(Arc::new(execution_plan));
        self
    }

    pub fn project_root(&self) -> PathBuf {
        self.task
            .project_root_path()
            .unwrap_or_else(|| self.shared.project_root.clone())
    }

    pub fn elapsed_ms(&self) -> u64 {
        Utc::now()
            .signed_duration_since(self.started_at)
            .num_milliseconds()
            .max(0) as u64
    }
}
