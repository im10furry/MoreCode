pub mod parallel_group;

use std::collections::{BTreeMap, HashMap, HashSet};

use async_trait::async_trait;
use chrono::Utc;
use mc_core::{
    AgentType, CommitPoint, Complexity, ContextAllocation, DependencyType, ExecutionPlan,
    ParallelGroup, PlanMetadata, ProjectContext, RiskLevel, SubTask, TaskDependency,
    TaskDescription,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use self::parallel_group::topological_layers;
use crate::support::complete_json;
use crate::{
    Agent, AgentConfig, AgentContext, AgentError, AgentExecutionReport, ImpactReport,
    SharedResources,
};

#[derive(Debug, Clone)]
pub struct Planner {
    config: AgentConfig,
}

#[derive(Debug, Clone, Deserialize)]
struct PlannerLlmSummary {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    review_focus: Vec<String>,
}

impl Planner {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    fn task_module(path: &str, project_ctx: &ProjectContext) -> String {
        project_ctx
            .structure
            .modules
            .iter()
            .find(|module| path.starts_with(&module.path))
            .map(|module| module.path.clone())
            .unwrap_or_else(|| path.to_string())
    }

    fn create_subtasks(
        &self,
        ctx: &AgentContext,
        project_ctx: &ProjectContext,
        impact: &ImpactReport,
    ) -> (Vec<SubTask>, Vec<TaskDependency>) {
        let mut tasks = Vec::new();
        let mut deps = Vec::new();
        let mut task_by_module = HashMap::<String, String>::new();
        let grouped = impact.direct_impacts.iter().fold(
            BTreeMap::<String, Vec<String>>::new(),
            |mut acc, change| {
                acc.entry(Self::task_module(&change.file, project_ctx))
                    .or_default()
                    .push(change.file.clone());
                acc
            },
        );

        for (index, (module, files)) in grouped.into_iter().enumerate() {
            let task_id = format!("T{}", index + 1);
            let risk = impact
                .direct_impacts
                .iter()
                .filter(|change| files.contains(&change.file))
                .fold(RiskLevel::Low, |acc, change| {
                    RiskLevel::max(acc, change.risk_level)
                });
            let complexity = match risk {
                RiskLevel::Low => Complexity::Simple,
                RiskLevel::Medium => Complexity::Medium,
                RiskLevel::High | RiskLevel::Critical => Complexity::Complex,
            };
            tasks.push(SubTask {
                id: task_id.clone(),
                description: format!("Implement requested changes in `{module}`"),
                target_files: files.clone(),
                expected_output: "Updated implementation aligned with the impacted module"
                    .to_string(),
                token_budget: 2_000 + (files.len() as u32 * 600),
                priority: index as u8,
                estimated_complexity: complexity,
                acceptance_criteria: vec![
                    "Target files compile and preserve module boundaries".to_string(),
                    "Impacted symbols remain compatible with downstream callers".to_string(),
                ],
                completed: false,
                assigned_agent: AgentType::Coder,
            });
            task_by_module.insert(module, task_id);
        }

        for edge in &project_ctx.dependency_graph.edges {
            if let (Some(upstream), Some(downstream)) =
                (task_by_module.get(&edge.to), task_by_module.get(&edge.from))
            {
                if upstream != downstream {
                    deps.push(TaskDependency {
                        upstream_task_id: upstream.clone(),
                        downstream_task_id: downstream.clone(),
                        dependency_type: DependencyType::Strong,
                        description: format!("`{}` depends on `{}`", edge.from, edge.to),
                    });
                }
            }
        }

        if tasks.is_empty() {
            tasks.push(SubTask {
                id: "T1".to_string(),
                description: "Implement the requested change".to_string(),
                target_files: ctx.task.affected_files.clone(),
                expected_output: "Updated implementation".to_string(),
                token_budget: 2_000,
                priority: 0,
                estimated_complexity: ctx.task.complexity,
                acceptance_criteria: vec!["Requested behavior is implemented".to_string()],
                completed: false,
                assigned_agent: AgentType::Coder,
            });
        }

        let coding_ids = tasks.iter().map(|task| task.id.clone()).collect::<Vec<_>>();
        let need_reviewer =
            impact.overall_risk_level.score() >= RiskLevel::Medium.score() || tasks.len() > 1;
        let need_tester = ctx.task.requires_testing
            || impact.overall_risk_level.score() >= RiskLevel::Medium.score();

        if need_reviewer {
            let reviewer_id = format!("T{}", tasks.len() + 1);
            tasks.push(SubTask {
                id: reviewer_id.clone(),
                description: "Review the implementation plan and risky changes".to_string(),
                target_files: impact
                    .direct_impacts
                    .iter()
                    .map(|change| change.file.clone())
                    .collect(),
                expected_output: "Review findings or approval".to_string(),
                token_budget: 1_500,
                priority: 250,
                estimated_complexity: Complexity::Medium,
                acceptance_criteria: vec!["High-risk edges are reviewed".to_string()],
                completed: false,
                assigned_agent: AgentType::Reviewer,
            });
            for task_id in &coding_ids {
                deps.push(TaskDependency {
                    upstream_task_id: task_id.clone(),
                    downstream_task_id: reviewer_id.clone(),
                    dependency_type: DependencyType::Weak,
                    description: "Review depends on coding output".to_string(),
                });
            }
        }

        if need_tester {
            let tester_id = format!("T{}", tasks.len() + 1);
            tasks.push(SubTask {
                id: tester_id.clone(),
                description: "Run focused verification for impacted behavior".to_string(),
                target_files: impact
                    .direct_impacts
                    .iter()
                    .map(|change| change.file.clone())
                    .collect(),
                expected_output: "Focused test evidence".to_string(),
                token_budget: 1_600,
                priority: 251,
                estimated_complexity: Complexity::Medium,
                acceptance_criteria: vec!["Impacted paths are verified".to_string()],
                completed: false,
                assigned_agent: AgentType::Tester,
            });
            for task_id in &coding_ids {
                deps.push(TaskDependency {
                    upstream_task_id: task_id.clone(),
                    downstream_task_id: tester_id.clone(),
                    dependency_type: DependencyType::Weak,
                    description: "Verification depends on coding output".to_string(),
                });
            }
        }

        (tasks, deps)
    }

    fn build_groups(
        &self,
        tasks: &[SubTask],
        deps: &[TaskDependency],
    ) -> (Vec<ParallelGroup>, HashMap<String, Vec<String>>) {
        let task_ids = tasks.iter().map(|task| task.id.clone()).collect::<Vec<_>>();
        let layers = topological_layers(&task_ids, deps);
        let mut groups = Vec::new();
        let mut task_to_group = HashMap::<String, String>::new();

        for (layer_idx, layer) in layers.into_iter().enumerate() {
            let mut by_agent = BTreeMap::<AgentType, Vec<SubTask>>::new();
            for task_id in layer {
                if let Some(task) = tasks.iter().find(|task| task.id == task_id) {
                    by_agent
                        .entry(task.assigned_agent)
                        .or_default()
                        .push(task.clone());
                }
            }

            for (slot, (agent_type, sub_tasks)) in by_agent.into_iter().enumerate() {
                let group_id = format!("G{}{}", layer_idx + 1, slot + 1);
                for task in &sub_tasks {
                    task_to_group.insert(task.id.clone(), group_id.clone());
                }
                groups.push(ParallelGroup {
                    id: group_id,
                    name: format!("{agent_type} layer {}", layer_idx + 1),
                    sub_tasks,
                    can_parallel: true,
                    depends_on: Vec::new(),
                    agent_type,
                });
            }
        }

        let mut group_deps = HashMap::<String, Vec<String>>::new();
        for dep in deps {
            let Some(from) = task_to_group.get(&dep.upstream_task_id) else {
                continue;
            };
            let Some(to) = task_to_group.get(&dep.downstream_task_id) else {
                continue;
            };
            if from == to {
                continue;
            }
            group_deps.entry(to.clone()).or_default().push(from.clone());
        }
        for group in &mut groups {
            let mut deps = group_deps.get(&group.id).cloned().unwrap_or_default();
            deps.sort();
            deps.dedup();
            group.depends_on = deps;
        }
        (groups, group_deps)
    }

    fn commit_points(groups: &[ParallelGroup]) -> Vec<CommitPoint> {
        groups
            .iter()
            .enumerate()
            .map(|(index, group)| CommitPoint {
                id: format!("C{}", index + 1),
                name: format!("checkpoint: {}", group.name),
                waiting_tasks: group.sub_tasks.iter().map(|task| task.id.clone()).collect(),
                target_branch: "main".to_string(),
                completed: false,
                merged_at: None,
            })
            .collect()
    }

    fn allocations(
        &self,
        project_ctx: &ProjectContext,
        impact: &ImpactReport,
        tasks: &[SubTask],
    ) -> Vec<ContextAllocation> {
        tasks
            .iter()
            .map(|task| ContextAllocation {
                sub_task_id: task.id.clone(),
                agent_type: task.assigned_agent,
                token_budget: task.token_budget,
                required_files: task.target_files.clone(),
                project_knowledge_subset: vec![
                    format!("root={}", project_ctx.root_path),
                    format!("modules={}", project_ctx.structure.modules.len()),
                    format!("overall_risk={:?}", impact.overall_risk_level),
                ],
                context_window_limit: self.config.planner.context_window_limit,
            })
            .collect()
    }

    pub fn validate_plan(&self, plan: &ExecutionPlan) -> Result<(), AgentError> {
        let layers = topological_layers(
            &plan
                .sub_tasks
                .iter()
                .map(|task| task.id.clone())
                .collect::<Vec<_>>(),
            &plan.dependencies,
        );
        if layers.iter().map(Vec::len).sum::<usize>() != plan.sub_tasks.len() {
            return Err(AgentError::ExecutionFailed {
                agent_type: AgentType::Planner,
                message: "execution plan contains a dependency cycle".to_string(),
            });
        }

        let grouped = plan
            .parallel_groups
            .iter()
            .flat_map(|group| group.sub_tasks.iter().map(|task| task.id.clone()))
            .collect::<HashSet<_>>();
        for task in &plan.sub_tasks {
            if !grouped.contains(&task.id) {
                return Err(AgentError::ExecutionFailed {
                    agent_type: AgentType::Planner,
                    message: format!("task `{}` is not in any parallel group", task.id),
                });
            }
        }

        if plan.parallel_groups.len() > self.config.planner.max_parallel_groups {
            return Err(AgentError::ResourceConstraint {
                message: "too many parallel groups".to_string(),
            });
        }
        if plan.total_estimated_tokens as u32 > self.config.planner.max_total_token_budget {
            return Err(AgentError::ResourceConstraint {
                message: "plan exceeds total token budget".to_string(),
            });
        }
        Ok(())
    }

    async fn enrich(
        &self,
        ctx: &AgentContext,
        plan: &ExecutionPlan,
        impact: &ImpactReport,
    ) -> Result<(PlannerLlmSummary, u32), AgentError> {
        let prompt = format!(
            "Task: {}\nSubtasks: {}\nGroups: {}\nOverall risk: {:?}",
            ctx.task.user_input,
            plan.sub_tasks.len(),
            plan.parallel_groups.len(),
            impact.overall_risk_level
        );
        complete_json(
            ctx.llm_provider.as_ref(),
            &ctx.config.llm_config.model_id,
            "Summarize the execution plan and return strict JSON.",
            &prompt,
            "planner_summary",
            json!({
                "type":"object",
                "additionalProperties":false,
                "required":["summary","review_focus"],
                "properties":{
                    "summary":{"type":"string"},
                    "review_focus":{"type":"array","items":{"type":"string"}}
                }
            }),
            ctx.config.llm_config.temperature,
            ctx.config.llm_config.max_output_tokens,
            ctx.cancel_token.child_token(),
        )
        .await
    }
}

#[async_trait]
impl Agent for Planner {
    fn agent_type(&self) -> AgentType {
        AgentType::Planner
    }

    fn default_config(&self) -> AgentConfig {
        AgentConfig::for_agent_type(AgentType::Planner)
    }

    fn update_config(&mut self, config: AgentConfig) {
        self.config = config;
    }

    async fn build_context(
        &self,
        task: &TaskDescription,
        project_ctx: Option<ProjectContext>,
        shared: &SharedResources,
    ) -> Result<AgentContext, AgentError> {
        let project_ctx = project_ctx.ok_or_else(|| AgentError::MissingContextData {
            data_type: "ProjectContext".to_string(),
        })?;
        Ok(AgentContext::new(task.clone(), shared, self.config.clone())
            .with_project_ctx(project_ctx))
    }

    async fn execute(&self, ctx: &AgentContext) -> Result<AgentExecutionReport, AgentError> {
        let project_ctx =
            ctx.project_ctx
                .as_deref()
                .cloned()
                .ok_or_else(|| AgentError::MissingContextData {
                    data_type: "ProjectContext".to_string(),
                })?;
        let impact = if let Some(impact) = ctx.impact_report.as_deref().cloned() {
            impact
        } else {
            ctx.handoff.get::<ImpactReport>().await.ok_or_else(|| {
                AgentError::MissingContextData {
                    data_type: "ImpactReport".to_string(),
                }
            })?
        };

        let (tasks, deps) = self.create_subtasks(ctx, &project_ctx, &impact);
        let (groups, group_deps) = self.build_groups(&tasks, &deps);
        let commits = Self::commit_points(&groups);
        let allocations = self.allocations(&project_ctx, &impact, &tasks);
        let mut plan = ExecutionPlan {
            plan_id: Uuid::new_v4().to_string(),
            task_description: ctx.task.user_input.clone(),
            summary: String::new(),
            parallel_groups: groups,
            group_dependencies: group_deps,
            sub_tasks: tasks,
            dependencies: deps,
            commit_points: commits,
            context_allocations: allocations,
            total_estimated_tokens: 0,
            total_estimated_duration_ms: 0,
            plan_metadata: PlanMetadata {
                generated_by: AgentType::Planner,
                generated_at: Utc::now(),
                model_used: ctx.config.llm_config.model_id.clone(),
                generation_duration_ms: 0,
                tokens_used: 0,
                version: 1,
            },
            created_at: Utc::now(),
        };
        plan.total_estimated_tokens = plan
            .sub_tasks
            .iter()
            .map(|task| task.token_budget as usize)
            .sum();
        plan.total_estimated_duration_ms = plan
            .sub_tasks
            .iter()
            .map(|task| task.token_budget as u64 * 3)
            .sum();
        let (summary, llm_tokens) = self.enrich(ctx, &plan, &impact).await?;
        plan.summary = summary.summary;
        if let Some(reviewer) = plan
            .context_allocations
            .iter_mut()
            .find(|allocation| allocation.agent_type == AgentType::Reviewer)
        {
            reviewer
                .project_knowledge_subset
                .extend(summary.review_focus);
        }
        plan.plan_metadata.generation_duration_ms = ctx.elapsed_ms();
        plan.plan_metadata.tokens_used = llm_tokens;

        self.validate_plan(&plan)?;
        ctx.handoff.put(plan.clone()).await;
        let result = serde_json::to_value(&plan).map_err(AgentError::serialization)?;
        Ok(AgentExecutionReport::success(
            AgentType::Planner,
            &ctx.execution_id,
            format!(
                "Planner generated {} tasks across {} groups",
                plan.sub_tasks.len(),
                plan.parallel_groups.len()
            ),
            result,
            ctx.elapsed_ms(),
            llm_tokens,
        ))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use mc_core::{
        AgentType, CommitPoint, Complexity, ContextAllocation, DependencyType, ExecutionPlan,
        ParallelGroup, PlanMetadata, SubTask, TaskDependency,
    };

    use super::Planner;
    use crate::AgentConfig;

    fn sample_task(id: &str) -> SubTask {
        SubTask {
            id: id.to_string(),
            description: format!("task {id}"),
            target_files: vec!["src/lib.rs".to_string()],
            expected_output: "out".to_string(),
            token_budget: 1_000,
            priority: 0,
            estimated_complexity: Complexity::Simple,
            acceptance_criteria: vec!["ok".to_string()],
            completed: false,
            assigned_agent: AgentType::Coder,
        }
    }

    #[test]
    fn planner_detects_cycles() {
        let planner = Planner::new(AgentConfig::for_agent_type(AgentType::Planner));
        let plan = ExecutionPlan {
            plan_id: "plan".to_string(),
            task_description: "cyclic".to_string(),
            summary: String::new(),
            parallel_groups: vec![ParallelGroup {
                id: "G1".to_string(),
                name: "group".to_string(),
                sub_tasks: vec![sample_task("T1"), sample_task("T2")],
                can_parallel: true,
                depends_on: Vec::new(),
                agent_type: AgentType::Coder,
            }],
            group_dependencies: Default::default(),
            sub_tasks: vec![sample_task("T1"), sample_task("T2")],
            dependencies: vec![
                TaskDependency {
                    upstream_task_id: "T1".to_string(),
                    downstream_task_id: "T2".to_string(),
                    dependency_type: DependencyType::Strong,
                    description: "a".to_string(),
                },
                TaskDependency {
                    upstream_task_id: "T2".to_string(),
                    downstream_task_id: "T1".to_string(),
                    dependency_type: DependencyType::Strong,
                    description: "b".to_string(),
                },
            ],
            commit_points: vec![CommitPoint {
                id: "C1".to_string(),
                name: "checkpoint".to_string(),
                waiting_tasks: vec!["T1".to_string(), "T2".to_string()],
                target_branch: "main".to_string(),
                completed: false,
                merged_at: None,
            }],
            context_allocations: vec![ContextAllocation {
                sub_task_id: "T1".to_string(),
                agent_type: AgentType::Coder,
                token_budget: 1_000,
                required_files: vec!["src/lib.rs".to_string()],
                project_knowledge_subset: Vec::new(),
                context_window_limit: 1_000,
            }],
            total_estimated_tokens: 2_000,
            total_estimated_duration_ms: 2_000,
            plan_metadata: PlanMetadata {
                generated_by: AgentType::Planner,
                generated_at: Utc::now(),
                model_used: "mock".to_string(),
                generation_duration_ms: 0,
                tokens_used: 0,
                version: 1,
            },
            created_at: Utc::now(),
        };

        assert!(planner.validate_plan(&plan).is_err());
    }
}
