use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::app::AppState;
use crate::theme::TuiTheme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    let lines = vec![
        Line::from("Tab / Shift+Tab: switch panels"),
        Line::from("1 / 2 / 3: progress, code, confirmation"),
        Line::from("[ / ]: cycle feedback mode"),
        Line::from("j / k or arrows: scroll"),
        Line::from("?: open help"),
        Line::from("q or Esc: quit"),
        Line::from(format!("active panel: {}", state.active_panel().title())),
        Line::from(format!("active stream: {}", state.stream_mode().title())),
    ];
    let paragraph = Paragraph::new(lines)
        .block(theme.panel_block("Help", true))
        .scroll((state.scroll_offset(state.active_panel()), 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
