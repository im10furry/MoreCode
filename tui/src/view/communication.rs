use std::collections::BTreeMap;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::widgets::{Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

use crate::app::{AppState, Endpoint};
use crate::i18n::{text, TextKey};
use crate::theme::TuiTheme;
use crate::widget::tree::{render_tree, TreeNode};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    if let Some(snapshot) = state.run_snapshot() {
        let sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(area);

        let rows = snapshot.events.iter().rev().take(20).map(|event| {
            Row::new(vec![
                Cell::from(event.sequence.to_string()),
                Cell::from(event.at.format("%H:%M:%S").to_string()),
                Cell::from(event_kind(&event.event)),
                Cell::from(event_summary(&event.event)),
            ])
        });
        let table = Table::new(
            rows,
            [
                Constraint::Length(6),
                Constraint::Length(10),
                Constraint::Length(14),
                Constraint::Min(20),
            ],
        )
        .header(
            Row::new(vec!["Seq", "Time", "Kind", "Summary"])
                .style(theme.accent().add_modifier(Modifier::BOLD)),
        )
        .block(theme.panel_block("Timeline", true))
        .column_spacing(1);
        frame.render_widget(table, sections[0]);

        let tree = snapshot
            .summary
            .steps
            .iter()
            .map(|step| TreeNode::leaf(format!("{} [{:?}]", step.step_id, step.status)))
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(render_tree(&tree))
                .block(theme.panel_block("Steps", false))
                .scroll((state.scroll_offset(state.active_panel()), 0))
                .wrap(Wrap { trim: false }),
            sections[1],
        );
        return;
    }

    let lang = state.language();
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let rows = state.communication_edges().iter().map(|edge| {
        Row::new(vec![
            Cell::from(edge.count.to_string()),
            Cell::from(edge.kind.to_string()),
            Cell::from(edge.from.to_string()),
            Cell::from(edge.to.to_string()),
            Cell::from(edge.last_summary.clone()),
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(11),
            Constraint::Length(16),
            Constraint::Length(16),
            Constraint::Min(12),
        ],
    )
    .header(
        Row::new(vec![
            text(lang, TextKey::CommunicationHeaderCount),
            text(lang, TextKey::CommunicationHeaderKind),
            text(lang, TextKey::CommunicationHeaderFrom),
            text(lang, TextKey::CommunicationHeaderTo),
            text(lang, TextKey::CommunicationHeaderSummary),
        ])
        .style(theme.accent().add_modifier(Modifier::BOLD)),
    )
    .block(theme.panel_block(text(lang, TextKey::CommunicationEdgesTitle), true))
    .column_spacing(1);
    frame.render_widget(table, sections[0]);

    let mut groups: BTreeMap<Endpoint, Vec<String>> = BTreeMap::new();
    for edge in state.communication_edges() {
        groups
            .entry(edge.from)
            .or_default()
            .push(format!("{} [{}:{}]", edge.to, edge.kind, edge.count));
    }
    let tree = groups
        .into_iter()
        .map(|(from, children)| TreeNode {
            label: from.to_string(),
            children: children.into_iter().map(TreeNode::leaf).collect(),
        })
        .collect::<Vec<_>>();
    let paragraph = Paragraph::new(render_tree(&tree))
        .block(theme.panel_block(text(lang, TextKey::CommunicationTopologyTitle), false))
        .scroll((state.scroll_offset(state.active_panel()), 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, sections[1]);
}

fn event_kind(event: &mc_core::RunEvent) -> String {
    match event {
        mc_core::RunEvent::RunStarted { .. } => "run".to_string(),
        mc_core::RunEvent::StepStarted { .. } | mc_core::RunEvent::StepFinished { .. } => {
            "step".to_string()
        }
        mc_core::RunEvent::Message { .. } => "message".to_string(),
        mc_core::RunEvent::ApprovalRequested { .. }
        | mc_core::RunEvent::ApprovalResolved { .. } => "approval".to_string(),
        mc_core::RunEvent::PatchProposed { .. } | mc_core::RunEvent::PatchResolved { .. } => {
            "patch".to_string()
        }
        mc_core::RunEvent::ArtifactWritten { .. } => "artifact".to_string(),
        mc_core::RunEvent::CommandStarted { .. }
        | mc_core::RunEvent::CommandOutput { .. }
        | mc_core::RunEvent::CommandFinished { .. } => "command".to_string(),
        mc_core::RunEvent::RunFinished { .. } => "finish".to_string(),
        mc_core::RunEvent::Error { .. } => "error".to_string(),
    }
}

fn event_summary(event: &mc_core::RunEvent) -> String {
    match event {
        mc_core::RunEvent::RunStarted { request, .. } => request.clone(),
        mc_core::RunEvent::StepStarted { step } => step.title.clone(),
        mc_core::RunEvent::StepFinished {
            step_id, summary, ..
        } => format!("{} {}", step_id, summary.clone().unwrap_or_default()),
        mc_core::RunEvent::Message { message, .. } => message.clone(),
        mc_core::RunEvent::ApprovalRequested { approval } => approval.title.clone(),
        mc_core::RunEvent::ApprovalResolved {
            approval_id,
            status,
            ..
        } => format!("{approval_id} {status:?}"),
        mc_core::RunEvent::PatchProposed { patch } => patch.file_path.clone(),
        mc_core::RunEvent::PatchResolved {
            patch_id, status, ..
        } => format!("{patch_id} {status:?}"),
        mc_core::RunEvent::ArtifactWritten { artifact } => artifact.title.clone(),
        mc_core::RunEvent::CommandStarted { command } => command.command.clone(),
        mc_core::RunEvent::CommandOutput { command_id, .. } => command_id.clone(),
        mc_core::RunEvent::CommandFinished {
            command_id, status, ..
        } => format!("{command_id} {status:?}"),
        mc_core::RunEvent::RunFinished { summary, .. } => {
            summary.clone().unwrap_or_else(|| "finished".to_string())
        }
        mc_core::RunEvent::Error { message, .. } => message.clone(),
    }
}
