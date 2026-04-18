use mc_core::AgentType;

use crate::intent::TaskType;
use crate::routing::RouteLevel;

pub fn select_agent_set(
    route_level: &RouteLevel,
    task_type: &TaskType,
    memory_aware_routing: bool,
    has_memory: bool,
    preflight_check: bool,
) -> Vec<AgentType> {
    let mut agents = match route_level {
        RouteLevel::Simple => vec![task_type.preferred_agent()],
        RouteLevel::Medium if memory_aware_routing && has_memory => {
            vec![AgentType::Coder, AgentType::Reviewer]
        }
        RouteLevel::Medium => vec![AgentType::Explorer, AgentType::Coder, AgentType::Reviewer],
        RouteLevel::Complex => vec![
            AgentType::Explorer,
            AgentType::ImpactAnalyzer,
            AgentType::Planner,
            AgentType::Coder,
            AgentType::Reviewer,
            AgentType::Tester,
        ],
        RouteLevel::Research => vec![AgentType::Research],
    };

    if !preflight_check {
        agents.retain(|agent_type| {
            *agent_type != AgentType::Reviewer
                || !matches!(route_level, RouteLevel::Medium | RouteLevel::Complex)
        });
    }

    agents
}
