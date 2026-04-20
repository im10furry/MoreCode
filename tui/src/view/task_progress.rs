use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Gauge, Paragraph, Tabs, Wrap};
use ratatui::Frame;

use crate::app::{active_stream_mode_index, status_label, AppState, StreamMode};
use crate::theme::TuiTheme;
use crate::widget::progress_bar::{progress_ratio, render_progress_bar};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

    let tabs = Tabs::new(
        StreamMode::ALL
            .into_iter()
            .map(|mode| Line::from(mode.title()))
            .collect::<Vec<_>>(),
    )
    .select(active_stream_mode_index(state.stream_mode()))
    .block(theme.panel_block("Feedback Modes", false))
    .style(theme.muted())
    .highlight_style(theme.accent());
    frame.render_widget(tabs, sections[0]);

    let gauge = Gauge::default()
        .block(theme.panel_block("Task Progress", false))
        .gauge_style(theme.accent())
        .ratio(progress_ratio(state.overall_progress()))
        .label(format!("{}%", state.overall_progress()));
    frame.render_widget(gauge, sections[1]);

    let lines = match state.stream_mode() {
        StreamMode::Progress => state
            .tasks()
            .iter()
            .map(|task| {
                Line::from(format!(
                    "{} {} {} {}",
                    task.agent_type,
                    status_label(task.status),
                    render_progress_bar(12, task.progress_percent),
                    task.summary
                ))
            })
            .collect::<Vec<_>>(),
        StreamMode::Code => state
            .code_stream()
            .iter()
            .map(|entry| Line::from(format!("{} {}", entry.kind, entry.content)))
            .collect::<Vec<_>>(),
        StreamMode::Confirmation => state
            .confirmations()
            .iter()
            .map(|entry| {
                let choice = entry.choice.clone().unwrap_or_else(|| "-".to_string());
                Line::from(format!(
                    "{} {} {} {}",
                    entry.agent_label, entry.status, choice, entry.reason
                ))
            })
            .collect::<Vec<_>>(),
    };

    let body = Paragraph::new(if lines.is_empty() {
        vec![Line::from("No events yet")]
    } else {
        lines
    })
    .block(theme.panel_block("Stream Output", true))
    .scroll((state.scroll_offset(state.active_panel()), 0))
    .wrap(Wrap { trim: false });
    frame.render_widget(body, sections[2]);
}
