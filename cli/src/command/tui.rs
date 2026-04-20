use chrono::Utc;
use mc_communication::{
    ApprovalRequest, ApprovalResponse, BroadcastEvent, ControlMessage, StateMessage,
};
use mc_core::{AgentExecutionReport, AgentType, Complexity, ResultType, SubTask, TaskResult};
use mc_tui::{LogLevel, Tui, TuiError, TuiHandle};
use tokio::time::{sleep, Duration};

use crate::init::AppContext;

pub async fn execute(_context: &AppContext) -> Result<String, String> {
    let (tui, handle) = Tui::new("MoreCode TUI");
    let demo = tokio::spawn(async move {
        let _ = emit_demo(handle).await;
    });

    let result = tui.run().await.map(|exit| exit.to_string());
    demo.abort();
    result.map_err(|error| error.to_string())
}

async fn emit_demo(handle: TuiHandle) -> Result<(), TuiError> {
    handle.log(LogLevel::Info, "Starting demo stream")?;
    handle.control(assign(
        AgentType::Explorer,
        "task-tui",
        "Scan workspace for TUI integration points",
        "Produce a project summary",
        3_000,
    ))?;
    sleep(Duration::from_millis(300)).await;

    handle.state(StateMessage::Progress {
        task_id: "task-tui".to_string(),
        agent_type: AgentType::Explorer,
        phase: "scan".to_string(),
        progress_percent: 35,
        message: "Reading README and communication contracts".to_string(),
    })?;
    handle.broadcast(BroadcastEvent::ProgressSnapshot {
        task_id: "task-tui".to_string(),
        agent_type: AgentType::Explorer,
        progress_percent: 35,
        summary: "Explorer is mapping the workspace".to_string(),
    })?;
    sleep(Duration::from_millis(300)).await;

    handle.state(StateMessage::Handoff {
        task_id: "task-tui".to_string(),
        from_agent: AgentType::Explorer,
        to_agent: AgentType::Planner,
        handoff: report("Workspace scan complete", 420),
    })?;
    handle.control(assign(
        AgentType::Coder,
        "task-tui",
        "Implement Ratatui dashboard and event loop",
        "Wire live status updates into the terminal UI",
        5_000,
    ))?;
    sleep(Duration::from_millis(300)).await;

    handle.state(StateMessage::Progress {
        task_id: "task-tui".to_string(),
        agent_type: AgentType::Coder,
        phase: "coding".to_string(),
        progress_percent: 60,
        message: "Rendering agent, topology, token and log panels".to_string(),
    })?;
    handle.state(StateMessage::PartialResult {
        task_id: "task-tui".to_string(),
        from_agent: AgentType::Coder,
        to_agent: AgentType::Reviewer,
        payload: serde_json::json!({
            "changed_files": ["tui/src/app.rs", "tui/src/view/dashboard.rs"],
            "status": "render pipeline ready"
        }),
    })?;
    handle.state(StateMessage::StreamChunk {
        task_id: "task-tui".to_string(),
        from_agent: AgentType::Coder,
        to_agent: AgentType::Reviewer,
        sequence: 1,
        payload: "fn render_dashboard(frame: &mut Frame, area: Rect) { ... }".to_string(),
        is_last: false,
    })?;
    sleep(Duration::from_millis(300)).await;

    handle.approval_request(ApprovalRequest {
        request_id: "approval-tui-demo".to_string(),
        task_id: "task-tui".to_string(),
        agent_type: "Coder".to_string(),
        reason: "Replace the placeholder TUI with a Ratatui implementation".to_string(),
        options: vec!["approve".to_string(), "reject".to_string()],
        recommendation: Some("approve".to_string()),
        created_at: Utc::now(),
        timeout_secs: 30,
    })?;
    sleep(Duration::from_millis(300)).await;

    handle.approval_response(ApprovalResponse {
        request_id: "approval-tui-demo".to_string(),
        choice: "approve".to_string(),
        approved: true,
        comment: Some("Proceed with the full UI module".to_string()),
        responded_at: Utc::now(),
    })?;
    handle.broadcast(BroadcastEvent::SystemNotification {
        level: "warn".to_string(),
        message: "Demo: token budget is nearing the warning threshold".to_string(),
    })?;
    handle.state(StateMessage::TaskCompleted {
        task_id: "task-tui".to_string(),
        agent_type: AgentType::Coder,
        result: result("TUI module implemented"),
        handoff: report("Implementation complete", 1_650),
        token_used: 1_650,
    })?;
    handle.log(
        LogLevel::Info,
        "Demo stream finished; inspect panels and press q",
    )?;
    Ok(())
}

fn assign(
    agent_type: AgentType,
    task_id: &str,
    description: &str,
    expected_output: &str,
    token_budget: u64,
) -> ControlMessage {
    ControlMessage::TaskAssigned {
        task_id: task_id.to_string(),
        agent_type,
        task: SubTask {
            id: format!("{task_id}-{agent_type}"),
            description: description.to_string(),
            target_files: vec!["tui/src".to_string()],
            expected_output: expected_output.to_string(),
            token_budget: token_budget as u32,
            priority: 0,
            estimated_complexity: Complexity::Medium,
            acceptance_criteria: vec!["cargo test -p tui".to_string()],
            completed: false,
            assigned_agent: agent_type,
        },
        context: Box::new(report("demo", 0)),
        token_budget,
    }
}

fn report(title: &str, token_used: u32) -> AgentExecutionReport {
    AgentExecutionReport {
        title: title.to_string(),
        key_findings: vec!["TUI scaffold is ready".to_string()],
        relevant_files: vec!["tui/src/app.rs".to_string()],
        recommendations: vec!["Run cargo test -p tui".to_string()],
        warnings: Vec::new(),
        token_used,
        timestamp: Utc::now(),
        extra: None,
    }
}

fn result(summary: &str) -> TaskResult {
    TaskResult {
        result_type: ResultType::CodeChange,
        success: true,
        data: serde_json::json!({ "summary": summary }),
        changed_files: vec!["tui/src/app.rs".to_string()],
        generated_content: Some(summary.to_string()),
        error_message: None,
    }
}
