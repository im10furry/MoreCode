use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use mc_communication::{
    ApprovalRequest, ApprovalResponse, BroadcastEvent, ControlMessage, StateMessage,
};

use crate::app::StreamMode;

/// Keyboard actions supported by the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    NextPanel,
    PreviousPanel,
    NextMode,
    PreviousMode,
    SetStreamMode(StreamMode),
    ScrollUp,
    ScrollDown,
    ToggleLanguage,
    Settings,
    SettingInc,
    SettingDec,
    ToggleSetting,
    Help,
    Quit,
    NextProject,
    PreviousProject,
    NextProjectMode,
    PreviousProjectMode,
}

impl KeyAction {
    /// Translate a crossterm key event into a semantic TUI action.
    pub fn from_crossterm(event: KeyEvent) -> Option<Self> {
        if !matches!(event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return None;
        }

        match (event.code, event.modifiers) {
            (KeyCode::Tab, _) | (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
                Some(Self::NextPanel)
            }
            (KeyCode::BackTab, _) | (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
                Some(Self::PreviousPanel)
            }
            (KeyCode::Right, KeyModifiers::ALT) => Some(Self::NextProject),
            (KeyCode::Left, KeyModifiers::ALT) => Some(Self::PreviousProject),
            (KeyCode::Up, KeyModifiers::ALT) => Some(Self::NextProjectMode),
            (KeyCode::Down, KeyModifiers::ALT) => Some(Self::PreviousProjectMode),
            (KeyCode::Char(']'), _) => Some(Self::NextMode),
            (KeyCode::Char('['), _) => Some(Self::PreviousMode),
            (KeyCode::Char('1'), _) => Some(Self::SetStreamMode(StreamMode::Progress)),
            (KeyCode::Char('2'), _) => Some(Self::SetStreamMode(StreamMode::Code)),
            (KeyCode::Char('3'), _) => Some(Self::SetStreamMode(StreamMode::Confirmation)),
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => Some(Self::ScrollUp),
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => Some(Self::ScrollDown),
            (KeyCode::Char('t'), _) => Some(Self::ToggleLanguage),
            (KeyCode::Char('s'), _) => Some(Self::Settings),
            (KeyCode::Char('+'), _) | (KeyCode::Char('='), _) => Some(Self::SettingInc),
            (KeyCode::Char('-'), _) => Some(Self::SettingDec),
            (KeyCode::Enter, _) => Some(Self::ToggleSetting),
            (KeyCode::Char('?'), _) | (KeyCode::F(1), _) => Some(Self::Help),
            (KeyCode::Esc, _)
            | (KeyCode::Char('q'), _)
            | (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Self::Quit),
            _ => None,
        }
    }
}

/// Log severity shown in the log panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

impl LogLevel {
    /// Stable short label for compact UI rendering.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Debug => "DEBUG",
        }
    }
}

/// External updates pushed into the TUI from the runtime or coordinator.
#[derive(Debug, Clone, PartialEq)]
pub enum TuiUpdate {
    Control(ControlMessage),
    State(StateMessage),
    Broadcast(BroadcastEvent),
    ApprovalRequest(ApprovalRequest),
    ApprovalResponse(ApprovalResponse),
    Log { level: LogLevel, message: String },
}

/// Application events consumed by the state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    Tick,
    Resize { width: u16, height: u16 },
    Key(KeyAction),
    Update(Box<TuiUpdate>),
}

impl AppEvent {
    /// Translate a crossterm event into a TUI event when it is relevant.
    pub fn from_crossterm(event: Event) -> Option<Self> {
        match event {
            Event::Key(key) => KeyAction::from_crossterm(key).map(Self::Key),
            Event::Resize(width, height) => Some(Self::Resize { width, height }),
            _ => None,
        }
    }
}
