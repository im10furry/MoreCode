use super::workflow::{export_snapshot, load_snapshot};
use crate::{AppContext, ExportCommand};

pub async fn execute(context: &AppContext, command: &ExportCommand) -> Result<String, String> {
    let snapshot = load_snapshot(context, &command.run_id)?;
    let path = export_snapshot(context, &snapshot, command.format)?;
    Ok(path.display().to_string())
}
