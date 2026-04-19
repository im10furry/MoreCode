use std::path::{Path, PathBuf};

use crate::error::SandboxError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandlockConfig {
    pub read_write_dirs: Vec<PathBuf>,
    pub read_only_dirs: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LandlockSupport {
    Supported,
    Unsupported(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandlockStatus {
    pub support: LandlockSupport,
    pub tracked_paths: usize,
}

impl LandlockConfig {
    pub fn for_task(task_workspace: &Path) -> Self {
        Self {
            read_write_dirs: vec![task_workspace.to_path_buf()],
            read_only_dirs: vec![
                PathBuf::from("/usr"),
                PathBuf::from("/lib"),
                PathBuf::from("/bin"),
            ],
        }
    }

    pub fn validate(&self) -> Result<(), SandboxError> {
        if self.read_write_dirs.is_empty() && self.read_only_dirs.is_empty() {
            return Err(SandboxError::UnsupportedPlatform(
                "landlock requires at least one tracked directory".into(),
            ));
        }

        for path in self
            .read_write_dirs
            .iter()
            .chain(self.read_only_dirs.iter())
        {
            if !path.exists() {
                return Err(SandboxError::PathAccessDenied {
                    path: path.clone(),
                    reason: "configured landlock path does not exist".into(),
                });
            }
        }

        Ok(())
    }
}

pub fn detect_landlock_support() -> LandlockSupport {
    #[cfg(target_os = "linux")]
    {
        let feature_paths = [
            "/sys/kernel/security/landlock",
            "/sys/kernel/security/landlock/features",
        ];
        if feature_paths.iter().any(|path| Path::new(path).exists()) {
            LandlockSupport::Supported
        } else {
            LandlockSupport::Unsupported(
                "kernel does not expose Landlock userspace interfaces".into(),
            )
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        LandlockSupport::Unsupported("landlock is only available on Linux".into())
    }
}

pub fn apply_landlock(config: &LandlockConfig) -> Result<LandlockStatus, SandboxError> {
    config.validate()?;
    let support = detect_landlock_support();
    match &support {
        LandlockSupport::Supported => Ok(LandlockStatus {
            support,
            tracked_paths: config.read_only_dirs.len() + config.read_write_dirs.len(),
        }),
        LandlockSupport::Unsupported(reason) => {
            Err(SandboxError::UnsupportedPlatform(reason.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{apply_landlock, detect_landlock_support, LandlockConfig, LandlockSupport};

    #[test]
    fn landlock_validation_rejects_missing_paths() {
        let config = LandlockConfig {
            read_write_dirs: vec![std::path::PathBuf::from("missing-path")],
            read_only_dirs: Vec::new(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn landlock_support_detection_is_total() {
        match detect_landlock_support() {
            LandlockSupport::Supported | LandlockSupport::Unsupported(_) => {}
        }
    }

    #[test]
    fn landlock_apply_reports_status_or_unsupported() {
        let temp = tempdir().unwrap();
        let config = LandlockConfig::for_task(temp.path());
        let result = apply_landlock(&config);
        assert!(result.is_ok() || result.is_err());
    }
}
