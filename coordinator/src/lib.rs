#![forbid(unsafe_code)]

mod config;
mod coordinator;
mod error;
mod intent;
mod phase;
mod response;
mod routing;

pub use config::CoordinatorConfig;
pub use coordinator::{Coordinator, DaemonTask, HandoffFormat, MemoryState};
pub use error::CoordinatorError;
pub use intent::{Clarification, IntentAnalysis, Question, TaskType, UserIntent};
pub use phase::{
    AgentExecutionState, AgentRuntimeStatus, ExecutionError, ExecutionPhase, ExecutionStatus,
};
pub use response::{CoordinatorResponse, ResponseType};
pub use routing::{
    CalibrationRecord, ComplexityConfig, ComplexityEvaluation, ComplexityEvaluator,
    ComplexityFactors, RouteLevel, RouteThresholds,
};
