use crate::app::AppState;

pub fn render(_state: &AppState) -> String {
    [
        "Help",
        "n: next panel",
        "p: previous panel",
        "h: help",
        "q: quit",
    ]
    .join("\n")
}
