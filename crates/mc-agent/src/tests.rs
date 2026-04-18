use async_trait::async_trait;
use chrono::Utc;
use mc_core::{
    ArchitecturePattern, CodeConventions, ContextAllocation, DependencyGraph, DocumentationConvention,
    ErrorHandlingPattern, ExecutionPlan, NamingConvention, PlanMetadata, ProjectContext,
    ProjectInfo, ProjectStructure, RiskAreas, ScanMetadata, TaskDescription, TechStack,
    TestingConvention,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::{
    execute_streaming_with_context, execute_with_context, Agent, AgentConfig, AgentContext,
    AgentError, AgentExecutionReport, AgentLifecycle, AgentRegistry, AgentType, ErrorCategory,
    EventBus, ExecutionStatus, GlobalConfig, ImpactReport, ImpactRiskLevel, InMemoryEventBus,
    LifecycleHandler, LlmClient, LlmRequest, LlmResponse, Logger, LoggingLifecycle,
    MemoryManager, SharedResources, StdoutStreamForwarder, StreamEvent, StreamForwarder,
    ToolRegistry,
};

struct DummyLlmClient;

#[async_trait]
impl LlmClient for DummyLlmClient {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, AgentError> {
        Ok(LlmResponse {
            content: request.prompt,
            model_id: "dummy".to_string(),
            metadata: request.metadata,
        })
    }
}

#[derive(Default)]
struct BufferLogger {
    messages: Mutex<Vec<String>>,
}

impl BufferLogger {
    fn snapshot(&self) -> Vec<String> {
        self.messages.lock().expect("buffer logger lock").clone()
    }
}

impl Logger for BufferLogger {
    fn info(&self, message: &str) {
        self.messages.lock().expect("buffer logger lock").push(message.to_string());
    }

    fn warn(&self, message: &str) {
        self.messages.lock().expect("buffer logger lock").push(message.to_string());
    }

    fn error(&self, message: &str) {
        self.messages.lock().expect("buffer logger lock").push(message.to_string());
    }
}

struct MockAgent {
    agent_type: AgentType,
    config: AgentConfig,
}

impl MockAgent {
    fn new(agent_type: AgentType) -> Self {
        Self {
            agent_type,
            config: AgentConfig::for_agent_type(agent_type),
        }
    }
}

#[async_trait]
impl Agent for MockAgent {
    fn agent_type(&self) -> AgentType {
        self.agent_type
    }

    async fn build_context(
        &self,
        task: &TaskDescription,
        project_ctx: Option<ProjectContext>,
        shared: &SharedResources,
    ) -> Result<AgentContext, AgentError> {
        let mut ctx = AgentContext::new(task.clone(), shared, self.config.clone());
        ctx.project_ctx = project_ctx.map(Arc::new);
        Ok(ctx)
    }

    async fn execute(&self, ctx: &AgentContext) -> Result<AgentExecutionReport, AgentError> {
        Ok(AgentExecutionReport::success(
            self.agent_type,
            ctx.execution_id,
            json!({ "agent": self.agent_type.identifier() }),
        ))
    }

    fn default_config(&self) -> AgentConfig {
        self.config.clone()
    }

    fn update_config(&mut self, config: AgentConfig) {
        self.config = config;
    }
}

struct SlowAgent {
    delay: Duration,
    config: AgentConfig,
}

impl SlowAgent {
    fn new(delay: Duration) -> Self {
        Self {
            delay,
            config: AgentConfig::for_agent_type(AgentType::Coder),
        }
    }
}

#[async_trait]
impl Agent for SlowAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Coder
    }

    async fn build_context(
        &self,
        task: &TaskDescription,
        project_ctx: Option<ProjectContext>,
        shared: &SharedResources,
    ) -> Result<AgentContext, AgentError> {
        let mut ctx = AgentContext::new(task.clone(), shared, self.config.clone());
        ctx.project_ctx = project_ctx.map(Arc::new);
        Ok(ctx)
    }

    async fn execute(&self, ctx: &AgentContext) -> Result<AgentExecutionReport, AgentError> {
        tokio::time::sleep(self.delay).await;
        Ok(AgentExecutionReport::success(
            AgentType::Coder,
            ctx.execution_id,
            json!({ "slow": true }),
        ))
    }

    fn default_config(&self) -> AgentConfig {
        self.config.clone()
    }

    fn update_config(&mut self, config: AgentConfig) {
        self.config = config;
    }
}

struct RetryableAgent {
    remaining_failures: AtomicUsize,
    config: AgentConfig,
}

impl RetryableAgent {
    fn new(failures: usize) -> Self {
        Self {
            remaining_failures: AtomicUsize::new(failures),
            config: AgentConfig::for_agent_type(AgentType::Coder),
        }
    }

    fn attempts(&self) -> usize {
        self.remaining_failures.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl Agent for RetryableAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Coder
    }

    async fn build_context(
        &self,
        task: &TaskDescription,
        project_ctx: Option<ProjectContext>,
        shared: &SharedResources,
    ) -> Result<AgentContext, AgentError> {
        let mut ctx = AgentContext::new(task.clone(), shared, self.config.clone());
        ctx.project_ctx = project_ctx.map(Arc::new);
        Ok(ctx)
    }

    async fn execute(&self, ctx: &AgentContext) -> Result<AgentExecutionReport, AgentError> {
        let remaining = self.remaining_failures.load(Ordering::SeqCst);
        if remaining > 0 {
            self.remaining_failures.fetch_sub(1, Ordering::SeqCst);
            return Err(AgentError::ToolError {
                tool_name: "retryable".to_string(),
                message: "transient failure".to_string(),
            });
        }

        Ok(AgentExecutionReport::success(
            AgentType::Coder,
            ctx.execution_id,
            json!({ "retried": true }),
        ))
    }

    fn default_config(&self) -> AgentConfig {
        self.config.clone()
    }

    fn update_config(&mut self, config: AgentConfig) {
        self.config = config;
    }
}

struct StreamingAgent {
    config: AgentConfig,
}

impl StreamingAgent {
    fn new() -> Self {
        let mut config = AgentConfig::for_agent_type(AgentType::Coder);
        config.streaming_enabled = true;
        Self { config }
    }
}

#[async_trait]
impl Agent for StreamingAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Coder
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    async fn build_context(
        &self,
        task: &TaskDescription,
        project_ctx: Option<ProjectContext>,
        shared: &SharedResources,
    ) -> Result<AgentContext, AgentError> {
        let mut ctx = AgentContext::new(task.clone(), shared, self.config.clone());
        ctx.project_ctx = project_ctx.map(Arc::new);
        Ok(ctx)
    }

    async fn execute(&self, ctx: &AgentContext) -> Result<AgentExecutionReport, AgentError> {
        Ok(AgentExecutionReport::success(
            AgentType::Coder,
            ctx.execution_id,
            json!({ "streaming": false }),
        ))
    }

    async fn execute_streaming(
        &self,
        ctx: &AgentContext,
        forwarder: &mut dyn StreamForwarder,
    ) -> Result<AgentExecutionReport, AgentError> {
        forwarder.forward_chunk("hello").await?;
        forwarder.forward_chunk(" world").await?;
        let report = AgentExecutionReport::success(
            AgentType::Coder,
            ctx.execution_id,
            json!({ "streaming": true }),
        );
        forwarder.forward_final(&report).await?;
        Ok(report)
    }

    fn default_config(&self) -> AgentConfig {
        self.config.clone()
    }

    fn update_config(&mut self, config: AgentConfig) {
        self.config = config;
    }
}

#[derive(Default)]
struct RecordingLifecycle {
    events: Mutex<Vec<&'static str>>,
}

impl RecordingLifecycle {
    fn snapshot(&self) -> Vec<&'static str> {
        self.events.lock().expect("lifecycle lock").clone()
    }
}

#[async_trait]
impl AgentLifecycle for RecordingLifecycle {
    async fn on_start(&self, _ctx: &AgentContext) -> Result<(), AgentError> {
        self.events.lock().expect("lifecycle lock").push("start");
        Ok(())
    }

    async fn on_complete(
        &self,
        _ctx: &AgentContext,
        _report: &AgentExecutionReport,
    ) -> Result<(), AgentError> {
        self.events.lock().expect("lifecycle lock").push("complete");
        Ok(())
    }

    async fn on_error(&self, _ctx: &AgentContext, _error: &AgentError) -> Result<(), AgentError> {
        self.events.lock().expect("lifecycle lock").push("error");
        Ok(())
    }

    async fn on_cancel(&self, _ctx: &AgentContext) -> Result<(), AgentError> {
        self.events.lock().expect("lifecycle lock").push("cancel");
        Ok(())
    }
}

#[derive(Default)]
struct RecordingStreamForwarder {
    chunks: Vec<String>,
    events: Vec<StreamEvent>,
    final_calls: usize,
}

#[async_trait]
impl StreamForwarder for RecordingStreamForwarder {
    async fn forward_chunk(&mut self, chunk: &str) -> Result<(), AgentError> {
        self.chunks.push(chunk.to_string());
        Ok(())
    }

    async fn forward_event(&mut self, event: StreamEvent) -> Result<(), AgentError> {
        self.events.push(event);
        Ok(())
    }

    async fn forward_final(&mut self, _report: &AgentExecutionReport) -> Result<(), AgentError> {
        self.final_calls += 1;
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), AgentError> {
        Ok(())
    }
}

fn create_test_shared_resources() -> SharedResources {
    SharedResources::new(
        Arc::new(DummyLlmClient),
        Arc::new(ToolRegistry::new()),
        Arc::new(MemoryManager::new()),
        Arc::new(InMemoryEventBus::default()),
        Arc::new(GlobalConfig::default()),
    )
}

fn create_test_logger() -> Arc<BufferLogger> {
    Arc::new(BufferLogger::default())
}

fn sample_project_context() -> ProjectContext {
    ProjectContext {
        project_info: ProjectInfo {
            name: "morecode".to_string(),
            description: "test".to_string(),
            version: Some("0.1.0".to_string()),
            language: "Rust".to_string(),
            framework: None,
            license: Some("MIT".to_string()),
            repository_url: None,
        },
        structure: ProjectStructure {
            directory_tree: ".".to_string(),
            total_files: 1,
            total_lines: 10,
            entry_files: vec!["src/main.rs".to_string()],
            config_files: vec!["Cargo.toml".to_string()],
            modules: Vec::new(),
        },
        tech_stack: TechStack {
            language_version: "1.85".to_string(),
            rust_edition: Some("2021".to_string()),
            framework: None,
            database: None,
            orm: None,
            auth: None,
            build_tool: Some("cargo".to_string()),
            package_manager: Some("cargo".to_string()),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            updated_at: Utc::now(),
        },
        architecture: ArchitecturePattern {
            name: "Monolith".to_string(),
            description: "single crate".to_string(),
            layers: Vec::new(),
            design_decisions: Vec::new(),
        },
        dependency_graph: DependencyGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
            circular_dependencies: Vec::new(),
        },
        conventions: CodeConventions {
            naming: NamingConvention {
                function: "snake_case".to_string(),
                struct_: "PascalCase".to_string(),
                constant: "SCREAMING_SNAKE_CASE".to_string(),
                module: "snake_case".to_string(),
                enum_variant: "PascalCase".to_string(),
                type_parameter: "UpperCamel".to_string(),
            },
            error_handling: ErrorHandlingPattern {
                custom_error_type: true,
                error_type_name: Some("AgentError".to_string()),
                unwrap_policy: "forbid".to_string(),
                propagation: "?".to_string(),
                error_crate: Some("thiserror".to_string()),
            },
            testing: TestingConvention {
                naming_pattern: "test_*".to_string(),
                test_attribute: "#[tokio::test]".to_string(),
                test_file_location: "src/tests.rs".to_string(),
                mock_framework: None,
                coverage_threshold: None,
            },
            documentation: DocumentationConvention {
                language: "zh-CN".to_string(),
                format: "Markdown".to_string(),
                code_example_style: "rust".to_string(),
                api_doc_tool: None,
            },
            custom_rules: Vec::new(),
        },
        risk_areas: RiskAreas { items: Vec::new() },
        scan_metadata: ScanMetadata {
            scanned_at: Utc::now(),
            files_scanned: 1,
            total_lines: 10,
            scan_duration_ms: 1,
            memory_version: 1,
            scanner_version: "test".to_string(),
        },
        root_path: ".".to_string(),
    }
}

fn sample_execution_plan() -> ExecutionPlan {
    ExecutionPlan {
        plan_id: "plan-1".to_string(),
        task_description: "test".to_string(),
        parallel_groups: Vec::new(),
        group_dependencies: HashMap::new(),
        sub_tasks: Vec::new(),
        dependencies: Vec::new(),
        commit_points: Vec::new(),
        context_allocations: vec![ContextAllocation {
            sub_task_id: "subtask-1".to_string(),
            agent_type: AgentType::Coder,
            token_budget: 256,
            required_files: vec!["src/lib.rs".to_string()],
            project_knowledge_subset: Vec::new(),
            context_window_limit: 1024,
        }],
        total_estimated_tokens: 256,
        total_estimated_duration_ms: 1_000,
        plan_metadata: PlanMetadata {
            generated_by: AgentType::Planner,
            generated_at: Utc::now(),
            model_used: "dummy".to_string(),
            generation_duration_ms: 1,
            tokens_used: 10,
            version: 1,
        },
        created_at: Utc::now(),
    }
}

fn create_test_context() -> AgentContext {
    let shared = create_test_shared_resources();
    let config = AgentConfig::for_agent_type(AgentType::Coder);
    let mut ctx = AgentContext::new(TaskDescription::simple("test task"), &shared, config);
    ctx.project_ctx = Some(Arc::new(sample_project_context()));
    ctx.impact_report = Some(Arc::new(ImpactReport {
        summary: "safe".to_string(),
        affected_files: vec!["src/lib.rs".to_string()],
        risk_level: ImpactRiskLevel::Low,
        breaking_change: false,
        notes: Vec::new(),
    }));
    ctx.execution_plan = Some(Arc::new(sample_execution_plan()));
    ctx
}

#[tokio::test]
async fn test_register_agent_success() {
    let registry = AgentRegistry::new();
    let result = registry.register(AgentType::Coder, |_shared, _config| {
        Box::new(MockAgent::new(AgentType::Coder))
    });
    assert!(result.is_ok());
    assert!(registry.is_registered(&AgentType::Coder));
}

#[tokio::test]
async fn test_register_duplicate_fails() {
    let registry = AgentRegistry::new();
    registry
        .register(AgentType::Coder, |_shared, _config| {
            Box::new(MockAgent::new(AgentType::Coder))
        })
        .unwrap();
    let result = registry.register(AgentType::Coder, |_shared, _config| {
        Box::new(MockAgent::new(AgentType::Coder))
    });
    assert!(matches!(result, Err(AgentError::DuplicateRegistration { .. })));
}

#[tokio::test]
async fn test_list_all_agents() {
    let registry = AgentRegistry::new();
    registry
        .register(AgentType::Coder, |_shared, _config| {
            Box::new(MockAgent::new(AgentType::Coder))
        })
        .unwrap();
    registry
        .register(AgentType::Reviewer, |_shared, _config| {
            Box::new(MockAgent::new(AgentType::Reviewer))
        })
        .unwrap();

    let all = registry.list_all();
    assert_eq!(all, vec![AgentType::Coder, AgentType::Reviewer]);
}

#[tokio::test]
async fn test_list_by_layer() {
    let registry = AgentRegistry::new();
    registry
        .register(AgentType::Explorer, |_shared, _config| {
            Box::new(MockAgent::new(AgentType::Explorer))
        })
        .unwrap();
    registry
        .register(AgentType::Coder, |_shared, _config| {
            Box::new(MockAgent::new(AgentType::Coder))
        })
        .unwrap();

    let cognitive = registry.list_by_layer(mc_core::AgentLayer::Cognitive);
    assert_eq!(cognitive, vec![AgentType::Explorer]);
}

#[tokio::test]
async fn test_get_singleton() {
    let registry = AgentRegistry::new();
    let shared = create_test_shared_resources();
    let config = AgentConfig::for_agent_type(AgentType::Coder);
    registry
        .register(AgentType::Coder, |_shared, _config| {
            Box::new(MockAgent::new(AgentType::Coder))
        })
        .unwrap();

    let i1 = registry.get(&AgentType::Coder, &shared, &config).unwrap();
    let i2 = registry.get(&AgentType::Coder, &shared, &config).unwrap();
    assert!(Arc::ptr_eq(&i1, &i2));
}

#[tokio::test]
async fn test_create_agent_new_instance() {
    let registry = AgentRegistry::new();
    let shared = create_test_shared_resources();
    let config = AgentConfig::for_agent_type(AgentType::Coder);
    registry
        .register(AgentType::Coder, |_shared, _config| {
            Box::new(MockAgent::new(AgentType::Coder))
        })
        .unwrap();

    let a1 = registry.create_agent(&AgentType::Coder, &shared, &config).unwrap();
    let a2 = registry.create_agent(&AgentType::Coder, &shared, &config).unwrap();
    assert_eq!(a1.agent_type(), AgentType::Coder);
    assert_eq!(a2.agent_type(), AgentType::Coder);
}

#[tokio::test]
async fn test_get_unregistered_agent_fails() {
    let registry = AgentRegistry::new();
    let shared = create_test_shared_resources();
    let config = AgentConfig::for_agent_type(AgentType::Coder);
    let result = registry.get(&AgentType::Coder, &shared, &config);
    assert!(matches!(result, Err(AgentError::AgentNotFound { .. })));
}

#[tokio::test]
async fn test_child_context_inheritance() {
    let parent_ctx = create_test_context();
    let child_ctx = parent_ctx.create_child_context(TaskDescription::simple("sub task"));
    assert!(child_ctx.project_ctx.is_some());
    assert!(child_ctx.impact_report.is_some());
    assert!(child_ctx.execution_plan.is_some());
    assert_eq!(child_ctx.parent_execution_id, Some(parent_ctx.execution_id));
    assert_ne!(child_ctx.execution_id, parent_ctx.execution_id);
}

#[tokio::test]
async fn test_cascading_cancel() {
    let parent_ctx = create_test_context();
    let child_ctx = parent_ctx.create_child_context(TaskDescription::simple("sub task"));
    parent_ctx.cancel_token.cancel();
    assert!(child_ctx.is_cancelled());
}

#[tokio::test]
async fn test_handoff_data_transfer() {
    let handoff = crate::AgentHandoff::new();
    handoff.put("hello".to_string()).await;
    let result: Option<String> = handoff.get().await;
    assert_eq!(result.as_deref(), Some("hello"));
}

#[tokio::test]
async fn test_execute_success() {
    let agent = MockAgent::new(AgentType::Coder);
    let ctx = create_test_context();
    let lifecycle = Arc::new(RecordingLifecycle::default());
    let result = execute_with_context(&agent, &ctx, Some(lifecycle.clone() as LifecycleHandler)).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, ExecutionStatus::Success);
    assert_eq!(lifecycle.snapshot(), vec!["start", "complete"]);
}

#[tokio::test]
async fn test_execute_timeout() {
    let agent = SlowAgent::new(Duration::from_millis(200));
    let mut ctx = create_test_context();
    let mut config = AgentConfig::for_agent_type(AgentType::Coder);
    config.timeout_ms = 10;
    ctx.config = Arc::new(config);
    ctx.max_retries = 0;

    let result = execute_with_context(&agent, &ctx, None).await;
    assert!(matches!(result, Err(AgentError::ExecutionTimeout { .. })));
}

#[tokio::test]
async fn test_execute_cancel() {
    let agent = SlowAgent::new(Duration::from_secs(10));
    let ctx = create_test_context();
    let cancel_token = ctx.cancel_token.clone();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(25)).await;
        cancel_token.cancel();
    });

    let result = execute_with_context(&agent, &ctx, None).await;
    assert!(matches!(result, Err(AgentError::ExecutionCancelled { .. })));
}

#[tokio::test]
async fn test_execute_retry() {
    let agent = RetryableAgent::new(2);
    let mut ctx = create_test_context();
    ctx.max_retries = 2;

    let result = execute_with_context(&agent, &ctx, None).await;
    assert!(result.is_ok());
    assert_eq!(agent.attempts(), 0);
}

#[tokio::test]
async fn test_execute_streaming() {
    let agent = StreamingAgent::new();
    let ctx = create_test_context();
    let mut forwarder = RecordingStreamForwarder::default();

    let result = execute_streaming_with_context(&agent, &ctx, &mut forwarder, None).await;
    assert!(result.is_ok());
    assert_eq!(forwarder.chunks, vec!["hello".to_string(), " world".to_string()]);
    assert_eq!(forwarder.final_calls, 1);
}

#[tokio::test]
async fn test_event_bus_broadcasts_execution_events() {
    let agent = MockAgent::new(AgentType::Coder);
    let ctx = create_test_context();
    let mut receiver = ctx.event_bus.subscribe();

    let result = execute_with_context(&agent, &ctx, None).await;
    assert!(result.is_ok());

    let started = receiver.recv().await.unwrap();
    let completed = receiver.recv().await.unwrap();
    assert_eq!(started.kind, crate::AgentEventKind::Started);
    assert_eq!(completed.kind, crate::AgentEventKind::Completed);
}

#[test]
fn test_error_category() {
    let err = AgentError::DuplicateRegistration {
        agent_type: AgentType::Coder,
        message: "test".into(),
    };
    assert_eq!(err.category(), ErrorCategory::Registration);

    let err = AgentError::LlmError {
        message: "rate limit".into(),
        source: None,
    };
    assert_eq!(err.category(), ErrorCategory::Llm);
}

#[test]
fn test_retryable_errors() {
    assert!(AgentError::LlmError {
        message: "timeout".into(),
        source: None,
    }
    .is_retryable());
    assert!(!AgentError::AgentNotFound {
        agent_type: AgentType::Coder,
        message: "not found".into(),
    }
    .is_retryable());
}

#[tokio::test]
async fn test_logging_lifecycle_emits_messages() {
    let agent = MockAgent::new(AgentType::Coder);
    let ctx = create_test_context();
    let logger = create_test_logger();
    let lifecycle = LoggingLifecycle::new(logger.clone());

    let result = execute_with_context(&agent, &ctx, Some(Arc::new(lifecycle))).await;
    assert!(result.is_ok());
    assert!(!logger.snapshot().is_empty());
}

#[tokio::test]
async fn test_stdout_forwarder_collects_buffer() {
    let mut forwarder = StdoutStreamForwarder::new();
    forwarder.forward_chunk("hello").await.unwrap();
    forwarder.forward_chunk(" world").await.unwrap();
    assert_eq!(forwarder.buffer(), "hello world");
}
