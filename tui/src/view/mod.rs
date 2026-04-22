pub mod agent_status;
pub mod communication;
pub mod dashboard;
pub mod help;
pub mod log;
pub mod project;
pub mod settings;
pub mod task_progress;
pub mod token_usage;

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::app::{AppState, Panel};
use crate::theme::TuiTheme;

pub fn render_active_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    match state.active_panel {
        Panel::Dashboard => dashboard::render(frame, area, state, theme),
        Panel::AgentStatus => agent_status::render(frame, area, state, theme),
        Panel::TaskProgress => task_progress::render(frame, area, state, theme),
        Panel::Communication => communication::render(frame, area, state, theme),
        Panel::TokenUsage => token_usage::render(frame, area, state, theme),
        Panel::Log => log::render(frame, area, state, theme),
        Panel::Projects => project::render(frame, area, state, theme),
        Panel::Settings => settings::render(frame, area, state, theme),
        Panel::Help => help::render(frame, area, state, theme),
    }
}
