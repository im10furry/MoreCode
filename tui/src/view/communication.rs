use std::collections::BTreeMap;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::widgets::{Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

use crate::app::{AppState, Endpoint};
use crate::theme::TuiTheme;
use crate::widget::tree::{render_tree, TreeNode};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
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
        Row::new(vec!["Count", "Kind", "From", "To", "Summary"])
            .style(theme.accent().add_modifier(Modifier::BOLD)),
    )
    .block(theme.panel_block("Communication Edges", true))
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
        .block(theme.panel_block("Topology View", false))
        .scroll((state.scroll_offset(state.active_panel()), 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, sections[1]);
}
