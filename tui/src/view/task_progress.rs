use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Gauge, Paragraph, Tabs, Wrap};
use ratatui::Frame;

use crate::app::{active_stream_mode_index, status_label, AppState, StreamMode};
use crate::i18n::{text, TextKey};
use crate::theme::TuiTheme;
use crate::widget::progress_bar::{progress_ratio, render_progress_bar};
use crate::widget::tree::{render_tree, TreeNode};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    if let Some(snapshot) = state.run_snapshot() {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(0)])
            .split(area);

        let gauge = Gauge::default()
            .block(theme.panel_block("Run Progress", false))
            .gauge_style(theme.accent())
            .ratio(progress_ratio(state.overall_progress()))
            .label(format!("{}%", state.overall_progress()));
        frame.render_widget(gauge, sections[0]);

        let tree = snapshot
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
                            "{} [{:?}] {}",
                            child.title,
                            child.status,
                            child.summary.clone().unwrap_or_default()
                        ))
                    })
                    .collect(),
            })
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(render_tree(&tree))
                .block(theme.panel_block("Step Tree", true))
                .scroll((state.scroll_offset(state.active_panel()), 0))
                .wrap(Wrap { trim: false }),
            sections[1],
        );
        return;
    }

    let lang = state.language();
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
            .map(|mode| Line::from(mode.title(lang)))
            .collect::<Vec<_>>(),
    )
    .select(active_stream_mode_index(state.stream_mode()))
    .block(theme.panel_block(text(lang, TextKey::TaskProgressFeedbackModes), false))
    .style(theme.muted())
    .highlight_style(theme.accent());
    frame.render_widget(tabs, sections[0]);

    let gauge = Gauge::default()
        .block(theme.panel_block(text(lang, TextKey::TaskProgressTaskProgress), false))
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
                    status_label(lang, task.status),
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
        vec![Line::from(text(lang, TextKey::EmptyNoEvents))]
    } else {
        lines
    })
    .block(theme.panel_block(text(lang, TextKey::TaskProgressStreamOutput), true))
    .scroll((state.scroll_offset(state.active_panel()), 0))
    .wrap(Wrap { trim: false });
    frame.render_widget(body, sections[2]);
}
