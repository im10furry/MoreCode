use crate::app::AppState;

pub fn render(state: &AppState) -> String {
    let mut lines = vec!["Agent Status".to_string()];
    for (agent, status) in &state.agent_statuses {
        lines.push(format!("- {agent}: {status}"));
    }
    lines.join("\n")
}
