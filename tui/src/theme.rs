use mc_core::{
    AgentExecutionStatus, AgentType, Color, DarkTheme, NamedColor, SemanticColor, Theme,
};
use ratatui::style::{Color as RColor, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders};

/// High-level color roles used by the TUI.
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

/// Thin theme adapter that bridges `mc-core` semantic colors into Ratatui styles.
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

    pub fn ratatui_color(&self, color: Color) -> RColor {
        match color {
            Color::Rgb(red, green, blue) => RColor::Rgb(red, green, blue),
            Color::Indexed(index) => RColor::Indexed(index),
            Color::Named(color) => named_color(color),
        }
    }

    pub fn semantic_color(&self, color: SemanticColor) -> RColor {
        self.ratatui_color(self.inner.color(color))
    }

    pub fn style(&self, role: ColorRole) -> Style {
        Style::default().fg(self.ratatui_color(self.color(role)))
    }

    pub fn background_style(&self) -> Style {
        Style::default()
            .fg(self.semantic_color(SemanticColor::TextPrimary))
            .bg(self.semantic_color(SemanticColor::BgPrimary))
    }

    pub fn text(&self) -> Style {
        self.style(ColorRole::Text)
    }

    pub fn muted(&self) -> Style {
        self.style(ColorRole::Muted)
    }

    pub fn accent(&self) -> Style {
        self.style(ColorRole::Accent)
    }

    pub fn warning(&self) -> Style {
        self.style(ColorRole::Warning).add_modifier(Modifier::BOLD)
    }

    pub fn error(&self) -> Style {
        self.style(ColorRole::Error).add_modifier(Modifier::BOLD)
    }

    pub fn status_style(&self, status: AgentExecutionStatus) -> Style {
        match status {
            AgentExecutionStatus::Pending => self.muted(),
            AgentExecutionStatus::Running => self.accent().add_modifier(Modifier::BOLD),
            AgentExecutionStatus::Completed => {
                self.style(ColorRole::Success).add_modifier(Modifier::BOLD)
            }
            AgentExecutionStatus::Failed | AgentExecutionStatus::Cancelled => self.error(),
        }
    }

    pub fn agent_style(&self, agent_type: AgentType) -> Style {
        Style::default().fg(self.semantic_color(agent_semantic_color(agent_type)))
    }

    pub fn panel_block<'a>(&self, title: impl Into<Line<'a>>, active: bool) -> Block<'a> {
        let border_color = if active {
            SemanticColor::BorderHighlight
        } else {
            SemanticColor::Border
        };

        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.semantic_color(border_color)))
            .style(
                Style::default()
                    .fg(self.semantic_color(SemanticColor::TextPrimary))
                    .bg(self.semantic_color(SemanticColor::BgPanel)),
            )
    }
}

impl Default for TuiTheme {
    fn default() -> Self {
        Self::dark()
    }
}

fn named_color(color: NamedColor) -> RColor {
    match color {
        NamedColor::Black => RColor::Black,
        NamedColor::Red => RColor::Red,
        NamedColor::Green => RColor::Green,
        NamedColor::Yellow => RColor::Yellow,
        NamedColor::Blue => RColor::Blue,
        NamedColor::Magenta => RColor::Magenta,
        NamedColor::Cyan => RColor::Cyan,
        NamedColor::White => RColor::White,
        NamedColor::DarkGray => RColor::DarkGray,
        NamedColor::LightRed => RColor::LightRed,
        NamedColor::LightGreen => RColor::LightGreen,
        NamedColor::LightYellow => RColor::LightYellow,
        NamedColor::LightBlue => RColor::LightBlue,
        NamedColor::LightMagenta => RColor::LightMagenta,
        NamedColor::LightCyan => RColor::LightCyan,
        NamedColor::LightGray => RColor::Gray,
    }
}

fn agent_semantic_color(agent_type: AgentType) -> SemanticColor {
    match agent_type {
        AgentType::Coordinator => SemanticColor::AgentCoordinator,
        AgentType::Explorer => SemanticColor::AgentExplorer,
        AgentType::ImpactAnalyzer => SemanticColor::AgentImpact,
        AgentType::Planner => SemanticColor::AgentPlanner,
        AgentType::Coder => SemanticColor::AgentCoder,
        AgentType::Reviewer => SemanticColor::AgentReviewer,
        AgentType::Tester => SemanticColor::AgentTester,
        AgentType::Debugger => SemanticColor::AgentDebugger,
        AgentType::Research => SemanticColor::AgentResearch,
        AgentType::DocWriter => SemanticColor::AgentDocWriter,
    }
}
