use super::workflow::{load_snapshot, render_review};
use crate::{AppContext, ReviewCommand, UiMode};

pub async fn execute(context: &AppContext, command: &ReviewCommand) -> Result<String, String> {
    let snapshot = load_snapshot(context, &command.run_id)?;
    match command.ui {
        UiMode::Cli => Ok(render_review(&snapshot)),
        UiMode::Tui => super::tui::open_run(context, snapshot).await,
        UiMode::Web => super::web::execute_from_review(context, command).await,
    }
}
