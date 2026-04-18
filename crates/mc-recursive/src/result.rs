use std::{collections::HashMap, time::Duration};

use mc_core::{agent::AgentType, token::TokenUsage};
use serde::{Deserialize, Serialize};

use crate::stats::RecursiveStats;

/// Final reduced result returned by the recursive orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AggregatedResult {
    pub findings: Vec<String>,
    pub confidence_scores: HashMap<String, f64>,
    pub contradictions: Vec<Contradiction>,
    pub total_tokens_used: usize,
    pub sub_agents_used: usize,
    pub depth_reached: usize,
    pub summary: String,
}

/// Record of a contradiction detected between two child-agent outputs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contradiction {
    pub description: String,
    pub agent_a_id: String,
    pub agent_a_conclusion: String,
    pub agent_b_id: String,
    pub agent_b_conclusion: String,
    pub severity: ContradictionSeverity,
}

/// Severity assigned to a contradiction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContradictionSeverity {
    Info,
    Warning,
    Error,
}

/// Generic wrapper that preserves the aggregate, sub-results, and stats together.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecursiveResult<T> {
    pub aggregated: T,
    pub sub_results: Vec<SubResult<T>>,
    pub stats: RecursiveStats,
    pub depth_reached: usize,
}

/// Generic child-result wrapper used by external callers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubResult<T> {
    pub sub_agent_id: String,
    pub agent_type: AgentType,
    pub result: Option<T>,
    pub token_usage: TokenUsage,
    pub duration: Duration,
    pub success: bool,
    pub error: Option<String>,
}

/// Compatibility alias kept for the terminology used in the architecture doc.
pub type RecursiveOrchestrationResult = AggregatedResult;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use mc_core::{agent::AgentType, token::TokenUsage};

    use super::{AggregatedResult, RecursiveResult, SubResult};
    use crate::RecursiveStats;

    #[test]
    fn recursive_result_is_generic() {
        let aggregated = AggregatedResult {
            findings: vec!["发现".to_string()],
            confidence_scores: HashMap::new(),
            contradictions: Vec::new(),
            total_tokens_used: 10,
            sub_agents_used: 1,
            depth_reached: 1,
            summary: "ok".to_string(),
        };

        let wrapper = RecursiveResult {
            aggregated: aggregated.clone(),
            sub_results: vec![SubResult {
                sub_agent_id: "sub-1".to_string(),
                agent_type: AgentType::Explorer,
                result: Some(aggregated),
                token_usage: TokenUsage::default(),
                duration: std::time::Duration::from_secs(1),
                success: true,
                error: None,
            }],
            stats: RecursiveStats::default(),
            depth_reached: 1,
        };

        assert_eq!(wrapper.sub_results.len(), 1);
    }
}
