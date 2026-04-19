use crate::cli::DaemonCommand;
use crate::init::AppContext;

pub async fn execute(_context: &AppContext, command: &DaemonCommand) -> Result<String, String> {
    match command {
        DaemonCommand::Status => Ok(
            "daemon runtime is not wired yet; config and memory subsystems are available"
                .to_string(),
        ),
    }
}
