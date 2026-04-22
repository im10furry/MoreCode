use std::io::Write;

use super::workflow::{execute_run, render_run_summary, RunEventSink, WorkflowOptions};
use crate::{AppContext, RunCommand, UiMode};

struct JsonSink;

impl RunEventSink for JsonSink {
    fn handle_event(&self, envelope: &mc_core::RunEventEnvelope) -> Result<(), String> {
        let line = serde_json::to_string(envelope).map_err(|error| error.to_string())?;
        println!("{line}");
        std::io::stdout().flush().map_err(|error| error.to_string())
    }
}

pub async fn execute(context: &AppContext, command: &RunCommand) -> Result<String, String> {
    match command.ui {
        UiMode::Cli => {
            let options = WorkflowOptions {
                plan_only: command.plan_only,
                approval: command.approval,
            };
            let output = if command.json {
                let sink = JsonSink;
                execute_run(context, &command.request, options, Some(&sink)).await?
            } else {
                execute_run(context, &command.request, options, None).await?
            };
            if command.json {
                Ok(String::new())
            } else {
                Ok(render_run_summary(&output))
            }
        }
        UiMode::Tui => super::tui::execute_run(context, command).await,
        UiMode::Web => super::web::execute_from_run(context, command).await,
    }
}
