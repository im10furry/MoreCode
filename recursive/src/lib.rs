#![forbid(unsafe_code)]

pub mod aggregator;
pub mod budget_allocator;
pub mod complexity;
pub mod config;
pub mod decision;
pub mod error;
pub mod filter;
pub mod limiter;
pub mod orchestrator;
pub mod result;
pub mod stats;
pub mod sub_agent;
pub mod task;

pub use aggregator::{detect_contradictions, ResultAggregator};
pub use budget_allocator::{AgentModelConfig, TokenBudgetAllocator};
pub use complexity::TaskComplexity;
pub use config::RecursiveConfig;
pub use decision::{should_recursively_split, RecursiveDecision};
pub use error::{RecursiveEngineResult, RecursiveError};
pub use filter::{
    code_reading_filter_strategy, estimate_tokens, matches_rule, CompressedEntry, FilterEngine,
    FilterRule, FilterRulePriority, FilterRuleType, FilterStrategy, FilteredResult, RegexCache,
};
pub use limiter::{ResourceLimiter, ResourcePermit};
pub use orchestrator::RecursiveOrchestrator;
pub use result::{
    AggregatedResult, Contradiction, ContradictionSeverity, RecursiveOrchestrationResult,
    RecursiveResult, SubResult,
};
pub use stats::RecursiveStats;
pub use sub_agent::{
    AgentFactory, FileSlice, SubAgentContext, SubAgentExecutor, SubAgentResult, SubAgentSpec,
    SubAgentStatus,
};
pub use task::{RecursiveTask, RecursiveTaskStatus};
