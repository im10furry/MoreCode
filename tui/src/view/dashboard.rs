use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Gauge, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{AppState, StreamMode};
use crate::i18n::{text, TextKey};
use crate::theme::TuiTheme;
use crate::widget::progress_bar::progress_ratio;
use crate::widget::tree::{render_tree, TreeNode};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    if let Some(snapshot) = state.run_snapshot() {
        render_run_dashboard(frame, area, state, snapshot, theme);
        return;
    }

    let lang = state.language();
    let outer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(area);
    let main = outer[0];
    let sidebar = outer[1];

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(0),
        ])
        .split(main);

    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(main_layout[0]);

    let agents = Paragraph::new(vec![
        Line::from(format!(
            "{} {}",
            text(lang, TextKey::DashboardRunning),
            state.running_agent_count()
        )),
        Line::from(format!(
            "{} {}",
            text(lang, TextKey::DashboardCompleted),
            state.completed_agent_count()
        )),
        Line::from(format!(
            "{} {}",
            text(lang, TextKey::DashboardFailed),
            state.failed_agent_count()
        )),
        Line::from(format!(
            "{} {}",
            text(lang, TextKey::DashboardPendingApprovals),
            state.pending_confirmation_count()
        )),
    ])
    .block(theme.panel_block(text(lang, TextKey::DashboardAgents), false));
    frame.render_widget(agents, cards[0]);

    let progress = Gauge::default()
        .block(theme.panel_block(text(lang, TextKey::DashboardOverallProgress), false))
        .gauge_style(theme.accent())
        .ratio(progress_ratio(state.overall_progress()))
        .label(format!("{}%", state.overall_progress()));
    frame.render_widget(progress, cards[1]);

    let token_lines = vec![
        Line::from(format!(
            "{} {}",
            text(lang, TextKey::DashboardTotal),
            state.token_total()
        )),
        Line::from(match state.token_budget_total() {
            Some(budget) => format!("{} {budget}", text(lang, TextKey::DashboardBudget)),
            None => format!("{} -", text(lang, TextKey::DashboardBudget)),
        }),
        Line::from(format!(
            "{} {}",
            text(lang, TextKey::DashboardSamples),
            state.token_history().len()
        )),
        Line::from(if state.has_budget_warning() {
            text(lang, TextKey::DashboardWarningBudget).to_string()
        } else {
            text(lang, TextKey::DashboardWarningNone).to_string()
        }),
    ];
    let tokens = Paragraph::new(token_lines)
        .block(theme.panel_block(text(lang, TextKey::DashboardTokens), false));
    frame.render_widget(tokens, cards[2]);

    let stream = match state.stream_mode() {
        StreamMode::Confirmation => state
            .confirmations()
            .iter()
            .rev()
            .take(5)
            .map(|entry| Line::from(format!("{}: {}", entry.agent_label, entry.status)))
            .collect::<Vec<_>>(),
        StreamMode::Code => state
            .code_stream()
            .iter()
            .rev()
            .take(5)
            .map(|entry| Line::from(format!("{} {}", entry.kind, entry.content)))
            .collect::<Vec<_>>(),
        StreamMode::Progress => state
            .tasks()
            .iter()
            .rev()
            .take(5)
            .map(|task| Line::from(format!("{} {}%", task.agent_type, task.progress_percent)))
            .collect::<Vec<_>>(),
    };
    let stream_widget = Paragraph::new(if stream.is_empty() {
        vec![Line::from(text(lang, TextKey::EmptyNoStream))]
    } else {
        stream
    })
    .block(theme.panel_block(text(lang, TextKey::DashboardCurrentStream), false))
    .wrap(Wrap { trim: false });
    frame.render_widget(stream_widget, main_layout[1]);

    let sidebar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Min(0),
        ])
        .split(sidebar);

    let topology = state
        .communication_edges()
        .iter()
        .rev()
        .filter(|edge| edge.count > 0)
        .take(4)
        .map(|edge| Line::from(format!("{} -> {} [{}]", edge.from, edge.to, edge.kind)))
        .collect::<Vec<_>>();
    let topology_widget = Paragraph::new(if topology.is_empty() {
        vec![Line::from(text(lang, TextKey::EmptyNoEdges))]
    } else {
        topology
    })
    .block(theme.panel_block(text(lang, TextKey::DashboardTopologyPreview), false))
    .wrap(Wrap { trim: false });
    frame.render_widget(topology_widget, sidebar_layout[0]);

    let logs = state
        .logs()
        .iter()
        .rev()
        .take(6)
        .map(|entry| Line::from(format!("[{}] {}", entry.level.label(), entry.message)))
        .collect::<Vec<_>>();
    let logs_widget = Paragraph::new(if logs.is_empty() {
        vec![Line::from(text(lang, TextKey::EmptyNoLogs))]
    } else {
        logs.into_iter().rev().collect()
    })
    .block(theme.panel_block(text(lang, TextKey::DashboardRecentLogs), false))
    .wrap(Wrap { trim: false });
    frame.render_widget(logs_widget, sidebar_layout[1]);
}

fn render_run_dashboard(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    snapshot: &mc_core::RunSnapshot,
    theme: TuiTheme,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(layout[1]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(top[1]);

    let summary = vec![
        Line::from(format!("Run {}", snapshot.summary.run_id)),
        Line::from(format!("status: {:?}", snapshot.summary.status)),
        Line::from(format!("request: {}", snapshot.summary.request)),
        Line::from(format!(
            "tokens: {}  approvals: {}  patches: {}",
            snapshot.summary.total_tokens,
            snapshot.summary.approvals.len(),
            snapshot.summary.patches.len()
        )),
    ];
    frame.render_widget(
        Paragraph::new(summary)
            .block(theme.panel_block("Run Summary", true))
            .wrap(Wrap { trim: false }),
        layout[0],
    );

    let steps = build_step_tree(snapshot);
    frame.render_widget(
        Paragraph::new(render_tree(&steps))
            .block(theme.panel_block("Step Tree", false))
            .scroll((state.scroll_offset(state.active_panel()), 0))
            .wrap(Wrap { trim: false }),
        top[0],
    );

    let timeline = snapshot
        .events
        .iter()
        .rev()
        .take(10)
        .map(|event| Line::from(event_label(event)))
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(if timeline.is_empty() {
            vec![Line::from("No timeline yet")]
        } else {
            timeline
        })
        .block(theme.panel_block("Timeline", false))
        .wrap(Wrap { trim: false }),
        bottom[0],
    );

    let detail = latest_detail(snapshot, state.patch_index);
    frame.render_widget(
        Paragraph::new(detail)
            .block(theme.panel_block("Diff / Detail", false))
            .scroll((state.scroll_offset(state.active_panel()), 0))
            .wrap(Wrap { trim: false }),
        bottom[1],
    );
}

fn build_step_tree(snapshot: &mc_core::RunSnapshot) -> Vec<TreeNode> {
    snapshot
        .summary
        .steps
        .iter()
        .filter(|step| step.parent_step_id.is_none())
        .map(|step| TreeNode {
            label: format!(
                "{} [{:?}] {}",
                step.title,
                step.status,
                step.summary.clone().unwrap_or_default()
            ),
            children: snapshot
                .summary
                .steps
                .iter()
                .filter(|child| child.parent_step_id.as_deref() == Some(step.step_id.as_str()))
                .map(|child| {
                    TreeNode::leaf(format!(
                        "{} [{} tokens] {}",
                        child.title,
                        child.token_used,
                        child.summary.clone().unwrap_or_default()
                    ))
                })
                .collect(),
        })
        .collect()
}

fn event_label(event: &mc_core::RunEventEnvelope) -> String {
    match &event.event {
        mc_core::RunEvent::RunStarted { request, .. } => format!("run started: {request}"),
        mc_core::RunEvent::StepStarted { step } => format!("step: {}", step.title),
        mc_core::RunEvent::StepFinished {
            step_id, summary, ..
        } => format!("done {step_id}: {}", summary.clone().unwrap_or_default()),
        mc_core::RunEvent::Message { message, .. } => message.clone(),
        mc_core::RunEvent::ApprovalRequested { approval } => {
            format!("approval: {}", approval.title)
        }
        mc_core::RunEvent::ApprovalResolved {
            approval_id,
            status,
            ..
        } => {
            format!("approval {approval_id}: {:?}", status)
        }
        mc_core::RunEvent::PatchProposed { patch } => format!("patch: {}", patch.file_path),
        mc_core::RunEvent::PatchResolved {
            patch_id, status, ..
        } => format!("patch {patch_id}: {:?}", status),
        mc_core::RunEvent::ArtifactWritten { artifact } => format!("artifact: {}", artifact.title),
        mc_core::RunEvent::CommandStarted { command } => format!("cmd: {}", command.command),
        mc_core::RunEvent::CommandOutput { command_id, .. } => format!("cmd output: {command_id}"),
        mc_core::RunEvent::CommandFinished {
            command_id, status, ..
        } => format!("cmd {command_id}: {:?}", status),
        mc_core::RunEvent::RunFinished { summary, .. } => summary
            .clone()
            .unwrap_or_else(|| "run finished".to_string()),
        mc_core::RunEvent::Error { message, .. } => format!("error: {message}"),
    }
}

fn latest_detail(snapshot: &mc_core::RunSnapshot, patch_index: usize) -> Vec<Line<'static>> {
    if !snapshot.summary.patches.is_empty() {
        let patch = &snapshot.summary.patches[patch_index.min(snapshot.summary.patches.len() - 1)];
        let mut lines = vec![
            Line::from(format!("selected patch: {}", patch.file_path)),
            Line::from(format!(
                "status: {:?}  controls: n/p switch  x accept  d reject",
                patch.status
            )),
            Line::from(""),
        ];
        lines.extend(
            patch
                .preview
                .lines()
                .map(|line| Line::from(line.to_string())),
        );
        return lines;
    }
    if let Some(command) = snapshot.summary.commands.last() {
        let mut lines = vec![
            Line::from(format!("command: {}", command.command)),
            Line::from(format!("status: {:?}", command.status)),
        ];
        if !command.stdout_tail.is_empty() {
            lines.push(Line::from("stdout:"));
            lines.extend(
                command
                    .stdout_tail
                    .lines()
                    .map(|line| Line::from(line.to_string())),
            );
        }
        return lines;
    }
    vec![Line::from(
        snapshot
            .summary
            .final_summary
            .clone()
            .unwrap_or_else(|| "No detail yet".to_string()),
    )]
}
