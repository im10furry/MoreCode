use std::path::Path;

use tokio::fs;

use crate::error::MemoryError;

use super::types::{RiskAreas, RiskInfo};

const FILE_NAME: &str = "risk-areas.json";

pub async fn load_risk_areas(memory_dir: &Path) -> Result<RiskAreas, MemoryError> {
    let path = memory_dir.join(FILE_NAME);
    if !fs::try_exists(&path).await? {
        return Ok(RiskAreas::default());
    }

    Ok(serde_json::from_str(&fs::read_to_string(path).await?)?)
}

pub async fn save_risk_areas(memory_dir: &Path, risk_areas: &RiskAreas) -> Result<(), MemoryError> {
    fs::create_dir_all(memory_dir).await?;
    fs::write(
        memory_dir.join(FILE_NAME),
        serde_json::to_vec_pretty(risk_areas)?,
    )
    .await?;
    Ok(())
}

pub async fn append_risk(memory_dir: &Path, risk: RiskInfo) -> Result<(), MemoryError> {
    let mut current = load_risk_areas(memory_dir).await?;
    current.risks.push(risk);
    save_risk_areas(memory_dir, &current).await
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use tempfile::tempdir;

    use crate::store::RiskInfo;

    use super::{append_risk, load_risk_areas};

    #[tokio::test]
    async fn risk_roundtrip_and_append() {
        let temp = tempdir().unwrap();
        append_risk(
            temp.path(),
            RiskInfo {
                area: "src/lib.rs:1".into(),
                r#type: "security".into(),
                description: "test".into(),
                severity: "high".into(),
                discovered_at: Utc::now(),
                discovered_by: "reviewer".into(),
            },
        )
        .await
        .unwrap();

        let risks = load_risk_areas(temp.path()).await.unwrap();
        assert_eq!(risks.risks.len(), 1);
    }
}
