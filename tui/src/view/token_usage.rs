use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Cell, Paragraph, Row, Sparkline, Table};
use ratatui::Frame;

use crate::app::AppState;
use crate::theme::TuiTheme;
use crate::widget::sparkline::compress_history;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(sections[0]);

    let summary = Paragraph::new(vec![
        Line::from(format!("total: {}", state.token_total())),
        Line::from(match state.token_budget_total() {
            Some(budget) => format!("budget: {budget}"),
            None => "budget: -".to_string(),
        }),
        Line::from(format!("running agents: {}", state.running_agent_count())),
        Line::from(if state.has_budget_warning() {
            "status: warning".to_string()
        } else {
            "status: healthy".to_string()
        }),
    ])
    .block(theme.panel_block("Token Summary", false));
    frame.render_widget(summary, top[0]);

    let history = state.token_history().iter().copied().collect::<Vec<_>>();
    let values = compress_history(&history, top[1].width as usize);
    let sparkline = Sparkline::default()
        .block(theme.panel_block("Consumption Trend", false))
        .data(&values)
        .style(theme.accent());
    frame.render_widget(sparkline, top[1]);

    let rows = state.agents().iter().map(|agent| {
        let budget = agent
            .token_budget
            .map(|budget| budget.to_string())
            .unwrap_or_else(|| "-".to_string());
        Row::new(vec![
            Cell::from(agent.agent_type.to_string()).style(theme.agent_style(agent.agent_type)),
            Cell::from(agent.token_used.to_string()),
            Cell::from(budget),
            Cell::from(agent.detail.clone()),
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["Agent", "Used", "Budget", "Last Detail"])
            .style(theme.accent().add_modifier(Modifier::BOLD)),
    )
    .block(theme.panel_block("Per-Agent Usage", true))
    .column_spacing(1);
    frame.render_widget(table, sections[1]);
}
