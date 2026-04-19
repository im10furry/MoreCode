pub mod framework;

use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use mc_core::{AgentType, ExecutionPlan, ProjectContext, TaskDescription};
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::time::timeout;

use crate::{
    Agent, AgentConfig, AgentContext, AgentError, AgentExecutionReport, SharedResources,
};

use self::framework::{
    detect_framework, derive_focus_filters, parse_test_output, FrameworkDetectionContext,
    TestCommand, TestFramework, TestRunSummary,
};

const DEFAULT_TEST_TIMEOUT_SECS: u64 = 600;

#[derive(Debug, Clone)]
pub struct Tester {
    config: AgentConfig,
    timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TesterExecutionReport {
    pub framework: TestFramework,
    pub command: TestCommand,
    pub summary: TestRunSummary,
    pub focused_targets: Vec<String>,
}

impl Tester {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            timeout: Duration::from_secs(DEFAULT_TEST_TIMEOUT_SECS),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn resolve_plan(ctx: &AgentContext) -> Option<ExecutionPlan> {
        if let Some(plan) = ctx.execution_plan.as_deref().cloned() {
            return Some(plan);
        }
        None
    }

    fn focused_targets(ctx: &AgentContext) -> Vec<String> {
        let mut targets = derive_focus_filters(
            Self::resolve_plan(ctx).as_ref(),
            Some(ctx.task.as_ref()),
        );

        if targets.is_empty() {
            targets = ctx.task.affected_files.clone();
        }
        targets.sort();
        targets.dedup();
        targets
    }

    fn choose_framework(ctx: &AgentContext, project_root: &PathBuf) -> TestFramework {
        let hint = ctx.task.user_input.as_str();
        detect_framework(project_root, &FrameworkDetectionContext { hint })
    }

    fn command_for_framework(
        framework: TestFramework,
        focused_targets: &[String],
        project_root: &PathBuf,
    ) -> TestCommand {
        framework.build_command(focused_targets, project_root)
    }

    async fn execute_command(
        &self,
        command: &TestCommand,
        project_root: &PathBuf,
    ) -> Result<(std::process::Output, u64), AgentError> {
        let started = std::time::Instant::now();
        let mut process = Command::new(&command.program);
        process.args(command.args.iter());
        process.current_dir(project_root);

        let result = timeout(self.timeout, process.output()).await.map_err(|_| {
            AgentError::ExecutionFailed {
                agent_type: AgentType::Tester,
                message: format!(
                    "test command timed out after {}s: {}",
                    self.timeout.as_secs(),
                    command.render()
                ),
            }
        })?;

        let output = result.map_err(|err| AgentError::ExecutionFailed {
            agent_type: AgentType::Tester,
            message: format!("failed to execute test command `{}`: {err}", command.render()),
        })?;

        Ok((output, started.elapsed().as_millis() as u64))
    }

    async fn execute_internal(&self, ctx: &AgentContext) -> Result<TesterExecutionReport, AgentError> {
        let project_root = ctx.project_root();
        let focused_targets = Self::focused_targets(ctx);
        let framework = Self::choose_framework(ctx, &project_root);
        let command = Self::command_for_framework(framework, &focused_targets, &project_root);

        let (output, duration_ms) = self.execute_command(&command, &project_root).await?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let exit_code = output.status.code();
        let summary = parse_test_output(framework, &stdout, &stderr, exit_code, duration_ms);

        Ok(TesterExecutionReport {
            framework,
            command,
            summary,
            focused_targets,
        })
    }
}

#[async_trait]
impl Agent for Tester {
    fn agent_type(&self) -> AgentType {
        AgentType::Tester
    }

    fn supports_parallel(&self) -> bool {
        false
    }

    fn default_config(&self) -> AgentConfig {
        AgentConfig::for_agent_type(AgentType::Tester)
    }

    fn update_config(&mut self, config: AgentConfig) {
        self.config = config;
    }

    async fn build_context(
        &self,
        task: &TaskDescription,
        project_ctx: Option<ProjectContext>,
        shared: &SharedResources,
    ) -> Result<AgentContext, AgentError> {
        let mut ctx = AgentContext::new(task.clone(), shared, self.config.clone());
        if let Some(project_ctx) = project_ctx {
            ctx = ctx.with_project_ctx(project_ctx);
        }
        Ok(ctx)
    }

    async fn execute(&self, ctx: &AgentContext) -> Result<AgentExecutionReport, AgentError> {
        let report = self.execute_internal(ctx).await?;
        ctx.handoff.put(report.clone()).await;

        let summary = if report.summary.success {
            format!(
                "Tester passed with {} ({} passed, {} failed, {} skipped)",
                report.framework,
                report.summary.passed,
                report.summary.failed,
                report.summary.skipped
            )
        } else {
            format!(
                "Tester found failures with {} ({} passed, {} failed, {} skipped)",
                report.framework,
                report.summary.passed,
                report.summary.failed,
                report.summary.skipped
            )
        };

        let result = serde_json::to_value(&report).map_err(AgentError::serialization)?;
        Ok(AgentExecutionReport::success(
            AgentType::Tester,
            &ctx.execution_id,
            summary,
            result,
            ctx.elapsed_ms(),
            report.summary.token_estimate,
        ))
    }
}

impl Default for Tester {
    fn default() -> Self {
        Self::new(AgentConfig::for_agent_type(AgentType::Tester))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use mc_llm::{
        CacheCapability, ChatRequest, ChatResponse, LlmError, LlmProvider, ModelInfo, StreamEvent,
    };
    use tempfile::TempDir;
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    use super::*;

    struct DummyLlmProvider {
        model_info: ModelInfo,
    }

    impl DummyLlmProvider {
        fn new() -> Self {
            Self {
                model_info: ModelInfo::new("dummy", "Dummy", "dummy"),
            }
        }
    }

    impl LlmProvider for DummyLlmProvider {
        fn provider_id(&self) -> &str {
            "dummy"
        }

        fn model_info(&self) -> &ModelInfo {
            &self.model_info
        }

        fn chat(
            &self,
            _request: ChatRequest,
            _cancel_token: CancellationToken,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<ChatResponse, LlmError>> + Send + '_>,
        > {
            Box::pin(async move {
                Err(LlmError::Internal(
                    "dummy provider should not be called in tester unit tests".to_string(),
                ))
            })
        }

        fn chat_stream(
            &self,
            _request: ChatRequest,
            _cancel_token: CancellationToken,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Result<mpsc::Receiver<StreamEvent>, LlmError>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async move {
                let (_tx, rx) = mpsc::channel(1);
                Ok(rx)
            })
        }

        fn cache_capability(&self) -> CacheCapability {
            CacheCapability::default()
        }

        fn list_models(
            &self,
            _cancel_token: CancellationToken,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Vec<ModelInfo>, LlmError>> + Send + '_>,
        > {
            Box::pin(async move { Ok(vec![self.model_info.clone()]) })
        }

        fn cancel_request(&self, _request_id: &str) -> Result<(), LlmError> {
            Ok(())
        }

        fn estimate_tokens(&self, text: &str) -> usize {
            text.len().div_ceil(4)
        }
    }

    fn build_shared(root: &std::path::Path) -> SharedResources {
        SharedResources::new(root, Arc::new(DummyLlmProvider::new()))
    }

    #[test]
    fn tester_focus_targets_prefers_plan_and_task_files() {
        let mut task = TaskDescription::simple("run focused tests");
        task.affected_files = vec!["src/lib.rs".to_string(), "tests/api.rs".to_string()];
        let shared = build_shared(std::path::Path::new("."));
        let mut ctx = AgentContext::new(
            task.clone(),
            &shared,
            AgentConfig::for_agent_type(AgentType::Tester),
        );
        let plan = ExecutionPlan {
            plan_id: "plan".to_string(),
            task_description: "desc".to_string(),
            summary: "summary".to_string(),
            parallel_groups: Vec::new(),
            group_dependencies: HashMap::new(),
            sub_tasks: Vec::new(),
            dependencies: Vec::new(),
            commit_points: Vec::new(),
            context_allocations: Vec::new(),
            total_estimated_tokens: 0,
            total_estimated_duration_ms: 0,
            plan_metadata: mc_core::PlanMetadata {
                generated_by: AgentType::Planner,
                generated_at: chrono::Utc::now(),
                model_used: "mock".to_string(),
                generation_duration_ms: 0,
                tokens_used: 0,
                version: 1,
            },
            created_at: chrono::Utc::now(),
        };
        ctx.execution_plan = Some(Arc::new(plan));
        let focused = Tester::focused_targets(&ctx);
        assert!(focused.iter().any(|item| item == "src/lib.rs"));
        assert!(focused.iter().any(|item| item == "tests/api.rs"));
    }

    #[tokio::test]
    async fn tester_uses_cargo_in_rust_workspace() {
        let temp = TempDir::new().expect("tempdir");
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname=\"demo\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
        )
        .expect("write cargo");

        let tester = Tester::default();
        let task = TaskDescription::with_root("run tests", temp.path());
        let shared = build_shared(temp.path());
        let ctx = tester
            .build_context(&task, None, &shared)
            .await
            .expect("build context");

        let framework = Tester::choose_framework(&ctx, &ctx.project_root());
        assert_eq!(framework, TestFramework::Cargo);
    }
}
