use serde::{Deserialize, Serialize};

/// Complexity heuristic used to decide whether a task is worth recursive splitting.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskComplexity {
    /// Small and shallow work.
    Light,
    /// Default complexity level.
    Normal,
    /// Broader multi-file or multi-track work.
    Heavy,
    /// Deep work that often benefits from another recursive layer.
    Deep,
}

impl TaskComplexity {
    /// Relative budgeting weight used by higher-level planning heuristics.
    pub const fn weight(self) -> f64 {
        match self {
            Self::Light => 0.5,
            Self::Normal => 1.0,
            Self::Heavy => 1.3,
            Self::Deep => 1.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TaskComplexity;

    #[test]
    fn weights_increase_with_complexity() {
        assert!(TaskComplexity::Light.weight() < TaskComplexity::Normal.weight());
        assert!(TaskComplexity::Normal.weight() < TaskComplexity::Heavy.weight());
        assert!(TaskComplexity::Heavy.weight() < TaskComplexity::Deep.weight());
    }
}
