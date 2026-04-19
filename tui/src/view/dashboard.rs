use crate::app::AppState;

pub fn render(state: &AppState) -> String {
    format!(
        "Dashboard\nprogress: {}%\ntokens: {}\nagents: {}",
        state.task_progress_percent,
        state.token_usage_total,
        state.agent_statuses.len()
    )
}
