use std::path::Path;

use tokio::fs;

use crate::error::MemoryError;

use super::types::TechStack;

const FILE_NAME: &str = "tech-stack.json";

pub async fn load_tech_stack(memory_dir: &Path) -> Result<TechStack, MemoryError> {
    let path = memory_dir.join(FILE_NAME);
    if !fs::try_exists(&path).await? {
        return Ok(TechStack::default());
    }

    Ok(serde_json::from_str(&fs::read_to_string(path).await?)?)
}

pub async fn save_tech_stack(memory_dir: &Path, tech_stack: &TechStack) -> Result<(), MemoryError> {
    fs::create_dir_all(memory_dir).await?;
    fs::write(
        memory_dir.join(FILE_NAME),
        serde_json::to_vec_pretty(tech_stack)?,
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{load_tech_stack, save_tech_stack};

    #[tokio::test]
    async fn tech_stack_roundtrip() {
        let temp = tempdir().unwrap();
        let tech_stack = crate::store::TechStack {
            language: "Rust".into(),
            edition: "2021".into(),
            framework: std::collections::BTreeMap::from([("web".into(), "axum".into())]),
            dev_tools: Default::default(),
            key_dependencies: vec!["serde".into()],
        };

        save_tech_stack(temp.path(), &tech_stack).await.unwrap();
        assert_eq!(load_tech_stack(temp.path()).await.unwrap(), tech_stack);
    }
}
