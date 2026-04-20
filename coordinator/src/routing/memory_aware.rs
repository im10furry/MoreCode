use mc_core::AgentType;

use crate::intent::TaskType;
use crate::routing::RouteLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRoutingMode {
    Disabled,
    ColdStart,
    WarmProjectMemory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryAwareRoutingDecision {
    pub mode: MemoryRoutingMode,
    pub route_level: RouteLevel,
    pub selected_agents: Vec<AgentType>,
    pub skipped_agents: Vec<AgentType>,
}

pub fn select_agent_set(
    route_level: &RouteLevel,
    task_type: &TaskType,
    memory_aware_routing: bool,
    has_memory: bool,
    preflight_check: bool,
) -> Vec<AgentType> {
    build_memory_aware_decision(
        route_level,
        task_type,
        memory_aware_routing,
        has_memory,
        preflight_check,
    )
    .selected_agents
}

pub fn build_memory_aware_decision(
    route_level: &RouteLevel,
    task_type: &TaskType,
    memory_aware_routing: bool,
    has_memory: bool,
    preflight_check: bool,
) -> MemoryAwareRoutingDecision {
    let warm_memory = memory_aware_routing && has_memory;
    let (selected_agents, skipped_agents) = match route_level {
        RouteLevel::Simple => (vec![task_type.preferred_agent()], Vec::new()),
        RouteLevel::Medium if warm_memory => (
            with_optional_preflight(
                vec![AgentType::Explorer, AgentType::Coder, AgentType::Reviewer],
                preflight_check,
            ),
            vec![AgentType::ImpactAnalyzer],
        ),
        RouteLevel::Medium => (
            with_optional_preflight(
                vec![
                    AgentType::Explorer,
                    AgentType::ImpactAnalyzer,
                    AgentType::Coder,
                    AgentType::Reviewer,
                ],
                preflight_check,
            ),
            Vec::new(),
        ),
        RouteLevel::Complex => (
            with_optional_preflight(
                vec![
                    AgentType::Explorer,
                    AgentType::ImpactAnalyzer,
                    AgentType::Planner,
                    AgentType::Coder,
                    AgentType::Reviewer,
                    AgentType::Tester,
                ],
                preflight_check,
            ),
            Vec::new(),
        ),
        RouteLevel::Research => (vec![AgentType::Research], Vec::new()),
    };

    MemoryAwareRoutingDecision {
        mode: if !memory_aware_routing {
            MemoryRoutingMode::Disabled
        } else if has_memory {
            MemoryRoutingMode::WarmProjectMemory
        } else {
            MemoryRoutingMode::ColdStart
        },
        route_level: route_level.clone(),
        selected_agents,
        skipped_agents,
    }
}

fn with_optional_preflight(mut agents: Vec<AgentType>, preflight_check: bool) -> Vec<AgentType> {
    if !preflight_check {
        agents.retain(|agent_type| *agent_type != AgentType::Reviewer);
    }
    agents
}

#[cfg(test)]
mod tests {
    use mc_core::AgentType;

    use crate::intent::TaskType;
    use crate::routing::RouteLevel;

    use super::{build_memory_aware_decision, select_agent_set, MemoryRoutingMode};

    #[test]
    fn medium_route_with_memory_skips_impact_analyzer_only() {
        let decision = build_memory_aware_decision(
            &RouteLevel::Medium,
            &TaskType::FeatureDevelopment,
            true,
            true,
            true,
        );

        assert_eq!(decision.mode, MemoryRoutingMode::WarmProjectMemory);
        assert_eq!(
            decision.selected_agents,
            vec![AgentType::Explorer, AgentType::Coder, AgentType::Reviewer]
        );
        assert_eq!(decision.skipped_agents, vec![AgentType::ImpactAnalyzer]);
    }

    #[test]
    fn medium_route_without_memory_uses_full_cognitive_path() {
        let agents = select_agent_set(&RouteLevel::Medium, &TaskType::BugFix, true, false, true);

        assert_eq!(
            agents,
            vec![
                AgentType::Explorer,
                AgentType::ImpactAnalyzer,
                AgentType::Coder,
                AgentType::Reviewer
            ]
        );
    }

    #[test]
    fn preflight_flag_removes_reviewer_from_routing_sets() {
        let agents = select_agent_set(
            &RouteLevel::Complex,
            &TaskType::Refactoring,
            true,
            true,
            false,
        );

        assert!(!agents.contains(&AgentType::Reviewer));
        assert!(agents.contains(&AgentType::Planner));
        assert!(agents.contains(&AgentType::Tester));
    }

    #[test]
    fn simple_route_always_prefers_task_specific_agent() {
        let agents = select_agent_set(
            &RouteLevel::Simple,
            &TaskType::Documentation,
            true,
            true,
            true,
        );

        assert_eq!(agents, vec![AgentType::DocWriter]);
    }
}
