use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::app::AppState;
use crate::i18n::{text, TextKey};
use crate::theme::TuiTheme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    let lang = state.language();
    let lines = state
        .logs()
        .iter()
        .map(|entry| Line::from(format!("[{}] {}", entry.level.label(), entry.message)))
        .collect::<Vec<_>>();
    let paragraph = Paragraph::new(if lines.is_empty() {
        vec![Line::from(text(lang, TextKey::EmptyNoLogs))]
    } else {
        lines
    })
    .block(theme.panel_block(text(lang, TextKey::PanelLogs), true))
    .scroll((state.scroll_offset(state.active_panel()), 0))
    .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
