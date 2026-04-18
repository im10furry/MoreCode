use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::SandboxError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathRestrictionType {
    Allow,
    Deny,
    ReadOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathRestriction {
    pub restriction_type: PathRestrictionType,
    pub path: PathBuf,
    pub recursive: bool,
    pub description: String,
}

impl PathRestriction {
    pub fn allow(path: impl Into<PathBuf>) -> Self {
        Self {
            restriction_type: PathRestrictionType::Allow,
            path: path.into(),
            recursive: true,
            description: String::new(),
        }
    }

    pub fn deny(path: impl Into<PathBuf>) -> Self {
        Self {
            restriction_type: PathRestrictionType::Deny,
            path: path.into(),
            recursive: true,
            description: String::new(),
        }
    }

    pub fn read_only(path: impl Into<PathBuf>) -> Self {
        Self {
            restriction_type: PathRestrictionType::ReadOnly,
            path: path.into(),
            recursive: true,
            description: String::new(),
        }
    }

    pub fn is_allowed(&self, target: &Path, is_write: bool) -> bool {
        let Ok(normalized_target) = normalize_path_no_symlink_escape(target) else {
            return false;
        };
        let Ok(normalized_rule) = normalize_path_no_symlink_escape(&self.path) else {
            return false;
        };
        self.is_allowed_normalized(&normalized_target, &normalized_rule, is_write)
    }

    pub fn allows_path(rules: &[Self], target: &Path, is_write: bool) -> bool {
        if rules.is_empty() {
            return true;
        }

        let Ok(normalized_target) = normalize_path_no_symlink_escape(target) else {
            return false;
        };

        let mut has_positive_rule = false;
        let mut matched_positive_rule = false;

        for rule in rules {
            let Ok(normalized_rule) = normalize_path_no_symlink_escape(&rule.path) else {
                return false;
            };

            if rule.matches_normalized(&normalized_target, &normalized_rule)
                && rule.restriction_type == PathRestrictionType::Deny
            {
                return false;
            }

            if matches!(
                rule.restriction_type,
                PathRestrictionType::Allow | PathRestrictionType::ReadOnly
            ) {
                has_positive_rule = true;
                if rule.is_allowed_normalized(&normalized_target, &normalized_rule, is_write) {
                    matched_positive_rule = true;
                }
            }
        }

        if has_positive_rule {
            matched_positive_rule
        } else {
            true
        }
    }

    fn is_allowed_normalized(
        &self,
        normalized_target: &Path,
        normalized_rule: &Path,
        is_write: bool,
    ) -> bool {
        let matches = self.matches_normalized(normalized_target, normalized_rule);
        match self.restriction_type {
            PathRestrictionType::Allow => matches,
            PathRestrictionType::Deny => !matches,
            PathRestrictionType::ReadOnly => matches && !is_write,
        }
    }

    fn matches_normalized(&self, normalized_target: &Path, normalized_rule: &Path) -> bool {
        if self.recursive {
            normalized_target.starts_with(normalized_rule)
        } else {
            normalized_target == normalized_rule
        }
    }
}

pub(crate) fn normalize_path_no_symlink_escape(path: &Path) -> Result<PathBuf, SandboxError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(SandboxError::from)?
            .join(path)
    };

    ensure_no_symlink_components(&absolute)?;

    let (existing_ancestor, missing_tail) = split_existing_ancestor(&absolute)?;
    let mut normalized = std::fs::canonicalize(&existing_ancestor).map_err(|error| {
        SandboxError::PathAccessDenied {
            path: absolute.clone(),
            reason: error.to_string(),
        }
    })?;

    for component in missing_tail {
        normalized.push(component);
    }

    Ok(normalized)
}

fn split_existing_ancestor(path: &Path) -> Result<(PathBuf, Vec<OsString>), SandboxError> {
    let mut cursor = path.to_path_buf();
    let mut tail = Vec::new();

    while !cursor.exists() {
        let file_name = cursor
            .file_name()
            .ok_or_else(|| SandboxError::PathAccessDenied {
                path: path.to_path_buf(),
                reason: "找不到可规范化的现存父路径".to_string(),
            })?;
        tail.push(file_name.to_os_string());
        cursor = cursor
            .parent()
            .ok_or_else(|| SandboxError::PathAccessDenied {
                path: path.to_path_buf(),
                reason: "路径没有可用父目录".to_string(),
            })?
            .to_path_buf();
    }

    tail.reverse();
    Ok((cursor, tail))
}

fn ensure_no_symlink_components(path: &Path) -> Result<(), SandboxError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(SandboxError::PathAccessDenied {
                        path: path.to_path_buf(),
                        reason: format!("路径包含符号链接组件: {}", current.display()),
                    });
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => break,
            Err(error) => {
                return Err(SandboxError::PathAccessDenied {
                    path: path.to_path_buf(),
                    reason: error.to_string(),
                })
            }
        }
    }

    Ok(())
}
