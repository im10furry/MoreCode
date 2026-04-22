use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use mc_agent::registry_min::AgentRegistry;
use mc_communication::{BroadcastEvent, StateMessage};
use mc_config::ResolvedProviderEntry;
use mc_coordinator::{AgentExecutionState, Coordinator, CoordinatorConfig, ExecutionStatus};
use mc_core::{AgentExecutionReport, AgentType, ResultType, TaskResult};
use mc_llm::{
    AnthropicProvider, AnthropicProviderConfig, GoogleProvider, GoogleProviderConfig, LlmProvider,
    ModelInfo, OpenAiProvider, OpenAiProviderConfig,
};
use mc_tui::{LogLevel, Tui, TuiHandle};
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;

use crate::init::AppContext;

pub async fn execute(context: &AppContext, request: Option<&str>) -> Result<String, String> {
    let (tui, handle) = Tui::new("MoreCode TUI");
    let mut tui = tui;
    tui.set_tick_rate(Duration::from_millis(context.config.tui.refresh_rate_ms.max(16)));

    let cancel = CancellationToken::new();
    let worker = match request.map(str::trim).filter(|value| !value.is_empty()) {
        Some(request) => Some(spawn_coordinator_job(
            context,
            handle.clone(),
            request.to_string(),
            cancel.clone(),
        )),
        None => {
            let _ = handle.log(
                LogLevel::Info,
                "No request provided. Usage: morecode tui <request>",
            );
            None
        }
    };

    let result = tui.run().await.map(|exit| exit.to_string());
    cancel.cancel();
    if let Some(worker) = worker {
        worker.abort();
    }
    result.map_err(|error| error.to_string())
}

fn spawn_coordinator_job(
    context: &AppContext,
    handle: TuiHandle,
    request: String,
    cancel: CancellationToken,
) -> JoinHandle<()> {
    let config = context.config.clone();
    let project_root = context.project_root.clone();

    tokio::spawn(async move {
        let task_id = format!("run-{}", uuid::Uuid::new_v4());
        let _ = handle.log(LogLevel::Info, format!("request: {request}"));

        let resolved = match config.provider.resolve_default_provider() {
            Some(resolved) => resolved,
            None => {
                let _ = handle.log(LogLevel::Error, "provider: cannot resolve default provider");
                return;
            }
        };

        let llm = match build_llm_provider(resolved) {
            Ok(provider) => provider,
            Err(error) => {
                let _ = handle.log(LogLevel::Error, format!("provider: {error}"));
                return;
            }
        };

        let registry = Arc::new(AgentRegistry::new());
        registry.register_defaults();

        let coordinator = match Coordinator::new(
            CoordinatorConfig {
                max_token_budget: config.coordinator.max_token_budget,
                max_recursion_depth: config.coordinator.max_recursion_depth,
                agent_timeout_secs: config.coordinator.agent_timeout_secs,
                max_retries: config.coordinator.max_retries,
                memory_aware_routing: config.coordinator.memory_aware_routing,
                recursive_orchestration: config.coordinator.recursive_orchestration,
                memory_stale_threshold_days: config.coordinator.memory_stale_threshold_days,
                preflight_check: config.coordinator.preflight_check,
                llm_weight_multiplier: config.coordinator.llm_weight_multiplier,
            },
            llm,
            registry,
            project_root,
        ) {
            Ok(coordinator) => Arc::new(coordinator),
            Err(error) => {
                let _ = handle.log(LogLevel::Error, format!("coordinator: {error}"));
                return;
            }
        };

        let status_task = tokio::spawn(poll_execution_status(
            coordinator.clone(),
            handle.clone(),
            task_id.clone(),
            cancel.clone(),
        ));

        let result = coordinator.handle_request(&request).await;
        cancel.cancel();
        status_task.abort();

        match result {
            Ok(response) => {
                let content = response.content;
                let _ = handle.log(LogLevel::Info, content.clone());
                let _ = handle.broadcast(BroadcastEvent::SystemNotification {
                    level: "info".to_string(),
                    message: "request completed".to_string(),
                });
                let _ = handle.state(StateMessage::TaskCompleted {
                    task_id,
                    agent_type: AgentType::Coordinator,
                    result: TaskResult {
                        result_type: ResultType::AnalysisReport,
                        success: true,
                        data: serde_json::json!({ "type": format!("{:?}", response.response_type) }),
                        changed_files: Vec::new(),
                        generated_content: Some(content),
                        error_message: None,
                    },
                    handoff: AgentExecutionReport {
                        title: "Coordinator finished".to_string(),
                        key_findings: Vec::new(),
                        relevant_files: Vec::new(),
                        recommendations: Vec::new(),
                        warnings: Vec::new(),
                        token_used: 0,
                        timestamp: Utc::now(),
                        extra: None,
                    },
                    token_used: 0,
                });
            }
            Err(error) => {
                let _ = handle.log(LogLevel::Error, error.to_string());
                let _ = handle.broadcast(BroadcastEvent::SystemNotification {
                    level: "error".to_string(),
                    message: "request failed".to_string(),
                });
                let _ = handle.state(StateMessage::TaskFailed {
                    task_id,
                    agent_type: AgentType::Coordinator,
                    error: error.to_string(),
                    retry_count: 0,
                    can_retry: false,
                });
            }
        }
    })
}

async fn poll_execution_status(
    coordinator: Arc<Coordinator>,
    handle: TuiHandle,
    task_id: String,
    cancel: CancellationToken,
) {
    let mut ticker = interval(Duration::from_millis(200));
    let mut last_phase: Option<String> = None;
    let mut last_overall: u8 = 0;
    let mut last_agents: HashMap<AgentType, AgentExecutionState> = HashMap::new();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = ticker.tick() => {}
        }

        let Some(status) = coordinator.execution_status().await else {
            continue;
        };
        apply_status_snapshot(
            &handle,
            &task_id,
            status,
            &mut last_phase,
            &mut last_overall,
            &mut last_agents,
        );
    }
}

fn apply_status_snapshot(
    handle: &TuiHandle,
    task_id: &str,
    status: ExecutionStatus,
    last_phase: &mut Option<String>,
    last_overall: &mut u8,
    last_agents: &mut HashMap<AgentType, AgentExecutionState>,
) {
    let phase = format!("{:?}", status.current_phase).to_ascii_lowercase();
    let overall = (status.progress_percent.max(0.0).min(1.0) * 100.0).round() as u8;

    let phase_changed = last_phase.as_deref() != Some(phase.as_str());
    let overall_changed = overall != *last_overall;
    if phase_changed {
        *last_phase = Some(phase.clone());
    }
    if overall_changed {
        *last_overall = overall;
    }

    for runtime in status.agent_statuses.values() {
        let agent_type = runtime.agent_type;
        let previous = last_agents.get(&agent_type).cloned();
        let state_changed = previous.as_ref() != Some(&runtime.state);
        if !(state_changed || phase_changed || (overall_changed && overall % 2 == 0)) {
            continue;
        }

        last_agents.insert(agent_type, runtime.state.clone());

        match &runtime.state {
            AgentExecutionState::Pending => {}
            AgentExecutionState::Running => {
                let _ = handle.state(StateMessage::Progress {
                    task_id: task_id.to_string(),
                    agent_type,
                    phase: phase.clone(),
                    progress_percent: overall,
                    message: String::new(),
                });
            }
            AgentExecutionState::Completed => {
                let _ = handle.state(StateMessage::TaskCompleted {
                    task_id: task_id.to_string(),
                    agent_type,
                    result: TaskResult {
                        result_type: ResultType::AnalysisReport,
                        success: true,
                        data: serde_json::json!({}),
                        changed_files: Vec::new(),
                        generated_content: None,
                        error_message: None,
                    },
                    handoff: AgentExecutionReport {
                        title: format!("{agent_type} completed"),
                        key_findings: Vec::new(),
                        relevant_files: Vec::new(),
                        recommendations: Vec::new(),
                        warnings: Vec::new(),
                        token_used: runtime.tokens_used.min(u32::MAX as usize) as u32,
                        timestamp: Utc::now(),
                        extra: None,
                    },
                    token_used: runtime.tokens_used as u64,
                });
            }
            AgentExecutionState::Failed(message) => {
                let _ = handle.state(StateMessage::TaskFailed {
                    task_id: task_id.to_string(),
                    agent_type,
                    error: message.clone(),
                    retry_count: 0,
                    can_retry: false,
                });
            }
        }
    }
}

fn build_llm_provider(
    resolved: ResolvedProviderEntry,
) -> Result<Arc<dyn LlmProvider>, String> {
    let provider_type = resolved.provider_type.as_str();
    if provider_type == "mock" {
        return Err("mock provider is not supported for TUI execution".to_string());
    }

    let base_url = resolved
        .base_url
        .ok_or_else(|| "provider.base_url is required".to_string())?;
    let api_key = resolved
        .api_key
        .ok_or_else(|| "provider.api_key/api_key_env is required".to_string())?;
    let model_id = resolved
        .default_model
        .ok_or_else(|| "provider.default_model is required".to_string())?;
    let model = ModelInfo::new(&model_id, &model_id, &resolved.name);

    match provider_type {
        "openai-compat" => {
            let provider = OpenAiProvider::from_config(OpenAiProviderConfig {
                base_url,
                api_key,
                model,
                default_headers: resolved.headers,
                request_timeout: Duration::from_secs(120),
                stream_buffer_size: 64,
            })
            .map_err(|error| error.to_string())?;
            Ok(Arc::new(provider))
        }
        "anthropic" => {
            let provider = AnthropicProvider::from_config(AnthropicProviderConfig {
                base_url,
                api_key,
                model,
                anthropic_version: "2023-06-01".to_string(),
                beta_headers: Vec::new(),
                default_headers: resolved.headers,
                request_timeout: Duration::from_secs(120),
                stream_buffer_size: 64,
                default_max_tokens: 4_096,
            })
            .map_err(|error| error.to_string())?;
            Ok(Arc::new(provider))
        }
        "google" => {
            let provider = GoogleProvider::from_config(GoogleProviderConfig {
                base_url,
                api_key,
                model,
                default_headers: resolved.headers,
                request_timeout: Duration::from_secs(120),
                stream_buffer_size: 64,
                default_max_output_tokens: 8_192,
            })
            .map_err(|error| error.to_string())?;
            Ok(Arc::new(provider))
        }
        other => Err(format!("unsupported provider_type: {other}")),
    }
}
