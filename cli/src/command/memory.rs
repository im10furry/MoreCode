use crate::cli::MemoryCommand;
use crate::init::AppContext;

pub async fn execute(context: &AppContext, command: &MemoryCommand) -> Result<String, String> {
    match command {
        MemoryCommand::Status => {
            let state = context
                .memory
                .load_project_memory_state()
                .await
                .map_err(|error| error.to_string())?;
            Ok(format!("{state:?}"))
        }
        MemoryCommand::Summary => context
            .memory
            .memory_summary()
            .await
            .map_err(|error| error.to_string()),
        MemoryCommand::Refresh => {
            let state = context
                .memory
                .refresh_project_memory()
                .await
                .map_err(|error| error.to_string())?;
            Ok(format!("memory refreshed: {state:?}"))
        }
        MemoryCommand::Clear => {
            let memory_dir = context.project_root.join(".assistant-memory");
            if tokio::fs::try_exists(&memory_dir)
                .await
                .map_err(|error| error.to_string())?
            {
                tokio::fs::remove_dir_all(&memory_dir)
                    .await
                    .map_err(|error| error.to_string())?;
            }
            Ok(format!("removed {}", memory_dir.display()))
        }
    }
}
