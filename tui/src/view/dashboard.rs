use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Gauge, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::AppState;
use crate::theme::TuiTheme;
use crate::widget::progress_bar::progress_ratio;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
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
        Line::from(format!("running: {}", state.running_agent_count())),
        Line::from(format!("completed: {}", state.completed_agent_count())),
        Line::from(format!("failed: {}", state.failed_agent_count())),
        Line::from(format!(
            "pending approvals: {}",
            state.pending_confirmation_count()
        )),
    ])
    .block(theme.panel_block("Agents", false));
    frame.render_widget(agents, cards[0]);

    let progress = Gauge::default()
        .block(theme.panel_block("Overall Progress", false))
        .gauge_style(theme.accent())
        .ratio(progress_ratio(state.overall_progress()))
        .label(format!("{}%", state.overall_progress()));
    frame.render_widget(progress, cards[1]);

    let token_lines = vec![
        Line::from(format!("total: {}", state.token_total())),
        Line::from(match state.token_budget_total() {
            Some(budget) => format!("budget: {budget}"),
            None => "budget: -".to_string(),
        }),
        Line::from(format!("samples: {}", state.token_history().len())),
        Line::from(if state.has_budget_warning() {
            "warning: >=80% budget".to_string()
        } else {
            "warning: none".to_string()
        }),
    ];
    let tokens = Paragraph::new(token_lines).block(theme.panel_block("Tokens", false));
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
        vec![Line::from("No active edges yet")]
    } else {
        topology
    })
    .block(theme.panel_block("Topology Preview", false))
    .wrap(Wrap { trim: false });
    frame.render_widget(topology, middle[0]);

    let stream = if state.stream_mode().title() == "Confirmation" {
        state
            .confirmations()
            .iter()
            .rev()
            .take(5)
            .map(|entry| Line::from(format!("{}: {}", entry.agent_label, entry.status)))
            .collect::<Vec<_>>()
    } else if state.stream_mode().title() == "Code" {
        state
            .code_stream()
            .iter()
            .rev()
            .take(5)
            .map(|entry| Line::from(format!("{} {}", entry.kind, entry.content)))
            .collect::<Vec<_>>()
    } else {
        state
            .tasks()
            .iter()
            .rev()
            .take(5)
            .map(|task| Line::from(format!("{} {}%", task.agent_type, task.progress_percent)))
            .collect::<Vec<_>>()
    };
    let stream = Paragraph::new(if stream.is_empty() {
        vec![Line::from("No stream events yet")]
    } else {
        stream
    })
    .block(theme.panel_block("Current Stream", false))
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
        vec![Line::from("No logs yet")]
    } else {
        logs.into_iter().rev().collect()
    })
    .block(theme.panel_block("Recent Logs", false))
    .wrap(Wrap { trim: false });
    frame.render_widget(logs, outer[2]);
}
