use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Cell, Paragraph, Row, Sparkline, Table};
use ratatui::Frame;

use crate::app::AppState;
use crate::i18n::{text, TextKey};
use crate::theme::TuiTheme;
use crate::widget::sparkline::compress_history;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    if let Some(snapshot) = state.run_snapshot() {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(8), Constraint::Min(0)])
            .split(area);
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(sections[0]);

        let summary = Paragraph::new(vec![
            Line::from(format!("total tokens: {}", snapshot.summary.total_tokens)),
            Line::from(format!("steps: {}", snapshot.summary.steps.len())),
            Line::from(format!("commands: {}", snapshot.summary.commands.len())),
            Line::from(format!("patches: {}", snapshot.summary.patches.len())),
        ])
        .block(theme.panel_block("Run Metrics", false));
        frame.render_widget(summary, top[0]);

        let history = state.token_history().iter().copied().collect::<Vec<_>>();
        let values = compress_history(&history, top[1].width as usize);
        let sparkline = Sparkline::default()
            .block(theme.panel_block("Token Trend", false))
            .data(&values)
            .style(theme.accent());
        frame.render_widget(sparkline, top[1]);

        let rows = snapshot.summary.commands.iter().map(|command| {
            Row::new(vec![
                Cell::from(command.title.clone()),
                Cell::from(format!("{:?}", command.status)),
                Cell::from(
                    command
                        .exit_code
                        .map(|code| code.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                ),
                Cell::from(command.command.clone()),
            ])
        });
        let table = Table::new(
            rows,
            [
                Constraint::Length(20),
                Constraint::Length(12),
                Constraint::Length(10),
                Constraint::Min(20),
            ],
        )
        .header(
            Row::new(vec!["Command", "Status", "Exit", "Invocation"])
                .style(theme.accent().add_modifier(Modifier::BOLD)),
        )
        .block(theme.panel_block("Commands", true))
        .column_spacing(1);
        frame.render_widget(table, sections[1]);
        return;
    }

    let lang = state.language();
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(sections[0]);

    let summary = Paragraph::new(vec![
        Line::from(format!(
            "{} {}",
            text(lang, TextKey::TokenLineTotal),
            state.token_total()
        )),
        Line::from(match state.token_budget_total() {
            Some(budget) => format!("{} {budget}", text(lang, TextKey::TokenLineBudget)),
            None => format!("{} -", text(lang, TextKey::TokenLineBudget)),
        }),
        Line::from(format!(
            "{} {}",
            text(lang, TextKey::TokenLineRunningAgents),
            state.running_agent_count()
        )),
        Line::from(if state.has_budget_warning() {
            text(lang, TextKey::TokenLineStatusWarning).to_string()
        } else {
            text(lang, TextKey::TokenLineStatusHealthy).to_string()
        }),
    ])
    .block(theme.panel_block(text(lang, TextKey::TokenSummaryTitle), false));
    frame.render_widget(summary, top[0]);

    let history = state.token_history().iter().copied().collect::<Vec<_>>();
    let values = compress_history(&history, top[1].width as usize);
    let sparkline = Sparkline::default()
        .block(theme.panel_block(text(lang, TextKey::TokenTrendTitle), false))
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
        Row::new(vec![
            text(lang, TextKey::TokenHeaderAgent),
            text(lang, TextKey::TokenHeaderUsed),
            text(lang, TextKey::TokenHeaderBudget),
            text(lang, TextKey::TokenHeaderLastDetail),
        ])
        .style(theme.accent().add_modifier(Modifier::BOLD)),
    )
    .block(theme.panel_block(text(lang, TextKey::TokenPerAgentTitle), true))
    .column_spacing(1);
    frame.render_widget(table, sections[1]);
}
