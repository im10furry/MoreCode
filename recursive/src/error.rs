use thiserror::Error;

/// Recursive orchestration specific failure modes.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RecursiveError {
    /// The configured or requested recursion depth is too large.
    #[error("递归深度超限: 当前深度 {current_depth} >= 最大深度 {max_depth}")]
    DepthLimitExceeded {
        current_depth: usize,
        max_depth: usize,
    },
    /// Too many child agents were requested for a single level.
    #[error("子 Agent 数量超限: 请求 {requested}, 最大 {max_sub_agents}")]
    TooManySubAgents {
        requested: usize,
        max_sub_agents: usize,
    },
    /// The global concurrency budget is exhausted.
    #[error("全局 Agent 总数超限: 请求 {requested}, 可用 {available}")]
    GlobalAgentLimitExceeded { requested: usize, available: usize },
    /// The requested child budget does not fit within the parent's budget.
    #[error("Token 预算不足: 请求 {requested}, 可用 {available}")]
    BudgetExceeded { requested: u64, available: u64 },
    /// Execution was cancelled by the parent.
    #[error("递归编排已取消")]
    Cancelled,
    /// No child-agent result could be reduced into a final output.
    #[error("没有可聚合的成功子 Agent 结果")]
    NoSuccessfulResults,
    /// Generic config validation error.
    #[error("递归配置无效: {0}")]
    InvalidConfig(String),
}

/// Local result alias used inside the crate for strongly typed errors.
pub type RecursiveEngineResult<T> = std::result::Result<T, RecursiveError>;
