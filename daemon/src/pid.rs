use std::path::{Path, PathBuf};

use crate::DaemonError;

#[derive(Debug)]
pub struct PidFileGuard {
    path: PathBuf,
}

impl PidFileGuard {
    pub fn acquire(path: impl Into<PathBuf>) -> Result<Self, DaemonError> {
        let path = path.into();
        if path.exists() {
            return Err(DaemonError::PidFile {
                path,
                reason: "pid file already exists".into(),
            });
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, std::process::id().to_string()).map_err(|error| DaemonError::PidFile {
            path: path.clone(),
            reason: error.to_string(),
        })?;

        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn read_pid(path: impl AsRef<Path>) -> Result<Option<u32>, DaemonError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(path).map_err(|error| DaemonError::PidFile {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
        let pid = contents.trim().parse::<u32>().map_err(|error| DaemonError::PidFile {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
        Ok(Some(pid))
    }
}

impl Drop for PidFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::PidFileGuard;

    #[test]
    fn pid_file_is_created_and_cleaned_up() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("daemon.pid");
        {
            let guard = PidFileGuard::acquire(&path).unwrap();
            assert_eq!(PidFileGuard::read_pid(&path).unwrap(), Some(std::process::id()));
            assert_eq!(guard.path(), path.as_path());
        }
        assert!(!path.exists());
    }
}
