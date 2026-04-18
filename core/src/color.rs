use serde::{Deserialize, Serialize};

/// Terminal color value independent of any UI crate.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Color {
    /// 24-bit RGB color.
    Rgb(u8, u8, u8),
    /// Indexed 256-color value.
    Indexed(u8),
    /// Named 16-color value.
    Named(NamedColor),
}

/// Backward-compatible alias for the terminal color enum.
pub type TerminalColor = Color;

/// Named 16-color palette entry.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum NamedColor {
    /// Black.
    Black,
    /// Red.
    Red,
    /// Green.
    Green,
    /// Yellow.
    Yellow,
    /// Blue.
    Blue,
    /// Magenta.
    Magenta,
    /// Cyan.
    Cyan,
    /// White.
    White,
    /// Dark gray.
    DarkGray,
    /// Bright red.
    LightRed,
    /// Bright green.
    LightGreen,
    /// Bright yellow.
    LightYellow,
    /// Bright blue.
    LightBlue,
    /// Bright magenta.
    LightMagenta,
    /// Bright cyan.
    LightCyan,
    /// Light gray.
    LightGray,
}

/// Semantic color key used across the CLI and TUI.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum SemanticColor {
    /// Success state color.
    Success,
    /// Error state color.
    Error,
    /// Warning state color.
    Warning,
    /// Informational state color.
    Info,
    /// Tip or hint color.
    Tips,
    /// Debugging state color.
    Debug,
    /// Coordinator agent color.
    AgentCoordinator,
    /// Explorer agent color.
    AgentExplorer,
    /// Impact analyzer agent color.
    AgentImpact,
    /// Planner agent color.
    AgentPlanner,
    /// Coder agent color.
    AgentCoder,
    /// Reviewer agent color.
    AgentReviewer,
    /// Tester agent color.
    AgentTester,
    /// Research agent color.
    AgentResearch,
    /// Documentation writer agent color.
    AgentDocWriter,
    /// Debugger agent color.
    AgentDebugger,
    /// Simple routing color.
    RouteSimple,
    /// Medium routing color.
    RouteMedium,
    /// Complex routing color.
    RouteComplex,
    /// Research routing color.
    RouteResearch,
    /// Command invocation color.
    CommandRun,
    /// Command output color.
    CommandOutput,
    /// Command error color.
    CommandError,
    /// Command path color.
    CommandPath,
    /// Command argument color.
    CommandArg,
    /// Command keyword color.
    CommandKeyword,
    /// Syntax keyword color.
    SyntaxKeyword,
    /// Syntax string literal color.
    SyntaxString,
    /// Syntax numeric literal color.
    SyntaxNumber,
    /// Syntax type name color.
    SyntaxType,
    /// Syntax comment color.
    SyntaxComment,
    /// Syntax function name color.
    SyntaxFunction,
    /// Syntax macro color.
    SyntaxMacro,
    /// Syntax lifetime color.
    SyntaxLifetime,
    /// Syntax attribute color.
    SyntaxAttribute,
    /// Syntax variable color.
    SyntaxVariable,
    /// Syntax operator color.
    SyntaxOperator,
    /// Syntax punctuation color.
    SyntaxPunctuation,
    /// Git added line color.
    GitAdded,
    /// Git deleted line color.
    GitDeleted,
    /// Git modified line color.
    GitModified,
    /// Git hash color.
    GitHash,
    /// Git branch color.
    GitBranch,
    /// Git tag color.
    GitTag,
    /// Git staged state color.
    GitStaged,
    /// Git unstaged state color.
    GitUnstaged,
    /// Git untracked state color.
    GitUntracked,
    /// Title bar background color.
    TitleBar,
    /// Title text color.
    TitleText,
    /// Status bar background color.
    StatusBar,
    /// Status text color.
    StatusText,
    /// Progress bar fill color.
    ProgressBar,
    /// Progress bar background color.
    ProgressBg,
    /// Border color.
    Border,
    /// Highlighted border color.
    BorderHighlight,
    /// Selection background color.
    Selection,
    /// Hover state color.
    Hover,
    /// Input field background color.
    InputField,
    /// Input cursor color.
    InputCursor,
    /// Scroll bar color.
    ScrollBar,
    /// Scroll thumb color.
    ScrollThumb,
    /// Separator color.
    Separator,
    /// Primary text color.
    TextPrimary,
    /// Secondary text color.
    TextSecondary,
    /// Muted text color.
    TextMuted,
    /// Accent text color.
    TextAccent,
    /// Link text color.
    TextLink,
    /// Tag text color.
    TextTag,
    /// User input text color.
    TextUserInput,
    /// Primary background color.
    BgPrimary,
    /// Secondary background color.
    BgSecondary,
    /// Panel background color.
    BgPanel,
    /// Code block background color.
    BgCode,
    /// Hover background color.
    BgHover,
}

impl SemanticColor {
    /// All semantic color variants.
    pub const ALL: [Self; 74] = [
        Self::Success,
        Self::Error,
        Self::Warning,
        Self::Info,
        Self::Tips,
        Self::Debug,
        Self::AgentCoordinator,
        Self::AgentExplorer,
        Self::AgentImpact,
        Self::AgentPlanner,
        Self::AgentCoder,
        Self::AgentReviewer,
        Self::AgentTester,
        Self::AgentResearch,
        Self::AgentDocWriter,
        Self::AgentDebugger,
        Self::RouteSimple,
        Self::RouteMedium,
        Self::RouteComplex,
        Self::RouteResearch,
        Self::CommandRun,
        Self::CommandOutput,
        Self::CommandError,
        Self::CommandPath,
        Self::CommandArg,
        Self::CommandKeyword,
        Self::SyntaxKeyword,
        Self::SyntaxString,
        Self::SyntaxNumber,
        Self::SyntaxType,
        Self::SyntaxComment,
        Self::SyntaxFunction,
        Self::SyntaxMacro,
        Self::SyntaxLifetime,
        Self::SyntaxAttribute,
        Self::SyntaxVariable,
        Self::SyntaxOperator,
        Self::SyntaxPunctuation,
        Self::GitAdded,
        Self::GitDeleted,
        Self::GitModified,
        Self::GitHash,
        Self::GitBranch,
        Self::GitTag,
        Self::GitStaged,
        Self::GitUnstaged,
        Self::GitUntracked,
        Self::TitleBar,
        Self::TitleText,
        Self::StatusBar,
        Self::StatusText,
        Self::ProgressBar,
        Self::ProgressBg,
        Self::Border,
        Self::BorderHighlight,
        Self::Selection,
        Self::Hover,
        Self::InputField,
        Self::InputCursor,
        Self::ScrollBar,
        Self::ScrollThumb,
        Self::Separator,
        Self::TextPrimary,
        Self::TextSecondary,
        Self::TextMuted,
        Self::TextAccent,
        Self::TextLink,
        Self::TextTag,
        Self::TextUserInput,
        Self::BgPrimary,
        Self::BgSecondary,
        Self::BgPanel,
        Self::BgCode,
        Self::BgHover,
    ];

    /// Number of semantic color variants.
    pub const COUNT: usize = Self::ALL.len();
}

/// Theme contract that resolves semantic colors into terminal colors.
pub trait Theme: Send + Sync {
    /// Resolve a semantic color into a concrete terminal color.
    fn color(&self, semantic: SemanticColor) -> Color;
    /// Whether the theme supports true-color rendering.
    fn supports_truecolor(&self) -> bool;
    /// Human-readable theme name.
    fn name(&self) -> &str;
}

/// Default dark theme used by MoreCode.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DarkTheme;

impl Theme for DarkTheme {
    fn color(&self, semantic: SemanticColor) -> Color {
        match semantic {
            SemanticColor::Success => Color::Rgb(74, 222, 128),
            SemanticColor::Error => Color::Rgb(248, 113, 113),
            SemanticColor::Warning => Color::Rgb(251, 191, 36),
            SemanticColor::Info => Color::Rgb(96, 165, 250),
            SemanticColor::Tips => Color::Rgb(252, 211, 77),
            SemanticColor::Debug => Color::Rgb(167, 139, 250),
            SemanticColor::AgentCoordinator => Color::Rgb(244, 114, 182),
            SemanticColor::AgentExplorer => Color::Rgb(52, 211, 153),
            SemanticColor::AgentImpact => Color::Rgb(251, 146, 60),
            SemanticColor::AgentPlanner => Color::Rgb(167, 139, 250),
            SemanticColor::AgentCoder => Color::Rgb(96, 165, 250),
            SemanticColor::AgentReviewer => Color::Rgb(45, 212, 191),
            SemanticColor::AgentTester => Color::Rgb(74, 222, 128),
            SemanticColor::AgentResearch => Color::Rgb(192, 132, 252),
            SemanticColor::AgentDocWriter => Color::Rgb(249, 168, 212),
            SemanticColor::AgentDebugger => Color::Rgb(248, 113, 113),
            SemanticColor::RouteSimple => Color::Rgb(74, 222, 128),
            SemanticColor::RouteMedium => Color::Rgb(251, 191, 36),
            SemanticColor::RouteComplex => Color::Rgb(248, 113, 113),
            SemanticColor::RouteResearch => Color::Rgb(96, 165, 250),
            SemanticColor::CommandRun => Color::Rgb(74, 222, 128),
            SemanticColor::CommandOutput => Color::Rgb(147, 197, 253),
            SemanticColor::CommandError => Color::Rgb(252, 165, 165),
            SemanticColor::CommandPath => Color::Rgb(134, 239, 172),
            SemanticColor::CommandArg => Color::Rgb(253, 230, 138),
            SemanticColor::CommandKeyword => Color::Rgb(196, 181, 253),
            SemanticColor::SyntaxKeyword => Color::Rgb(192, 132, 252),
            SemanticColor::SyntaxString => Color::Rgb(74, 222, 128),
            SemanticColor::SyntaxNumber => Color::Rgb(251, 191, 36),
            SemanticColor::SyntaxType => Color::Rgb(96, 165, 250),
            SemanticColor::SyntaxComment => Color::Rgb(107, 114, 128),
            SemanticColor::SyntaxFunction => Color::Rgb(244, 114, 182),
            SemanticColor::SyntaxMacro => Color::Rgb(251, 146, 60),
            SemanticColor::SyntaxLifetime => Color::Rgb(45, 212, 191),
            SemanticColor::SyntaxAttribute => Color::Rgb(252, 211, 77),
            SemanticColor::SyntaxVariable => Color::Rgb(229, 231, 235),
            SemanticColor::SyntaxOperator => Color::Rgb(249, 168, 212),
            SemanticColor::SyntaxPunctuation => Color::Rgb(156, 163, 175),
            SemanticColor::GitAdded => Color::Rgb(74, 222, 128),
            SemanticColor::GitDeleted => Color::Rgb(248, 113, 113),
            SemanticColor::GitModified => Color::Rgb(251, 191, 36),
            SemanticColor::GitHash => Color::Rgb(252, 211, 77),
            SemanticColor::GitBranch => Color::Rgb(192, 132, 252),
            SemanticColor::GitTag => Color::Rgb(96, 165, 250),
            SemanticColor::GitStaged => Color::Rgb(52, 211, 153),
            SemanticColor::GitUnstaged => Color::Rgb(251, 146, 60),
            SemanticColor::GitUntracked => Color::Rgb(156, 163, 175),
            SemanticColor::TitleBar => Color::Rgb(30, 41, 59),
            SemanticColor::TitleText => Color::Rgb(241, 245, 249),
            SemanticColor::StatusBar => Color::Rgb(30, 41, 59),
            SemanticColor::StatusText => Color::Rgb(148, 163, 184),
            SemanticColor::ProgressBar => Color::Rgb(59, 130, 246),
            SemanticColor::ProgressBg => Color::Rgb(55, 65, 81),
            SemanticColor::Border => Color::Rgb(71, 85, 105),
            SemanticColor::BorderHighlight => Color::Rgb(96, 165, 250),
            SemanticColor::Selection => Color::Rgb(30, 58, 95),
            SemanticColor::Hover => Color::Rgb(51, 65, 85),
            SemanticColor::InputField => Color::Rgb(15, 23, 42),
            SemanticColor::InputCursor => Color::Rgb(244, 114, 182),
            SemanticColor::ScrollBar => Color::Rgb(71, 85, 105),
            SemanticColor::ScrollThumb => Color::Rgb(148, 163, 184),
            SemanticColor::Separator => Color::Rgb(51, 65, 85),
            SemanticColor::TextPrimary => Color::Rgb(241, 245, 249),
            SemanticColor::TextSecondary => Color::Rgb(148, 163, 184),
            SemanticColor::TextMuted => Color::Rgb(100, 116, 139),
            SemanticColor::TextAccent => Color::Rgb(96, 165, 250),
            SemanticColor::TextLink => Color::Rgb(147, 197, 253),
            SemanticColor::TextTag => Color::Rgb(252, 211, 77),
            SemanticColor::TextUserInput => Color::Rgb(74, 222, 128),
            SemanticColor::BgPrimary => Color::Rgb(15, 23, 42),
            SemanticColor::BgSecondary => Color::Rgb(30, 41, 59),
            SemanticColor::BgPanel => Color::Rgb(30, 41, 59),
            SemanticColor::BgCode => Color::Rgb(15, 23, 42),
            SemanticColor::BgHover => Color::Rgb(51, 65, 85),
        }
    }

    fn supports_truecolor(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "dark"
    }
}
