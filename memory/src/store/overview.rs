use std::path::Path;

use tokio::fs;

use crate::error::MemoryError;

const FILE_NAME: &str = "project-overview.md";

pub async fn load_overview(memory_dir: &Path) -> Result<String, MemoryError> {
    let path = memory_dir.join(FILE_NAME);
    if !fs::try_exists(&path).await? {
        return Ok(String::new());
    }

    Ok(fs::read_to_string(path).await?)
}

pub async fn save_overview(memory_dir: &Path, overview: &str) -> Result<(), MemoryError> {
    fs::create_dir_all(memory_dir).await?;
    fs::write(memory_dir.join(FILE_NAME), overview).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{load_overview, save_overview};

    #[tokio::test]
    async fn overview_roundtrip() {
        let temp = tempdir().unwrap();
        save_overview(temp.path(), "# Project").await.unwrap();
        assert_eq!(load_overview(temp.path()).await.unwrap(), "# Project");
    }
}
