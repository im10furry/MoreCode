use std::collections::{BTreeMap, HashMap};

use chrono::Utc;
use mc_context::ProjectContext;
use mc_core::{
    AgentType, CommitPoint, ExecutionPlan, ParallelGroup, PlanMetadata, SubTask, TaskDependency,
    TaskDescription,
};

use crate::plan::allocator::{allocate_agent_budgets, ContextAllocator};
use crate::plan::dependency::{
    build_group_dependencies, topological_layers, validate_dependencies,
};
use crate::routing::RouteLevel;
use crate::CoordinatorError;

pub struct ExecutionPlanBuilder {
    model_used: String,
    target_branch: String,
    context_allocator: ContextAllocator,
}

impl ExecutionPlanBuilder {
    pub fn new(model_used: impl Into<String>) -> Self {
        Self {
            model_used: model_used.into(),
            target_branch: "main".to_string(),
            context_allocator: ContextAllocator::default(),
        }
    }

    pub fn with_target_branch(mut self, target_branch: impl Into<String>) -> Self {
        self.target_branch = target_branch.into();
        self
    }

    pub fn with_context_allocator(mut self, context_allocator: ContextAllocator) -> Self {
        self.context_allocator = context_allocator;
        self
    }

    pub fn build(
        &self,
        task: &TaskDescription,
        route_level: &RouteLevel,
        max_token_budget: u32,
        sub_tasks: Vec<SubTask>,
        dependencies: Vec<TaskDependency>,
        project_ctx: Option<&ProjectContext>,
    ) -> Result<ExecutionPlan, CoordinatorError> {
        if sub_tasks.is_empty() {
            return Err(CoordinatorError::Internal(
                "execution plan requires at least one subtask".into(),
            ));
        }

        let task_ids = sub_tasks
            .iter()
            .map(|task| task.id.clone())
            .collect::<Vec<_>>();
        validate_dependencies(&task_ids, &dependencies)?;
        let layers = topological_layers(&task_ids, &dependencies)?;
        let agents = sub_tasks
            .iter()
            .map(|task| task.assigned_agent)
            .collect::<Vec<_>>();
        let agent_budgets = allocate_agent_budgets(max_token_budget, &agents, route_level);
        let context_allocations =
            self.context_allocator
                .allocate(task, project_ctx, &sub_tasks, &agent_budgets);

        let mut parallel_groups = build_parallel_groups(&sub_tasks, &layers);
        let group_dependencies = build_group_dependencies(&parallel_groups, &dependencies);
        for group in &mut parallel_groups {
            group.depends_on = group_dependencies
                .get(&group.id)
                .cloned()
                .unwrap_or_default();
        }

        let commit_points = parallel_groups
            .iter()
            .enumerate()
            .map(|(index, group)| CommitPoint {
                id: format!("C{}", index + 1),
                name: format!("checkpoint: {}", group.name),
                waiting_tasks: group.sub_tasks.iter().map(|task| task.id.clone()).collect(),
                target_branch: self.target_branch.clone(),
                completed: false,
                merged_at: None,
            })
            .collect::<Vec<_>>();

        let total_estimated_tokens = context_allocations
            .iter()
            .map(|allocation| allocation.token_budget as usize)
            .sum();
        let total_estimated_duration_ms = sub_tasks
            .iter()
            .map(|task| task.token_budget as u64 * 3)
            .sum();

        Ok(ExecutionPlan {
            plan_id: format!("plan-{}", uuid::Uuid::new_v4()),
            task_description: task.user_input.clone(),
            summary: format!(
                "{} subtasks across {} parallel groups",
                sub_tasks.len(),
                parallel_groups.len()
            ),
            parallel_groups,
            group_dependencies,
            sub_tasks,
            dependencies,
            commit_points,
            context_allocations,
            total_estimated_tokens,
            total_estimated_duration_ms,
            plan_metadata: PlanMetadata {
                generated_by: AgentType::Planner,
                generated_at: Utc::now(),
                model_used: self.model_used.clone(),
                generation_duration_ms: 0,
                tokens_used: 0,
                version: 1,
            },
            created_at: Utc::now(),
        })
    }
}

impl Default for ExecutionPlanBuilder {
    fn default() -> Self {
        Self::new("planner")
    }
}

fn build_parallel_groups(sub_tasks: &[SubTask], layers: &[Vec<String>]) -> Vec<ParallelGroup> {
    let tasks_by_id = sub_tasks
        .iter()
        .cloned()
        .map(|task| (task.id.clone(), task))
        .collect::<HashMap<_, _>>();
    let mut groups = Vec::new();

    for (layer_index, layer) in layers.iter().enumerate() {
        let mut grouped = BTreeMap::<AgentType, Vec<SubTask>>::new();
        for task_id in layer {
            if let Some(task) = tasks_by_id.get(task_id).cloned() {
                grouped.entry(task.assigned_agent).or_default().push(task);
            }
        }

        for (slot, (agent_type, tasks)) in grouped.into_iter().enumerate() {
            groups.push(ParallelGroup {
                id: format!("G{}{}", layer_index + 1, slot + 1),
                name: format!("{agent_type} layer {}", layer_index + 1),
                sub_tasks: tasks,
                can_parallel: layer.len() > 1,
                depends_on: Vec::new(),
                agent_type,
            });
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use mc_core::{AgentType, Complexity, DependencyType, SubTask, TaskDescription, TaskIntent};

    use crate::routing::RouteLevel;

    use super::ExecutionPlanBuilder;

    fn task() -> TaskDescription {
        TaskDescription {
            id: "task-1".into(),
            user_input: "implement auth".into(),
            intent: TaskIntent::FeatureAddition,
            complexity: Complexity::Complex,
            affected_files: vec!["src/auth/mod.rs".into()],
            requires_new_dependency: false,
            involves_architecture_change: false,
            needs_external_research: false,
            requires_testing: true,
            forced_agents: None,
            constraints: Vec::new(),
            details: None,
            project_root: Some("C:/repo".into()),
            created_at: Utc::now(),
        }
    }

    fn subtask(id: &str, agent_type: AgentType, priority: u8) -> SubTask {
        SubTask {
            id: id.into(),
            description: format!("task-{id}"),
            target_files: vec![format!("src/{id}.rs")],
            expected_output: "done".into(),
            token_budget: 2_000,
            priority,
            estimated_complexity: Complexity::Medium,
            acceptance_criteria: vec!["ok".into()],
            completed: false,
            assigned_agent: agent_type,
        }
    }

    #[test]
    fn builder_creates_parallel_groups_and_commit_points() {
        let plan = ExecutionPlanBuilder::new("gpt-5.4")
            .build(
                &task(),
                &RouteLevel::Complex,
                24_000,
                vec![
                    subtask("T1", AgentType::Coder, 0),
                    subtask("T2", AgentType::Coder, 1),
                    subtask("T3", AgentType::Reviewer, 2),
                ],
                vec![
                    mc_core::TaskDependency {
                        upstream_task_id: "T1".into(),
                        downstream_task_id: "T3".into(),
                        dependency_type: DependencyType::Strong,
                        description: "review after first coding".into(),
                    },
                    mc_core::TaskDependency {
                        upstream_task_id: "T2".into(),
                        downstream_task_id: "T3".into(),
                        dependency_type: DependencyType::Weak,
                        description: "review after second coding".into(),
                    },
                ],
                None,
            )
            .unwrap();

        assert_eq!(plan.parallel_groups.len(), 2);
        assert_eq!(plan.commit_points.len(), 2);
        assert_eq!(plan.group_dependencies.len(), 1);
        assert!(plan.summary.contains("3 subtasks"));
    }

    #[test]
    fn builder_rejects_empty_subtasks() {
        let error = ExecutionPlanBuilder::default()
            .build(
                &task(),
                &RouteLevel::Simple,
                8_000,
                Vec::new(),
                Vec::new(),
                None,
            )
            .unwrap_err();

        assert!(error.to_string().contains("at least one subtask"));
    }
}
