use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use mc_agent::registry_min::AgentRegistry;
use mc_agent::trait_def_min::{Agent, AgentResult, ReviewVerdict, TestReport};
use mc_agent::{
    AgentConfig, Coder, CognitivePipeline, Explorer, ImpactAnalyzer, Planner, Reviewer,
    SharedResources, Tester,
};
use mc_context::{CodeConventions, ProjectContext, ProjectInfo, RiskArea, ScanMetadata, TechStack};
use mc_core::{AgentType, ResultType, TaskDescription, TaskIntent, TaskResult};
use mc_llm::{
    ChatMessage, ChatRequest, EventBus, InMemoryEventBus, LlmProvider, MessageRole, ResponseFormat,
};
use mc_recursive::RecursiveStats;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::CoordinatorConfig;
use crate::error::CoordinatorError;
use crate::intent::{
    intent_analysis_schema, keyword_fallback, keyword_fast_path, IntentAnalysis, UserIntent,
};
use crate::phase::{
    AgentExecutionState, AgentRuntimeStatus, ExecutionError, ExecutionPhase, ExecutionStatus,
};
use crate::plan::allocate_agent_budgets;
use crate::response::{CoordinatorResponse, ResponseType};
use crate::routing::{
    select_agent_set, CalibrationRecord, ComplexityConfig, ComplexityEvaluation,
    ComplexityEvaluator, RouteLevel,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HandoffFormat {
    Structured,
    CompressedSummary,
    DirectInjection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryState {
    Empty,
    Valid,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonTask {
    pub description: String,
    pub route_level: RouteLevel,
}

impl DaemonTask {
    pub fn new(description: impl Into<String>, route_level: RouteLevel) -> Self {
        Self {
            description: description.into(),
            route_level,
        }
    }
}

#[derive(Debug, Clone)]
struct IntegrationContext {
    project_context: Option<ProjectContext>,
}

#[derive(Debug, Clone)]
struct IntegrationResult {
    changed_files: Vec<String>,
    review_issues: Vec<String>,
    test_results: Vec<TestReport>,
    total_tokens_used: usize,
    total_duration_ms: u64,
    handoff_format: HandoffFormat,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MemoryMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    updated_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ttl_seconds: Option<u64>,
    #[serde(default)]
    files: Vec<String>,
}

impl MemoryMeta {
    fn is_valid(&self, stale_days: i64) -> bool {
        let Some(updated_at) = self.updated_at else {
            return false;
        };

        let ttl_seconds = self
            .ttl_seconds
            .unwrap_or_else(|| stale_days.max(1) as u64 * 24 * 60 * 60);

        Utc::now()
            .signed_duration_since(updated_at)
            .num_seconds()
            .max(0) as u64
            <= ttl_seconds
    }
}

pub struct Coordinator {
    pub config: CoordinatorConfig,
    pub(crate) llm_client: Arc<dyn LlmProvider>,
    pub(crate) agent_registry: Arc<AgentRegistry>,
    pub(crate) project_root: PathBuf,
    pub(crate) project_context: Option<Arc<ProjectContext>>,
    pub(crate) complexity_evaluator: Arc<RwLock<ComplexityEvaluator>>,
    pub(crate) execution_status: Arc<RwLock<Option<ExecutionStatus>>>,
    pub(crate) event_bus: Arc<dyn EventBus>,
    pub(crate) memory_state: Arc<RwLock<Option<MemoryState>>>,
    pub(crate) recursive_stats: Option<RecursiveStats>,
}

impl Coordinator {
    pub fn new(
        config: CoordinatorConfig,
        llm_client: Arc<dyn LlmProvider>,
        agent_registry: Arc<AgentRegistry>,
        project_root: PathBuf,
    ) -> Result<Self, CoordinatorError> {
        let mut complexity_config = Self::load_complexity_config(&project_root)?;
        complexity_config.llm_weight_multiplier = config.llm_weight_multiplier;
        let project_context = Self::load_project_memory_sync(&project_root, &config).map(Arc::new);
        let memory_state = if project_context.is_some() {
            Some(MemoryState::Valid)
        } else {
            Some(MemoryState::Empty)
        };

        Ok(Self {
            config,
            llm_client,
            agent_registry,
            project_root,
            project_context,
            complexity_evaluator: Arc::new(RwLock::new(ComplexityEvaluator::new(
                complexity_config,
            ))),
            execution_status: Arc::new(RwLock::new(None)),
            event_bus: Arc::new(InMemoryEventBus::default()),
            memory_state: Arc::new(RwLock::new(memory_state)),
            recursive_stats: None,
        })
    }

    pub fn config(&self) -> &CoordinatorConfig {
        &self.config
    }

    pub fn project_root(&self) -> &PathBuf {
        &self.project_root
    }

    pub async fn execution_status(&self) -> Option<ExecutionStatus> {
        self.execution_status.read().await.clone()
    }

    pub async fn is_busy(&self) -> bool {
        self.execution_status.read().await.is_some()
    }

    pub fn project_context(&self) -> Option<&Arc<ProjectContext>> {
        self.project_context.as_ref()
    }

    pub async fn complexity_config(&self) -> ComplexityConfig {
        self.complexity_evaluator.read().await.config.clone()
    }

    pub async fn memory_state(&self) -> Option<MemoryState> {
        self.memory_state.read().await.clone()
    }

    pub fn registered_agents(&self) -> Vec<AgentType> {
        self.agent_registry.list_types()
    }

    pub async fn recursive_stats(&self) -> Option<RecursiveStats> {
        self.recursive_stats.clone()
    }

    pub async fn handle_request(
        &self,
        request: &str,
    ) -> Result<CoordinatorResponse, CoordinatorError> {
        let started = Instant::now();
        self.begin_execution().await;
        self.set_phase(ExecutionPhase::Receiving, 0.05).await;
        info!(request_len = request.len(), "coordinator received request");

        self.set_phase(ExecutionPhase::Understanding, 0.15).await;
        let intent_analysis = self.recognize_intent(request).await?;
        if let Some(clarification) = &intent_analysis.clarifications {
            if !clarification.questions.is_empty() {
                self.set_phase(ExecutionPhase::Clarifying, 0.25).await;
                self.finish_execution(ExecutionPhase::Completed, 1.0).await;
                return Ok(CoordinatorResponse {
                    response_type: ResponseType::ClarificationNeeded,
                    content: serde_json::to_string_pretty(clarification)?,
                    ..CoordinatorResponse::default()
                });
            }
        }
        let intent = intent_analysis.intent;

        self.set_phase(ExecutionPhase::LoadingMemory, 0.35).await;
        let project_ctx = self.load_project_memory(&self.project_root).await?;
        {
            let mut memory_state = self.memory_state.write().await;
            *memory_state = Some(if project_ctx.is_some() {
                MemoryState::Valid
            } else {
                MemoryState::Empty
            });
        }

        self.set_phase(ExecutionPhase::EvaluatingComplexity, 0.5)
            .await;
        let evaluation = {
            let evaluator = self.complexity_evaluator.read().await;
            evaluator.evaluate_with_details(&intent, project_ctx.as_ref())
        };
        info!(route_level = ?evaluation.route_level, score = evaluation.score, "complexity evaluation completed");

        self.set_phase(ExecutionPhase::Routing, 0.6).await;
        let agents = self.select_agents(&evaluation.route_level, &intent, project_ctx.is_some());
        let budget = self.allocate_budget(&agents, &evaluation.route_level);

        if self.should_use_cognitive_pipeline(&intent, &evaluation) {
            self.set_phase(ExecutionPhase::Dispatching, 0.75).await;
            match self
                .handle_with_cognitive_pipeline(request, &intent, &evaluation)
                .await
            {
                Ok(response) => {
                    self.finish_execution(ExecutionPhase::Completed, 1.0).await;
                    return Ok(response);
                }
                Err(error) => {
                    warn!(error = %error, "specialized cognitive pipeline failed, falling back to legacy coordinator path");
                }
            }
        }

        self.set_phase(ExecutionPhase::Dispatching, 0.75).await;
        let results = self
            .dispatch_tasks_streaming(&agents, &intent, &budget)
            .await?;

        self.set_phase(ExecutionPhase::Integrating, 0.9).await;
        let integration = self
            .integrate_results(
                &results,
                &IntegrationContext {
                    project_context: project_ctx.clone(),
                },
            )
            .await;

        self.set_phase(ExecutionPhase::Delivering, 0.97).await;
        let response = CoordinatorResponse {
            response_type: ResponseType::Completed,
            content: self.format_delivery(&intent, &evaluation, &integration),
            changed_files: integration.changed_files.clone(),
            review_issues: integration.review_issues.clone(),
            test_results: integration.test_results.clone(),
            total_tokens_used: integration.total_tokens_used,
            total_duration_ms: started.elapsed().as_millis() as u64,
        };

        self.record_calibration(&evaluation, &integration).await;
        self.finish_execution(ExecutionPhase::Completed, 1.0).await;
        Ok(response)
    }

    pub async fn handle_daemon_task(
        &self,
        task: DaemonTask,
    ) -> Result<TaskResult, CoordinatorError> {
        let shared_context = self
            .project_context
            .as_ref()
            .ok_or(CoordinatorError::DaemonContextUnavailable)?;

        match task.route_level {
            RouteLevel::Simple => {
                let coder = self.create_agent(AgentType::Coder)?;
                coder
                    .execute_with_context(&task.description, shared_context)
                    .await
                    .map_err(Into::into)
            }
            RouteLevel::Medium => {
                let planner = self.create_agent(AgentType::Planner)?;
                let plan = planner.plan(&task.description, shared_context).await?;
                let coder = self.create_agent(AgentType::Coder)?;
                coder
                    .execute_plan(plan, shared_context)
                    .await
                    .map_err(Into::into)
            }
            _ => self.dispatch_daemon_complex(task, shared_context).await,
        }
    }

    pub async fn load_project_memory(
        &self,
        project_root: &Path,
    ) -> Result<Option<ProjectContext>, CoordinatorError> {
        Ok(Self::load_project_memory_async(project_root, &self.config).await)
    }

    fn should_use_cognitive_pipeline(
        &self,
        intent: &UserIntent,
        evaluation: &ComplexityEvaluation,
    ) -> bool {
        let supported_task = matches!(
            intent.task_type,
            crate::TaskType::FeatureDevelopment
                | crate::TaskType::BugFix
                | crate::TaskType::Refactoring
                | crate::TaskType::Testing
                | crate::TaskType::CodeReview
                | crate::TaskType::Debugging
        );

        supported_task
            && !intent.needs_research
            && self.project_root.join("Cargo.toml").exists()
            && !matches!(evaluation.route_level, RouteLevel::Simple)
    }

    async fn handle_with_cognitive_pipeline(
        &self,
        request: &str,
        intent: &UserIntent,
        evaluation: &ComplexityEvaluation,
    ) -> Result<CoordinatorResponse, CoordinatorError> {
        let task = self.build_cognitive_task(request, intent);
        let shared = SharedResources::new(self.project_root.clone(), Arc::clone(&self.llm_client));
        let pipeline = CognitivePipeline::new(
            Explorer::new(AgentConfig::for_agent_type(AgentType::Explorer)),
            ImpactAnalyzer::new(AgentConfig::for_agent_type(AgentType::ImpactAnalyzer)),
            Planner::new(AgentConfig::for_agent_type(AgentType::Planner)),
        )
        .with_execution_agents(
            Coder::new(AgentConfig::for_agent_type(AgentType::Coder)),
            Reviewer::new(AgentConfig::for_agent_type(AgentType::Reviewer)),
            Tester::new(AgentConfig::for_agent_type(AgentType::Tester)),
        );

        let started = Instant::now();
        let result = pipeline
            .execute_full(&task, &shared)
            .await
            .map_err(|error| CoordinatorError::Internal(error.to_string()))?;

        let review_issues = result
            .review_report
            .findings
            .iter()
            .filter(|finding| {
                matches!(
                    finding.severity,
                    mc_agent::ReviewSeverity::Blocker | mc_agent::ReviewSeverity::Warning
                )
            })
            .map(|finding| finding.title.clone())
            .collect::<Vec<_>>();
        let changed_files = result
            .coder_output
            .changes
            .iter()
            .map(|change| change.path.clone())
            .collect::<Vec<_>>();
        let test_results = vec![TestReport {
            summary: if result.tester_report.summary.stdout_tail.trim().is_empty() {
                result.tester_execution.summary.clone()
            } else {
                result.tester_report.summary.stdout_tail.clone()
            },
            passed: result.tester_report.summary.passed,
            failed: result.tester_report.summary.failed,
            coverage: None,
        }];

        Ok(CoordinatorResponse {
            response_type: ResponseType::Completed,
            content: format!(
                "Task type: {}\nRoute level: {:?}\nPipeline: cognitive\nPlanning: {}\nCoder: {}\nReviewer: {}\nTester: {}",
                intent.task_type.as_key(),
                evaluation.route_level,
                result.planning.planner_report.summary,
                result.coder_report.summary,
                result.reviewer_execution.summary,
                result.tester_execution.summary,
            ),
            changed_files,
            review_issues,
            test_results,
            total_tokens_used: result.planning.explorer_report.metrics.tokens_used as usize
                + result.planning.impact_report_execution.metrics.tokens_used as usize
                + result.planning.planner_report.metrics.tokens_used as usize
                + result.coder_report.metrics.tokens_used as usize
                + result.reviewer_execution.metrics.tokens_used as usize
                + result.tester_execution.metrics.tokens_used as usize,
            total_duration_ms: started.elapsed().as_millis() as u64,
        })
    }

    fn build_cognitive_task(&self, request: &str, intent: &UserIntent) -> TaskDescription {
        TaskDescription {
            id: Uuid::new_v4().to_string(),
            user_input: request.to_string(),
            intent: map_task_type_to_core(&intent.task_type),
            complexity: intent.estimated_complexity,
            affected_files: intent.target_files.clone(),
            requires_new_dependency: matches!(intent.task_type, crate::TaskType::Configuration),
            involves_architecture_change: matches!(
                intent.estimated_complexity,
                mc_core::Complexity::Complex
            ),
            needs_external_research: intent.needs_research,
            requires_testing: matches!(
                intent.task_type,
                crate::TaskType::FeatureDevelopment
                    | crate::TaskType::BugFix
                    | crate::TaskType::Refactoring
                    | crate::TaskType::Testing
                    | crate::TaskType::Debugging
            ),
            forced_agents: None,
            constraints: if intent.domains.is_empty() {
                Vec::new()
            } else {
                vec![format!("domains={}", intent.domains.join(","))]
            },
            details: None,
            project_root: Some(self.project_root.to_string_lossy().into_owned()),
            created_at: Utc::now(),
        }
    }

    async fn recognize_intent(&self, request: &str) -> Result<IntentAnalysis, CoordinatorError> {
        if let Some(intent) = keyword_fast_path(request) {
            info!(task_type = %intent.task_type.as_key(), "keyword fast path matched");
            return Ok(IntentAnalysis {
                intent,
                clarifications: None,
            });
        }

        match self.llm_intent_analysis(request).await {
            Ok(analysis) => Ok(analysis),
            Err(error @ CoordinatorError::IntentParseFailed { .. }) => Err(error),
            Err(error) => {
                warn!(error = %error, "intent llm failed, falling back to keyword heuristics");
                Ok(IntentAnalysis {
                    intent: keyword_fallback(request),
                    clarifications: None,
                })
            }
        }
    }

    async fn llm_intent_analysis(&self, request: &str) -> Result<IntentAnalysis, CoordinatorError> {
        let prompt = format!(
            "Analyze the user request and return JSON only.\n\
Return both the inferred intent and any clarification questions.\n\
Do not add facts that are not supported by the request.\n\
If the request is actionable without clarification, return null for clarifications.\n\n\
User request:\n{request}"
        );

        let response = self
            .llm_client
            .chat(
                ChatRequest {
                    messages: vec![
                        ChatMessage::text(MessageRole::System, prompt),
                        ChatMessage::text(MessageRole::User, request),
                    ],
                    model: None,
                    temperature: 0.0,
                    top_p: None,
                    max_tokens: Some(800),
                    stop_sequences: Vec::new(),
                    tools: Vec::new(),
                    response_format: Some(ResponseFormat::JsonSchema {
                        schema: intent_analysis_schema(),
                        name: "intent_analysis".into(),
                        strict: true,
                    }),
                    timeout: None,
                    user_id: None,
                    cache_control_points: Vec::new(),
                    extra_headers: HashMap::new(),
                    request_id: Some(Uuid::new_v4().to_string()),
                },
                CancellationToken::new(),
            )
            .await?;

        let raw = response.message.content.to_text();
        self.parse_or_repair_json("intent_analysis", request, &raw)
            .await
    }

    async fn parse_or_repair_json<T>(
        &self,
        schema_name: &str,
        user_request: &str,
        raw: &str,
    ) -> Result<T, CoordinatorError>
    where
        T: DeserializeOwned,
    {
        match serde_json::from_str(raw) {
            Ok(value) => Ok(value),
            Err(first_err) => {
                warn!(%schema_name, error = %first_err, "llm json parse failed, attempting repair");
                let extract_prompt = format!(
                    "Extract structured data from the following text and return valid JSON only.\n\
Requirements:\n\
1. Only use facts explicitly present in the text.\n\
2. Use null when a field cannot be determined.\n\
3. Do not explain the answer.\n\n\
Original user request:\n{user_request}\n\n\
Schema name: {schema_name}\n\n\
Source text:\n{raw}"
                );

                let repaired = self
                    .llm_client
                    .chat(
                        ChatRequest {
                            messages: vec![ChatMessage::text(MessageRole::System, extract_prompt)],
                            model: None,
                            temperature: 0.0,
                            top_p: None,
                            max_tokens: Some(500),
                            stop_sequences: Vec::new(),
                            tools: Vec::new(),
                            response_format: Some(ResponseFormat::JsonObject),
                            timeout: None,
                            user_id: None,
                            cache_control_points: Vec::new(),
                            extra_headers: HashMap::new(),
                            request_id: Some(Uuid::new_v4().to_string()),
                        },
                        CancellationToken::new(),
                    )
                    .await?
                    .message
                    .content
                    .to_text();

                serde_json::from_str(&repaired).map_err(|repair_err| {
                    CoordinatorError::IntentParseFailed {
                        schema: schema_name.to_string(),
                        first_error: first_err.to_string(),
                        repair_error: repair_err.to_string(),
                    }
                })
            }
        }
    }

    fn select_agents(
        &self,
        route_level: &RouteLevel,
        intent: &UserIntent,
        has_memory: bool,
    ) -> Vec<AgentType> {
        select_agent_set(
            route_level,
            &intent.task_type,
            self.config.memory_aware_routing,
            has_memory,
            self.config.preflight_check,
        )
    }

    fn allocate_budget(
        &self,
        agents: &[AgentType],
        route_level: &RouteLevel,
    ) -> HashMap<AgentType, u32> {
        allocate_agent_budgets(self.config.max_token_budget, agents, route_level)
    }

    async fn dispatch_tasks_streaming(
        &self,
        agents: &[AgentType],
        intent: &UserIntent,
        budget: &HashMap<AgentType, u32>,
    ) -> Result<Vec<AgentResult>, CoordinatorError> {
        let mut handles = Vec::new();

        for agent_type in agents {
            let agent = self.create_agent(*agent_type)?;
            let agent_key = format!("{}:{}", agent.agent_type().as_str(), agent.agent_id());
            let event_bus = Arc::clone(&self.event_bus);
            let task_description = intent.raw_request.clone();
            let token_budget = budget.get(agent_type).copied().unwrap_or(0) as u64;
            let execution_status = Arc::clone(&self.execution_status);
            let agent_type_copy = *agent_type;

            {
                let mut status_guard = execution_status.write().await;
                if let Some(status) = status_guard.as_mut() {
                    status.agent_statuses.insert(
                        agent_key.clone(),
                        AgentRuntimeStatus {
                            agent_type: agent_type_copy,
                            state: AgentExecutionState::Pending,
                            started_at: None,
                            completed_at: None,
                            tokens_used: 0,
                        },
                    );
                }
            }

            let handle = tokio::spawn(async move {
                {
                    let mut status_guard = execution_status.write().await;
                    if let Some(status) = status_guard.as_mut() {
                        if let Some(agent_status) = status.agent_statuses.get_mut(&agent_key) {
                            agent_status.state = AgentExecutionState::Running;
                            agent_status.started_at = Some(Utc::now());
                        }
                    }
                }

                let result = agent
                    .execute_streaming(&task_description, token_budget, event_bus)
                    .await;

                let mut status_guard = execution_status.write().await;
                if let Some(status) = status_guard.as_mut() {
                    if let Some(agent_status) = status.agent_statuses.get_mut(&agent_key) {
                        match &result {
                            Ok(agent_result) => {
                                agent_status.state = AgentExecutionState::Completed;
                                agent_status.tokens_used = agent_result.tokens_used;
                                agent_status.completed_at = Some(Utc::now());
                                status.tokens_used += agent_result.tokens_used;
                                status.tokens_remaining = status
                                    .tokens_remaining
                                    .saturating_sub(agent_result.tokens_used);
                            }
                            Err(error) => {
                                agent_status.state = AgentExecutionState::Failed(error.to_string());
                                agent_status.completed_at = Some(Utc::now());
                                status.errors.push(ExecutionError {
                                    phase: ExecutionPhase::Dispatching,
                                    message: format!(
                                        "{} failed: {error}",
                                        agent_type_copy.as_str()
                                    ),
                                    occurred_at: Utc::now(),
                                });
                            }
                        }
                    }
                }

                result
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(error)) => warn!(error = %error, "agent execution failed during dispatch"),
                Err(error) => warn!(error = %error, "agent task join failed"),
            }
        }

        Ok(results)
    }

    async fn integrate_results(
        &self,
        results: &[AgentResult],
        ctx: &IntegrationContext,
    ) -> IntegrationResult {
        let mut changed_files = HashSet::new();
        let mut review_issues = Vec::new();
        let mut test_results = Vec::new();

        for result in results {
            if let Some(handoff) = &result.handoff {
                changed_files.extend(handoff.changed_files.iter().cloned());
            }

            if let Some(task_result) = &result.task_result {
                changed_files.extend(task_result.changed_files.iter().cloned());
            }

            if let Some(review_report) = &result.review_report {
                if !matches!(review_report.verdict, ReviewVerdict::Passed) {
                    review_issues.extend(review_report.issues.clone());
                }
            }

            if let Some(test_report) = &result.test_report {
                test_results.push(test_report.clone());
            }
        }

        if let Some(project_context) = &ctx.project_context {
            changed_files.extend(
                project_context
                    .notes
                    .iter()
                    .filter_map(|note| note.strip_prefix("Focus file: ").map(ToOwned::to_owned)),
            );
        }

        let mut changed_files = changed_files.into_iter().collect::<Vec<_>>();
        changed_files.sort();

        IntegrationResult {
            changed_files,
            review_issues,
            test_results,
            total_tokens_used: results.iter().map(|result| result.tokens_used).sum(),
            total_duration_ms: results.iter().map(|result| result.duration_ms).sum(),
            handoff_format: HandoffFormat::Structured,
        }
    }

    fn format_delivery(
        &self,
        intent: &UserIntent,
        evaluation: &ComplexityEvaluation,
        integration: &IntegrationResult,
    ) -> String {
        let mut lines = vec![
            format!("Task type: {}", intent.task_type.as_key()),
            format!("Route level: {:?}", evaluation.route_level),
            format!("Handoff format: {:?}", integration.handoff_format),
        ];

        if !integration.changed_files.is_empty() {
            lines.push(format!(
                "Changed files: {}",
                integration.changed_files.join(", ")
            ));
        }

        if !integration.test_results.is_empty() {
            lines.push(format!("Test reports: {}", integration.test_results.len()));
        }

        if !integration.review_issues.is_empty() {
            lines.push(format!(
                "Review issues: {}",
                integration.review_issues.join("; ")
            ));
        }

        if integration.total_duration_ms > 0 {
            lines.push(format!(
                "Integrated duration: {} ms",
                integration.total_duration_ms
            ));
        }

        lines.join("\n")
    }

    async fn record_calibration(
        &self,
        evaluation: &ComplexityEvaluation,
        integration: &IntegrationResult,
    ) {
        let mut evaluator = self.complexity_evaluator.write().await;
        evaluator.record_calibration(CalibrationRecord {
            factors: evaluation.factors.clone(),
            evaluated_score: evaluation.score,
            evaluated_route: evaluation.route_level.clone(),
            actual_needed_more_agents: integration.review_issues.len() > 2,
            actual_tokens_used: integration.total_tokens_used as u64,
            timestamp: Utc::now(),
        });
        let _ = evaluator.calibrate_from_history();
    }

    async fn dispatch_daemon_complex(
        &self,
        task: DaemonTask,
        shared_context: &Arc<ProjectContext>,
    ) -> Result<TaskResult, CoordinatorError> {
        let intent = keyword_fallback(&task.description);
        let agents = self.select_agents(&task.route_level, &intent, true);
        let mut last_result = TaskResult {
            result_type: ResultType::CodeChange,
            success: true,
            data: serde_json::json!({}),
            changed_files: Vec::new(),
            generated_content: None,
            error_message: None,
        };
        let mut plan = None;

        for agent_type in agents {
            let agent = self.create_agent(agent_type)?;
            match agent_type {
                AgentType::Planner => {
                    plan = Some(agent.plan(&task.description, shared_context).await?);
                }
                AgentType::Coder => {
                    last_result = if let Some(plan) = plan.clone() {
                        agent.execute_plan(plan, shared_context).await?
                    } else {
                        agent
                            .execute_with_context(&task.description, shared_context)
                            .await?
                    };
                }
                _ => {
                    last_result = agent
                        .execute_with_context(&task.description, shared_context)
                        .await?;
                }
            }
        }

        Ok(last_result)
    }

    fn create_agent(&self, agent_type: AgentType) -> Result<Arc<dyn Agent>, CoordinatorError> {
        self.agent_registry
            .create_agent(agent_type)
            .map_err(Into::into)
    }

    async fn begin_execution(&self) {
        let mut status = self.execution_status.write().await;
        *status = Some(ExecutionStatus::new(self.config.max_token_budget as usize));
    }

    async fn set_phase(&self, phase: ExecutionPhase, progress: f32) {
        let mut status = self.execution_status.write().await;
        if let Some(status) = status.as_mut() {
            status.current_phase = phase;
            status.progress_percent = progress;
        }
    }

    async fn finish_execution(&self, phase: ExecutionPhase, progress: f32) {
        let mut status = self.execution_status.write().await;
        if let Some(status) = status.as_mut() {
            status.current_phase = phase;
            status.progress_percent = progress;
        }
    }

    fn load_complexity_config(project_root: &Path) -> Result<ComplexityConfig, CoordinatorError> {
        let candidates = [
            project_root.join(".morecode").join("routing.toml"),
            project_root.join(".morecode").join("routing.json"),
        ];

        for path in candidates {
            if path.exists() {
                return ComplexityEvaluator::from_file(&path).map(|evaluator| evaluator.config);
            }
        }

        Ok(ComplexityConfig::default())
    }

    fn load_project_memory_sync(
        project_root: &Path,
        config: &CoordinatorConfig,
    ) -> Option<ProjectContext> {
        let memory_dir = project_root.join(".assistant-memory");
        let meta_path = memory_dir.join("memory-meta.json");
        if !meta_path.exists() {
            return None;
        }

        let meta = std::fs::read_to_string(&meta_path)
            .ok()
            .and_then(|content| serde_json::from_str::<MemoryMeta>(&content).ok())?;
        if !meta.is_valid(config.memory_stale_threshold_days) {
            return None;
        }

        Self::build_project_context_from_disk(project_root, &memory_dir, &meta)
    }

    async fn load_project_memory_async(
        project_root: &Path,
        config: &CoordinatorConfig,
    ) -> Option<ProjectContext> {
        let memory_dir = project_root.join(".assistant-memory");
        let meta_path = memory_dir.join("memory-meta.json");
        if !meta_path.exists() {
            return None;
        }

        let meta = tokio::fs::read_to_string(&meta_path)
            .await
            .ok()
            .and_then(|content| serde_json::from_str::<MemoryMeta>(&content).ok())?;
        if !meta.is_valid(config.memory_stale_threshold_days) {
            return None;
        }

        Self::build_project_context_from_disk(project_root, &memory_dir, &meta)
    }

    fn build_project_context_from_disk(
        project_root: &Path,
        memory_dir: &Path,
        meta: &MemoryMeta,
    ) -> Option<ProjectContext> {
        let context_path = memory_dir.join("project-context.json");
        if context_path.exists() {
            let mut context = std::fs::read_to_string(&context_path)
                .ok()
                .and_then(|content| serde_json::from_str::<ProjectContext>(&content).ok())?;
            if context.info.root_dir.as_os_str().is_empty() {
                context.info.root_dir = project_root.to_path_buf();
            }
            if context.info.name.is_empty() {
                context.info.name = project_root
                    .file_name()
                    .and_then(|item| item.to_str())
                    .unwrap_or("workspace")
                    .to_string();
            }
            return Some(context);
        }

        let overview = std::fs::read_to_string(memory_dir.join("project-overview.md")).ok();
        let tech_stack = read_optional_json_sync::<TechStack>(&memory_dir.join("tech-stack.json"))
            .unwrap_or_default();
        let conventions =
            read_optional_json_sync::<CodeConventions>(&memory_dir.join("conventions.json"))
                .unwrap_or_default();
        let risk_areas =
            read_optional_json_sync::<Vec<RiskArea>>(&memory_dir.join("risk-areas.json"))
                .unwrap_or_default();
        let scan_metadata =
            read_optional_json_sync::<ScanMetadata>(&memory_dir.join("scan-metadata.json"))
                .unwrap_or_default();

        if overview.is_none()
            && tech_stack == TechStack::default()
            && conventions == CodeConventions::default()
            && risk_areas.is_empty()
        {
            return None;
        }

        let primary_language = tech_stack.primary_language().map(ToOwned::to_owned);
        Some(ProjectContext {
            info: ProjectInfo {
                name: project_root
                    .file_name()
                    .and_then(|item| item.to_str())
                    .unwrap_or("workspace")
                    .to_string(),
                root_dir: project_root.to_path_buf(),
                primary_language,
                repository_url: None,
                summary: overview,
            },
            tech_stack,
            conventions,
            risk_areas,
            scan_metadata,
            impact_report: None,
            notes: meta.files.clone(),
        })
    }
}

fn read_optional_json_sync<T: DeserializeOwned>(path: &Path) -> Option<T> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
}

fn map_task_type_to_core(task_type: &crate::TaskType) -> TaskIntent {
    match task_type {
        crate::TaskType::FeatureDevelopment => TaskIntent::FeatureAddition,
        crate::TaskType::BugFix => TaskIntent::BugFix,
        crate::TaskType::Refactoring => TaskIntent::Refactoring,
        crate::TaskType::Documentation => TaskIntent::Documentation,
        crate::TaskType::Testing => TaskIntent::Other("testing".to_string()),
        crate::TaskType::Configuration => TaskIntent::Other("configuration".to_string()),
        crate::TaskType::CodeReview => TaskIntent::Other("code_review".to_string()),
        crate::TaskType::Debugging => TaskIntent::Other("debugging".to_string()),
        crate::TaskType::Other(value) if value.eq_ignore_ascii_case("research") => {
            TaskIntent::Research
        }
        crate::TaskType::Other(value) => TaskIntent::Other(value.clone()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use mc_llm::{CacheCapability, ChatResponse, FinishReason, ModelInfo, StreamEvent, TokenUsage};
    use tempfile::TempDir;

    use super::*;

    struct FakeLlmProvider {
        responses: Mutex<VecDeque<Result<String, mc_llm::LlmError>>>,
        call_count: AtomicUsize,
        model_info: ModelInfo,
    }

    impl FakeLlmProvider {
        fn new(responses: Vec<Result<String, mc_llm::LlmError>>) -> Self {
            Self {
                responses: Mutex::new(VecDeque::from(responses)),
                call_count: AtomicUsize::new(0),
                model_info: ModelInfo::new("fake-model", "Fake", "fake"),
            }
        }

        fn call_count(&self) -> usize {
            self.call_count.load(Ordering::Relaxed)
        }

        fn next_response(&self) -> Result<String, mc_llm::LlmError> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            self.responses
                .lock()
                .expect("fake llm queue lock poisoned")
                .pop_front()
                .unwrap_or_else(|| {
                    Ok(serde_json::json!({
                        "intent": {
                            "raw_request": "default",
                            "task_type": "FeatureDevelopment",
                            "target_files": [],
                            "domains": [],
                            "estimated_complexity": "simple",
                            "needs_project_context": false,
                            "needs_research": false
                        },
                        "clarifications": null
                    })
                    .to_string())
                })
        }
    }

    impl LlmProvider for FakeLlmProvider {
        fn provider_id(&self) -> &str {
            "fake"
        }

        fn model_info(&self) -> &ModelInfo {
            &self.model_info
        }

        fn chat(
            &self,
            _request: ChatRequest,
            _cancel_token: CancellationToken,
        ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, mc_llm::LlmError>> + Send + '_>>
        {
            Box::pin(async move {
                let content = self.next_response()?;
                Ok(ChatResponse {
                    id: Uuid::new_v4().to_string(),
                    model: self.model_info.id.clone(),
                    message: ChatMessage::text(MessageRole::Assistant, content),
                    usage: TokenUsage::default(),
                    finish_reason: FinishReason::Stop,
                    latency_ms: 1,
                    raw_response: None,
                })
            })
        }

        fn chat_stream(
            &self,
            _request: ChatRequest,
            _cancel_token: CancellationToken,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<tokio::sync::mpsc::Receiver<StreamEvent>, mc_llm::LlmError>,
                    > + Send
                    + '_,
            >,
        > {
            Box::pin(async move {
                let (tx, rx) = tokio::sync::mpsc::channel(4);
                let _ = tx
                    .send(StreamEvent::Finish {
                        reason: FinishReason::Stop,
                        usage: Some(TokenUsage::default()),
                        response_id: Uuid::new_v4().to_string(),
                    })
                    .await;
                Ok(rx)
            })
        }

        fn cache_capability(&self) -> CacheCapability {
            CacheCapability::default()
        }

        fn list_models(
            &self,
            _cancel_token: CancellationToken,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<ModelInfo>, mc_llm::LlmError>> + Send + '_>>
        {
            Box::pin(async move { Ok(vec![self.model_info.clone()]) })
        }

        fn cancel_request(&self, _request_id: &str) -> Result<(), mc_llm::LlmError> {
            Ok(())
        }

        fn estimate_tokens(&self, text: &str) -> usize {
            text.split_whitespace().count().max(1)
        }
    }

    fn build_coordinator(
        root: &Path,
        responses: Vec<Result<String, mc_llm::LlmError>>,
    ) -> Coordinator {
        let provider = Arc::new(FakeLlmProvider::new(responses));
        let registry = Arc::new(AgentRegistry::new());
        registry.register_defaults();

        Coordinator::new(
            CoordinatorConfig::default(),
            provider,
            registry,
            root.to_path_buf(),
        )
        .expect("coordinator should build")
    }

    fn build_rust_project() -> TempDir {
        let temp = TempDir::new().expect("temp dir should exist");
        std::fs::create_dir_all(temp.path().join("src")).expect("src dir");
        std::fs::write(
            temp.path().join("Cargo.toml"),
            r#"[package]
name = "pipeline-demo"
version = "0.1.0"
edition = "2021"
"#,
        )
        .expect("cargo manifest");
        std::fs::write(
            temp.path().join("src/lib.rs"),
            r#"pub fn compute() -> usize { 42 }"#,
        )
        .expect("lib file");
        temp
    }

    #[tokio::test]
    async fn recognize_intent_uses_keyword_fast_path_without_llm() {
        let temp = TempDir::new().expect("temp dir should exist");
        let provider = Arc::new(FakeLlmProvider::new(Vec::new()));
        let registry = Arc::new(AgentRegistry::new());
        registry.register_defaults();
        let coordinator = Coordinator::new(
            CoordinatorConfig::default(),
            Arc::clone(&provider) as Arc<dyn LlmProvider>,
            registry,
            temp.path().to_path_buf(),
        )
        .expect("coordinator should build");

        let analysis = coordinator
            .recognize_intent("修复 typo in src/lib.rs")
            .await
            .expect("intent recognition should succeed");

        assert_eq!(provider.call_count(), 0);
        assert_eq!(analysis.intent.task_type, crate::TaskType::BugFix);
    }

    #[tokio::test]
    async fn parse_or_repair_json_returns_explicit_intent_parse_failed() {
        let temp = TempDir::new().expect("temp dir should exist");
        let coordinator = build_coordinator(temp.path(), vec![Ok("{\"still\":\"broken\"".into())]);

        let error = coordinator
            .parse_or_repair_json::<crate::IntentAnalysis>("intent_analysis", "request", "{invalid")
            .await
            .expect_err("repair should fail");

        assert!(matches!(error, CoordinatorError::IntentParseFailed { .. }));
    }

    #[tokio::test]
    async fn recognize_intent_falls_back_when_llm_transport_fails() {
        let temp = TempDir::new().expect("temp dir should exist");
        let provider = Arc::new(FakeLlmProvider::new(vec![Err(mc_llm::LlmError::ApiError(
            "boom".into(),
        ))]));
        let registry = Arc::new(AgentRegistry::new());
        registry.register_defaults();
        let coordinator = Coordinator::new(
            CoordinatorConfig::default(),
            Arc::clone(&provider) as Arc<dyn LlmProvider>,
            registry,
            temp.path().to_path_buf(),
        )
        .expect("coordinator should build");

        let analysis = coordinator
            .recognize_intent("fix bug in src/lib.rs")
            .await
            .expect("fallback should succeed");

        assert_eq!(provider.call_count(), 1);
        assert_eq!(analysis.intent.task_type, crate::TaskType::BugFix);
    }

    #[tokio::test]
    async fn handle_request_runs_full_nine_step_pipeline() {
        let temp = TempDir::new().expect("temp dir should exist");
        let coordinator = build_coordinator(temp.path(), Vec::new());

        let response = coordinator
            .handle_request("修复 typo in src/lib.rs")
            .await
            .expect("request should succeed");

        assert_eq!(response.response_type, ResponseType::Completed);
        assert!(!response.changed_files.is_empty());
        assert!(!response.content.is_empty());
    }

    #[tokio::test]
    async fn handle_request_uses_cognitive_pipeline_when_project_is_available() {
        let temp = build_rust_project();
        let coordinator = build_coordinator(
            temp.path(),
            vec![
                Ok(serde_json::json!({
                    "project_summary": "Single-crate workspace",
                    "architecture_name": "Library",
                    "architecture_description": "A small Rust crate",
                    "design_decisions": ["Keep the crate lightweight"],
                    "notable_patterns": ["library"]
                })
                .to_string()),
                Ok(serde_json::json!({
                    "compatibility_notes": ["Review downstream users of compute"],
                    "recommendations": ["Run cargo test"],
                    "risk_assessment": []
                })
                .to_string()),
                Ok(serde_json::json!({
                    "summary": "Update compute implementation, then review and test",
                    "review_focus": ["public function compute"]
                })
                .to_string()),
                Ok(serde_json::json!({
                    "summary": "Adjust compute implementation in src/lib.rs",
                    "implementation_notes": ["Preserve the exported API"],
                    "changes": [{
                        "path": "src/lib.rs",
                        "change_kind": "modify",
                        "rationale": "The task targets the library implementation",
                        "patch_preview": "",
                        "acceptance_checks": ["cargo test"]
                    }],
                    "validation_steps": ["cargo test"],
                    "risks": []
                })
                .to_string()),
                Ok(serde_json::json!({
                    "summary": "Review completed without blocking issues.",
                    "verdict": "approved",
                    "additional_findings": []
                })
                .to_string()),
            ],
        );

        let response = coordinator
            .handle_request("deep debug src/lib.rs")
            .await
            .expect("request should succeed");

        assert_eq!(response.response_type, ResponseType::Completed);
        assert!(response.content.contains("Pipeline: cognitive"));
        assert!(response
            .changed_files
            .iter()
            .any(|path| path == "src/lib.rs"));
        assert_eq!(response.test_results.len(), 1);
    }

    #[tokio::test]
    async fn load_project_memory_handles_missing_and_valid_memory() {
        let temp = TempDir::new().expect("temp dir should exist");
        let coordinator = build_coordinator(temp.path(), Vec::new());
        let missing = coordinator
            .load_project_memory(temp.path())
            .await
            .expect("load should not error");
        assert!(missing.is_none());

        let memory_dir = temp.path().join(".assistant-memory");
        std::fs::create_dir_all(&memory_dir).expect("memory dir should be created");
        std::fs::write(
            memory_dir.join("memory-meta.json"),
            serde_json::json!({
                "updated_at": Utc::now(),
                "ttl_seconds": 3600,
                "files": ["src/lib.rs"]
            })
            .to_string(),
        )
        .expect("meta should be written");
        std::fs::write(
            memory_dir.join("project-overview.md"),
            "Coordinator memory overview",
        )
        .expect("overview should be written");
        std::fs::write(
            memory_dir.join("tech-stack.json"),
            serde_json::json!({
                "languages": ["Rust"],
                "frameworks": [],
                "package_managers": ["cargo"],
                "databases": []
            })
            .to_string(),
        )
        .expect("tech stack should be written");

        let loaded = coordinator
            .load_project_memory(temp.path())
            .await
            .expect("load should not error");
        assert!(loaded.is_some());
    }

    #[test]
    fn complexity_config_default_simple_threshold_is_fifteen() {
        assert_eq!(ComplexityConfig::default().route_thresholds.simple_max, 15);
    }

    #[test]
    fn calibrate_from_history_requires_minimum_records() {
        let mut evaluator = ComplexityEvaluator::new(ComplexityConfig::default());
        for _ in 0..19 {
            evaluator.record_calibration(CalibrationRecord {
                factors: crate::ComplexityFactors {
                    file_count: 1,
                    task_type: crate::TaskType::BugFix.as_key(),
                    needs_context: false,
                    domain_count: 1,
                    project_size: 10,
                    llm_complexity: mc_core::Complexity::Simple,
                },
                evaluated_score: 10,
                evaluated_route: RouteLevel::Simple,
                actual_needed_more_agents: false,
                actual_tokens_used: 1_000,
                timestamp: Utc::now(),
            });
        }
        assert!(evaluator.calibrate_from_history().is_none());

        evaluator.record_calibration(CalibrationRecord {
            factors: crate::ComplexityFactors {
                file_count: 2,
                task_type: crate::TaskType::FeatureDevelopment.as_key(),
                needs_context: true,
                domain_count: 2,
                project_size: 100,
                llm_complexity: mc_core::Complexity::Medium,
            },
            evaluated_score: 40,
            evaluated_route: RouteLevel::Medium,
            actual_needed_more_agents: true,
            actual_tokens_used: 8_000,
            timestamp: Utc::now(),
        });

        assert!(evaluator.calibrate_from_history().is_some());
    }
}
