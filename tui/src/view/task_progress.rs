use crate::app::AppState;
use crate::widget::progress_bar::render_progress_bar;

pub fn render(state: &AppState) -> String {
    format!(
        "Task Progress\n{}",
        render_progress_bar(24, state.task_progress_percent)
    )
}
