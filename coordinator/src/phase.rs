use std::collections::HashMap;

use chrono::{DateTime, Utc};
use mc_core::AgentType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionPhase {
    Receiving,
    Understanding,
    Clarifying,
    LoadingMemory,
    EvaluatingComplexity,
    Routing,
    Dispatching,
    Integrating,
    Delivering,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentExecutionState {
    Pending,
    Running,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentRuntimeStatus {
    pub agent_type: AgentType,
    pub state: AgentExecutionState,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub tokens_used: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionError {
    pub phase: ExecutionPhase,
    pub message: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionStatus {
    pub current_phase: ExecutionPhase,
    pub agent_statuses: HashMap<String, AgentRuntimeStatus>,
    pub progress_percent: f32,
    pub tokens_used: usize,
    pub tokens_remaining: usize,
    pub started_at: DateTime<Utc>,
    pub errors: Vec<ExecutionError>,
}

impl ExecutionStatus {
    pub fn new(tokens_remaining: usize) -> Self {
        Self {
            current_phase: ExecutionPhase::Receiving,
            agent_statuses: HashMap::new(),
            progress_percent: 0.0,
            tokens_used: 0,
            tokens_remaining,
            started_at: Utc::now(),
            errors: Vec::new(),
        }
    }
}
