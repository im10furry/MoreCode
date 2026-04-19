use crate::app::AppState;

pub fn render(state: &AppState) -> String {
    let mut lines = vec!["Communication".to_string()];
    for (from, to) in &state.communication_edges {
        lines.push(format!("{from} -> {to}"));
    }
    lines.join("\n")
}
