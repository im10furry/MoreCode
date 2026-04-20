use mc_core::AgentType;

use crate::error::TuiError;
use crate::event::{AppEvent, KeyAction};
use crate::theme::TuiTheme;
use crate::view;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Dashboard,
    AgentStatus,
    TaskProgress,
    Communication,
    TokenUsage,
    Log,
    Help,
}

impl Panel {
    pub const ALL: [Panel; 7] = [
        Panel::Dashboard,
        Panel::AgentStatus,
        Panel::TaskProgress,
        Panel::Communication,
        Panel::TokenUsage,
        Panel::Log,
        Panel::Help,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub active_panel: Panel,
    pub title: String,
    pub logs: Vec<String>,
    pub task_progress_percent: u8,
    pub token_usage_total: u64,
    pub communication_edges: Vec<(AgentType, AgentType)>,
    pub agent_statuses: Vec<(AgentType, String)>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_panel: Panel::Dashboard,
            title: "MoreCode".into(),
            logs: Vec::new(),
            task_progress_percent: 0,
            token_usage_total: 0,
            communication_edges: Vec::new(),
            agent_statuses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFrame {
    pub title: String,
    pub body: String,
    pub footer: String,
}

pub struct App {
    state: AppState,
    theme: TuiTheme,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
            theme: TuiTheme::default(),
        }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    pub fn handle_event(&mut self, event: AppEvent) -> Result<(), TuiError> {
        match event {
            AppEvent::Tick => Ok(()),
            AppEvent::Resize { width, height } if width == 0 || height == 0 => Err(
                TuiError::InvalidLayout("terminal size must be greater than zero".into()),
            ),
            AppEvent::Resize { .. } => Ok(()),
            AppEvent::Key(KeyAction::NextPanel) => {
                self.state.active_panel = next_panel(self.state.active_panel);
                Ok(())
            }
            AppEvent::Key(KeyAction::PreviousPanel) => {
                self.state.active_panel = previous_panel(self.state.active_panel);
                Ok(())
            }
            AppEvent::Key(KeyAction::ScrollUp) => Ok(()),
            AppEvent::Key(KeyAction::ScrollDown) => Ok(()),
            AppEvent::Key(KeyAction::Help) => {
                self.state.active_panel = Panel::Help;
                Ok(())
            }
            AppEvent::Key(KeyAction::Quit) => Ok(()),
        }
    }

    pub fn render(&self) -> Result<RenderFrame, TuiError> {
        let body = match self.state.active_panel {
            Panel::Dashboard => view::dashboard::render(&self.state),
            Panel::AgentStatus => view::agent_status::render(&self.state),
            Panel::TaskProgress => view::task_progress::render(&self.state),
            Panel::Communication => view::communication::render(&self.state),
            Panel::TokenUsage => view::token_usage::render(&self.state),
            Panel::Log => view::log::render(&self.state),
            Panel::Help => view::help::render(&self.state),
        };

        Ok(RenderFrame {
            title: format!("{} [{:?}]", self.state.title, self.state.active_panel),
            body,
            footer: format!("theme={} | n/next p/prev h/help q/quit", self.theme.name()),
        })
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn next_panel(current: Panel) -> Panel {
    let index = Panel::ALL
        .iter()
        .position(|panel| *panel == current)
        .unwrap_or(0);
    Panel::ALL[(index + 1) % Panel::ALL.len()]
}

fn previous_panel(current: Panel) -> Panel {
    let index = Panel::ALL
        .iter()
        .position(|panel| *panel == current)
        .unwrap_or(0);
    Panel::ALL[(index + Panel::ALL.len() - 1) % Panel::ALL.len()]
}

#[cfg(test)]
mod tests {
    use crate::event::{AppEvent, KeyAction};

    use super::{App, Panel};

    #[test]
    fn app_cycles_panels() {
        let mut app = App::new();
        assert_eq!(app.state().active_panel, Panel::Dashboard);
        app.handle_event(AppEvent::Key(KeyAction::NextPanel))
            .unwrap();
        assert_eq!(app.state().active_panel, Panel::AgentStatus);
        app.handle_event(AppEvent::Key(KeyAction::PreviousPanel))
            .unwrap();
        assert_eq!(app.state().active_panel, Panel::Dashboard);
    }
}
