use std::collections::{BTreeMap, VecDeque};
use std::fmt;

use chrono::{DateTime, Utc};
use mc_communication::{
    ApprovalRequest, ApprovalResponse, BroadcastEvent, ControlMessage, StateMessage,
};
use mc_core::{AgentExecutionStatus, AgentType};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Tabs};
use ratatui::Frame;

use crate::error::TuiError;
use crate::event::{AppEvent, KeyAction, LogLevel, TuiUpdate};
use crate::i18n::{text, Language, TextKey};
use crate::theme::TuiTheme;
use crate::view;

const MAX_LOG_ENTRIES: usize = 200;
const MAX_CODE_STREAM_ENTRIES: usize = 80;
const MAX_CONFIRMATIONS: usize = 32;
const MAX_TOKEN_HISTORY: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Panel {
    Dashboard,
    AgentStatus,
    TaskProgress,
    Communication,
    TokenUsage,
    Log,
    Help,
}

impl Panel {
    pub const ALL: [Self; 7] = [
        Self::Dashboard,
        Self::AgentStatus,
        Self::TaskProgress,
        Self::Communication,
        Self::TokenUsage,
        Self::Log,
        Self::Help,
    ];

    pub fn title(self, lang: Language) -> &'static str {
        match self {
            Self::Dashboard => text(lang, TextKey::PanelDashboard),
            Self::AgentStatus => text(lang, TextKey::PanelAgents),
            Self::TaskProgress => text(lang, TextKey::PanelProgress),
            Self::Communication => text(lang, TextKey::PanelTopology),
            Self::TokenUsage => text(lang, TextKey::PanelTokens),
            Self::Log => text(lang, TextKey::PanelLogs),
            Self::Help => text(lang, TextKey::PanelHelp),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StreamMode {
    Progress,
    Code,
    Confirmation,
}

impl StreamMode {
    pub const ALL: [Self; 3] = [Self::Progress, Self::Code, Self::Confirmation];

    pub fn title(self, lang: Language) -> &'static str {
        match self {
            Self::Progress => text(lang, TextKey::StreamProgress),
            Self::Code => text(lang, TextKey::StreamCode),
            Self::Confirmation => text(lang, TextKey::StreamConfirmation),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Endpoint {
    Coordinator,
    Agent(AgentType),
    User,
    Ui,
    System,
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Coordinator => f.write_str("Coordinator"),
            Self::Agent(agent) => write!(f, "{agent}"),
            Self::User => f.write_str("User"),
            Self::Ui => f.write_str("UI"),
            Self::System => f.write_str("System"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommunicationKind {
    Control,
    State,
    Data,
    Approval,
    Broadcast,
}

impl fmt::Display for CommunicationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Control => f.write_str("control"),
            Self::State => f.write_str("state"),
            Self::Data => f.write_str("data"),
            Self::Approval => f.write_str("approval"),
            Self::Broadcast => f.write_str("broadcast"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSnapshot {
    pub agent_type: AgentType,
    pub status: AgentExecutionStatus,
    pub task_id: Option<String>,
    pub phase: Option<String>,
    pub detail: String,
    pub progress_percent: u8,
    pub token_used: u64,
    pub token_budget: Option<u64>,
    pub last_update: DateTime<Utc>,
}

impl AgentSnapshot {
    fn pending(agent_type: AgentType) -> Self {
        Self {
            agent_type,
            status: AgentExecutionStatus::Pending,
            task_id: None,
            phase: None,
            detail: "Idle".to_string(),
            progress_percent: 0,
            token_used: 0,
            token_budget: None,
            last_update: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskProgressEntry {
    pub task_id: String,
    pub agent_type: AgentType,
    pub status: AgentExecutionStatus,
    pub phase: String,
    pub progress_percent: u8,
    pub summary: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommunicationEdge {
    pub from: Endpoint,
    pub to: Endpoint,
    pub kind: CommunicationKind,
    pub count: u64,
    pub last_summary: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeStreamKind {
    Handoff,
    PartialResult,
    StreamChunk,
}

impl fmt::Display for CodeStreamKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Handoff => f.write_str("handoff"),
            Self::PartialResult => f.write_str("partial"),
            Self::StreamChunk => f.write_str("chunk"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeStreamEntry {
    pub kind: CodeStreamKind,
    pub from: Endpoint,
    pub to: Endpoint,
    pub task_id: String,
    pub content: String,
    pub sequence: Option<u32>,
    pub is_terminal: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationStatus {
    Pending,
    Approved,
    Rejected,
}

impl fmt::Display for ConfirmationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => f.write_str("pending"),
            Self::Approved => f.write_str("approved"),
            Self::Rejected => f.write_str("rejected"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationEntry {
    pub request_id: String,
    pub task_id: String,
    pub agent_label: String,
    pub reason: String,
    pub recommendation: Option<String>,
    pub status: ConfirmationStatus,
    pub choice: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub(crate) active_panel: Panel,
    pub(crate) stream_mode: StreamMode,
    pub(crate) language: Language,
    pub(crate) title: String,
    pub(crate) should_quit: bool,
    pub(crate) terminal_size: (u16, u16),
    pub(crate) agents: Vec<AgentSnapshot>,
    pub(crate) tasks: Vec<TaskProgressEntry>,
    pub(crate) communication_edges: Vec<CommunicationEdge>,
    pub(crate) code_stream: VecDeque<CodeStreamEntry>,
    pub(crate) confirmations: VecDeque<ConfirmationEntry>,
    pub(crate) logs: VecDeque<UiLogEntry>,
    pub(crate) token_total: u64,
    pub(crate) token_history: VecDeque<u64>,
    pub(crate) scroll_offsets: BTreeMap<Panel, u16>,
}

#[derive(Debug, Clone)]
pub struct App {
    state: AppState,
    theme: TuiTheme,
}

impl App {
    pub fn new() -> Self {
        Self::with_title("MoreCode")
    }

    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            state: AppState::new(title),
            theme: TuiTheme::default(),
        }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    pub fn handle_event(&mut self, event: AppEvent) -> Result<(), TuiError> {
        match event {
            AppEvent::Tick => Ok(()),
            AppEvent::Resize { width, height } => {
                if width == 0 || height == 0 {
                    return Err(TuiError::InvalidLayout(
                        "terminal size must be greater than zero".to_string(),
                    ));
                }
                self.state.terminal_size = (width, height);
                Ok(())
            }
            AppEvent::Key(KeyAction::NextPanel) => {
                self.state.active_panel = next_panel(self.state.active_panel);
                Ok(())
            }
            AppEvent::Key(KeyAction::PreviousPanel) => {
                self.state.active_panel = previous_panel(self.state.active_panel);
                Ok(())
            }
            AppEvent::Key(KeyAction::NextMode) => {
                self.state.stream_mode = next_stream_mode(self.state.stream_mode);
                Ok(())
            }
            AppEvent::Key(KeyAction::PreviousMode) => {
                self.state.stream_mode = previous_stream_mode(self.state.stream_mode);
                Ok(())
            }
            AppEvent::Key(KeyAction::SetStreamMode(mode)) => {
                self.state.stream_mode = mode;
                Ok(())
            }
            AppEvent::Key(KeyAction::ScrollUp) => {
                self.adjust_scroll(-1);
                Ok(())
            }
            AppEvent::Key(KeyAction::ScrollDown) => {
                self.adjust_scroll(1);
                Ok(())
            }
            AppEvent::Key(KeyAction::ToggleLanguage) => {
                self.state.language = self.state.language.toggle();
                self.push_log(LogLevel::Info, format!("language: {:?}", self.state.language));
                Ok(())
            }
            AppEvent::Key(KeyAction::Help) => {
                self.state.active_panel = Panel::Help;
                Ok(())
            }
            AppEvent::Key(KeyAction::Quit) => {
                self.state.should_quit = true;
                Ok(())
            }
            AppEvent::Update(update) => self.apply_update(*update),
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Block::default().style(self.theme.background_style()), area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(2),
            ])
            .split(area);
        self.render_header(frame, layout[0]);
        view::render_active_panel(frame, layout[1], &self.state, self.theme);
        self.render_footer(frame, layout[2]);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let lang = self.state.language;
        let titles = Panel::ALL
            .into_iter()
            .map(|panel| Line::from(panel.title(lang)))
            .collect::<Vec<_>>();
        let title = Line::from(vec![
            Span::styled(
                self.state.title.clone(),
                self.theme.text().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("stream: {}", self.state.stream_mode.title(lang)),
                self.theme.accent(),
            ),
        ]);
        let tabs = Tabs::new(titles)
            .select(active_panel_index(self.state.active_panel))
            .block(self.theme.panel_block(title, true))
            .style(self.theme.muted())
            .highlight_style(self.theme.accent().add_modifier(Modifier::BOLD))
            .divider(" | ");
        frame.render_widget(tabs, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let lang = self.state.language;
        let progress = self.state.overall_progress();
        let token_summary = match self.state.token_budget_total() {
            Some(budget) => format!("tokens {}/{}", self.state.token_total, budget),
            None => format!("tokens {}", self.state.token_total),
        };
        let mut spans = vec![
            Span::styled("Tab", self.theme.accent()),
            Span::raw(text(lang, TextKey::FooterNext)),
            Span::styled("Shift+Tab", self.theme.accent()),
            Span::raw(text(lang, TextKey::FooterPrev)),
            Span::styled("1/2/3", self.theme.accent()),
            Span::raw(text(lang, TextKey::FooterStreams)),
            Span::styled("j/k", self.theme.accent()),
            Span::raw(text(lang, TextKey::FooterScroll)),
            Span::styled("?/q", self.theme.accent()),
            Span::raw(text(lang, TextKey::FooterHelpQuit)),
            Span::styled(
                format!("{} {progress}%", text(lang, TextKey::FooterProgress)),
                self.theme.text(),
            ),
            Span::raw("  "),
        ];
        let token_style = if self.state.has_budget_warning() {
            self.theme.warning()
        } else {
            self.theme.text()
        };
        spans.push(Span::styled(token_summary, token_style));

        let footer = Paragraph::new(Line::from(spans))
            .block(self.theme.panel_block(text(lang, TextKey::FooterControls), false))
            .style(self.theme.muted());
        frame.render_widget(footer, area);
    }

    fn apply_update(&mut self, update: TuiUpdate) -> Result<(), TuiError> {
        match update {
            TuiUpdate::Control(message) => self.apply_control_message(message),
            TuiUpdate::State(message) => self.apply_state_message(message),
            TuiUpdate::Broadcast(event) => self.apply_broadcast_event(event),
            TuiUpdate::ApprovalRequest(request) => self.apply_approval_request(request),
            TuiUpdate::ApprovalResponse(response) => self.apply_approval_response(response),
            TuiUpdate::Log { level, message } => {
                self.push_log(level, message);
                Ok(())
            }
        }
    }

    fn apply_control_message(&mut self, message: ControlMessage) -> Result<(), TuiError> {
        match message {
            ControlMessage::TaskAssigned {
                task_id,
                agent_type,
                task,
                token_budget,
                ..
            } => {
                let agent = self.agent_mut(agent_type)?;
                agent.status = AgentExecutionStatus::Running;
                agent.task_id = Some(task_id.clone());
                agent.phase = Some("assigned".to_string());
                agent.detail = task.description.clone();
                agent.progress_percent = 0;
                agent.token_budget = Some(token_budget);
                agent.last_update = Utc::now();

                let progress = self.upsert_task(&task_id, agent_type);
                progress.status = AgentExecutionStatus::Running;
                progress.phase = "assigned".to_string();
                progress.progress_percent = 0;
                progress.summary = task.expected_output.clone();
                progress.updated_at = Utc::now();

                self.record_edge(
                    Endpoint::Coordinator,
                    Endpoint::Agent(agent_type),
                    CommunicationKind::Control,
                    format!("assigned {}", task.description),
                );
                self.push_log(
                    LogLevel::Info,
                    format!("Coordinator assigned {agent_type}: {}", task.description),
                );
                Ok(())
            }
            ControlMessage::Cancel { task_id, reason } => {
                let matched_agents = self
                    .state
                    .agents
                    .iter_mut()
                    .filter(|agent| agent.task_id.as_deref() == Some(task_id.as_str()))
                    .map(|agent| {
                        agent.status = AgentExecutionStatus::Cancelled;
                        agent.phase = Some("cancelled".to_string());
                        agent.detail = reason.clone();
                        agent.last_update = Utc::now();
                        agent.agent_type
                    })
                    .collect::<Vec<_>>();

                for task in self
                    .state
                    .tasks
                    .iter_mut()
                    .filter(|task| task.task_id == task_id)
                {
                    task.status = AgentExecutionStatus::Cancelled;
                    task.summary = reason.clone();
                    task.updated_at = Utc::now();
                }

                for agent_type in matched_agents {
                    self.record_edge(
                        Endpoint::Coordinator,
                        Endpoint::Agent(agent_type),
                        CommunicationKind::Control,
                        format!("cancelled {task_id}"),
                    );
                }

                self.push_log(LogLevel::Warn, format!("Cancelled {task_id}: {reason}"));
                Ok(())
            }
            ControlMessage::ApprovalRequired {
                task_id,
                agent_type,
                reason,
                recommendation,
                ..
            } => {
                self.record_edge(
                    Endpoint::Agent(agent_type),
                    Endpoint::User,
                    CommunicationKind::Approval,
                    format!("approval for {task_id}"),
                );
                self.push_log(
                    LogLevel::Warn,
                    format!(
                        "{agent_type} requested approval: {}{}",
                        reason,
                        recommendation
                            .as_ref()
                            .map(|value| format!(" (recommended: {value})"))
                            .unwrap_or_default()
                    ),
                );
                Ok(())
            }
            ControlMessage::CollaborationRequest {
                from_agent,
                to_agent,
                request_type,
                ..
            } => {
                self.record_edge(
                    Endpoint::Agent(from_agent),
                    Endpoint::Agent(to_agent),
                    CommunicationKind::Control,
                    request_type.clone(),
                );
                self.push_log(
                    LogLevel::Debug,
                    format!("{from_agent} requested {request_type} from {to_agent}"),
                );
                Ok(())
            }
        }
    }

    fn apply_state_message(&mut self, message: StateMessage) -> Result<(), TuiError> {
        match message {
            StateMessage::Progress {
                task_id,
                agent_type,
                phase,
                progress_percent,
                message,
            } => {
                let progress_percent = progress_percent.min(100);
                let summary = if message.is_empty() {
                    phase.clone()
                } else {
                    message.clone()
                };

                let agent = self.agent_mut(agent_type)?;
                agent.status = AgentExecutionStatus::Running;
                agent.task_id = Some(task_id.clone());
                agent.phase = Some(phase.clone());
                agent.detail = summary.clone();
                agent.progress_percent = progress_percent;
                agent.last_update = Utc::now();

                let progress = self.upsert_task(&task_id, agent_type);
                progress.status = AgentExecutionStatus::Running;
                progress.phase = phase.clone();
                progress.progress_percent = progress_percent;
                progress.summary = summary.clone();
                progress.updated_at = Utc::now();

                self.record_edge(
                    Endpoint::Agent(agent_type),
                    Endpoint::Coordinator,
                    CommunicationKind::State,
                    format!("{phase} {progress_percent}%"),
                );
                self.push_log(
                    LogLevel::Info,
                    format!("{agent_type} {phase} {progress_percent}% - {summary}"),
                );
                Ok(())
            }
            StateMessage::TaskCompleted {
                task_id,
                agent_type,
                result,
                handoff,
                token_used,
            } => {
                let summary = result
                    .generated_content
                    .clone()
                    .filter(|content| !content.trim().is_empty())
                    .unwrap_or_else(|| handoff.title.clone());

                let agent = self.agent_mut(agent_type)?;
                agent.status = AgentExecutionStatus::Completed;
                agent.task_id = Some(task_id.clone());
                agent.phase = Some("completed".to_string());
                agent.detail = summary;
                agent.progress_percent = 100;
                agent.token_used = agent.token_used.saturating_add(token_used);
                agent.last_update = Utc::now();

                let progress = self.upsert_task(&task_id, agent_type);
                progress.status = AgentExecutionStatus::Completed;
                progress.phase = "completed".to_string();
                progress.progress_percent = 100;
                progress.summary = handoff.title.clone();
                progress.updated_at = Utc::now();

                self.state.token_total = self.state.token_total.saturating_add(token_used);
                self.push_token_sample(self.state.token_total);
                self.record_edge(
                    Endpoint::Agent(agent_type),
                    Endpoint::Coordinator,
                    CommunicationKind::State,
                    format!("completed {task_id}"),
                );
                self.push_log(
                    LogLevel::Info,
                    format!("{agent_type} completed {task_id} (+{token_used} tokens)"),
                );
                Ok(())
            }
            StateMessage::TaskFailed {
                task_id,
                agent_type,
                error,
                retry_count,
                can_retry,
            } => {
                let detail = if can_retry {
                    format!("{error} (retry {retry_count})")
                } else {
                    error
                };

                let agent = self.agent_mut(agent_type)?;
                agent.status = AgentExecutionStatus::Failed;
                agent.task_id = Some(task_id.clone());
                agent.phase = Some("failed".to_string());
                agent.detail = detail.clone();
                agent.last_update = Utc::now();

                let progress = self.upsert_task(&task_id, agent_type);
                progress.status = AgentExecutionStatus::Failed;
                progress.phase = "failed".to_string();
                progress.summary = detail.clone();
                progress.updated_at = Utc::now();

                self.record_edge(
                    Endpoint::Agent(agent_type),
                    Endpoint::Coordinator,
                    CommunicationKind::State,
                    format!("failed {task_id}"),
                );
                self.push_log(
                    LogLevel::Error,
                    format!("{agent_type} failed {task_id}: {detail}"),
                );
                Ok(())
            }
            StateMessage::Handoff {
                task_id,
                from_agent,
                to_agent,
                handoff,
            } => {
                self.record_edge(
                    Endpoint::Agent(from_agent),
                    Endpoint::Agent(to_agent),
                    CommunicationKind::Data,
                    handoff.title.clone(),
                );
                self.push_code_stream(CodeStreamEntry {
                    kind: CodeStreamKind::Handoff,
                    from: Endpoint::Agent(from_agent),
                    to: Endpoint::Agent(to_agent),
                    task_id,
                    content: format!(
                        "{} | focus: {}",
                        handoff.title,
                        join_or_dash(&handoff.recommendations)
                    ),
                    sequence: None,
                    is_terminal: true,
                    created_at: Utc::now(),
                });
                Ok(())
            }
            StateMessage::PartialResult {
                task_id,
                from_agent,
                to_agent,
                payload,
            } => {
                self.record_edge(
                    Endpoint::Agent(from_agent),
                    Endpoint::Agent(to_agent),
                    CommunicationKind::Data,
                    "partial result".to_string(),
                );
                self.push_code_stream(CodeStreamEntry {
                    kind: CodeStreamKind::PartialResult,
                    from: Endpoint::Agent(from_agent),
                    to: Endpoint::Agent(to_agent),
                    task_id,
                    content: compact_json(&payload),
                    sequence: None,
                    is_terminal: false,
                    created_at: Utc::now(),
                });
                Ok(())
            }
            StateMessage::StreamChunk {
                task_id,
                from_agent,
                to_agent,
                sequence,
                payload,
                is_last,
            } => {
                self.record_edge(
                    Endpoint::Agent(from_agent),
                    Endpoint::Agent(to_agent),
                    CommunicationKind::Data,
                    format!("chunk #{sequence}"),
                );
                self.push_code_stream(CodeStreamEntry {
                    kind: CodeStreamKind::StreamChunk,
                    from: Endpoint::Agent(from_agent),
                    to: Endpoint::Agent(to_agent),
                    task_id,
                    content: payload,
                    sequence: Some(sequence),
                    is_terminal: is_last,
                    created_at: Utc::now(),
                });
                Ok(())
            }
        }
    }

    fn apply_broadcast_event(&mut self, event: BroadcastEvent) -> Result<(), TuiError> {
        match event {
            BroadcastEvent::ProgressSnapshot {
                task_id,
                agent_type,
                progress_percent,
                summary,
            } => {
                let progress_percent = progress_percent.min(100);
                let agent = self.agent_mut(agent_type)?;
                if agent.status == AgentExecutionStatus::Pending {
                    agent.status = AgentExecutionStatus::Running;
                }
                agent.task_id = Some(task_id.clone());
                agent.detail = summary.clone();
                agent.progress_percent = progress_percent;
                agent.last_update = Utc::now();

                let progress = self.upsert_task(&task_id, agent_type);
                progress.progress_percent = progress_percent;
                progress.summary = summary;
                progress.updated_at = Utc::now();

                self.record_edge(
                    Endpoint::System,
                    Endpoint::Ui,
                    CommunicationKind::Broadcast,
                    format!("{agent_type} snapshot"),
                );
                Ok(())
            }
            BroadcastEvent::SystemNotification { level, message } => {
                self.record_edge(
                    Endpoint::System,
                    Endpoint::Ui,
                    CommunicationKind::Broadcast,
                    level.clone(),
                );
                self.push_log(level_from_str(&level), message);
                Ok(())
            }
        }
    }

    fn apply_approval_request(&mut self, request: ApprovalRequest) -> Result<(), TuiError> {
        let agent_endpoint = parse_agent_label(&request.agent_type)
            .map(Endpoint::Agent)
            .unwrap_or(Endpoint::System);

        self.record_edge(
            agent_endpoint,
            Endpoint::User,
            CommunicationKind::Approval,
            request.reason.clone(),
        );

        if let Some(entry) = self
            .state
            .confirmations
            .iter_mut()
            .find(|entry| entry.request_id == request.request_id)
        {
            entry.reason = request.reason.clone();
            entry.recommendation = request.recommendation.clone();
            entry.status = ConfirmationStatus::Pending;
            entry.choice = None;
            entry.updated_at = Utc::now();
        } else {
            if self.state.confirmations.len() >= MAX_CONFIRMATIONS {
                self.state.confirmations.pop_front();
            }
            self.state.confirmations.push_back(ConfirmationEntry {
                request_id: request.request_id.clone(),
                task_id: request.task_id.clone(),
                agent_label: request.agent_type.clone(),
                reason: request.reason.clone(),
                recommendation: request.recommendation.clone(),
                status: ConfirmationStatus::Pending,
                choice: None,
                updated_at: Utc::now(),
            });
        }

        self.push_log(
            LogLevel::Warn,
            format!("Approval requested by {}", request.agent_type),
        );
        Ok(())
    }

    fn apply_approval_response(&mut self, response: ApprovalResponse) -> Result<(), TuiError> {
        let status = if response.approved {
            ConfirmationStatus::Approved
        } else {
            ConfirmationStatus::Rejected
        };
        let mut endpoint = Endpoint::System;

        if let Some(entry) = self
            .state
            .confirmations
            .iter_mut()
            .find(|entry| entry.request_id == response.request_id)
        {
            entry.status = status;
            entry.choice = Some(response.choice.clone());
            entry.updated_at = Utc::now();
            if let Some(agent_type) = parse_agent_label(&entry.agent_label) {
                endpoint = Endpoint::Agent(agent_type);
            }
        }

        self.record_edge(
            Endpoint::User,
            endpoint,
            CommunicationKind::Approval,
            response.choice.clone(),
        );
        self.push_log(
            if response.approved {
                LogLevel::Info
            } else {
                LogLevel::Warn
            },
            format!("Approval response {}", response.choice),
        );
        Ok(())
    }

    fn adjust_scroll(&mut self, delta: i16) {
        let offset = self
            .state
            .scroll_offsets
            .entry(self.state.active_panel)
            .or_insert(0);
        if delta.is_negative() {
            *offset = offset.saturating_sub(delta.unsigned_abs());
        } else {
            *offset = offset.saturating_add(delta as u16);
        }
    }

    fn agent_mut(&mut self, agent_type: AgentType) -> Result<&mut AgentSnapshot, TuiError> {
        self.state
            .agents
            .iter_mut()
            .find(|agent| agent.agent_type == agent_type)
            .ok_or_else(|| {
                TuiError::EventHandling(format!("missing agent snapshot for {agent_type}"))
            })
    }

    fn upsert_task(&mut self, task_id: &str, agent_type: AgentType) -> &mut TaskProgressEntry {
        if let Some(index) = self
            .state
            .tasks
            .iter()
            .position(|task| task.task_id == task_id && task.agent_type == agent_type)
        {
            return &mut self.state.tasks[index];
        }

        self.state.tasks.push(TaskProgressEntry {
            task_id: task_id.to_string(),
            agent_type,
            status: AgentExecutionStatus::Pending,
            phase: "queued".to_string(),
            progress_percent: 0,
            summary: String::new(),
            updated_at: Utc::now(),
        });

        let index = self.state.tasks.len() - 1;
        &mut self.state.tasks[index]
    }

    fn record_edge(
        &mut self,
        from: Endpoint,
        to: Endpoint,
        kind: CommunicationKind,
        summary: String,
    ) {
        if let Some(edge) = self
            .state
            .communication_edges
            .iter_mut()
            .find(|edge| edge.from == from && edge.to == to && edge.kind == kind)
        {
            edge.count = edge.count.saturating_add(1);
            edge.last_summary = summary;
            edge.updated_at = Utc::now();
            return;
        }

        self.state.communication_edges.push(CommunicationEdge {
            from,
            to,
            kind,
            count: 1,
            last_summary: summary,
            updated_at: Utc::now(),
        });
    }

    fn push_code_stream(&mut self, entry: CodeStreamEntry) {
        if self.state.code_stream.len() >= MAX_CODE_STREAM_ENTRIES {
            self.state.code_stream.pop_front();
        }
        self.state.code_stream.push_back(entry);
    }

    fn push_log(&mut self, level: LogLevel, message: impl Into<String>) {
        if self.state.logs.len() >= MAX_LOG_ENTRIES {
            self.state.logs.pop_front();
        }
        self.state.logs.push_back(UiLogEntry {
            level,
            message: message.into(),
            timestamp: Utc::now(),
        });
    }

    fn push_token_sample(&mut self, total: u64) {
        if self.state.token_history.len() >= MAX_TOKEN_HISTORY {
            self.state.token_history.pop_front();
        }
        self.state.token_history.push_back(total);
    }
}

impl AppState {
    pub fn new(title: impl Into<String>) -> Self {
        let mut scroll_offsets = BTreeMap::new();
        for panel in Panel::ALL {
            scroll_offsets.insert(panel, 0);
        }
        Self {
            active_panel: Panel::Dashboard,
            stream_mode: StreamMode::Progress,
            language: Language::detect(),
            title: title.into(),
            should_quit: false,
            terminal_size: (0, 0),
            agents: AgentType::ALL
                .into_iter()
                .map(AgentSnapshot::pending)
                .collect(),
            tasks: Vec::new(),
            communication_edges: seed_default_topology(),
            code_stream: VecDeque::new(),
            confirmations: VecDeque::new(),
            logs: VecDeque::new(),
            token_total: 0,
            token_history: VecDeque::from([0]),
            scroll_offsets,
        }
    }

    pub fn active_panel(&self) -> Panel {
        self.active_panel
    }

    pub fn stream_mode(&self) -> StreamMode {
        self.stream_mode
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn agents(&self) -> &[AgentSnapshot] {
        &self.agents
    }

    pub fn tasks(&self) -> &[TaskProgressEntry] {
        &self.tasks
    }

    pub fn communication_edges(&self) -> &[CommunicationEdge] {
        &self.communication_edges
    }

    pub fn code_stream(&self) -> &VecDeque<CodeStreamEntry> {
        &self.code_stream
    }

    pub fn confirmations(&self) -> &VecDeque<ConfirmationEntry> {
        &self.confirmations
    }

    pub fn logs(&self) -> &VecDeque<UiLogEntry> {
        &self.logs
    }

    pub fn token_total(&self) -> u64 {
        self.token_total
    }

    pub fn token_history(&self) -> &VecDeque<u64> {
        &self.token_history
    }

    pub fn scroll_offset(&self, panel: Panel) -> u16 {
        self.scroll_offsets.get(&panel).copied().unwrap_or(0)
    }

    pub fn overall_progress(&self) -> u8 {
        let mut total = 0u64;
        let mut count = 0u64;

        for agent in &self.agents {
            if agent.status == AgentExecutionStatus::Pending {
                continue;
            }
            total += u64::from(agent.progress_percent);
            count += 1;
        }

        if count == 0 {
            0
        } else {
            (total / count) as u8
        }
    }

    pub fn token_budget_total(&self) -> Option<u64> {
        let total = self
            .agents
            .iter()
            .filter_map(|agent| agent.token_budget)
            .sum::<u64>();
        if total == 0 {
            None
        } else {
            Some(total)
        }
    }

    pub fn has_budget_warning(&self) -> bool {
        match self.token_budget_total() {
            Some(budget) => self.token_total.saturating_mul(100) >= budget.saturating_mul(80),
            None => false,
        }
    }

    pub fn running_agent_count(&self) -> usize {
        self.agents
            .iter()
            .filter(|agent| agent.status == AgentExecutionStatus::Running)
            .count()
    }

    pub fn completed_agent_count(&self) -> usize {
        self.agents
            .iter()
            .filter(|agent| agent.status == AgentExecutionStatus::Completed)
            .count()
    }

    pub fn failed_agent_count(&self) -> usize {
        self.agents
            .iter()
            .filter(|agent| {
                matches!(
                    agent.status,
                    AgentExecutionStatus::Failed | AgentExecutionStatus::Cancelled
                )
            })
            .count()
    }

    pub fn pending_confirmation_count(&self) -> usize {
        self.confirmations
            .iter()
            .filter(|entry| entry.status == ConfirmationStatus::Pending)
            .count()
    }
}

fn next_panel(current: Panel) -> Panel {
    let index = active_panel_index(current);
    Panel::ALL[(index + 1) % Panel::ALL.len()]
}

fn previous_panel(current: Panel) -> Panel {
    let index = active_panel_index(current);
    Panel::ALL[(index + Panel::ALL.len() - 1) % Panel::ALL.len()]
}

fn active_panel_index(current: Panel) -> usize {
    Panel::ALL
        .iter()
        .position(|panel| *panel == current)
        .unwrap_or(0)
}

fn next_stream_mode(current: StreamMode) -> StreamMode {
    let index = active_stream_mode_index(current);
    StreamMode::ALL[(index + 1) % StreamMode::ALL.len()]
}

fn previous_stream_mode(current: StreamMode) -> StreamMode {
    let index = active_stream_mode_index(current);
    StreamMode::ALL[(index + StreamMode::ALL.len() - 1) % StreamMode::ALL.len()]
}

pub(crate) fn active_stream_mode_index(current: StreamMode) -> usize {
    StreamMode::ALL
        .iter()
        .position(|mode| *mode == current)
        .unwrap_or(0)
}

pub(crate) fn status_label(lang: Language, status: AgentExecutionStatus) -> &'static str {
    match status {
        AgentExecutionStatus::Pending => text(lang, TextKey::StatusPending),
        AgentExecutionStatus::Running => text(lang, TextKey::StatusRunning),
        AgentExecutionStatus::Completed => text(lang, TextKey::StatusCompleted),
        AgentExecutionStatus::Failed => text(lang, TextKey::StatusFailed),
        AgentExecutionStatus::Cancelled => text(lang, TextKey::StatusCancelled),
    }
}

pub(crate) fn level_from_str(level: &str) -> LogLevel {
    match level {
        "warn" | "warning" => LogLevel::Warn,
        "error" => LogLevel::Error,
        "debug" => LogLevel::Debug,
        _ => LogLevel::Info,
    }
}

pub(crate) fn parse_agent_label(label: &str) -> Option<AgentType> {
    match label {
        "Coordinator" => Some(AgentType::Coordinator),
        "Explorer" => Some(AgentType::Explorer),
        "ImpactAnalyzer" => Some(AgentType::ImpactAnalyzer),
        "Planner" => Some(AgentType::Planner),
        "Coder" => Some(AgentType::Coder),
        "Reviewer" => Some(AgentType::Reviewer),
        "Tester" => Some(AgentType::Tester),
        "Debugger" => Some(AgentType::Debugger),
        "Research" => Some(AgentType::Research),
        "DocWriter" => Some(AgentType::DocWriter),
        _ => None,
    }
}

pub(crate) fn compact_json(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
}

pub(crate) fn join_or_dash(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(", ")
    }
}

fn seed_default_topology() -> Vec<CommunicationEdge> {
    let mut edges = Vec::new();
    let now = Utc::now();

    for agent in AgentType::ALL {
        if agent == AgentType::Coordinator {
            continue;
        }

        edges.push(CommunicationEdge {
            from: Endpoint::Coordinator,
            to: Endpoint::Agent(agent),
            kind: CommunicationKind::Control,
            count: 0,
            last_summary: "idle".to_string(),
            updated_at: now,
        });
        edges.push(CommunicationEdge {
            from: Endpoint::Agent(agent),
            to: Endpoint::Coordinator,
            kind: CommunicationKind::State,
            count: 0,
            last_summary: "idle".to_string(),
            updated_at: now,
        });
    }

    edges
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new("MoreCode")
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use mc_communication::StateMessage;
    use mc_core::{AgentExecutionReport, ResultType, TaskResult};

    use crate::event::{KeyAction, TuiUpdate};

    use super::{App, AppEvent, ConfirmationStatus, Panel, StreamMode};

    fn sample_result() -> TaskResult {
        TaskResult {
            result_type: ResultType::CodeChange,
            success: true,
            data: serde_json::json!({ "ok": true }),
            changed_files: vec!["tui/src/app.rs".to_string()],
            generated_content: Some("done".to_string()),
            error_message: None,
        }
    }

    fn sample_report() -> AgentExecutionReport {
        AgentExecutionReport {
            title: "handoff".to_string(),
            key_findings: vec!["finding".to_string()],
            relevant_files: vec!["tui/src/app.rs".to_string()],
            recommendations: vec!["cargo test -p tui".to_string()],
            warnings: Vec::new(),
            token_used: 128,
            timestamp: Utc::now(),
            extra: None,
        }
    }

    #[test]
    fn panel_and_mode_switching_works() {
        let mut app = App::new();
        assert_eq!(app.state().active_panel(), Panel::Dashboard);

        app.handle_event(AppEvent::Key(KeyAction::NextPanel))
            .expect("next panel");
        assert_eq!(app.state().active_panel(), Panel::AgentStatus);

        app.handle_event(AppEvent::Key(KeyAction::SetStreamMode(StreamMode::Code)))
            .expect("mode");
        assert_eq!(app.state().stream_mode(), StreamMode::Code);
    }

    #[test]
    fn state_updates_flow_into_ui_state() {
        let mut app = App::new();

        app.handle_event(AppEvent::Update(Box::new(TuiUpdate::State(
            StateMessage::Progress {
                task_id: "task-1".to_string(),
                agent_type: mc_core::AgentType::Coder,
                phase: "editing".to_string(),
                progress_percent: 50,
                message: "updating".to_string(),
            },
        ))))
        .expect("progress");
        app.handle_event(AppEvent::Update(Box::new(TuiUpdate::State(
            StateMessage::TaskCompleted {
                task_id: "task-1".to_string(),
                agent_type: mc_core::AgentType::Coder,
                result: sample_result(),
                handoff: sample_report(),
                token_used: 512,
            },
        ))))
        .expect("completed");

        assert_eq!(app.state().token_total(), 512);
        assert_eq!(app.state().tasks().len(), 1);
        assert_eq!(app.state().code_stream().len(), 0);

        app.handle_event(AppEvent::Update(Box::new(TuiUpdate::ApprovalRequest(
            mc_communication::ApprovalRequest {
                request_id: "approval-1".to_string(),
                task_id: "task-1".to_string(),
                agent_type: "Coder".to_string(),
                reason: "needs confirmation".to_string(),
                options: vec!["approve".to_string(), "reject".to_string()],
                recommendation: Some("approve".to_string()),
                created_at: Utc::now(),
                timeout_secs: 30,
            },
        ))))
        .expect("approval request");
        app.handle_event(AppEvent::Update(Box::new(TuiUpdate::ApprovalResponse(
            mc_communication::ApprovalResponse {
                request_id: "approval-1".to_string(),
                choice: "approve".to_string(),
                approved: true,
                comment: Some("ok".to_string()),
                responded_at: Utc::now(),
            },
        ))))
        .expect("approval response");

        assert_eq!(
            app.state().confirmations().front().map(|item| item.status),
            Some(ConfirmationStatus::Approved)
        );
    }
}
