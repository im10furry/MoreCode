use mc_core::{Color, DarkTheme, SemanticColor, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRole {
    Title,
    Border,
    Accent,
    Success,
    Warning,
    Error,
    Text,
    Muted,
}

#[derive(Debug, Clone, Copy)]
pub struct TuiTheme {
    inner: DarkTheme,
}

impl TuiTheme {
    pub fn dark() -> Self {
        Self { inner: DarkTheme }
    }

    pub fn name(&self) -> &str {
        self.inner.name()
    }

    pub fn color(&self, role: ColorRole) -> Color {
        self.inner.color(match role {
            ColorRole::Title => SemanticColor::TitleText,
            ColorRole::Border => SemanticColor::Border,
            ColorRole::Accent => SemanticColor::TextAccent,
            ColorRole::Success => SemanticColor::Success,
            ColorRole::Warning => SemanticColor::Warning,
            ColorRole::Error => SemanticColor::Error,
            ColorRole::Text => SemanticColor::TextPrimary,
            ColorRole::Muted => SemanticColor::TextMuted,
        })
    }
}

impl Default for TuiTheme {
    fn default() -> Self {
        Self::dark()
    }
}
