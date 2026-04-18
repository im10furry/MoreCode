use chrono::Utc;
use mc_core::{AgentExecutionReport, AgentType, Complexity, ResultType, SubTask, TaskResult};

pub(crate) fn sample_subtask(agent_type: AgentType) -> SubTask {
    SubTask {
        id: "subtask-1".to_string(),
        description: "Inspect and update auth flow".to_string(),
        target_files: vec!["src/auth/mod.rs".to_string()],
        expected_output: "Updated auth module".to_string(),
        token_budget: 2_000,
        priority: 1,
        estimated_complexity: Complexity::Medium,
        acceptance_criteria: vec!["Tests pass".to_string()],
        completed: false,
        assigned_agent: agent_type,
    }
}

pub(crate) fn sample_report() -> AgentExecutionReport {
    AgentExecutionReport {
        title: "Explorer handoff".to_string(),
        key_findings: vec!["Found auth entry point".to_string()],
        relevant_files: vec!["src/auth/mod.rs".to_string()],
        recommendations: vec!["Reuse existing token validation".to_string()],
        warnings: vec!["Login handler is long".to_string()],
        token_used: 1_024,
        timestamp: Utc::now(),
        extra: Some(serde_json::json!({ "module": "auth" })),
    }
}

pub(crate) fn sample_task_result() -> TaskResult {
    TaskResult {
        result_type: ResultType::CodeChange,
        success: true,
        data: serde_json::json!({ "patched": true }),
        changed_files: vec!["src/auth/mod.rs".to_string()],
        generated_content: Some("patched".to_string()),
        error_message: None,
    }
}
