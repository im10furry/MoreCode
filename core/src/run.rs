use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    Running,
    WaitingApproval,
    Failed,
    Succeeded,
    Canceled,
}

impl RunStatus {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Failed | Self::Succeeded | Self::Canceled)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Done,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    ProjectContext,
    ImpactReport,
    ExecutionPlan,
    CodeDraft,
    Patch,
    ReviewReport,
    TestReport,
    Markdown,
    Html,
    Json,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalLevel {
    P0,
    P1,
    P2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatchKind {
    Add,
    Modify,
    Delete,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatchStatus {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunArtifact {
    pub artifact_id: String,
    pub kind: ArtifactKind,
    pub title: String,
    pub relative_path: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunStep {
    pub step_id: String,
    pub parent_step_id: Option<String>,
    pub title: String,
    pub agent: Option<String>,
    pub status: StepStatus,
    pub summary: Option<String>,
    pub token_used: u64,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl RunStep {
    pub fn new(
        step_id: impl Into<String>,
        title: impl Into<String>,
        parent_step_id: Option<String>,
        agent: Option<String>,
    ) -> Self {
        Self {
            step_id: step_id.into(),
            parent_step_id,
            title: title.into(),
            agent,
            status: StepStatus::Pending,
            summary: None,
            token_used: 0,
            started_at: None,
            finished_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunApproval {
    pub approval_id: String,
    pub step_id: String,
    pub title: String,
    pub reason: String,
    pub level: ApprovalLevel,
    pub options: Vec<String>,
    pub recommended: Option<String>,
    pub status: ApprovalStatus,
    pub choice: Option<String>,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchHunk {
    pub hunk_id: String,
    pub header: String,
    pub body: String,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub status: PatchStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunPatch {
    pub patch_id: String,
    pub step_id: String,
    pub file_path: String,
    pub kind: PatchKind,
    pub rationale: String,
    pub preview: String,
    pub acceptance_checks: Vec<String>,
    pub hunks: Vec<PatchHunk>,
    pub status: PatchStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunCommand {
    pub command_id: String,
    pub step_id: String,
    pub title: String,
    pub command: String,
    pub cwd: String,
    pub status: CommandStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub stdout_tail: String,
    pub stderr_tail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunEvent {
    RunStarted {
        run_id: String,
        session_id: String,
        request: String,
        project_root: String,
    },
    StepStarted {
        step: RunStep,
    },
    StepFinished {
        step_id: String,
        status: StepStatus,
        summary: Option<String>,
        token_used: u64,
    },
    Message {
        step_id: Option<String>,
        level: MessageLevel,
        message: String,
    },
    ApprovalRequested {
        approval: RunApproval,
    },
    ApprovalResolved {
        approval_id: String,
        status: ApprovalStatus,
        choice: Option<String>,
        comment: Option<String>,
    },
    PatchProposed {
        patch: RunPatch,
    },
    PatchResolved {
        patch_id: String,
        hunk_id: Option<String>,
        status: PatchStatus,
    },
    ArtifactWritten {
        artifact: RunArtifact,
    },
    CommandStarted {
        command: RunCommand,
    },
    CommandOutput {
        command_id: String,
        stream: OutputStream,
        chunk: String,
    },
    CommandFinished {
        command_id: String,
        status: CommandStatus,
        exit_code: Option<i32>,
        stdout_tail: String,
        stderr_tail: String,
    },
    RunFinished {
        status: RunStatus,
        summary: Option<String>,
        total_tokens: u64,
        total_duration_ms: u64,
        review_verdict: Option<String>,
        changed_files: Vec<String>,
    },
    Error {
        step_id: Option<String>,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunEventEnvelope {
    pub sequence: u64,
    pub at: DateTime<Utc>,
    pub event: RunEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunSummary {
    pub run_id: String,
    pub session_id: String,
    pub request: String,
    pub project_root: String,
    pub status: RunStatus,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub total_tokens: u64,
    pub total_duration_ms: Option<u64>,
    pub review_verdict: Option<String>,
    pub final_summary: Option<String>,
    pub changed_files: Vec<String>,
    pub steps: Vec<RunStep>,
    pub approvals: Vec<RunApproval>,
    pub patches: Vec<RunPatch>,
    pub artifacts: Vec<RunArtifact>,
    pub commands: Vec<RunCommand>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunSnapshot {
    pub summary: RunSummary,
    pub events: Vec<RunEventEnvelope>,
}

impl RunSnapshot {
    pub fn new(
        run_id: impl Into<String>,
        session_id: impl Into<String>,
        request: impl Into<String>,
        project_root: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            summary: RunSummary {
                run_id: run_id.into(),
                session_id: session_id.into(),
                request: request.into(),
                project_root: project_root.into(),
                status: RunStatus::Queued,
                started_at: now,
                updated_at: now,
                finished_at: None,
                total_tokens: 0,
                total_duration_ms: None,
                review_verdict: None,
                final_summary: None,
                changed_files: Vec::new(),
                steps: Vec::new(),
                approvals: Vec::new(),
                patches: Vec::new(),
                artifacts: Vec::new(),
                commands: Vec::new(),
                errors: Vec::new(),
            },
            events: Vec::new(),
        }
    }

    pub fn apply(&mut self, envelope: &RunEventEnvelope) {
        self.summary.updated_at = envelope.at;
        match &envelope.event {
            RunEvent::RunStarted {
                run_id,
                session_id,
                request,
                project_root,
            } => {
                self.summary.run_id = run_id.clone();
                self.summary.session_id = session_id.clone();
                self.summary.request = request.clone();
                self.summary.project_root = project_root.clone();
                self.summary.status = RunStatus::Running;
                self.summary.started_at = envelope.at;
            }
            RunEvent::StepStarted { step } => {
                let mut current = step.clone();
                current.status = StepStatus::Running;
                current.started_at = Some(envelope.at);
                upsert_step(&mut self.summary.steps, current);
            }
            RunEvent::StepFinished {
                step_id,
                status,
                summary,
                token_used,
            } => {
                let step = ensure_step(&mut self.summary.steps, step_id);
                step.status = *status;
                step.summary = summary.clone();
                step.token_used = *token_used;
                step.finished_at = Some(envelope.at);
                self.summary.total_tokens = self.summary.total_tokens.saturating_add(*token_used);
            }
            RunEvent::Message { .. } => {}
            RunEvent::ApprovalRequested { approval } => {
                let mut current = approval.clone();
                current.status = ApprovalStatus::Pending;
                upsert_approval(&mut self.summary.approvals, current);
                self.summary.status = RunStatus::WaitingApproval;
            }
            RunEvent::ApprovalResolved {
                approval_id,
                status,
                choice,
                comment,
            } => {
                if let Some(approval) = self
                    .summary
                    .approvals
                    .iter_mut()
                    .find(|item| item.approval_id == *approval_id)
                {
                    approval.status = *status;
                    approval.choice = choice.clone();
                    approval.comment = comment.clone();
                    approval.responded_at = Some(envelope.at);
                }
                if !self
                    .summary
                    .approvals
                    .iter()
                    .any(|item| item.status == ApprovalStatus::Pending)
                    && !self.summary.status.is_terminal()
                {
                    self.summary.status = RunStatus::Running;
                }
            }
            RunEvent::PatchProposed { patch } => {
                upsert_patch(&mut self.summary.patches, patch.clone());
            }
            RunEvent::PatchResolved {
                patch_id,
                hunk_id,
                status,
            } => {
                if let Some(patch) = self
                    .summary
                    .patches
                    .iter_mut()
                    .find(|item| item.patch_id == *patch_id)
                {
                    if let Some(hunk_id) = hunk_id {
                        if let Some(hunk) =
                            patch.hunks.iter_mut().find(|item| item.hunk_id == *hunk_id)
                        {
                            hunk.status = *status;
                        }
                        patch.status = summarize_patch_status(&patch.hunks);
                    } else {
                        patch.status = *status;
                        for hunk in &mut patch.hunks {
                            hunk.status = *status;
                        }
                    }
                }
                self.summary.changed_files = self
                    .summary
                    .patches
                    .iter()
                    .filter(|patch| patch.status == PatchStatus::Accepted)
                    .map(|patch| patch.file_path.clone())
                    .collect();
            }
            RunEvent::ArtifactWritten { artifact } => {
                if !self
                    .summary
                    .artifacts
                    .iter()
                    .any(|item| item.artifact_id == artifact.artifact_id)
                {
                    self.summary.artifacts.push(artifact.clone());
                }
            }
            RunEvent::CommandStarted { command } => {
                let mut current = command.clone();
                current.status = CommandStatus::Running;
                current.started_at = Some(envelope.at);
                upsert_command(&mut self.summary.commands, current);
            }
            RunEvent::CommandOutput {
                command_id,
                stream,
                chunk,
            } => {
                if let Some(command) = self
                    .summary
                    .commands
                    .iter_mut()
                    .find(|item| item.command_id == *command_id)
                {
                    let target = match stream {
                        OutputStream::Stdout => &mut command.stdout_tail,
                        OutputStream::Stderr => &mut command.stderr_tail,
                    };
                    push_tail(target, chunk);
                }
            }
            RunEvent::CommandFinished {
                command_id,
                status,
                exit_code,
                stdout_tail,
                stderr_tail,
            } => {
                if let Some(command) = self
                    .summary
                    .commands
                    .iter_mut()
                    .find(|item| item.command_id == *command_id)
                {
                    command.status = *status;
                    command.exit_code = *exit_code;
                    command.finished_at = Some(envelope.at);
                    if !stdout_tail.is_empty() {
                        command.stdout_tail = stdout_tail.clone();
                    }
                    if !stderr_tail.is_empty() {
                        command.stderr_tail = stderr_tail.clone();
                    }
                }
            }
            RunEvent::RunFinished {
                status,
                summary,
                total_tokens,
                total_duration_ms,
                review_verdict,
                changed_files,
            } => {
                self.summary.status = *status;
                self.summary.final_summary = summary.clone();
                self.summary.total_tokens = *total_tokens;
                self.summary.total_duration_ms = Some(*total_duration_ms);
                self.summary.review_verdict = review_verdict.clone();
                self.summary.changed_files = changed_files.clone();
                self.summary.finished_at = Some(envelope.at);
            }
            RunEvent::Error { message, .. } => {
                self.summary.errors.push(message.clone());
            }
        }
        self.events.push(envelope.clone());
    }
}

pub struct RunRecorder {
    run_dir: PathBuf,
    events_path: PathBuf,
    summary_path: PathBuf,
    sequence: u64,
    snapshot: RunSnapshot,
}

impl RunRecorder {
    pub fn create(project_root: &Path, request: impl Into<String>) -> io::Result<Self> {
        let run_id = format!("run-{}", Uuid::new_v4());
        let session_id = format!("session-{}", Uuid::new_v4());
        let request = request.into();
        let run_dir = project_root.join(".morecode").join("runs").join(&run_id);
        fs::create_dir_all(run_dir.join("artifacts"))?;

        let snapshot = RunSnapshot::new(
            run_id.clone(),
            session_id.clone(),
            request.clone(),
            project_root.to_string_lossy().into_owned(),
        );
        let mut recorder = Self {
            events_path: run_dir.join("events.jsonl"),
            summary_path: run_dir.join("summary.json"),
            run_dir,
            sequence: 0,
            snapshot,
        };
        recorder.persist_summary()?;
        recorder.emit(RunEvent::RunStarted {
            run_id,
            session_id,
            request,
            project_root: project_root.to_string_lossy().into_owned(),
        })?;
        Ok(recorder)
    }

    pub fn open(project_root: &Path, run_id: &str) -> io::Result<Self> {
        Self::open_dir(project_root.join(".morecode").join("runs").join(run_id))
    }

    pub fn open_dir(run_dir: PathBuf) -> io::Result<Self> {
        let events_path = run_dir.join("events.jsonl");
        let summary_path = run_dir.join("summary.json");
        let snapshot = load_snapshot_from_paths(&summary_path, &events_path)?;
        let sequence = snapshot
            .events
            .last()
            .map(|event| event.sequence)
            .unwrap_or(0);

        Ok(Self {
            run_dir,
            events_path,
            summary_path,
            sequence,
            snapshot,
        })
    }

    pub fn emit(&mut self, event: RunEvent) -> io::Result<RunEventEnvelope> {
        self.refresh_from_disk()?;
        self.sequence = self.sequence.saturating_add(1);
        let envelope = RunEventEnvelope {
            sequence: self.sequence,
            at: Utc::now(),
            event,
        };
        append_json_line(&self.events_path, &envelope)?;
        self.snapshot.apply(&envelope);
        self.persist_summary()?;
        Ok(envelope)
    }

    pub fn write_text_artifact(
        &mut self,
        file_name: impl AsRef<Path>,
        title: impl Into<String>,
        kind: ArtifactKind,
        contents: &str,
        description: Option<String>,
    ) -> io::Result<RunArtifact> {
        let relative_path = PathBuf::from("artifacts").join(file_name.as_ref());
        let absolute_path = self.run_dir.join(&relative_path);
        if let Some(parent) = absolute_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&absolute_path, contents)?;
        let artifact = RunArtifact {
            artifact_id: format!("artifact-{}", Uuid::new_v4()),
            kind,
            title: title.into(),
            relative_path: relative_path.to_string_lossy().into_owned(),
            description,
            created_at: Utc::now(),
        };
        let _ = self.emit(RunEvent::ArtifactWritten {
            artifact: artifact.clone(),
        })?;
        Ok(artifact)
    }

    pub fn write_json_artifact<T: Serialize>(
        &mut self,
        file_name: impl AsRef<Path>,
        title: impl Into<String>,
        kind: ArtifactKind,
        value: &T,
        description: Option<String>,
    ) -> io::Result<RunArtifact> {
        let contents = serde_json::to_string_pretty(value).map_err(json_to_io)?;
        self.write_text_artifact(file_name, title, kind, &contents, description)
    }

    pub fn run_dir(&self) -> &Path {
        &self.run_dir
    }

    pub fn snapshot(&self) -> &RunSnapshot {
        &self.snapshot
    }

    fn refresh_from_disk(&mut self) -> io::Result<()> {
        if self.summary_path.exists() {
            self.snapshot = load_snapshot_from_paths(&self.summary_path, &self.events_path)?;
            self.sequence = self
                .snapshot
                .events
                .last()
                .map(|event| event.sequence)
                .unwrap_or(0);
        }
        Ok(())
    }

    fn persist_summary(&self) -> io::Result<()> {
        let contents = serde_json::to_string_pretty(&self.snapshot.summary).map_err(json_to_io)?;
        fs::write(&self.summary_path, contents)
    }
}

#[derive(Debug, Clone)]
pub struct RunStore {
    root: PathBuf,
}

impl RunStore {
    pub fn new(project_root: &Path) -> Self {
        Self {
            root: project_root.join(".morecode").join("runs"),
        }
    }

    pub fn runs_root(&self) -> &Path {
        &self.root
    }

    pub fn run_dir(&self, run_id: &str) -> PathBuf {
        self.root.join(run_id)
    }

    pub fn list_summaries(&self) -> io::Result<Vec<RunSummary>> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }

        let mut runs = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let path = entry.path().join("summary.json");
            if !path.exists() {
                continue;
            }
            let contents = fs::read_to_string(path)?;
            let summary = serde_json::from_str::<RunSummary>(&contents).map_err(json_to_io)?;
            runs.push(summary);
        }
        runs.sort_by(|left, right| right.started_at.cmp(&left.started_at));
        Ok(runs)
    }

    pub fn load_snapshot(&self, run_id: &str) -> io::Result<RunSnapshot> {
        let run_dir = self.run_dir(run_id);
        let summary_path = run_dir.join("summary.json");
        let events_path = run_dir.join("events.jsonl");
        load_snapshot_from_paths(&summary_path, &events_path)
    }

    pub fn open_recorder(&self, run_id: &str) -> io::Result<RunRecorder> {
        RunRecorder::open_dir(self.run_dir(run_id))
    }
}

pub fn build_patch_hunks(preview: &str) -> Vec<PatchHunk> {
    if preview.trim().is_empty() {
        return Vec::new();
    }

    let lines = preview.lines().collect::<Vec<_>>();
    let has_explicit_hunks = lines.iter().any(|line| line.starts_with("@@"));
    if !has_explicit_hunks {
        return vec![build_hunk(
            "hunk-1".to_string(),
            "@@".to_string(),
            preview.to_string(),
        )];
    }

    let mut hunks = Vec::new();
    let mut current_header: Option<String> = None;
    let mut current_lines = Vec::new();

    for line in lines {
        if line.starts_with("@@") {
            if let Some(header) = current_header.take() {
                let body = current_lines.join("\n");
                hunks.push(build_hunk(
                    format!("hunk-{}", hunks.len() + 1),
                    header,
                    body,
                ));
                current_lines.clear();
            }
            current_header = Some(line.to_string());
            continue;
        }

        if current_header.is_some() {
            current_lines.push(line.to_string());
        }
    }

    if let Some(header) = current_header {
        let body = current_lines.join("\n");
        hunks.push(build_hunk(
            format!("hunk-{}", hunks.len() + 1),
            header,
            body,
        ));
    }

    hunks
}

fn build_hunk(hunk_id: String, header: String, body: String) -> PatchHunk {
    let (added_lines, removed_lines) = count_patch_lines(&body);
    PatchHunk {
        hunk_id,
        header,
        body,
        added_lines,
        removed_lines,
        status: PatchStatus::Pending,
    }
}

fn count_patch_lines(body: &str) -> (usize, usize) {
    let mut added = 0usize;
    let mut removed = 0usize;
    for line in body.lines() {
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }
        if line.starts_with('+') {
            added += 1;
        } else if line.starts_with('-') {
            removed += 1;
        }
    }
    (added, removed)
}

fn append_json_line<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(value).map_err(json_to_io)?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    file.flush()
}

fn load_snapshot_from_paths(summary_path: &Path, events_path: &Path) -> io::Result<RunSnapshot> {
    let summary_contents = fs::read_to_string(summary_path)?;
    let summary = serde_json::from_str::<RunSummary>(&summary_contents).map_err(json_to_io)?;

    let mut events = Vec::new();
    if events_path.exists() {
        let file = fs::File::open(events_path)?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let envelope = serde_json::from_str::<RunEventEnvelope>(&line).map_err(json_to_io)?;
            events.push(envelope);
        }
    }

    Ok(RunSnapshot { summary, events })
}

fn json_to_io(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

fn upsert_step(steps: &mut Vec<RunStep>, step: RunStep) {
    if let Some(index) = steps.iter().position(|item| item.step_id == step.step_id) {
        steps[index] = step;
    } else {
        steps.push(step);
    }
}

fn ensure_step<'a>(steps: &'a mut Vec<RunStep>, step_id: &str) -> &'a mut RunStep {
    if let Some(index) = steps.iter().position(|item| item.step_id == step_id) {
        return &mut steps[index];
    }
    steps.push(RunStep::new(
        step_id.to_string(),
        step_id.to_string(),
        None,
        None,
    ));
    let index = steps.len() - 1;
    &mut steps[index]
}

fn upsert_approval(approvals: &mut Vec<RunApproval>, approval: RunApproval) {
    if let Some(index) = approvals
        .iter()
        .position(|item| item.approval_id == approval.approval_id)
    {
        approvals[index] = approval;
    } else {
        approvals.push(approval);
    }
}

fn upsert_patch(patches: &mut Vec<RunPatch>, patch: RunPatch) {
    if let Some(index) = patches
        .iter()
        .position(|item| item.patch_id == patch.patch_id)
    {
        patches[index] = patch;
    } else {
        patches.push(patch);
    }
}

fn upsert_command(commands: &mut Vec<RunCommand>, command: RunCommand) {
    if let Some(index) = commands
        .iter()
        .position(|item| item.command_id == command.command_id)
    {
        commands[index] = command;
    } else {
        commands.push(command);
    }
}

fn summarize_patch_status(hunks: &[PatchHunk]) -> PatchStatus {
    if hunks.is_empty() {
        return PatchStatus::Pending;
    }
    if hunks
        .iter()
        .all(|item| item.status == PatchStatus::Accepted)
    {
        PatchStatus::Accepted
    } else if hunks
        .iter()
        .all(|item| item.status == PatchStatus::Rejected)
    {
        PatchStatus::Rejected
    } else {
        PatchStatus::Pending
    }
}

fn push_tail(target: &mut String, chunk: &str) {
    if chunk.trim().is_empty() {
        return;
    }
    if !target.is_empty() {
        target.push('\n');
    }
    target.push_str(chunk.trim());
    // Keep only the last 40 lines efficiently
    let mut newline_count = 0usize;
    let mut truncate_at = target.len();
    for (i, ch) in target.char_indices().rev() {
        if ch == '\n' {
            newline_count += 1;
            if newline_count >= 40 {
                truncate_at = i;
                break;
            }
        }
    }
    if truncate_at < target.len() {
        *target = target[truncate_at + 1..].to_string();
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{
        build_patch_hunks, ApprovalLevel, ApprovalStatus, ArtifactKind, PatchKind, PatchStatus,
        RunApproval, RunEvent, RunPatch, RunRecorder, RunStatus, RunStep, RunStore, StepStatus,
    };

    #[test]
    fn recorder_roundtrip_persists_summary_and_events() {
        let temp = tempdir().expect("tempdir");
        let mut recorder = RunRecorder::create(temp.path(), "ship the UI").expect("recorder");

        recorder
            .emit(RunEvent::StepStarted {
                step: RunStep::new("plan", "Plan", None, Some("Planner".to_string())),
            })
            .expect("step started");
        recorder
            .emit(RunEvent::StepFinished {
                step_id: "plan".to_string(),
                status: StepStatus::Done,
                summary: Some("plan ready".to_string()),
                token_used: 32,
            })
            .expect("step finished");

        let patch = RunPatch {
            patch_id: "patch-1".to_string(),
            step_id: "review".to_string(),
            file_path: "src/lib.rs".to_string(),
            kind: PatchKind::Modify,
            rationale: "Update API".to_string(),
            preview: "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@\n-old\n+new".to_string(),
            acceptance_checks: vec!["cargo test".to_string()],
            hunks: build_patch_hunks("--- a/src/lib.rs\n+++ b/src/lib.rs\n@@\n-old\n+new"),
            status: PatchStatus::Pending,
        };
        recorder
            .emit(RunEvent::PatchProposed { patch })
            .expect("patch proposed");

        let approval = RunApproval {
            approval_id: "approval-1".to_string(),
            step_id: "test".to_string(),
            title: "Run tests".to_string(),
            reason: "cargo test touches the workspace".to_string(),
            level: ApprovalLevel::P1,
            options: vec!["approve".to_string(), "reject".to_string()],
            recommended: Some("approve".to_string()),
            status: ApprovalStatus::Pending,
            choice: None,
            comment: None,
            created_at: chrono::Utc::now(),
            responded_at: None,
        };
        recorder
            .emit(RunEvent::ApprovalRequested { approval })
            .expect("approval");

        recorder
            .write_text_artifact(
                "notes.md",
                "Run Notes",
                ArtifactKind::Markdown,
                "# Notes",
                None,
            )
            .expect("artifact");

        recorder
            .emit(RunEvent::RunFinished {
                status: RunStatus::Succeeded,
                summary: Some("done".to_string()),
                total_tokens: 64,
                total_duration_ms: 500,
                review_verdict: Some("approved".to_string()),
                changed_files: vec!["src/lib.rs".to_string()],
            })
            .expect("run finished");

        let store = RunStore::new(temp.path());
        let run_id = recorder.snapshot().summary.run_id.clone();
        let snapshot = store.load_snapshot(&run_id).expect("snapshot");

        assert_eq!(snapshot.summary.status, RunStatus::Succeeded);
        assert_eq!(snapshot.summary.steps.len(), 1);
        assert_eq!(snapshot.summary.patches.len(), 1);
        assert_eq!(snapshot.summary.approvals.len(), 1);
        assert_eq!(snapshot.summary.artifacts.len(), 1);
        assert!(!snapshot.events.is_empty());
    }

    #[test]
    fn recorder_can_reopen_and_append_events() {
        let temp = tempdir().expect("tempdir");
        let mut recorder = RunRecorder::create(temp.path(), "review patches").expect("recorder");
        let run_id = recorder.snapshot().summary.run_id.clone();
        recorder
            .emit(RunEvent::RunFinished {
                status: RunStatus::Succeeded,
                summary: Some("done".to_string()),
                total_tokens: 12,
                total_duration_ms: 30,
                review_verdict: None,
                changed_files: Vec::new(),
            })
            .expect("finish");

        let store = RunStore::new(temp.path());
        let mut reopened = store.open_recorder(&run_id).expect("open recorder");
        reopened
            .emit(RunEvent::Message {
                step_id: None,
                level: super::MessageLevel::Info,
                message: "post-review note".to_string(),
            })
            .expect("append event");

        let snapshot = store.load_snapshot(&run_id).expect("snapshot");
        assert_eq!(snapshot.events.len(), reopened.snapshot().events.len());
        assert!(snapshot
            .events
            .last()
            .is_some_and(|event| matches!(&event.event, RunEvent::Message { message, .. } if message == "post-review note")));
    }

    #[test]
    fn patch_hunks_split_explicit_sections() {
        let hunks = build_patch_hunks(
            "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-old\n+new\n@@ -4 +4 @@\n-left\n+right",
        );
        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[0].added_lines, 1);
        assert_eq!(hunks[1].removed_lines, 1);
    }
}
