use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskArea {
    pub name: String,
    pub level: RiskLevel,
    pub rationale: String,
    pub mitigation: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::RiskLevel;

    #[test]
    fn risk_level_orders_by_severity() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }
}
