use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Tabs, Wrap};
use ratatui::Frame;

use crate::app::{AppState, Panel};
use crate::i18n::{text, TextKey};
use crate::theme::TuiTheme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: TuiTheme) {
    let lang = state.language();
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let projects = state.project_manager().projects.iter()
        .map(|project| Line::from(format!("{} ({})", project.info.name, project.mode.as_str())))
        .collect::<Vec<_>>();

    let active_index = state.project_manager().active_project_index.unwrap_or(0);
    let tabs = Tabs::new(projects)
        .select(active_index)
        .block(theme.panel_block(text(lang, TextKey::PanelProjects), false))
        .style(theme.muted())
        .highlight_style(theme.accent());
    frame.render_widget(tabs, sections[0]);

    let mut lines = Vec::new();
    
    if let Some(project) = state.project_manager().active_project() {
        lines.push(Line::from(format!("{}: {}", text(lang, TextKey::ProjectName), project.info.name)));
        lines.push(Line::from(format!("{}: {}", text(lang, TextKey::ProjectPath), project.root_path.to_string_lossy())));
        lines.push(Line::from(format!("{}: {}", text(lang, TextKey::ProjectMode), project.mode.as_str())));
        lines.push(Line::from(format!("{}: {}", text(lang, TextKey::ProjectStatus), match project.status {
            crate::mc_core::ProjectStatus::Active => text(lang, TextKey::StatusActive),
            crate::mc_core::ProjectStatus::Paused => text(lang, TextKey::StatusPaused),
            crate::mc_core::ProjectStatus::Completed => text(lang, TextKey::StatusCompleted),
            crate::mc_core::ProjectStatus::Error => text(lang, TextKey::StatusError),
        })));
        lines.push(Line::from(format!("{}: {}", text(lang, TextKey::ProjectLanguage), project.info.language)));
        if let Some(version) = &project.info.version {
            lines.push(Line::from(format!("{}: {}", text(lang, TextKey::ProjectVersion), version)));
        }
        if let Some(framework) = &project.info.framework {
            lines.push(Line::from(format!("{}: {}", text(lang, TextKey::ProjectFramework), framework)));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(format!("{}: {}", text(lang, TextKey::ProjectTasks), project.task_ids.len())));
    } else {
        lines.push(Line::from(text(lang, TextKey::ProjectNoActive)));
        lines.push(Line::from(text(lang, TextKey::ProjectAddHint)));
    }

    let body = Paragraph::new(lines)
        .block(theme.panel_block(text(lang, TextKey::ProjectDetails), true))
        .scroll((state.scroll_offset(Panel::Projects), 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(body, sections[1]);
}