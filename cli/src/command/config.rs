use crate::cli::ConfigCommand;
use crate::init::AppContext;

pub async fn execute(context: &AppContext, command: &ConfigCommand) -> Result<String, String> {
    match command {
        ConfigCommand::Show => {
            serde_json::to_string_pretty(&context.config).map_err(|error| error.to_string())
        }
    }
}
