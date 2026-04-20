use std::path::Path;

use tokio::fs;

use crate::error::MemoryError;

const FILE_NAME: &str = "conventions.md";

pub async fn load_conventions(memory_dir: &Path) -> Result<String, MemoryError> {
    let path = memory_dir.join(FILE_NAME);
    if !fs::try_exists(&path).await? {
        return Ok(String::new());
    }

    Ok(fs::read_to_string(path).await?)
}

pub async fn save_conventions(memory_dir: &Path, conventions: &str) -> Result<(), MemoryError> {
    fs::create_dir_all(memory_dir).await?;
    fs::write(memory_dir.join(FILE_NAME), conventions).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{load_conventions, save_conventions};

    #[tokio::test]
    async fn conventions_roundtrip() {
        let temp = tempdir().unwrap();
        save_conventions(temp.path(), "- no unwrap").await.unwrap();
        assert_eq!(load_conventions(temp.path()).await.unwrap(), "- no unwrap");
    }
}
