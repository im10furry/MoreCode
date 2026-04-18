use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::project::risk::RiskLevel;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    AddFile,
    ModifyFile,
    DeleteFile,
    AddDependency,
    ModifyConfig,
    DatabaseMigration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImpactChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImpactReport {
    pub direct_impacts: Vec<ImpactChange>,
    pub indirect_impacts: Vec<ImpactChange>,
    pub risk_assessment: RiskLevel,
}

impl ImpactReport {
    pub fn all_changes(&self) -> impl Iterator<Item = &ImpactChange> {
        self.direct_impacts
            .iter()
            .chain(self.indirect_impacts.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::{ChangeType, ImpactChange, ImpactReport};
    use crate::project::risk::RiskLevel;

    #[test]
    fn all_changes_includes_direct_and_indirect() {
        let report = ImpactReport {
            direct_impacts: vec![ImpactChange {
                path: "src/lib.rs".into(),
                change_type: ChangeType::ModifyFile,
                note: "update".into(),
            }],
            indirect_impacts: vec![ImpactChange {
                path: "Cargo.toml".into(),
                change_type: ChangeType::ModifyConfig,
                note: "deps".into(),
            }],
            risk_assessment: RiskLevel::Medium,
        };

        assert_eq!(report.all_changes().count(), 2);
    }
}
