use serde::{Deserialize, Serialize};

use crate::complexity::TaskComplexity;

/// Decision returned by the recursion policy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecursiveDecision {
    /// Continue with recursive splitting.
    Proceed {
        suggested_sub_agents: usize,
        split_dimension: String,
    },
    /// Refuse to recurse because the tree is already too deep.
    TooDeep {
        current_depth: usize,
        max_depth: usize,
    },
    /// Refuse to recurse because there are too few tasks to justify it.
    TooFewTasks {
        actual_count: usize,
        min_threshold: usize,
    },
    /// Refuse to recurse because even the minimum budget is not available.
    InsufficientBudget { required: u64, available: u64 },
    /// Fall back to a non-recursive parallel execution.
    Degrade { reason: String },
}

/// Decide whether a task should recurse or stay at the current parallel layer.
pub fn should_recursively_split(
    task_complexity: TaskComplexity,
    current_depth: usize,
    max_depth: usize,
    available_budget: u64,
    estimated_child_budget: u64,
    min_child_budget: u64,
    sub_task_count: usize,
) -> RecursiveDecision {
    if current_depth >= max_depth {
        return RecursiveDecision::TooDeep {
            current_depth,
            max_depth,
        };
    }

    if sub_task_count < 2 {
        return RecursiveDecision::TooFewTasks {
            actual_count: sub_task_count,
            min_threshold: 2,
        };
    }

    let required_per_child = estimated_child_budget.max(min_child_budget);
    let required = required_per_child.saturating_mul(sub_task_count as u64);
    if required > available_budget {
        if sub_task_count <= 2 && available_budget >= min_child_budget.saturating_mul(2) {
            return RecursiveDecision::Degrade {
                reason: format!(
                    "预算不足以递归拆分（需要 {required}，可用 {available_budget}），降级为并行执行"
                ),
            };
        }
        return RecursiveDecision::InsufficientBudget {
            required,
            available: available_budget,
        };
    }

    if sub_task_count <= 2 {
        return RecursiveDecision::Degrade {
            reason: format!("子任务数为 {sub_task_count}，按约束使用并行执行"),
        };
    }

    RecursiveDecision::Proceed {
        suggested_sub_agents: sub_task_count,
        split_dimension: match task_complexity {
            TaskComplexity::Light => "files",
            TaskComplexity::Normal => "modules",
            TaskComplexity::Heavy => "concerns",
            TaskComplexity::Deep => "recursive-dimensions",
        }
        .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{should_recursively_split, RecursiveDecision};
    use crate::complexity::TaskComplexity;

    #[test]
    fn returns_too_deep_when_depth_is_exceeded() {
        let decision =
            should_recursively_split(TaskComplexity::Deep, 2, 2, 20_000, 4_000, 2_000, 3);
        assert_eq!(
            decision,
            RecursiveDecision::TooDeep {
                current_depth: 2,
                max_depth: 2,
            }
        );
    }

    #[test]
    fn returns_degrade_for_small_parallel_batches() {
        let decision =
            should_recursively_split(TaskComplexity::Normal, 0, 2, 20_000, 4_000, 2_000, 2);
        assert!(matches!(decision, RecursiveDecision::Degrade { .. }));
    }

    #[test]
    fn returns_proceed_for_large_complex_batches() {
        let decision =
            should_recursively_split(TaskComplexity::Heavy, 0, 2, 30_000, 4_000, 2_000, 3);
        assert!(matches!(decision, RecursiveDecision::Proceed { .. }));
    }
}
