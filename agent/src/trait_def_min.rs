use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use chrono::Utc;
use mc_context::{ImpactChange, ImpactReport, ProjectContext, RiskLevel};
use mc_core::{
    AgentType, CommitPoint, Complexity, ContextAllocation, ExecutionPlan, ParallelGroup,
    PlanMetadata, ResultType, SubTask, TaskDependency, TaskResult,
};
use mc_llm::{EventBus, FinishReason, LlmError, StreamEvent, StreamForwarder, TokenUsage};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::handoff_min::AgentHandoff;

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("agent not found: {0}")]
    AgentNotFound(AgentType),
    #[error("streaming failed: {0}")]
    Streaming(String),
    #[error("internal agent error: {0}")]
    Internal(String),
}

impl From<LlmError> for AgentError {
    fn from(value: LlmError) -> Self {
        Self::Streaming(value.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewVerdict {
    Passed,
    NeedsChanges,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewReport {
    pub verdict: ReviewVerdict,
    pub issues: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestReport {
    pub summary: String,
    pub passed: usize,
    pub failed: usize,
    pub coverage: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResearchReport {
    pub findings: Vec<String>,
    pub recommendations: Vec<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentResult {
    pub agent_type: AgentType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_result: Option<TaskResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handoff: Option<AgentHandoff>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_report: Option<ReviewReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_report: Option<TestReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact_report: Option<ImpactReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub research_report: Option<ResearchReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_context: Option<ProjectContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_plan: Option<ExecutionPlan>,
    pub tokens_used: usize,
    pub duration_ms: u64,
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn agent_type(&self) -> AgentType;
    fn agent_id(&self) -> &str;

    async fn execute(&self, task: &str, token_budget: u64) -> Result<AgentResult, AgentError>;

    async fn execute_streaming(
        &self,
        task: &str,
        token_budget: u64,
        event_bus: Arc<dyn EventBus>,
    ) -> Result<AgentResult, AgentError>;

    async fn execute_with_context(
        &self,
        task: &str,
        shared_context: &Arc<ProjectContext>,
    ) -> Result<TaskResult, AgentError>;

    async fn plan(
        &self,
        task: &str,
        shared_context: &Arc<ProjectContext>,
    ) -> Result<ExecutionPlan, AgentError>;

    async fn execute_plan(
        &self,
        plan: ExecutionPlan,
        shared_context: &Arc<ProjectContext>,
    ) -> Result<TaskResult, AgentError>;
}

#[derive(Debug)]
pub struct DefaultAgent {
    agent_type: AgentType,
    agent_id: String,
}

impl DefaultAgent {
    pub fn new(agent_type: AgentType) -> Self {
        Self {
            agent_type,
            agent_id: Uuid::new_v4().to_string(),
        }
    }

    fn build_result(
        &self,
        task: &str,
        stream_content: String,
        usage: TokenUsage,
        duration_ms: u64,
    ) -> AgentResult {
        let changed_files = infer_files(task);
        let tokens_used = usage.total_tokens.max(estimate_tokens(&stream_content)) as usize;

        match self.agent_type {
            AgentType::Explorer => {
                let mut project_context = ProjectContext::default();
                project_context.info.name = "Workspace".into();
                project_context.info.summary = Some(stream_content);
                project_context.notes = changed_files
                    .iter()
                    .map(|file| format!("Focus file: {file}"))
                    .collect();

                AgentResult {
                    agent_type: self.agent_type,
                    task_result: None,
                    handoff: None,
                    review_report: None,
                    test_report: None,
                    impact_report: None,
                    research_report: None,
                    project_context: Some(project_context),
                    execution_plan: None,
                    tokens_used,
                    duration_ms,
                }
            }
            AgentType::ImpactAnalyzer => {
                let impact_report = ImpactReport {
                    direct_impacts: changed_files
                        .iter()
                        .map(|file| ImpactChange {
                            path: PathBuf::from(file),
                            change_type: mc_context::ChangeType::ModifyFile,
                            note: "Predicted direct impact".into(),
                        })
                        .collect(),
                    indirect_impacts: Vec::new(),
                    risk_assessment: if changed_files.len() > 3 {
                        RiskLevel::High
                    } else {
                        RiskLevel::Medium
                    },
                };

                AgentResult {
                    agent_type: self.agent_type,
                    task_result: None,
                    handoff: None,
                    review_report: None,
                    test_report: None,
                    impact_report: Some(impact_report),
                    research_report: None,
                    project_context: None,
                    execution_plan: None,
                    tokens_used,
                    duration_ms,
                }
            }
            AgentType::Planner => AgentResult {
                agent_type: self.agent_type,
                task_result: None,
                handoff: None,
                review_report: None,
                test_report: None,
                impact_report: None,
                research_report: None,
                project_context: None,
                execution_plan: Some(simulated_plan(task, &changed_files)),
                tokens_used,
                duration_ms,
            },
            AgentType::Reviewer => AgentResult {
                agent_type: self.agent_type,
                task_result: Some(TaskResult {
                    result_type: ResultType::ReviewResult,
                    success: true,
                    data: serde_json::json!({ "verdict": "passed" }),
                    changed_files: Vec::new(),
                    generated_content: Some("Review passed".into()),
                    error_message: None,
                }),
                handoff: None,
                review_report: Some(ReviewReport {
                    verdict: ReviewVerdict::Passed,
                    issues: Vec::new(),
                    summary: "Simulated review passed".into(),
                }),
                test_report: None,
                impact_report: None,
                research_report: None,
                project_context: None,
                execution_plan: None,
                tokens_used,
                duration_ms,
            },
            AgentType::Tester => AgentResult {
                agent_type: self.agent_type,
                task_result: Some(TaskResult {
                    result_type: ResultType::TestResult,
                    success: true,
                    data: serde_json::json!({ "passed": 1, "failed": 0 }),
                    changed_files: Vec::new(),
                    generated_content: Some("Tests passed".into()),
                    error_message: None,
                }),
                handoff: None,
                review_report: None,
                test_report: Some(TestReport {
                    summary: "Simulated tests passed".into(),
                    passed: 1,
                    failed: 0,
                    coverage: Some(0.82),
                }),
                impact_report: None,
                research_report: None,
                project_context: None,
                execution_plan: None,
                tokens_used,
                duration_ms,
            },
            AgentType::Research => AgentResult {
                agent_type: self.agent_type,
                task_result: Some(TaskResult {
                    result_type: ResultType::ResearchReport,
                    success: true,
                    data: serde_json::json!({ "findings": ["simulated research"] }),
                    changed_files: Vec::new(),
                    generated_content: Some("Research completed".into()),
                    error_message: None,
                }),
                handoff: None,
                review_report: None,
                test_report: None,
                impact_report: None,
                research_report: Some(ResearchReport {
                    findings: vec!["Simulated external research".into()],
                    recommendations: vec!["Proceed with the implementation".into()],
                    sources: vec!["simulated://source".into()],
                }),
                project_context: None,
                execution_plan: None,
                tokens_used,
                duration_ms,
            },
            AgentType::DocWriter => {
                let handoff = AgentHandoff {
                    changed_files: changed_files.clone(),
                    new_files: Vec::new(),
                    deleted_files: Vec::new(),
                    summary: stream_content.clone(),
                    structured: Some(serde_json::json!({ "kind": "docs", "files": changed_files })),
                };

                AgentResult {
                    agent_type: self.agent_type,
                    task_result: Some(TaskResult {
                        result_type: ResultType::Documentation,
                        success: true,
                        data: serde_json::json!({ "summary": stream_content }),
                        changed_files: handoff.changed_files.clone(),
                        generated_content: Some(handoff.summary.clone()),
                        error_message: None,
                    }),
                    handoff: Some(handoff),
                    review_report: None,
                    test_report: None,
                    impact_report: None,
                    research_report: None,
                    project_context: None,
                    execution_plan: None,
                    tokens_used,
                    duration_ms,
                }
            }
            _ => {
                let handoff = AgentHandoff {
                    changed_files: changed_files.clone(),
                    new_files: Vec::new(),
                    deleted_files: Vec::new(),
                    summary: stream_content.clone(),
                    structured: Some(serde_json::json!({
                        "agent": self.agent_type.as_str(),
                        "files": changed_files,
                    })),
                };

                AgentResult {
                    agent_type: self.agent_type,
                    task_result: Some(TaskResult {
                        result_type: ResultType::CodeChange,
                        success: true,
                        data: serde_json::json!({ "summary": stream_content }),
                        changed_files: handoff.changed_files.clone(),
                        generated_content: Some(handoff.summary.clone()),
                        error_message: None,
                    }),
                    handoff: Some(handoff),
                    review_report: None,
                    test_report: None,
                    impact_report: None,
                    research_report: None,
                    project_context: None,
                    execution_plan: None,
                    tokens_used,
                    duration_ms,
                }
            }
        }
    }
}

#[async_trait]
impl Agent for DefaultAgent {
    fn agent_type(&self) -> AgentType {
        self.agent_type
    }

    fn agent_id(&self) -> &str {
        &self.agent_id
    }

    async fn execute(&self, task: &str, _token_budget: u64) -> Result<AgentResult, AgentError> {
        let start = Instant::now();
        let content = format!("{} completed task: {task}", self.agent_type.as_str());
        Ok(self.build_result(
            task,
            content.clone(),
            TokenUsage {
                prompt_tokens: estimate_tokens(task),
                completion_tokens: estimate_tokens(&content),
                cached_tokens: 0,
                total_tokens: estimate_tokens(task) + estimate_tokens(&content),
            },
            start.elapsed().as_millis() as u64,
        ))
    }

    async fn execute_streaming(
        &self,
        task: &str,
        _token_budget: u64,
        event_bus: Arc<dyn EventBus>,
    ) -> Result<AgentResult, AgentError> {
        let start = Instant::now();
        let content = format!("{} streaming task: {task}", self.agent_type.as_str());
        let usage = TokenUsage {
            prompt_tokens: estimate_tokens(task),
            completion_tokens: estimate_tokens(&content),
            cached_tokens: 0,
            total_tokens: estimate_tokens(task) + estimate_tokens(&content),
        };

        let (tx, rx) = mpsc::channel(8);
        let midpoint = content.chars().count().max(1) / 2;
        let mut first = String::new();
        let mut second = String::new();
        for (index, ch) in content.chars().enumerate() {
            if index < midpoint {
                first.push(ch);
            } else {
                second.push(ch);
            }
        }
        let usage_clone = usage;
        tokio::spawn(async move {
            let _ = tx
                .send(StreamEvent::Delta {
                    content: first,
                    cumulative_tokens: Some(usage_clone.prompt_tokens / 2 + 1),
                })
                .await;
            let _ = tx
                .send(StreamEvent::Delta {
                    content: second,
                    cumulative_tokens: Some(usage_clone.total_tokens),
                })
                .await;
            let _ = tx
                .send(StreamEvent::Finish {
                    reason: FinishReason::Stop,
                    usage: Some(usage_clone),
                    response_id: Uuid::new_v4().to_string(),
                })
                .await;
        });

        let (stream_content, usage) = StreamForwarder::new(rx, event_bus).into_content().await?;
        Ok(self.build_result(
            task,
            stream_content,
            usage,
            start.elapsed().as_millis() as u64,
        ))
    }

    async fn execute_with_context(
        &self,
        task: &str,
        shared_context: &Arc<ProjectContext>,
    ) -> Result<TaskResult, AgentError> {
        let changed_files = infer_files(task);
        Ok(TaskResult {
            result_type: ResultType::CodeChange,
            success: true,
            data: serde_json::json!({
                "project": shared_context.info.name,
                "notes": shared_context.notes,
            }),
            changed_files,
            generated_content: Some(format!(
                "{} used shared context for {}",
                self.agent_type.as_str(),
                shared_context.info.name
            )),
            error_message: None,
        })
    }

    async fn plan(
        &self,
        task: &str,
        _shared_context: &Arc<ProjectContext>,
    ) -> Result<ExecutionPlan, AgentError> {
        Ok(simulated_plan(task, &infer_files(task)))
    }

    async fn execute_plan(
        &self,
        plan: ExecutionPlan,
        shared_context: &Arc<ProjectContext>,
    ) -> Result<TaskResult, AgentError> {
        let changed_files = if plan.sub_tasks.is_empty() {
            infer_files(&plan.task_description)
        } else {
            plan.sub_tasks
                .iter()
                .flat_map(|sub_task| sub_task.target_files.clone())
                .collect()
        };

        Ok(TaskResult {
            result_type: ResultType::CodeChange,
            success: true,
            data: serde_json::json!({
                "plan_id": plan.plan_id,
                "project": shared_context.info.name,
            }),
            changed_files,
            generated_content: Some("Plan executed with shared context".into()),
            error_message: None,
        })
    }
}

fn infer_files(task: &str) -> Vec<String> {
    let mut files = task
        .split_whitespace()
        .map(|token| {
            token.trim_matches(|ch: char| {
                matches!(ch, ',' | '.' | '"' | '\'' | '(' | ')' | '[' | ']')
            })
        })
        .filter(|token| {
            token.contains('/')
                || token.ends_with(".rs")
                || token.ends_with(".md")
                || token.ends_with(".toml")
        })
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if files.is_empty() {
        files.push("src/lib.rs".into());
    }

    files
}

fn estimate_tokens(text: &str) -> u32 {
    text.split_whitespace().count().max(1) as u32
}

fn simulated_plan(task: &str, files: &[String]) -> ExecutionPlan {
    let sub_task = SubTask {
        id: Uuid::new_v4().to_string(),
        description: task.into(),
        target_files: files.to_vec(),
        expected_output: "Apply requested changes".into(),
        token_budget: 4_000,
        priority: 0,
        estimated_complexity: Complexity::Medium,
        acceptance_criteria: vec!["Requested behavior is implemented".into()],
        completed: false,
        assigned_agent: AgentType::Coder,
    };

    ExecutionPlan {
        plan_id: Uuid::new_v4().to_string(),
        task_description: task.into(),
        summary: "Simulated single-step execution plan".into(),
        parallel_groups: vec![ParallelGroup {
            id: Uuid::new_v4().to_string(),
            name: "primary".into(),
            sub_tasks: vec![sub_task.clone()],
            can_parallel: false,
            depends_on: Vec::new(),
            agent_type: AgentType::Coder,
        }],
        group_dependencies: HashMap::new(),
        sub_tasks: vec![sub_task.clone()],
        dependencies: vec![TaskDependency {
            upstream_task_id: sub_task.id.clone(),
            downstream_task_id: sub_task.id.clone(),
            dependency_type: mc_core::DependencyType::Weak,
            description: "Single-step plan".into(),
        }],
        commit_points: vec![CommitPoint {
            id: Uuid::new_v4().to_string(),
            name: "default".into(),
            waiting_tasks: vec![sub_task.id.clone()],
            target_branch: "main".into(),
            completed: false,
            merged_at: None,
        }],
        context_allocations: vec![ContextAllocation {
            sub_task_id: sub_task.id.clone(),
            agent_type: AgentType::Coder,
            token_budget: sub_task.token_budget,
            required_files: files.to_vec(),
            project_knowledge_subset: Vec::new(),
            context_window_limit: 16_000,
        }],
        total_estimated_tokens: sub_task.token_budget as usize,
        total_estimated_duration_ms: 1_000,
        plan_metadata: PlanMetadata {
            generated_by: AgentType::Planner,
            generated_at: Utc::now(),
            model_used: "simulated".into(),
            generation_duration_ms: 10,
            tokens_used: 128,
            version: 1,
        },
        created_at: Utc::now(),
    }
}
