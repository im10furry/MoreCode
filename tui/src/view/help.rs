use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::app::AppState;
use crate::i18n::{text, TextKey};
use crate::theme::TuiTheme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    let lang = state.language();
    let lines = vec![
        Line::from(text(lang, TextKey::HelpSwitchPanels)),
        Line::from(text(lang, TextKey::HelpSwitchStreams)),
        Line::from(text(lang, TextKey::HelpCycleMode)),
        Line::from(text(lang, TextKey::HelpScroll)),
        Line::from(text(lang, TextKey::HelpToggleLanguage)),
        Line::from(text(lang, TextKey::HelpOpenSettings)),
        Line::from(text(lang, TextKey::HelpOpenHelp)),
        Line::from(text(lang, TextKey::HelpQuit)),
        Line::from("n / p: next/previous approval or patch"),
        Line::from("a / r: approve or reject selected approval"),
        Line::from("x / d: accept or reject selected patch"),
        Line::from("mouse: click tabs/settings, wheel scroll, click selected rows to act"),
        Line::from(format!(
            "{} {}",
            text(lang, TextKey::HelpActivePanel),
            state.active_panel().title(lang)
        )),
        Line::from(format!(
            "{} {}",
            text(lang, TextKey::HelpActiveStream),
            state.stream_mode().title(lang)
        )),
    ];
    let paragraph = Paragraph::new(lines)
        .block(theme.panel_block(text(lang, TextKey::HelpTitle), true))
        .scroll((state.scroll_offset(state.active_panel()), 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
