use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

use crate::app::{status_label, AppState};
use crate::i18n::{text, TextKey};
use crate::theme::TuiTheme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    if let Some(snapshot) = state.run_snapshot() {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(9),
                Constraint::Percentage(45),
                Constraint::Min(0),
            ])
            .split(area);

        let approval_rows = snapshot.summary.approvals.iter().map(|approval| {
            let selected = snapshot
                .summary
                .approvals
                .get(state.approval_index)
                .is_some_and(|current| current.approval_id == approval.approval_id);
            Row::new(vec![
                Cell::from(format!(
                    "{}{}",
                    if selected { "> " } else { "  " },
                    approval.title
                )),
                Cell::from(format!("{:?}", approval.level)),
                Cell::from(format!("{:?}", approval.status)),
                Cell::from(approval.choice.clone().unwrap_or_else(|| "-".to_string())),
            ])
        });
        let approvals = Table::new(
            approval_rows,
            [
                Constraint::Percentage(40),
                Constraint::Length(10),
                Constraint::Length(12),
                Constraint::Percentage(20),
            ],
        )
        .header(
            Row::new(vec!["Approval", "Level", "Status", "Choice"])
                .style(theme.accent().add_modifier(Modifier::BOLD)),
        )
        .block(theme.panel_block("Approvals", true))
        .column_spacing(1);
        frame.render_widget(approvals, sections[0]);

        let patch_lines = snapshot
            .summary
            .patches
            .get(state.patch_index)
            .map(|patch| {
                let mut lines = vec![
                    Line::from(format!("selected patch: {}", patch.file_path)),
                    Line::from(format!(
                        "status: {:?}  kind: {:?}",
                        patch.status, patch.kind
                    )),
                    Line::from(format!("rationale: {}", patch.rationale)),
                    Line::from("controls: n/p switch  x accept  d reject"),
                    Line::from(""),
                ];
                lines.extend(
                    patch
                        .preview
                        .lines()
                        .take(18)
                        .map(|line| Line::from(line.to_string())),
                );
                lines
            })
            .unwrap_or_else(|| vec![Line::from("No patches yet")]);
        frame.render_widget(
            Paragraph::new(patch_lines)
                .block(theme.panel_block("Patch Review", false))
                .wrap(Wrap { trim: false }),
            sections[1],
        );

        let artifact_lines = snapshot
            .summary
            .artifacts
            .iter()
            .map(|artifact| {
                let path = format!("{} -> {}", artifact.title, artifact.relative_path);
                ratatui::text::Line::from(path)
            })
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(if artifact_lines.is_empty() {
                vec![ratatui::text::Line::from("No artifacts yet")]
            } else {
                artifact_lines
            })
            .block(theme.panel_block("Artifacts", false))
            .scroll((state.scroll_offset(state.active_panel()), 0))
            .wrap(Wrap { trim: false }),
            sections[2],
        );
        return;
    }

    let lang = state.language();
    let rows = state.agents().iter().map(|agent| {
        let budget = agent
            .token_budget
            .map(|budget| budget.to_string())
            .unwrap_or_else(|| "-".to_string());
        Row::new(vec![
            Cell::from(agent.agent_type.to_string()).style(theme.agent_style(agent.agent_type)),
            Cell::from(status_label(lang, agent.status)).style(theme.status_style(agent.status)),
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
                text(lang, TextKey::AgentStatusHeaderAgent),
                text(lang, TextKey::AgentStatusHeaderStatus),
                text(lang, TextKey::AgentStatusHeaderTask),
                text(lang, TextKey::AgentStatusHeaderPhase),
                text(lang, TextKey::AgentStatusHeaderProgress),
                text(lang, TextKey::AgentStatusHeaderTokens),
                text(lang, TextKey::AgentStatusHeaderBudget),
            ])
            .style(theme.accent().add_modifier(Modifier::BOLD)),
        )
        .block(theme.panel_block(text(lang, TextKey::AgentStatusTitle), true))
        .column_spacing(1);
    frame.render_widget(table, area);
}
