use mc_core::{RunSnapshot, CONFIG_FILE_NAME, PROJECT_CONFIG_SUBDIR};
use mc_tui::{Language, LogLevel, Tui, TuiHandle};
use tokio::task::JoinHandle;

use super::workflow::{
    execute_run as execute_workflow, load_snapshot, RunEventSink, WorkflowOptions,
};
use crate::{AppContext, ApprovalMode, RunCommand, TuiCommand};

struct TuiRunSink {
    handle: TuiHandle,
}

impl RunEventSink for TuiRunSink {
    fn handle_event(&self, envelope: &mc_core::RunEventEnvelope) -> Result<(), String> {
        self.handle
            .run_event(envelope.clone())
            .map_err(|error| error.to_string())
    }
}

pub async fn execute(context: &AppContext, command: &TuiCommand) -> Result<String, String> {
    if let Some(run_id) = &command.run_id {
        let snapshot = load_snapshot(context, run_id)?;
        return open_run(context, snapshot).await;
    }

    let run_command = RunCommand {
        request: command
            .request
            .clone()
            .unwrap_or_else(|| "review the current workspace".to_string()),
        ui: crate::UiMode::Tui,
        plan_only: false,
        json: false,
        approval: ApprovalMode::Prompt,
    };
    execute_run(context, &run_command).await
}

pub async fn execute_run(context: &AppContext, command: &RunCommand) -> Result<String, String> {
    if !mc_tui::Tui::is_terminal_available() {
        eprintln!("TUI unavailable (no interactive terminal), falling back to CLI mode");
        let options = super::workflow::WorkflowOptions {
            plan_only: command.plan_only,
            approval: command.approval,
        };
        let output =
            super::workflow::execute_run(context, &command.request, options, None).await?;
        return Ok(super::workflow::render_run_summary(&output));
    }

    let (tui, handle) = Tui::new("MoreCode Run");
    let mut tui = tui;
    tui.set_tick_rate(std::time::Duration::from_millis(
        context.config.tui.refresh_rate_ms.max(16),
    ));
    {
        let state = tui.app_mut().state_mut();
        let language = context.config.tui.language.trim().to_ascii_lowercase();
        match language.as_str() {
            "zh" | "zh-cn" | "zh_cn" => state.set_language(Language::ZhCn),
            "en" | "en-us" | "en_us" => state.set_language(Language::En),
            _ => {}
        }
        state.set_tick_rate_ms(context.config.tui.refresh_rate_ms);
        state.set_max_log_entries(context.config.tui.max_log_lines);
        state.set_mouse_support(context.config.tui.mouse_support);
        state.set_settings_persist_path(
            context
                .project_root
                .join(PROJECT_CONFIG_SUBDIR)
                .join(CONFIG_FILE_NAME),
        );
    }

    let worker = spawn_run_worker(context, handle.clone(), command.clone());
    let result = tui.run().await.map(|_| String::new());
    worker.abort();

    match result {
        Ok(output) => Ok(output),
        Err(error) => {
            let msg = error.to_string();
            if msg.contains("Initial console modes not set") || msg.contains("terminal io failed")
            {
                eprintln!("TUI unavailable ({msg}), falling back to CLI mode");
                let options = super::workflow::WorkflowOptions {
                    plan_only: command.plan_only,
                    approval: command.approval,
                };
                let output =
                    super::workflow::execute_run(context, &command.request, options, None).await?;
                Ok(super::workflow::render_run_summary(&output))
            } else {
                Err(msg)
            }
        }
    }
}

pub async fn open_run(context: &AppContext, snapshot: RunSnapshot) -> Result<String, String> {
    if !mc_tui::Tui::is_terminal_available() {
        eprintln!("TUI unavailable (no interactive terminal), falling back to CLI mode");
        return Ok(super::workflow::render_review(&snapshot));
    }

    let (tui, _handle) = Tui::new("MoreCode Review");
    let mut tui = tui;
    tui.set_tick_rate(std::time::Duration::from_millis(
        context.config.tui.refresh_rate_ms.max(16),
    ));
    {
        let state = tui.app_mut().state_mut();
        let language = context.config.tui.language.trim().to_ascii_lowercase();
        match language.as_str() {
            "zh" | "zh-cn" | "zh_cn" => state.set_language(Language::ZhCn),
            "en" | "en-us" | "en_us" => state.set_language(Language::En),
            _ => {}
        }
        state.set_tick_rate_ms(context.config.tui.refresh_rate_ms);
        state.set_max_log_entries(context.config.tui.max_log_lines);
        state.set_mouse_support(context.config.tui.mouse_support);
        state.set_settings_persist_path(
            context
                .project_root
                .join(PROJECT_CONFIG_SUBDIR)
                .join(CONFIG_FILE_NAME),
        );
    }
    tui.app_mut().load_run(snapshot.clone());
    match tui.run().await {
        Ok(_) => Ok(String::new()),
        Err(error) => {
            let msg = error.to_string();
            if msg.contains("Initial console modes not set") || msg.contains("terminal io failed")
            {
                eprintln!("TUI unavailable ({msg}), falling back to CLI mode");
                Ok(super::workflow::render_review(&snapshot))
            } else {
                Err(msg)
            }
        }
    }
}

fn spawn_run_worker(
    context: &AppContext,
    handle: TuiHandle,
    command: RunCommand,
) -> JoinHandle<()> {
    let project_root = context.project_root.clone();
    let config = context.config.clone();
    tokio::spawn(async move {
        let memory = match mc_memory::MemorySystem::new(&project_root).await {
            Ok(memory) => memory,
            Err(error) => {
                let _ = handle.log(LogLevel::Error, format!("memory: {error}"));
                return;
            }
        };
        let context = AppContext {
            cwd: project_root.clone(),
            project_root,
            config,
            memory,
        };
        let sink = TuiRunSink {
            handle: handle.clone(),
        };
        let options = WorkflowOptions {
            plan_only: command.plan_only,
            approval: command.approval,
        };
        if let Err(error) = execute_workflow(&context, &command.request, options, Some(&sink)).await
        {
            let _ = handle.log(LogLevel::Error, error);
        }
    })
}
