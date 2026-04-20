use std::collections::HashMap;

use mc_context::ProjectContext;
use mc_core::{AgentType, ContextAllocation, SubTask, TaskDescription};

use crate::routing::RouteLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanAllocationConfig {
    pub default_context_window_limit: usize,
    pub max_project_knowledge_items: usize,
}

impl Default for PlanAllocationConfig {
    fn default() -> Self {
        Self {
            default_context_window_limit: 16_000,
            max_project_knowledge_items: 6,
        }
    }
}

pub fn allocate_agent_budgets(
    max_token_budget: u32,
    agents: &[AgentType],
    route_level: &RouteLevel,
) -> HashMap<AgentType, u32> {
    if agents.is_empty() {
        return HashMap::new();
    }

    let total_budget = match route_level {
        RouteLevel::Simple => max_token_budget / 4,
        RouteLevel::Medium => max_token_budget / 2,
        RouteLevel::Complex => max_token_budget,
        RouteLevel::Research => max_token_budget * 3 / 4,
    };

    let total_weight = agents
        .iter()
        .map(|agent_type| agent_weight(*agent_type))
        .sum::<u32>()
        .max(1);

    agents
        .iter()
        .map(|agent_type| {
            (
                *agent_type,
                (total_budget * agent_weight(*agent_type) / total_weight).max(1_000),
            )
        })
        .collect()
}

pub struct ContextAllocator {
    config: PlanAllocationConfig,
}

impl ContextAllocator {
    pub fn new(config: PlanAllocationConfig) -> Self {
        Self { config }
    }

    pub fn allocate(
        &self,
        task: &TaskDescription,
        project_ctx: Option<&ProjectContext>,
        sub_tasks: &[SubTask],
        agent_budgets: &HashMap<AgentType, u32>,
    ) -> Vec<ContextAllocation> {
        sub_tasks
            .iter()
            .map(|sub_task| {
                let token_budget = if sub_task.token_budget > 0 {
                    sub_task.token_budget
                } else {
                    agent_budgets
                        .get(&sub_task.assigned_agent)
                        .copied()
                        .unwrap_or(1_000)
                };

                ContextAllocation {
                    sub_task_id: sub_task.id.clone(),
                    agent_type: sub_task.assigned_agent,
                    token_budget,
                    required_files: if sub_task.target_files.is_empty() {
                        task.affected_files.clone()
                    } else {
                        sub_task.target_files.clone()
                    },
                    project_knowledge_subset: project_knowledge_subset(
                        project_ctx,
                        sub_task,
                        self.config.max_project_knowledge_items,
                    ),
                    context_window_limit: self
                        .config
                        .default_context_window_limit
                        .max(token_budget as usize * 8),
                }
            })
            .collect()
    }
}

impl Default for ContextAllocator {
    fn default() -> Self {
        Self::new(PlanAllocationConfig::default())
    }
}

fn agent_weight(agent_type: AgentType) -> u32 {
    match agent_type {
        AgentType::Coder | AgentType::Research => 2,
        AgentType::Planner | AgentType::Reviewer | AgentType::Tester => 1,
        AgentType::Explorer
        | AgentType::ImpactAnalyzer
        | AgentType::Debugger
        | AgentType::DocWriter => 1,
        AgentType::Coordinator => 1,
    }
}

fn project_knowledge_subset(
    project_ctx: Option<&ProjectContext>,
    sub_task: &SubTask,
    limit: usize,
) -> Vec<String> {
    let Some(project_ctx) = project_ctx else {
        return Vec::new();
    };

    let mut items = Vec::new();
    items.push(format!("project={}", project_ctx.info.name));
    if let Some(language) = primary_language(project_ctx) {
        items.push(format!("language={language}"));
    }
    if let Some(framework) = project_ctx.tech_stack.frameworks.first() {
        items.push(format!("framework={framework}"));
    }
    if !project_ctx.notes.is_empty() {
        items.push(format!("notes={}", project_ctx.notes.join(",")));
    }

    for risk in &project_ctx.risk_areas {
        if sub_task
            .target_files
            .iter()
            .any(|file| file == &risk.name || file.contains(&risk.name) || risk.name.contains(file))
        {
            items.push(format!("risk={}::{:?}", risk.name, risk.level));
        }
    }

    items.sort();
    items.dedup();
    items.truncate(limit);
    items
}

fn primary_language(project_ctx: &ProjectContext) -> Option<&str> {
    project_ctx
        .info
        .primary_language
        .as_deref()
        .or_else(|| project_ctx.tech_stack.primary_language())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;
    use mc_context::{ProjectContext, ProjectInfo, RiskArea, RiskLevel, TechStack};
    use mc_core::{AgentType, Complexity, SubTask, TaskDescription, TaskIntent};

    use crate::routing::RouteLevel;

    use super::{allocate_agent_budgets, ContextAllocator};

    fn sample_task() -> TaskDescription {
        TaskDescription {
            id: "task-1".into(),
            user_input: "add auth".into(),
            intent: TaskIntent::FeatureAddition,
            complexity: Complexity::Medium,
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

    fn sample_subtask() -> SubTask {
        SubTask {
            id: "sub-1".into(),
            description: "implement auth".into(),
            target_files: vec!["src/auth/mod.rs".into()],
            expected_output: "done".into(),
            token_budget: 2_000,
            priority: 0,
            estimated_complexity: Complexity::Medium,
            acceptance_criteria: vec!["tests pass".into()],
            completed: false,
            assigned_agent: AgentType::Coder,
        }
    }

    fn sample_project_context() -> ProjectContext {
        ProjectContext {
            info: ProjectInfo {
                name: "MoreCode".into(),
                root_dir: "C:/repo".into(),
                primary_language: Some("Rust".into()),
                repository_url: None,
                summary: Some("workspace".into()),
            },
            tech_stack: TechStack::default(),
            conventions: Default::default(),
            risk_areas: vec![RiskArea {
                name: "src/auth/mod.rs".into(),
                level: RiskLevel::High,
                rationale: "auth is sensitive".into(),
                mitigation: Some("run focused review".into()),
            }],
            scan_metadata: Default::default(),
            impact_report: None,
            notes: vec!["auth module".into()],
        }
    }

    #[test]
    fn allocate_agent_budgets_matches_route_weights() {
        let budgets = allocate_agent_budgets(
            24_000,
            &[AgentType::Coder, AgentType::Reviewer, AgentType::Tester],
            &RouteLevel::Medium,
        );

        assert!(budgets[&AgentType::Coder] > budgets[&AgentType::Reviewer]);
        assert_eq!(budgets.len(), 3);
    }

    #[test]
    fn context_allocator_uses_subtask_files_and_project_knowledge() {
        let allocator = ContextAllocator::default();
        let allocations = allocator.allocate(
            &sample_task(),
            Some(&sample_project_context()),
            &[sample_subtask()],
            &HashMap::from([(AgentType::Coder, 4_000)]),
        );

        assert_eq!(allocations.len(), 1);
        assert_eq!(
            allocations[0].required_files,
            vec!["src/auth/mod.rs".to_string()]
        );
        assert!(allocations[0]
            .project_knowledge_subset
            .iter()
            .any(|item| item.contains("risk=src/auth/mod.rs")));
        assert!(allocations[0].context_window_limit >= 16_000);
    }
}
