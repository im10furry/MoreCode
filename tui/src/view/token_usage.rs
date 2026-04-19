use crate::app::AppState;
use crate::widget::sparkline::render_sparkline;

pub fn render(state: &AppState) -> String {
    let values = [0, state.token_usage_total / 2, state.token_usage_total];
    format!(
        "Token Usage\n{}\n{}",
        state.token_usage_total,
        render_sparkline(&values)
    )
}
