use crate::app::AppState;

pub fn render(state: &AppState) -> String {
    let mut lines = vec!["Log".to_string()];
    lines.extend(state.logs.iter().cloned());
    lines.join("\n")
}
