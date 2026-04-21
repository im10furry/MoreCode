use serde::{Deserialize, Serialize};

use crate::store::{MetaJson, ProjectMemory, ProjectMemorySnapshot};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectMemoryState {
    Empty,
    Stale {
        meta: MetaJson,
        stale_threshold_days: i64,
    },
    Valid(Box<ProjectMemorySnapshot>),
}

impl ProjectMemoryState {
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Valid(_))
    }

    pub fn meta(&self) -> Option<&MetaJson> {
        match self {
            Self::Empty => None,
            Self::Stale { meta, .. } => Some(meta),
            Self::Valid(snapshot) => Some(&snapshot.meta),
        }
    }

    pub fn snapshot(&self) -> Option<&ProjectMemorySnapshot> {
        match self {
            Self::Valid(snapshot) => Some(snapshot.as_ref()),
            Self::Empty | Self::Stale { .. } => None,
        }
    }
}

impl From<ProjectMemory> for ProjectMemoryState {
    fn from(value: ProjectMemory) -> Self {
        match value {
            ProjectMemory::Empty => Self::Empty,
            ProjectMemory::Stale(meta) => Self::Stale {
                stale_threshold_days: meta.stale_threshold_days,
                meta,
            },
            ProjectMemory::Valid(snapshot) => Self::Valid(snapshot),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectMemoryState;
    use crate::store::{MetaJson, ProjectMemory, ProjectMemorySnapshot};

    #[test]
    fn project_memory_state_conversion_preserves_variants() {
        assert_eq!(
            ProjectMemoryState::from(ProjectMemory::Empty),
            ProjectMemoryState::Empty
        );

        let meta = MetaJson::default();
        let stale = ProjectMemoryState::from(ProjectMemory::Stale(meta.clone()));
        assert!(matches!(stale, ProjectMemoryState::Stale { .. }));
        assert_eq!(stale.meta(), Some(&meta));

        let snapshot = ProjectMemorySnapshot::default();
        let valid = ProjectMemoryState::from(ProjectMemory::Valid(Box::new(snapshot.clone())));
        assert_eq!(valid.snapshot(), Some(&snapshot));
        assert!(valid.is_usable());
    }
}
