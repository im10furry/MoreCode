use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::app::AppState;
use crate::i18n::{text, Language, TextKey};
use crate::theme::TuiTheme;

fn format_language(lang: Language) -> &'static str {
    match lang {
        Language::En => "English",
        Language::ZhCn => "中文",
    }
}

fn yes_no(lang: Language, value: bool) -> &'static str {
    match (lang, value) {
        (Language::ZhCn, true) => "开",
        (Language::ZhCn, false) => "关",
        (_, true) => "on",
        (_, false) => "off",
    }
}

fn row(selected: bool, theme: TuiTheme, label: &'static str, value: String) -> Line<'static> {
    let value_span = if selected {
        Span::styled(value, theme.accent().add_modifier(Modifier::BOLD))
    } else {
        Span::styled(value, theme.text())
    };
    Line::from(vec![
        Span::raw("  "),
        Span::styled(label, theme.muted()),
        Span::raw(": "),
        value_span,
    ])
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    let lang = state.language();
    let selected = state.settings_index();

    let lines = vec![
        row(
            selected == 0,
            theme,
            text(lang, TextKey::SettingsItemLanguage),
            format_language(lang).to_string(),
        ),
        row(
            selected == 1,
            theme,
            text(lang, TextKey::SettingsItemTickRate),
            state.tick_rate_ms().to_string(),
        ),
        row(
            selected == 2,
            theme,
            text(lang, TextKey::SettingsItemMaxLogs),
            state.max_log_entries().to_string(),
        ),
        row(
            selected == 3,
            theme,
            text(lang, TextKey::SettingsItemMouse),
            yes_no(lang, state.mouse_support()).to_string(),
        ),
        Line::from(""),
        Line::from(Span::styled(
            text(lang, TextKey::SettingsHint),
            theme.muted(),
        )),
        Line::from(Span::styled(
            text(lang, TextKey::SettingsPersistHint),
            theme.muted(),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(theme.panel_block(text(lang, TextKey::SettingsTitle), true))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
