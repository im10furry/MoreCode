use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use mc_core::{AgentType, ProjectContext, TaskDescription};
use regex::Regex;
use serde_json::json;
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::execution_report::{build_report, serialize_extra};
use crate::{Agent, AgentConfig, AgentContext, AgentError, SharedResources};

pub mod diagnosis;
pub use diagnosis::{
    ErrorAnalysis, ErrorType, FixReport, FixSuggestion, LogPattern, ParsedStackTrace, StackFrame,
    SuggestedChange,
};

pub trait LogAnalyzer: Send + Sync {
    fn analyze(&self, log_text: &str) -> Vec<LogPattern>;
}

pub trait StackTraceParser: Send + Sync {
    fn parse(&self, trace: &str) -> ParsedStackTrace;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultLogAnalyzer;

impl LogAnalyzer for DefaultLogAnalyzer {
    fn analyze(&self, log_text: &str) -> Vec<LogPattern> {
        let mut grouped: HashMap<String, (String, usize)> = HashMap::new();
        for line in log_text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            let severity = if contains_any(line, &["error", "panic", "fatal"]) {
                "error"
            } else if contains_any(line, &["warn", "warning"]) {
                "warning"
            } else {
                "info"
            };

            let normalized = normalize_log_line(line);
            let entry = grouped
                .entry(normalized)
                .or_insert_with(|| (severity.to_string(), 0));
            entry.1 += 1;
        }

        let mut patterns = grouped
            .into_iter()
            .map(|(pattern, (severity, frequency))| LogPattern {
                pattern,
                severity,
                frequency,
            })
            .collect::<Vec<_>>();
        patterns.sort_by(|left, right| right.frequency.cmp(&left.frequency));
        patterns
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultStackTraceParser;

impl StackTraceParser for DefaultStackTraceParser {
    fn parse(&self, trace: &str) -> ParsedStackTrace {
        let frame_regex = Regex::new(
            r"(?:(?P<function>[A-Za-z0-9_:<>$]+).*)?(?P<file>[A-Za-z0-9_./\\-]+\.rs):(?P<line>\d+)(?::(?P<column>\d+))?",
        )
        .expect("frame regex must compile");

        let mut frames = Vec::new();
        let mut causes = Vec::new();

        for line in trace.lines().map(str::trim).filter(|line| !line.is_empty()) {
            if let Some(cause) = line
                .strip_prefix("Caused by:")
                .or_else(|| line.strip_prefix("caused by:"))
            {
                causes.push(cause.trim().to_string());
            }

            if let Some(captures) = frame_regex.captures(line) {
                frames.push(StackFrame {
                    function: captures
                        .name("function")
                        .map(|capture| capture.as_str().trim().to_string())
                        .filter(|value| !value.is_empty()),
                    file: captures
                        .name("file")
                        .map(|capture| capture.as_str().to_string()),
                    line: captures
                        .name("line")
                        .and_then(|capture| capture.as_str().parse::<u32>().ok()),
                    column: captures
                        .name("column")
                        .and_then(|capture| capture.as_str().parse::<u32>().ok()),
                    raw: line.to_string(),
                });
            }
        }

        ParsedStackTrace { frames, causes }
    }
}

#[derive(Clone)]
pub struct Debugger {
    config: AgentConfig,
    log_analyzer: Arc<dyn LogAnalyzer>,
    stack_trace_parser: Arc<dyn StackTraceParser>,
    log_tool: String,
}

impl Debugger {
    pub fn new(
        config: AgentConfig,
        log_analyzer: Arc<dyn LogAnalyzer>,
        stack_trace_parser: Arc<dyn StackTraceParser>,
    ) -> Self {
        Self {
            config,
            log_analyzer,
            stack_trace_parser,
            log_tool: "file_read".to_string(),
        }
    }

    pub fn with_log_tool(mut self, log_tool: impl Into<String>) -> Self {
        self.log_tool = log_tool.into();
        self
    }

    fn analyze_bundle(
        &self,
        ctx: AgentContext,
    ) -> Pin<Box<dyn Future<Output = Result<FixReport, AgentError>> + Send + '_>> {
        Box::pin(async move {
            if ctx.is_cancelled() {
                return Err(AgentError::Cancelled {
                    reason: "debugger cancelled".to_string(),
                });
            }

            if ctx.recursion_depth > ctx.config.max_recursion_depth {
                return Err(AgentError::RecursionDepthExceeded {
                    current: ctx.recursion_depth,
                    max: ctx.config.max_recursion_depth,
                });
            }

            let inputs = self.collect_inputs(&ctx).await?;
            let error_blocks = collect_error_blocks(&ctx, &inputs);
            let mut join_set = JoinSet::new();
            for block in error_blocks
                .into_iter()
                .take(ctx.config.max_parallel_tasks.max(1))
            {
                let agent = self.clone();
                let child_task = clone_task_with_input(ctx.task.as_ref(), block);
                let child_ctx = ctx.create_child_context(child_task);
                let inputs = inputs.clone();
                join_set.spawn(async move { agent.analyze_single_error(child_ctx, inputs).await });
            }

            let mut analyses = Vec::new();
            while let Some(result) = join_set.join_next().await {
                match result {
                    Ok(Ok(analysis)) => analyses.push(analysis),
                    Ok(Err(error)) => {
                        analyses.push(ErrorAnalysis {
                            root_cause: error.to_string(),
                            error_type: ErrorType::Unknown,
                            affected_files: Vec::new(),
                            confidence: 0.1,
                            evidence: vec![error.to_string()],
                        });
                    }
                    Err(error) => {
                        analyses.push(ErrorAnalysis {
                            root_cause: error.to_string(),
                            error_type: ErrorType::Unknown,
                            affected_files: Vec::new(),
                            confidence: 0.1,
                            evidence: vec![error.to_string()],
                        });
                    }
                }
            }

            let best = analyses
                .iter()
                .max_by(|left, right| left.confidence.total_cmp(&right.confidence))
                .cloned()
                .unwrap_or_else(|| ErrorAnalysis {
                    root_cause: "No error analysis was produced.".to_string(),
                    error_type: ErrorType::Unknown,
                    affected_files: Vec::new(),
                    confidence: 0.0,
                    evidence: Vec::new(),
                });

            let fix_suggestion = self.generate_fix_suggestion(&best, &ctx);
            let verified = self.verify_fix(&ctx, &fix_suggestion).await;

            Ok(FixReport {
                root_cause: best.root_cause,
                error_type: best.error_type,
                affected_files: best.affected_files,
                fix_suggestion,
                verified,
                confidence: best.confidence,
            })
        })
    }

    fn analyze_single_error(
        &self,
        ctx: AgentContext,
        inputs: DebugInputs,
    ) -> Pin<Box<dyn Future<Output = Result<ErrorAnalysis, AgentError>> + Send + '_>> {
        Box::pin(async move {
            let error_text = ctx.task.user_input.clone();
            let stack = self.stack_trace_parser.parse(&format!(
                "{}\n{}",
                error_text,
                inputs.stack_traces.join("\n")
            ));
            let log_patterns = self.log_analyzer.analyze(&inputs.logs.join("\n"));

            let nested_causes = extract_nested_causes(&error_text);
            let mut nested_analysis: Option<ErrorAnalysis> = None;
            if !nested_causes.is_empty() && ctx.recursion_depth < ctx.config.max_recursion_depth {
                let mut join_set = JoinSet::new();
                for cause in nested_causes.into_iter().take(2) {
                    let agent = self.clone();
                    let child_task = clone_task_with_input(ctx.task.as_ref(), cause);
                    let child_ctx = ctx.create_child_context(child_task);
                    let inputs = inputs.clone();
                    join_set
                        .spawn(async move { agent.analyze_single_error(child_ctx, inputs).await });
                }

                while let Some(result) = join_set.join_next().await {
                    if let Ok(Ok(analysis)) = result {
                        nested_analysis = Some(match nested_analysis {
                            Some(existing) if existing.confidence >= analysis.confidence => {
                                existing
                            }
                            _ => analysis,
                        });
                    }
                }
            }

            let mut affected_files = collect_affected_files(&error_text, &stack);
            let error_type = infer_error_type(&error_text, &log_patterns);
            let root_cause = infer_root_cause(
                &error_text,
                error_type,
                &log_patterns,
                &stack,
                nested_analysis.as_ref(),
            );
            let mut evidence = build_evidence(&error_text, &log_patterns, &stack);

            let confidence = compute_confidence(
                &affected_files,
                &log_patterns,
                &stack,
                nested_analysis.as_ref(),
            );

            if let Some(nested) = nested_analysis {
                affected_files.extend(nested.affected_files);
                evidence.extend(nested.evidence);
            }

            affected_files = dedupe_strings(affected_files);
            evidence = dedupe_strings(evidence);

            Ok(ErrorAnalysis {
                root_cause,
                error_type,
                affected_files,
                confidence,
                evidence,
            })
        })
    }

    async fn collect_inputs(&self, ctx: &AgentContext) -> Result<DebugInputs, AgentError> {
        let mut logs = ctx.get_metadata::<Vec<String>>("logs").unwrap_or_default();
        let mut stack_traces = ctx
            .get_metadata::<Vec<String>>("stack_traces")
            .unwrap_or_default();

        if let Some(log_paths) = ctx.get_metadata::<Vec<String>>("log_paths") {
            for path in log_paths {
                let value = if ctx.has_tool(&self.log_tool).await {
                    ctx.call_tool_value(
                        AgentType::Debugger,
                        &self.log_tool,
                        json!({
                            "path": path,
                            "offset": 0,
                            "limit": 400,
                        }),
                    )
                    .await?
                } else {
                    json!({})
                };
                if let Some(content) = value.get("content").and_then(serde_json::Value::as_str) {
                    logs.push(content.to_string());
                }
            }
        }

        if stack_traces.is_empty() && contains_any(&ctx.task.user_input, &["backtrace", ".rs:"]) {
            stack_traces.push(ctx.task.user_input.clone());
        }

        Ok(DebugInputs { logs, stack_traces })
    }

    fn generate_fix_suggestion(
        &self,
        analysis: &ErrorAnalysis,
        ctx: &AgentContext,
    ) -> FixSuggestion {
        let changes = analysis
            .affected_files
            .iter()
            .map(|file| SuggestedChange {
                file: file.clone(),
                change_type: change_type_for_error(analysis.error_type).to_string(),
                description: fix_description_for_error(analysis.error_type, &analysis.root_cause),
                code_diff: None,
            })
            .collect::<Vec<_>>();

        let steps = vec![
            format!(
                "Inspect the primary failure context: {}",
                analysis.root_cause
            ),
            "Apply the change in the affected file or configuration location.".to_string(),
            format!("Re-run the failing workflow for task `{}`.", ctx.task.id),
            "Add a regression test or validation step that reproduces the issue.".to_string(),
        ];

        let prevention = prevention_for_error(analysis.error_type);

        FixSuggestion {
            description: fix_description_for_error(analysis.error_type, &analysis.root_cause),
            changes,
            steps,
            prevention,
        }
    }

    async fn verify_fix(&self, ctx: &AgentContext, fix: &FixSuggestion) -> bool {
        if fix.changes.is_empty() {
            return false;
        }

        if let Some(project_root) = ctx.project_root() {
            return fix
                .changes
                .iter()
                .all(|change| change_path_exists(&project_root, &change.file));
        }

        false
    }
}

#[async_trait]
impl Agent for Debugger {
    fn agent_type(&self) -> AgentType {
        AgentType::Debugger
    }

    fn supports_recursion(&self) -> bool {
        true
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn default_config(&self) -> AgentConfig {
        AgentConfig::for_agent_type(AgentType::Debugger)
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
            ctx.project_ctx = Some(Arc::new(project_ctx));
        }
        Ok(ctx)
    }

    async fn execute(
        &self,
        ctx: &AgentContext,
    ) -> Result<mc_core::AgentExecutionReport, AgentError> {
        let report = self.analyze_bundle(ctx.clone()).await?;
        ctx.handoff.put(report.clone()).await;

        Ok(build_report(
            AgentType::Debugger,
            "fix report generated",
            vec![report.root_cause.clone()],
            report.affected_files.clone(),
            report.fix_suggestion.steps.clone(),
            Vec::new(),
            (report.affected_files.len() + report.fix_suggestion.steps.len()) as u32,
            Some(serialize_extra(&report)?),
        ))
    }
}

#[derive(Debug, Clone, Default)]
struct DebugInputs {
    logs: Vec<String>,
    stack_traces: Vec<String>,
}

fn collect_error_blocks(ctx: &AgentContext, inputs: &DebugInputs) -> Vec<String> {
    if let Some(errors) = ctx.get_metadata::<Vec<String>>("errors") {
        if !errors.is_empty() {
            return errors;
        }
    }

    let mut blocks = ctx
        .task
        .user_input
        .split("\n---\n")
        .map(str::trim)
        .filter(|block| !block.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if blocks.is_empty() {
        blocks.push(ctx.task.user_input.clone());
    }

    if blocks.len() == 1 && !inputs.stack_traces.is_empty() {
        blocks[0].push('\n');
        blocks[0].push_str(&inputs.stack_traces.join("\n"));
    }

    blocks
}

fn clone_task_with_input(task: &TaskDescription, user_input: String) -> TaskDescription {
    let mut cloned = task.clone();
    cloned.id = Uuid::new_v4().to_string();
    cloned.user_input = user_input;
    cloned
}

fn extract_nested_causes(error_text: &str) -> Vec<String> {
    error_text
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("Caused by:")
                .or_else(|| line.trim().strip_prefix("caused by:"))
                .map(|cause| cause.trim().to_string())
        })
        .filter(|cause| !cause.is_empty())
        .collect()
}

fn collect_affected_files(error_text: &str, stack: &ParsedStackTrace) -> Vec<String> {
    let file_regex = Regex::new(r"([A-Za-z0-9_./\\-]+\.rs):\d+").expect("file regex must compile");

    let mut files = stack
        .frames
        .iter()
        .filter_map(|frame| frame.file.clone())
        .collect::<Vec<_>>();

    files.extend(
        file_regex
            .captures_iter(error_text)
            .filter_map(|captures| captures.get(1).map(|capture| capture.as_str().to_string())),
    );

    dedupe_strings(files)
}

fn infer_error_type(error_text: &str, patterns: &[LogPattern]) -> ErrorType {
    let lower = error_text.to_lowercase();
    if contains_any(
        &lower,
        &[
            "syntax error",
            "unexpected token",
            "expected `",
            "unexpected `",
        ],
    ) {
        return ErrorType::SyntaxError;
    }
    if contains_any(
        &lower,
        &[
            "mismatched types",
            "trait bound",
            "cannot infer",
            "type mismatch",
        ],
    ) {
        return ErrorType::TypeError;
    }
    if contains_any(&lower, &["panic", "thread '", "backtrace", "runtime error"]) {
        return ErrorType::RuntimeError;
    }
    if contains_any(
        &lower,
        &["config", "configuration", "env var", "environment variable"],
    ) {
        return ErrorType::ConfigurationError;
    }
    if contains_any(
        &lower,
        &["dependency", "crate", "feature", "version conflict"],
    ) {
        return ErrorType::DependencyError;
    }
    if contains_any(
        &lower,
        &["deadlock", "race", "concurrent", "send", "sync", "poisoned"],
    ) {
        return ErrorType::ConcurrencyError;
    }
    if contains_any(
        &lower,
        &[
            "assertion failed",
            "unexpected behavior",
            "incorrect result",
        ],
    ) {
        return ErrorType::LogicError;
    }
    if patterns
        .iter()
        .any(|pattern| contains_any(&pattern.pattern.to_lowercase(), &["panic", "error"]))
    {
        return ErrorType::RuntimeError;
    }
    ErrorType::Unknown
}

fn infer_root_cause(
    error_text: &str,
    error_type: ErrorType,
    patterns: &[LogPattern],
    stack: &ParsedStackTrace,
    nested: Option<&ErrorAnalysis>,
) -> String {
    if let Some(nested) = nested {
        return format!("Nested cause identified: {}", nested.root_cause);
    }

    if let Some(primary_pattern) = patterns.first() {
        return format!(
            "{} pattern observed: {}",
            primary_pattern.severity, primary_pattern.pattern
        );
    }

    if let Some(frame) = stack.frames.first() {
        let file = frame
            .file
            .clone()
            .unwrap_or_else(|| "unknown file".to_string());
        let function = frame
            .function
            .clone()
            .unwrap_or_else(|| "unknown function".to_string());
        return format!("Likely failure around {function} in {file}");
    }

    let description = match error_type {
        ErrorType::SyntaxError => "Syntax issue reported by the parser or compiler.",
        ErrorType::TypeError => {
            "Type contract mismatch detected during compilation or runtime coercion."
        }
        ErrorType::LogicError => {
            "Control-flow or business-rule branch is inconsistent with expectations."
        }
        ErrorType::RuntimeError => "Unhandled runtime failure propagated to the caller.",
        ErrorType::ConfigurationError => {
            "Configuration or environment input is missing or malformed."
        }
        ErrorType::DependencyError => "Dependency graph or feature selection is inconsistent.",
        ErrorType::ConcurrencyError => "Shared-state or async coordination issue detected.",
        ErrorType::Unknown => "Root cause could not be classified precisely.",
    };

    if error_text.trim().is_empty() {
        description.to_string()
    } else {
        format!("{description} Raw error: {}", first_sentence(error_text))
    }
}

fn build_evidence(
    error_text: &str,
    patterns: &[LogPattern],
    stack: &ParsedStackTrace,
) -> Vec<String> {
    let mut evidence = Vec::new();
    evidence.push(first_sentence(error_text));
    evidence.extend(patterns.iter().take(3).map(|pattern| {
        format!(
            "{} x{}: {}",
            pattern.severity, pattern.frequency, pattern.pattern
        )
    }));
    evidence.extend(stack.frames.iter().take(3).map(|frame| frame.raw.clone()));
    dedupe_strings(evidence)
}

fn compute_confidence(
    affected_files: &[String],
    patterns: &[LogPattern],
    stack: &ParsedStackTrace,
    nested: Option<&ErrorAnalysis>,
) -> f64 {
    let mut confidence: f64 = 0.35;
    if !affected_files.is_empty() {
        confidence += 0.2;
    }
    if !patterns.is_empty() {
        confidence += 0.15;
    }
    if !stack.frames.is_empty() {
        confidence += 0.15;
    }
    if nested.is_some() {
        confidence += 0.1;
    }
    confidence.min(0.95)
}

fn change_type_for_error(error_type: ErrorType) -> &'static str {
    match error_type {
        ErrorType::SyntaxError => "syntax_fix",
        ErrorType::TypeError => "type_fix",
        ErrorType::LogicError => "logic_fix",
        ErrorType::RuntimeError => "runtime_guard",
        ErrorType::ConfigurationError => "config_update",
        ErrorType::DependencyError => "dependency_alignment",
        ErrorType::ConcurrencyError => "concurrency_control",
        ErrorType::Unknown => "investigation",
    }
}

fn fix_description_for_error(error_type: ErrorType, root_cause: &str) -> String {
    let prefix = match error_type {
        ErrorType::SyntaxError => "Correct the invalid syntax near the reported location",
        ErrorType::TypeError => "Align the source and target types across the failing boundary",
        ErrorType::LogicError => "Rework the faulty branch or state transition",
        ErrorType::RuntimeError => "Add defensive handling for the failing runtime path",
        ErrorType::ConfigurationError => "Fix the missing or invalid configuration input",
        ErrorType::DependencyError => "Align dependency versions or enabled features",
        ErrorType::ConcurrencyError => "Stabilize concurrent access or async coordination",
        ErrorType::Unknown => "Reproduce the issue with narrower instrumentation",
    };
    format!("{prefix}: {root_cause}")
}

fn prevention_for_error(error_type: ErrorType) -> Vec<String> {
    match error_type {
        ErrorType::SyntaxError => vec![
            "Run formatting and lint checks before committing.".to_string(),
            "Keep parser-visible macros and generated code covered by tests.".to_string(),
        ],
        ErrorType::TypeError => vec![
            "Prefer explicit type boundaries at module and API edges.".to_string(),
            "Add compile-time tests for generic or trait-heavy paths.".to_string(),
        ],
        ErrorType::LogicError => vec![
            "Add regression tests for the failing branch.".to_string(),
            "Document the expected invariant near the implementation.".to_string(),
        ],
        ErrorType::RuntimeError => vec![
            "Replace unwrap/expect on non-guaranteed paths with explicit error handling."
                .to_string(),
            "Capture enough telemetry to reproduce the runtime failure.".to_string(),
        ],
        ErrorType::ConfigurationError => vec![
            "Validate required configuration at startup.".to_string(),
            "Keep environment examples and runtime validation in sync.".to_string(),
        ],
        ErrorType::DependencyError => vec![
            "Pin critical dependency versions and feature sets.".to_string(),
            "Add CI checks for dependency drift.".to_string(),
        ],
        ErrorType::ConcurrencyError => vec![
            "Minimize shared mutable state across async tasks.".to_string(),
            "Stress-test high-contention paths with deterministic fixtures.".to_string(),
        ],
        ErrorType::Unknown => vec![
            "Preserve logs, stack traces, and reproduction steps for follow-up analysis."
                .to_string(),
        ],
    }
}

fn normalize_log_line(line: &str) -> String {
    let timestamp_regex = Regex::new(r"^\[?\d{4}-\d{2}-\d{2}[T ][0-9:.+-]+]?\s*")
        .expect("timestamp regex must compile");
    timestamp_regex
        .replace(line, "")
        .replace("ERROR", "")
        .replace("WARN", "")
        .replace("INFO", "")
        .trim()
        .to_string()
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    let haystack = haystack.to_lowercase();
    needles
        .iter()
        .any(|needle| haystack.contains(&needle.to_lowercase()))
}

fn first_sentence(text: &str) -> String {
    text.split(['.', '!', '?', '\n', '。', '！', '？'])
        .map(str::trim)
        .find(|segment| !segment.is_empty())
        .unwrap_or_default()
        .to_string()
}

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

fn change_path_exists(project_root: &std::path::Path, file: &str) -> bool {
    let raw = std::path::PathBuf::from(file);
    if raw.is_absolute() {
        return raw.exists();
    }

    if project_root.join(&raw).exists() {
        return true;
    }

    let normalized_suffix = file.replace('\\', "/");
    let mut stack = vec![project_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            let candidate = path.to_string_lossy().replace('\\', "/");
            if candidate.ends_with(&normalized_suffix) {
                return true;
            }
        }
    }

    false
}
