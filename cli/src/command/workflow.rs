use std::fmt::Write as _;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::io::AsyncBufReadExt;
use mc_agent::tester::framework::{
    derive_focus_filters, detect_framework, FrameworkDetectionContext,
};
use mc_agent::{
    Agent, AgentConfig, AgentHandoff, CodeChangeKind, CodeGenerationOutput, Coder, Explorer,
    ImpactAnalyzer, ImpactReport, Planner, ReviewReport, Reviewer, SharedResources, Tester,
    TesterExecutionReport,
};
use mc_config::ResolvedProviderEntry;
use mc_core::{
    build_patch_hunks, ApprovalLevel, ApprovalStatus, ArtifactKind, CommandStatus, Complexity,
    MessageLevel, PatchKind, PatchStatus, RunApproval, RunCommand, RunEvent, RunEventEnvelope,
    RunPatch, RunRecorder, RunSnapshot, RunStatus, RunStep, RunStore, StepStatus, TaskDescription,
    TaskIntent,
};
use mc_llm::{
    AnthropicProvider, AnthropicProviderConfig, LlmProvider, ModelInfo, OpenAiProvider,
    OpenAiProviderConfig,
};
use mc_llm::{GoogleProvider, GoogleProviderConfig};
use uuid::Uuid;

use crate::{AppContext, ApprovalMode, ExportFormat};

pub trait RunEventSink: Send + Sync {
    fn handle_event(&self, envelope: &RunEventEnvelope) -> Result<(), String>;
}

#[derive(Debug, Clone, Copy)]
pub struct WorkflowOptions {
    pub plan_only: bool,
    pub approval: ApprovalMode,
}

#[derive(Debug, Clone)]
pub struct RunExecutionOutput {
    pub snapshot: RunSnapshot,
    pub run_dir: PathBuf,
}

pub fn run_store(context: &AppContext) -> RunStore {
    RunStore::new(&context.project_root)
}

pub fn load_snapshot(context: &AppContext, run_id: &str) -> Result<RunSnapshot, String> {
    run_store(context)
        .load_snapshot(run_id)
        .map_err(|error| error.to_string())
}

pub async fn execute_run(
    context: &AppContext,
    request: &str,
    options: WorkflowOptions,
    sink: Option<&dyn RunEventSink>,
) -> Result<RunExecutionOutput, String> {
    let recorder =
        RunRecorder::create(&context.project_root, request).map_err(|error| error.to_string())?;
    execute_run_with_recorder(context, request, options, sink, recorder).await
}

fn agent_config(agent_type: mc_core::AgentType, model_id: &str) -> AgentConfig {
    let mut config = AgentConfig::for_agent_type(agent_type);
    config.llm_config.model_id = model_id.to_string();
    config
}

pub async fn execute_run_with_recorder(
    context: &AppContext,
    request: &str,
    options: WorkflowOptions,
    sink: Option<&dyn RunEventSink>,
    mut recorder: RunRecorder,
) -> Result<RunExecutionOutput, String> {
    let llm = build_llm_provider(resolve_default_provider(&context.config.provider)?)?;
    let shared = SharedResources::new(context.project_root.clone(), llm);
    let task = build_task_description(request, &context.project_root);
    let model_id = &context.config.agent.default_model;

    let run_dir_label = recorder.run_dir().display().to_string();
    emit(
        &mut recorder,
        sink,
        RunEvent::Message {
            step_id: None,
            level: MessageLevel::Info,
            message: format!("run created at {run_dir_label}"),
        },
    )?;

    let started_at = Utc::now();
    let handoff = Arc::new(AgentHandoff::new());

    let understand = RunStep::new("understand", "Understand", None, None);
    emit(
        &mut recorder,
        sink,
        RunEvent::StepStarted { step: understand },
    )?;

    let explorer_ctx = Explorer::new(agent_config(mc_core::AgentType::Explorer, model_id))
        .build_context(&task, None, &shared)
        .await
        .map_err(|error| error.to_string())?
        .with_handoff(Arc::clone(&handoff));
    run_agent_step(
        &mut recorder,
        sink,
        RunStep::new(
            "understand.explorer",
            "Scan project context",
            Some("understand".to_string()),
            Some("Explorer".to_string()),
        ),
        async {
            let report = Explorer::new(agent_config(mc_core::AgentType::Explorer, model_id))
                .execute(&explorer_ctx)
                .await
                .map_err(|error| error.to_string())?;
            let project_context = handoff
                .get::<mc_core::ProjectContext>()
                .await
                .ok_or_else(|| "explorer did not produce project context".to_string())?;
            Ok((
                report.summary,
                u64::from(report.metrics.tokens_used),
                project_context,
            ))
        },
        |recorder, project_context| {
            recorder
                .write_json_artifact(
                    "project-context.json",
                    "Project Context",
                    ArtifactKind::ProjectContext,
                    project_context,
                    Some("Explorer output".to_string()),
                )
                .map(|_| ())
        },
    )
    .await?;
    let project_context = handoff
        .get::<mc_core::ProjectContext>()
        .await
        .ok_or_else(|| "missing project context after explorer".to_string())?;

    let impact_ctx = ImpactAnalyzer::new(agent_config(
        mc_core::AgentType::ImpactAnalyzer, model_id,
    ))
    .build_context(&task, Some(project_context.clone()), &shared)
    .await
    .map_err(|error| error.to_string())?
    .with_handoff(Arc::clone(&handoff));
    run_agent_step(
        &mut recorder,
        sink,
        RunStep::new(
            "understand.impact",
            "Assess impact and risk",
            Some("understand".to_string()),
            Some("ImpactAnalyzer".to_string()),
        ),
        async {
            let report = ImpactAnalyzer::new(agent_config(
                mc_core::AgentType::ImpactAnalyzer, model_id,
            ))
            .execute(&impact_ctx)
            .await
            .map_err(|error| error.to_string())?;
            let impact_report = handoff
                .get::<ImpactReport>()
                .await
                .ok_or_else(|| "impact analyzer did not produce impact report".to_string())?;
            Ok((
                report.summary,
                u64::from(report.metrics.tokens_used),
                impact_report,
            ))
        },
        |recorder, impact_report| {
            recorder
                .write_json_artifact(
                    "impact-report.json",
                    "Impact Report",
                    ArtifactKind::ImpactReport,
                    impact_report,
                    Some("Impact analyzer output".to_string()),
                )
                .map(|_| ())
        },
    )
    .await?;
    let impact_report = handoff
        .get::<ImpactReport>()
        .await
        .ok_or_else(|| "missing impact report after analyzer".to_string())?;

    let understand_tokens = sum_child_tokens(recorder.snapshot(), "understand");
    emit(
        &mut recorder,
        sink,
        RunEvent::StepFinished {
            step_id: "understand".to_string(),
            status: StepStatus::Done,
            summary: Some("project scan and impact analysis finished".to_string()),
            token_used: understand_tokens,
        },
    )?;

    let plan = RunStep::new("plan", "Plan", None, None);
    emit(&mut recorder, sink, RunEvent::StepStarted { step: plan })?;
    let planner_ctx = Planner::new(agent_config(mc_core::AgentType::Planner, model_id))
        .build_context(&task, Some(project_context.clone()), &shared)
        .await
        .map_err(|error| error.to_string())?
        .with_handoff(Arc::clone(&handoff))
        .with_impact_report(impact_report.clone());
    run_agent_step(
        &mut recorder,
        sink,
        RunStep::new(
            "plan.execution",
            "Build execution plan",
            Some("plan".to_string()),
            Some("Planner".to_string()),
        ),
        async {
            let report = Planner::new(agent_config(mc_core::AgentType::Planner, model_id))
                .execute(&planner_ctx)
                .await
                .map_err(|error| error.to_string())?;
            let execution_plan = handoff
                .get::<mc_core::ExecutionPlan>()
                .await
                .ok_or_else(|| "planner did not produce execution plan".to_string())?;
            Ok((
                report.summary,
                u64::from(report.metrics.tokens_used),
                execution_plan,
            ))
        },
        |recorder, execution_plan| {
            recorder
                .write_json_artifact(
                    "execution-plan.json",
                    "Execution Plan",
                    ArtifactKind::ExecutionPlan,
                    execution_plan,
                    Some("Planner output".to_string()),
                )
                .map(|_| ())
        },
    )
    .await?;
    let execution_plan = handoff
        .get::<mc_core::ExecutionPlan>()
        .await
        .ok_or_else(|| "missing execution plan after planner".to_string())?;

    let plan_tokens = sum_child_tokens(recorder.snapshot(), "plan");
    emit(
        &mut recorder,
        sink,
        RunEvent::StepFinished {
            step_id: "plan".to_string(),
            status: StepStatus::Done,
            summary: Some(format!(
                "{} tasks across {} groups",
                execution_plan.sub_tasks.len(),
                execution_plan.parallel_groups.len()
            )),
            token_used: plan_tokens,
        },
    )?;

    if options.plan_only {
        finalize_run(
            &mut recorder,
            sink,
            RunStatus::Succeeded,
            "plan generated without execution".to_string(),
            None,
            started_at,
        )?;
        return Ok(RunExecutionOutput {
            snapshot: recorder.snapshot().clone(),
            run_dir: recorder.run_dir().to_path_buf(),
        });
    }

    let execute = RunStep::new("execute", "Execute", None, None);
    emit(&mut recorder, sink, RunEvent::StepStarted { step: execute })?;

    let coder_ctx = Coder::new(agent_config(mc_core::AgentType::Coder, model_id))
        .build_context(&task, Some(project_context.clone()), &shared)
        .await
        .map_err(|error| error.to_string())?
        .with_handoff(Arc::clone(&handoff))
        .with_impact_report(impact_report.clone())
        .with_execution_plan(execution_plan.clone());
    run_agent_step(
        &mut recorder,
        sink,
        RunStep::new(
            "execute.codegen",
            "Draft code changes",
            Some("execute".to_string()),
            Some("Coder".to_string()),
        ),
        async {
            let report = Coder::new(agent_config(mc_core::AgentType::Coder, model_id))
                .execute(&coder_ctx)
                .await
                .map_err(|error| error.to_string())?;
            let coder_output = handoff
                .get::<CodeGenerationOutput>()
                .await
                .ok_or_else(|| "coder did not produce code generation output".to_string())?;
            Ok((
                report.summary,
                u64::from(report.metrics.tokens_used),
                coder_output,
            ))
        },
        |recorder, coder_output| {
            recorder
                .write_json_artifact(
                    "code-draft.json",
                    "Code Draft",
                    ArtifactKind::CodeDraft,
                    coder_output,
                    Some("Coder draft output".to_string()),
                )
                .map(|_| ())
        },
    )
    .await?;
    let coder_output = handoff
        .get::<CodeGenerationOutput>()
        .await
        .ok_or_else(|| "missing code generation output after coder".to_string())?;
    record_patch_artifacts(&mut recorder, sink, &coder_output, "execute.codegen")?;

    let review_root = RunStep::new("review", "Review", None, None);
    emit(
        &mut recorder,
        sink,
        RunEvent::StepStarted { step: review_root },
    )?;

    let reviewer_ctx = Reviewer::new(agent_config(mc_core::AgentType::Reviewer, model_id))
        .build_context(&task, Some(project_context.clone()), &shared)
        .await
        .map_err(|error| error.to_string())?
        .with_handoff(Arc::clone(&handoff))
        .with_impact_report(impact_report.clone())
        .with_execution_plan(execution_plan.clone());
    run_agent_step(
        &mut recorder,
        sink,
        RunStep::new(
            "review.reviewer",
            "Review generated changes",
            Some("review".to_string()),
            Some("Reviewer".to_string()),
        ),
        async {
            let report = Reviewer::new(agent_config(mc_core::AgentType::Reviewer, model_id))
                .execute(&reviewer_ctx)
                .await
                .map_err(|error| error.to_string())?;
            let review_report = handoff
                .get::<ReviewReport>()
                .await
                .ok_or_else(|| "reviewer did not produce review report".to_string())?;
            Ok((
                report.summary,
                u64::from(report.metrics.tokens_used),
                review_report,
            ))
        },
        |recorder, review_report| {
            recorder
                .write_json_artifact(
                    "review-report.json",
                    "Review Report",
                    ArtifactKind::ReviewReport,
                    review_report,
                    Some("Reviewer output".to_string()),
                )
                .map(|_| ())
        },
    )
    .await?;
    let review_report = handoff
        .get::<ReviewReport>()
        .await
        .ok_or_else(|| "missing review report after reviewer".to_string())?;

    let validation_command =
        build_validation_command(&task, &execution_plan, &context.project_root).unwrap_or_else(
            || RunCommand {
                command_id: format!("cmd-{}", Uuid::new_v4()),
                step_id: "execute.tests".to_string(),
                title: "Focused validation".to_string(),
                command: "validation unavailable".to_string(),
                cwd: context.project_root.to_string_lossy().into_owned(),
                status: CommandStatus::Skipped,
                started_at: None,
                finished_at: None,
                exit_code: None,
                stdout_tail: String::new(),
                stderr_tail: String::new(),
            },
        );
    let tests_approved = request_approval(
        &mut recorder,
        sink,
        options.approval,
        RunApproval {
            approval_id: format!("approval-{}", Uuid::new_v4()),
            step_id: "execute.tests".to_string(),
            title: "Run validation command".to_string(),
            reason: format!("About to run `{}`", validation_command.command),
            level: ApprovalLevel::P1,
            options: vec!["approve".to_string(), "reject".to_string()],
            recommended: Some("approve".to_string()),
            status: ApprovalStatus::Pending,
            choice: None,
            comment: None,
            created_at: Utc::now(),
            responded_at: None,
        },
    )
    .await?;

    let test_result = if tests_approved {
        emit(
            &mut recorder,
            sink,
            RunEvent::CommandStarted {
                command: validation_command.clone(),
            },
        )?;
        let tester_ctx = Tester::new(agent_config(mc_core::AgentType::Tester, model_id))
            .build_context(&task, Some(project_context.clone()), &shared)
            .await
            .map_err(|error| error.to_string())?
            .with_handoff(Arc::clone(&handoff))
            .with_impact_report(impact_report.clone())
            .with_execution_plan(execution_plan.clone());
        let result = run_agent_step(
            &mut recorder,
            sink,
            RunStep::new(
                "execute.tests",
                "Run focused validation",
                Some("execute".to_string()),
                Some("Tester".to_string()),
            ),
            async {
                let report = Tester::new(agent_config(mc_core::AgentType::Tester, model_id))
                    .execute(&tester_ctx)
                    .await
                    .map_err(|error| error.to_string())?;
                let tester_report = handoff
                    .get::<TesterExecutionReport>()
                    .await
                    .ok_or_else(|| "tester did not produce test report".to_string())?;
                Ok((
                    report.summary,
                    u64::from(report.metrics.tokens_used),
                    tester_report,
                ))
            },
            |recorder, tester_report| {
                recorder
                    .write_json_artifact(
                        "test-report.json",
                        "Test Report",
                        ArtifactKind::TestReport,
                        tester_report,
                        Some("Tester output".to_string()),
                    )
                    .map(|_| ())
            },
        )
        .await;
        let tester_report = handoff.get::<TesterExecutionReport>().await;
        if let Some(tester_report) = tester_report.as_ref() {
            let status = if tester_report.summary.success {
                CommandStatus::Completed
            } else {
                CommandStatus::Failed
            };
            emit(
                &mut recorder,
                sink,
                RunEvent::CommandFinished {
                    command_id: validation_command.command_id.clone(),
                    status,
                    exit_code: tester_report.summary.exit_code,
                    stdout_tail: tester_report.summary.stdout_tail.clone(),
                    stderr_tail: tester_report.summary.stderr_tail.clone(),
                },
            )?;
        }
        result.ok();
        tester_report
    } else {
        emit(
            &mut recorder,
            sink,
            RunEvent::CommandStarted {
                command: validation_command.clone(),
            },
        )?;
        emit(
            &mut recorder,
            sink,
            RunEvent::CommandFinished {
                command_id: validation_command.command_id.clone(),
                status: CommandStatus::Skipped,
                exit_code: None,
                stdout_tail: String::new(),
                stderr_tail: "skipped by approval policy".to_string(),
            },
        )?;
        emit(
            &mut recorder,
            sink,
            RunEvent::StepStarted {
                step: RunStep::new(
                    "execute.tests",
                    "Run focused validation",
                    Some("execute".to_string()),
                    Some("Tester".to_string()),
                ),
            },
        )?;
        emit(
            &mut recorder,
            sink,
            RunEvent::StepFinished {
                step_id: "execute.tests".to_string(),
                status: StepStatus::Skipped,
                summary: Some("validation skipped by approval policy".to_string()),
                token_used: 0,
            },
        )?;
        None
    };

    let execute_tokens = sum_child_tokens(recorder.snapshot(), "execute");
    emit(
        &mut recorder,
        sink,
        RunEvent::StepFinished {
            step_id: "execute".to_string(),
            status: StepStatus::Done,
            summary: Some(format!(
                "{} draft file(s) prepared",
                coder_output.changes.len()
            )),
            token_used: execute_tokens,
        },
    )?;

    let patch_decision = request_approval(
        &mut recorder,
        sink,
        options.approval,
        RunApproval {
            approval_id: format!("approval-{}", Uuid::new_v4()),
            step_id: "review.patch".to_string(),
            title: "Accept generated patch set".to_string(),
            reason: format!("Review verdict: {:?}", review_report.verdict),
            level: ApprovalLevel::P0,
            options: vec!["accept".to_string(), "reject".to_string()],
            recommended: Some("accept".to_string()),
            status: ApprovalStatus::Pending,
            choice: None,
            comment: None,
            created_at: Utc::now(),
            responded_at: None,
        },
    )
    .await?;
    let patch_status = if patch_decision {
        PatchStatus::Accepted
    } else {
        PatchStatus::Rejected
    };
    let patch_ids = recorder
        .snapshot()
        .summary
        .patches
        .iter()
        .map(|patch| patch.patch_id.clone())
        .collect::<Vec<_>>();
    for patch_id in patch_ids {
        emit(
            &mut recorder,
            sink,
            RunEvent::PatchResolved {
                patch_id,
                hunk_id: None,
                status: patch_status,
            },
        )?;
    }
    let review_tokens = sum_child_tokens(recorder.snapshot(), "review");
    emit(
        &mut recorder,
        sink,
        RunEvent::StepFinished {
            step_id: "review".to_string(),
            status: StepStatus::Done,
            summary: Some(format!("review verdict: {:?}", review_report.verdict)),
            token_used: review_tokens,
        },
    )?;

    let final_summary = build_markdown_summary(recorder.snapshot(), test_result.as_ref());
    recorder
        .write_text_artifact(
            "summary.md",
            "Run Summary",
            ArtifactKind::Markdown,
            &final_summary,
            Some("Human-readable run summary".to_string()),
        )
        .map_err(|error| error.to_string())?;

    finalize_run(
        &mut recorder,
        sink,
        RunStatus::Succeeded,
        format!(
            "generated {} patch drafts, review verdict {:?}",
            coder_output.changes.len(),
            review_report.verdict
        ),
        Some(format!("{:?}", review_report.verdict).to_ascii_lowercase()),
        started_at,
    )?;

    Ok(RunExecutionOutput {
        snapshot: recorder.snapshot().clone(),
        run_dir: recorder.run_dir().to_path_buf(),
    })
}

pub fn render_run_summary(output: &RunExecutionOutput) -> String {
    let snapshot = &output.snapshot;
    let mut text = String::new();
    let _ = writeln!(&mut text, "run_id: {}", snapshot.summary.run_id);
    let _ = writeln!(&mut text, "status: {:?}", snapshot.summary.status);
    if let Some(summary) = &snapshot.summary.final_summary {
        let _ = writeln!(&mut text, "summary: {summary}");
    }
    let _ = writeln!(&mut text, "tokens: {}", snapshot.summary.total_tokens);
    let _ = writeln!(&mut text, "artifacts: {}", snapshot.summary.artifacts.len());
    let _ = writeln!(&mut text, "run_dir: {}", output.run_dir.display());
    text.trim_end().to_string()
}

pub fn render_review(snapshot: &RunSnapshot) -> String {
    let mut text = String::new();
    let _ = writeln!(&mut text, "Run {}", snapshot.summary.run_id);
    let _ = writeln!(&mut text, "Request: {}", snapshot.summary.request);
    let _ = writeln!(&mut text, "Status: {:?}", snapshot.summary.status);
    if let Some(verdict) = &snapshot.summary.review_verdict {
        let _ = writeln!(&mut text, "Review verdict: {verdict}");
    }
    if !snapshot.summary.errors.is_empty() {
        let _ = writeln!(&mut text, "Errors:");
        for error in &snapshot.summary.errors {
            let _ = writeln!(&mut text, "- {error}");
        }
    }
    if !snapshot.summary.patches.is_empty() {
        let _ = writeln!(&mut text, "Patches:");
        for patch in &snapshot.summary.patches {
            let _ = writeln!(
                &mut text,
                "- [{}] {} ({:?})",
                format!("{:?}", patch.status).to_ascii_lowercase(),
                patch.file_path,
                patch.kind
            );
            let _ = writeln!(&mut text, "  rationale: {}", patch.rationale);
            if !patch.acceptance_checks.is_empty() {
                let _ = writeln!(
                    &mut text,
                    "  checks: {}",
                    patch.acceptance_checks.join(", ")
                );
            }
        }
    }
    text.trim_end().to_string()
}

pub fn render_replay(snapshot: &RunSnapshot, json: bool) -> String {
    if json {
        return snapshot
            .events
            .iter()
            .filter_map(|event| serde_json::to_string(event).ok())
            .collect::<Vec<_>>()
            .join("\n");
    }

    let mut text = String::new();
    for event in &snapshot.events {
        let _ = writeln!(
            &mut text,
            "[{}] {}",
            event.sequence,
            replay_label(&event.event)
        );
    }
    text.trim_end().to_string()
}

pub fn export_snapshot(
    context: &AppContext,
    snapshot: &RunSnapshot,
    format: ExportFormat,
) -> Result<PathBuf, String> {
    let run_dir = run_store(context).run_dir(&snapshot.summary.run_id);
    let export_dir = run_dir.join("artifacts").join("exports");
    std::fs::create_dir_all(&export_dir).map_err(|error| error.to_string())?;

    let (file_name, contents) = match format {
        ExportFormat::Md => ("summary-export.md", build_markdown_summary(snapshot, None)),
        ExportFormat::Jsonl => (
            "events-export.jsonl",
            snapshot
                .events
                .iter()
                .filter_map(|event| serde_json::to_string(event).ok())
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        ExportFormat::Html => ("run-review.html", build_html_export(snapshot)),
    };

    let path = export_dir.join(file_name);
    std::fs::write(&path, contents).map_err(|error| error.to_string())?;
    Ok(path)
}

async fn run_agent_step<T, F, G>(
    recorder: &mut RunRecorder,
    sink: Option<&dyn RunEventSink>,
    step: RunStep,
    future: F,
    persist: G,
) -> Result<(), String>
where
    F: std::future::Future<Output = Result<(String, u64, T), String>>,
    G: FnOnce(&mut RunRecorder, &T) -> io::Result<()>,
{
    let step_id = step.step_id.clone();
    emit(recorder, sink, RunEvent::StepStarted { step })?;
    let (summary, token_used, value) = future.await?;
    persist(recorder, &value).map_err(|error| error.to_string())?;
    emit(
        recorder,
        sink,
        RunEvent::StepFinished {
            step_id,
            status: StepStatus::Done,
            summary: Some(summary),
            token_used,
        },
    )?;
    Ok(())
}

fn emit(
    recorder: &mut RunRecorder,
    sink: Option<&dyn RunEventSink>,
    event: RunEvent,
) -> Result<(), String> {
    let envelope = recorder.emit(event).map_err(|error| error.to_string())?;
    if let Some(sink) = sink {
        sink.handle_event(&envelope)?;
    }
    Ok(())
}

fn build_llm_provider(resolved: ResolvedProviderEntry) -> Result<Arc<dyn LlmProvider>, String> {
    let provider_type = resolved.provider_type.as_str();

    let base_url = resolved
        .base_url
        .ok_or_else(|| "provider.base_url is required".to_string())?;
    let api_key = resolved
        .api_key
        .ok_or_else(|| "provider.api_key/api_key_env is required".to_string())?;
    let model_id = resolved
        .default_model
        .ok_or_else(|| "provider.default_model is required".to_string())?;
    let model = ModelInfo::new(&model_id, &model_id, &resolved.name);

    match provider_type {
        "openai-compat" => {
            let provider = OpenAiProvider::from_config(OpenAiProviderConfig {
                base_url,
                api_key,
                model,
                default_headers: resolved.headers,
                request_timeout: Duration::from_secs(120),
                stream_buffer_size: 64,
            })
            .map_err(|error| error.to_string())?;
            Ok(Arc::new(provider))
        }
        "anthropic" => {
            let provider = AnthropicProvider::from_config(AnthropicProviderConfig {
                base_url,
                api_key,
                model,
                anthropic_version: "2023-06-01".to_string(),
                beta_headers: Vec::new(),
                default_headers: resolved.headers,
                request_timeout: Duration::from_secs(120),
                stream_buffer_size: 64,
                default_max_tokens: 4_096,
            })
            .map_err(|error| error.to_string())?;
            Ok(Arc::new(provider))
        }
        "google" => {
            let provider = GoogleProvider::from_config(GoogleProviderConfig {
                base_url,
                api_key,
                model,
                default_headers: resolved.headers,
                request_timeout: Duration::from_secs(120),
                stream_buffer_size: 64,
                default_max_output_tokens: 8_192,
            })
            .map_err(|error| error.to_string())?;
            Ok(Arc::new(provider))
        }
        other => Err(format!("unsupported provider_type: {other}")),
    }
}

fn resolve_default_provider(
    config: &mc_config::ProviderConfig,
) -> Result<ResolvedProviderEntry, String> {
    let default_provider = config.default_provider.trim();
    let resolved = config.resolve_default_provider().ok_or_else(|| {
        let names = config.provider_names();
        if names.is_empty() {
            format!("cannot resolve default provider: {default_provider}")
        } else {
            format!(
                "cannot resolve default provider: {default_provider} (configured providers: {})",
                names.join(", ")
            )
        }
    })?;

    if resolved.provider_type == "mock" {
        return Err(format!(
            "mock provider is not supported for interactive execution (provider.default_provider = '{}')",
            resolved.name
        ));
    }

    Ok(resolved)
}

fn build_task_description(request: &str, project_root: &Path) -> TaskDescription {
    let mut task = TaskDescription::with_root(request, project_root);
    task.intent = infer_intent(request);
    task.affected_files = detect_affected_files(request, project_root);
    task.complexity = infer_complexity(request, &task.affected_files);
    task.requires_testing = !request.to_ascii_lowercase().contains("doc");
    task.details = Some("Generated by the interactive run workflow".to_string());
    task
}

fn infer_intent(request: &str) -> TaskIntent {
    let lowered = request.to_ascii_lowercase();
    if lowered.contains("fix") || lowered.contains("bug") {
        TaskIntent::BugFix
    } else if lowered.contains("refactor") {
        TaskIntent::Refactoring
    } else if lowered.contains("doc") || lowered.contains("readme") {
        TaskIntent::Documentation
    } else {
        TaskIntent::FeatureAddition
    }
}

fn infer_complexity(request: &str, files: &[String]) -> Complexity {
    if files.len() > 3 || request.split_whitespace().count() > 18 {
        Complexity::Complex
    } else if files.len() > 1 || request.split_whitespace().count() > 8 {
        Complexity::Medium
    } else {
        Complexity::Simple
    }
}

fn detect_affected_files(request: &str, project_root: &Path) -> Vec<String> {
    let mut files = request
        .split_whitespace()
        .map(|token| {
            token
                .trim_matches(|ch: char| {
                    !ch.is_ascii_alphanumeric()
                        && ch != '/'
                        && ch != '\\'
                        && ch != '.'
                        && ch != '_'
                        && ch != '-'
                })
                .replace('\\', "/")
        })
        .filter(|token| token.contains('/') || token.contains('.'))
        .filter(|token| project_root.join(token).exists())
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    files
}

fn record_patch_artifacts(
    recorder: &mut RunRecorder,
    sink: Option<&dyn RunEventSink>,
    coder_output: &CodeGenerationOutput,
    step_id: &str,
) -> Result<(), String> {
    for (index, change) in coder_output.changes.iter().enumerate() {
        let patch = RunPatch {
            patch_id: format!("patch-{}", Uuid::new_v4()),
            step_id: step_id.to_string(),
            file_path: change.path.clone(),
            kind: map_patch_kind(change.change_kind),
            rationale: change.rationale.clone(),
            preview: change.patch_preview.clone(),
            acceptance_checks: change.acceptance_checks.clone(),
            hunks: build_patch_hunks(&change.patch_preview),
            status: PatchStatus::Pending,
        };
        recorder
            .write_text_artifact(
                PathBuf::from("patches").join(format!(
                    "{}-{}.patch",
                    index + 1,
                    sanitize_file_name(&change.path)
                )),
                format!("Patch {}", change.path),
                ArtifactKind::Patch,
                &change.patch_preview,
                Some(change.rationale.clone()),
            )
            .map_err(|error| error.to_string())?;
        emit(recorder, sink, RunEvent::PatchProposed { patch })?;
    }
    Ok(())
}

fn sum_child_tokens(snapshot: &mc_core::RunSnapshot, parent_step_id: &str) -> u64 {
    let prefix = format!("{parent_step_id}.");
    snapshot
        .events
        .iter()
        .filter_map(|envelope| match &envelope.event {
            RunEvent::StepFinished { step_id, token_used, .. }
                if step_id.starts_with(&prefix) =>
            {
                Some(*token_used)
            }
            _ => None,
        })
        .sum()
}

async fn request_approval(
    recorder: &mut RunRecorder,
    sink: Option<&dyn RunEventSink>,
    policy: ApprovalMode,
    approval: RunApproval,
) -> Result<bool, String> {
    let approval_id = approval.approval_id.clone();
    emit(recorder, sink, RunEvent::ApprovalRequested { approval })?;

    let (status, choice, comment) = match policy {
        ApprovalMode::Auto => (
            ApprovalStatus::Approved,
            Some("approve".to_string()),
            Some("auto-approved".to_string()),
        ),
        ApprovalMode::Deny => (
            ApprovalStatus::Rejected,
            Some("reject".to_string()),
            Some("blocked by approval policy".to_string()),
        ),
        ApprovalMode::Prompt => prompt_for_approval(recorder.snapshot(), &approval_id).await?,
    };

    emit(
        recorder,
        sink,
        RunEvent::ApprovalResolved {
            approval_id,
            status,
            choice,
            comment,
        },
    )?;

    Ok(status == ApprovalStatus::Approved)
}

async fn prompt_for_approval(
    snapshot: &RunSnapshot,
    approval_id: &str,
) -> Result<(ApprovalStatus, Option<String>, Option<String>), String> {
    let Some(approval) = snapshot
        .summary
        .approvals
        .iter()
        .find(|item| item.approval_id == approval_id)
    else {
        return Ok((
            ApprovalStatus::Rejected,
            Some("reject".to_string()),
            Some("approval request missing".to_string()),
        ));
    };

    if !std::io::stdin().is_terminal() {
        return Ok((
            ApprovalStatus::Rejected,
            Some("reject".to_string()),
            Some("non-interactive session".to_string()),
        ));
    }

    print!(
        "\n[approval:{}] {} [{}]\n{}\noptions: {}\nchoice: ",
        approval.approval_id,
        approval.title,
        format!("{:?}", approval.level).to_ascii_lowercase(),
        approval.reason,
        approval.options.join("/")
    );
    io::stdout().flush().map_err(|error| error.to_string())?;

    let mut line = String::new();
    tokio::io::BufReader::new(tokio::io::stdin())
        .read_line(&mut line)
        .await
        .map_err(|error| error.to_string())?;
    let choice = line.trim().to_ascii_lowercase();
    if choice.is_empty() {
        return Ok((
            ApprovalStatus::Rejected,
            Some("reject".to_string()),
            Some("empty choice".to_string()),
        ));
    }

    let approved = choice == "approve" || choice == "accept" || choice == "yes" || choice == "y";
    Ok((
        if approved {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Rejected
        },
        Some(choice),
        Some("interactive decision".to_string()),
    ))
}

fn build_validation_command(
    task: &TaskDescription,
    execution_plan: &mc_core::ExecutionPlan,
    project_root: &Path,
) -> Option<RunCommand> {
    let focused = derive_focus_filters(Some(execution_plan), Some(task));
    let framework = detect_framework(
        project_root,
        &FrameworkDetectionContext {
            hint: task.user_input.as_str(),
        },
    );
    let command = framework.build_command(&focused, project_root);
    Some(RunCommand {
        command_id: format!("cmd-{}", Uuid::new_v4()),
        step_id: "execute.tests".to_string(),
        title: "Focused validation".to_string(),
        command: command.render(),
        cwd: command.cwd.to_string_lossy().into_owned(),
        status: CommandStatus::Running,
        started_at: None,
        finished_at: None,
        exit_code: None,
        stdout_tail: String::new(),
        stderr_tail: String::new(),
    })
}

fn finalize_run(
    recorder: &mut RunRecorder,
    sink: Option<&dyn RunEventSink>,
    status: RunStatus,
    summary: String,
    review_verdict: Option<String>,
    started_at: chrono::DateTime<Utc>,
) -> Result<(), String> {
    let total_tokens = recorder.snapshot().summary.total_tokens;
    let changed_files = recorder
        .snapshot()
        .summary
        .patches
        .iter()
        .filter(|patch| patch.status == PatchStatus::Accepted)
        .map(|patch| patch.file_path.clone())
        .collect::<Vec<_>>();
    let duration_ms = Utc::now()
        .signed_duration_since(started_at)
        .num_milliseconds()
        .max(0) as u64;
    emit(
        recorder,
        sink,
        RunEvent::RunFinished {
            status,
            summary: Some(summary),
            total_tokens,
            total_duration_ms: duration_ms,
            review_verdict,
            changed_files,
        },
    )
}

fn build_markdown_summary(
    snapshot: &RunSnapshot,
    test_report: Option<&TesterExecutionReport>,
) -> String {
    let mut text = String::new();
    let _ = writeln!(&mut text, "# Run {}", snapshot.summary.run_id);
    let _ = writeln!(&mut text);
    let _ = writeln!(&mut text, "- Request: {}", snapshot.summary.request);
    let _ = writeln!(&mut text, "- Status: {:?}", snapshot.summary.status);
    let _ = writeln!(&mut text, "- Tokens: {}", snapshot.summary.total_tokens);
    if let Some(verdict) = &snapshot.summary.review_verdict {
        let _ = writeln!(&mut text, "- Review verdict: {verdict}");
    }
    let _ = writeln!(&mut text);
    let _ = writeln!(&mut text, "## Steps");
    for step in &snapshot.summary.steps {
        let indent = if step.parent_step_id.is_some() {
            "  "
        } else {
            ""
        };
        let _ = writeln!(
            &mut text,
            "{}- {} [{:?}] {}",
            indent,
            step.title,
            step.status,
            step.summary.clone().unwrap_or_default()
        );
    }
    let _ = writeln!(&mut text);
    let _ = writeln!(&mut text, "## Patches");
    for patch in &snapshot.summary.patches {
        let _ = writeln!(
            &mut text,
            "- {} [{:?}] {:?}",
            patch.file_path, patch.status, patch.kind
        );
    }
    if let Some(test_report) = test_report {
        let _ = writeln!(&mut text);
        let _ = writeln!(&mut text, "## Tests");
        let _ = writeln!(
            &mut text,
            "- {} (passed: {}, failed: {}, skipped: {})",
            test_report.command.render(),
            test_report.summary.passed,
            test_report.summary.failed,
            test_report.summary.skipped
        );
    }
    text
}

fn build_html_export(snapshot: &RunSnapshot) -> String {
    let steps = snapshot
        .summary
        .steps
        .iter()
        .map(|step| {
            format!(
                "<li><strong>{}</strong> <span>{:?}</span><div>{}</div></li>",
                escape_html(&step.title),
                step.status,
                escape_html(step.summary.as_deref().unwrap_or(""))
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let patches = snapshot
        .summary
        .patches
        .iter()
        .map(|patch| {
            format!(
                "<section class=\"patch\"><h3>{}</h3><p>{:?} · {:?}</p><pre>{}</pre></section>",
                escape_html(&patch.file_path),
                patch.kind,
                patch.status,
                escape_html(&patch.preview)
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{}</title><style>\
        :root{{color-scheme:dark;--bg:#0d0f12;--panel:#161b21;--edge:rgba(255,255,255,0.08);--text:#ebe7de;--muted:#9aa1ac;--accent:#d7c2a4;}}\
        body{{font-family:ui-sans-serif,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;background:radial-gradient(circle at top left,rgba(215,194,164,0.08),transparent 26%),linear-gradient(180deg,#0d0f12 0%,#11151a 44%,#0a0c10 100%);color:var(--text);margin:0;padding:24px;}}\
        .grid{{display:grid;grid-template-columns:1fr 1fr;gap:24px;align-items:start;}}\
        .panel{{border:1px solid var(--edge);border-radius:18px;padding:20px;background:linear-gradient(180deg,rgba(27,32,38,0.96),rgba(18,22,27,0.96));box-shadow:0 24px 70px rgba(0,0,0,0.38);}}\
        h1,h2,h3{{margin-top:0;}} h1{{font-size:clamp(2rem,4vw,3.5rem);line-height:0.94;letter-spacing:-0.04em;}}\
        p,li,span{{color:var(--muted);}} strong{{color:var(--text);}}\
        pre{{white-space:pre-wrap;background:#0f141b;color:#f0f3f7;padding:14px;border-radius:12px;overflow:auto;border:1px solid rgba(255,255,255,0.05);}}\
        .patch + .patch{{margin-top:16px;}}</style></head><body>\
        <h1>Run {}</h1><div class=\"grid\">\
        <section class=\"panel\"><h2>Summary</h2><p>Status: {:?}</p><p>Request: {}</p><p>Tokens: {}</p><h2>Steps</h2><ol>{}</ol></section>\
        <section class=\"panel\"><h2>Patches</h2>{}</section></div></body></html>",
        escape_html(&snapshot.summary.run_id),
        escape_html(&snapshot.summary.run_id),
        snapshot.summary.status,
        escape_html(&snapshot.summary.request),
        snapshot.summary.total_tokens,
        steps,
        patches
    )
}

fn replay_label(event: &RunEvent) -> String {
    match event {
        RunEvent::RunStarted { request, .. } => format!("run started: {request}"),
        RunEvent::StepStarted { step } => format!("step started: {}", step.title),
        RunEvent::StepFinished {
            step_id, summary, ..
        } => format!(
            "step finished: {} {}",
            step_id,
            summary.clone().unwrap_or_default()
        ),
        RunEvent::Message { message, .. } => format!("message: {message}"),
        RunEvent::ApprovalRequested { approval } => {
            format!("approval requested: {}", approval.title)
        }
        RunEvent::ApprovalResolved {
            approval_id,
            status,
            ..
        } => {
            format!("approval resolved: {approval_id} -> {status:?}")
        }
        RunEvent::PatchProposed { patch } => format!("patch proposed: {}", patch.file_path),
        RunEvent::PatchResolved {
            patch_id, status, ..
        } => format!("patch resolved: {patch_id} -> {status:?}"),
        RunEvent::ArtifactWritten { artifact } => format!("artifact: {}", artifact.title),
        RunEvent::CommandStarted { command } => format!("command started: {}", command.command),
        RunEvent::CommandOutput { command_id, .. } => format!("command output: {command_id}"),
        RunEvent::CommandFinished {
            command_id, status, ..
        } => format!("command finished: {command_id} -> {status:?}"),
        RunEvent::RunFinished { status, .. } => format!("run finished: {status:?}"),
        RunEvent::Error { message, .. } => format!("error: {message}"),
    }
}

fn map_patch_kind(kind: CodeChangeKind) -> PatchKind {
    match kind {
        CodeChangeKind::Add => PatchKind::Add,
        CodeChangeKind::Modify => PatchKind::Modify,
        CodeChangeKind::Delete => PatchKind::Delete,
    }
}

fn sanitize_file_name(path: &str) -> String {
    path.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
