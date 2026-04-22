use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Gauge, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{AppState, StreamMode};
use crate::i18n::{text, TextKey};
use crate::theme::TuiTheme;
use crate::widget::progress_bar::progress_ratio;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    let lang = state.language();
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(9),
            Constraint::Min(8),
        ])
        .split(area);
    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(outer[0]);
    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(outer[1]);

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
    let tokens = Paragraph::new(token_lines).block(theme.panel_block(text(lang, TextKey::DashboardTokens), false));
    frame.render_widget(tokens, cards[2]);

    let topology = state
        .communication_edges()
        .iter()
        .rev()
        .filter(|edge| edge.count > 0)
        .take(6)
        .map(|edge| Line::from(format!("{} -> {} [{}]", edge.from, edge.to, edge.kind)))
        .collect::<Vec<_>>();
    let topology = Paragraph::new(if topology.is_empty() {
        vec![Line::from(text(lang, TextKey::EmptyNoEdges))]
    } else {
        topology
    })
    .block(theme.panel_block(text(lang, TextKey::DashboardTopologyPreview), false))
    .wrap(Wrap { trim: false });
    frame.render_widget(topology, middle[0]);

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
    let stream = Paragraph::new(if stream.is_empty() {
        vec![Line::from(text(lang, TextKey::EmptyNoStream))]
    } else {
        stream
    })
    .block(theme.panel_block(text(lang, TextKey::DashboardCurrentStream), false))
    .wrap(Wrap { trim: false });
    frame.render_widget(stream, middle[1]);

    let logs = state
        .logs()
        .iter()
        .rev()
        .take(8)
        .map(|entry| Line::from(format!("[{}] {}", entry.level.label(), entry.message)))
        .collect::<Vec<_>>();
    let logs = Paragraph::new(if logs.is_empty() {
        vec![Line::from(text(lang, TextKey::EmptyNoLogs))]
    } else {
        logs.into_iter().rev().collect()
    })
    .block(theme.panel_block(text(lang, TextKey::DashboardRecentLogs), false))
    .wrap(Wrap { trim: false });
    frame.render_widget(logs, outer[2]);
}
