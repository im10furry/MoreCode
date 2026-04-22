use super::workflow::{load_snapshot, render_replay};
use crate::{AppContext, ReplayCommand};

pub async fn execute(context: &AppContext, command: &ReplayCommand) -> Result<String, String> {
    let snapshot = load_snapshot(context, &command.run_id)?;
    Ok(render_replay(&snapshot, command.json))
}
