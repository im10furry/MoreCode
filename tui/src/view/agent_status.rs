use ratatui::layout::{Constraint, Rect};
use ratatui::style::Modifier;
use ratatui::widgets::{Cell, Row, Table};
use ratatui::Frame;

use crate::app::{status_label, AppState};
use crate::theme::TuiTheme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    let rows = state.agents().iter().map(|agent| {
        let budget = agent
            .token_budget
            .map(|budget| budget.to_string())
            .unwrap_or_else(|| "-".to_string());
        Row::new(vec![
            Cell::from(agent.agent_type.to_string()).style(theme.agent_style(agent.agent_type)),
            Cell::from(status_label(agent.status)).style(theme.status_style(agent.status)),
            Cell::from(agent.task_id.clone().unwrap_or_else(|| "-".to_string())),
            Cell::from(agent.phase.clone().unwrap_or_else(|| "-".to_string())),
            Cell::from(format!("{}%", agent.progress_percent)),
            Cell::from(agent.token_used.to_string()),
            Cell::from(budget),
        ])
    });

    let widths = [
        Constraint::Length(16),
        Constraint::Length(11),
        Constraint::Min(16),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Length(12),
        Constraint::Length(12),
    ];
    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                "Agent", "Status", "Task", "Phase", "Progress", "Tokens", "Budget",
            ])
            .style(theme.accent().add_modifier(Modifier::BOLD)),
        )
        .block(theme.panel_block("Agent Status", true))
        .column_spacing(1);
    frame.render_widget(table, area);
}
