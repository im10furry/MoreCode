use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use std::mem::size_of;
#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

#[cfg(target_os = "linux")]
use nix::fcntl::{open, OFlag};
#[cfg(target_os = "linux")]
use nix::libc;
#[cfg(target_os = "linux")]
use nix::sys::stat::Mode;

use crate::error::SandboxError;
use crate::path_restriction::{PathRestriction, PathRestrictionType};

#[cfg(any(target_os = "linux", test))]
const LANDLOCK_ABI_REFER: u32 = 2;
#[cfg(any(target_os = "linux", test))]
const LANDLOCK_ABI_TRUNCATE: u32 = 3;
#[cfg(any(target_os = "linux", test))]
const LANDLOCK_ABI_IOCTL_DEV: u32 = 5;

#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_EXECUTE: u64 = 1 << 0;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_WRITE_FILE: u64 = 1 << 1;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_READ_FILE: u64 = 1 << 2;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_READ_DIR: u64 = 1 << 3;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_REMOVE_DIR: u64 = 1 << 4;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_REMOVE_FILE: u64 = 1 << 5;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_MAKE_CHAR: u64 = 1 << 6;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_MAKE_DIR: u64 = 1 << 7;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_MAKE_REG: u64 = 1 << 8;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_MAKE_SOCK: u64 = 1 << 9;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_MAKE_FIFO: u64 = 1 << 10;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_MAKE_BLOCK: u64 = 1 << 11;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_MAKE_SYM: u64 = 1 << 12;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_REFER: u64 = 1 << 13;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_TRUNCATE: u64 = 1 << 14;
#[cfg(any(target_os = "linux", test))]
const ACCESS_FS_IOCTL_DEV: u64 = 1 << 15;

/// Landlock is fundamentally allow-list based. Denied paths are kept for Guardian
/// compatibility and validation, but configs that would require subtractive kernel
/// rules are rejected during validation and application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandlockConfig {
    pub read_write_dirs: Vec<PathBuf>,
    pub read_only_dirs: Vec<PathBuf>,
    pub denied_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LandlockSupport {
    Supported { abi_version: u32 },
    Unsupported(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandlockStatus {
    pub support: LandlockSupport,
    pub tracked_paths: usize,
}

impl LandlockConfig {
    /// Build the default Landlock policy for a task workspace.
    pub fn for_task(task_workspace: &Path) -> Self {
        Self {
            read_write_dirs: vec![task_workspace.to_path_buf()],
            read_only_dirs: default_read_only_dirs(),
            denied_paths: Vec::new(),
        }
    }

    /// Build a Landlock policy from Guardian path restrictions.
    pub fn from_path_restrictions(
        task_workspace: &Path,
        restrictions: &[PathRestriction],
    ) -> Result<Self, SandboxError> {
        Self::for_task(task_workspace).with_path_restrictions(restrictions)
    }

    /// Merge Guardian path restrictions into the default task policy.
    pub fn with_path_restrictions(
        mut self,
        restrictions: &[PathRestriction],
    ) -> Result<Self, SandboxError> {
        for restriction in restrictions {
            match restriction.restriction_type {
                PathRestrictionType::Allow => {
                    self.read_write_dirs.push(restriction.path.clone());
                }
                PathRestrictionType::ReadOnly => {
                    self.read_only_dirs.push(restriction.path.clone());
                }
                PathRestrictionType::Deny => {
                    self.denied_paths.push(restriction.path.clone());
                }
            }
        }

        PreparedLandlockConfig::from_config(&self).map(Into::into)
    }

    pub fn validate(&self) -> Result<(), SandboxError> {
        let _ = PreparedLandlockConfig::from_config(self)?;
        Ok(())
    }
}

impl From<PreparedLandlockConfig> for LandlockConfig {
    fn from(value: PreparedLandlockConfig) -> Self {
        Self {
            read_write_dirs: value.read_write_paths,
            read_only_dirs: value.read_only_paths,
            denied_paths: value.denied_paths,
        }
    }
}

pub fn detect_landlock_support() -> LandlockSupport {
    #[cfg(target_os = "linux")]
    {
        match query_landlock_abi() {
            Ok(abi_version) if abi_version > 0 => LandlockSupport::Supported { abi_version },
            Ok(_) => {
                LandlockSupport::Unsupported("Landlock ABI query returned version 0".to_string())
            }
            Err(error) => map_support_error(error),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        LandlockSupport::Unsupported("landlock is only available on Linux".into())
    }
}

pub fn apply_landlock(config: &LandlockConfig) -> Result<LandlockStatus, SandboxError> {
    let prepared = PreparedLandlockConfig::from_config(config)?;

    #[cfg(target_os = "linux")]
    {
        let abi_version = match detect_landlock_support() {
            LandlockSupport::Supported { abi_version } => abi_version,
            LandlockSupport::Unsupported(reason) => {
                return Err(SandboxError::UnsupportedPlatform(reason));
            }
        };

        let ruleset = create_ruleset(handled_access_fs_for_abi(abi_version))?;
        let read_only_access = read_only_access_fs();
        let read_write_access = read_write_access_fs_for_abi(abi_version);

        for path in &prepared.read_only_paths {
            add_path_rule(&ruleset, path, read_only_access)?;
        }

        for path in &prepared.read_write_paths {
            add_path_rule(&ruleset, path, read_write_access)?;
        }

        set_no_new_privs()?;
        restrict_self(&ruleset)?;

        Ok(LandlockStatus {
            support: LandlockSupport::Supported { abi_version },
            tracked_paths: prepared.read_only_paths.len() + prepared.read_write_paths.len(),
        })
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = prepared;
        Err(SandboxError::UnsupportedPlatform(
            "landlock is only available on Linux".into(),
        ))
    }
}

fn default_read_only_dirs() -> Vec<PathBuf> {
    ["/usr", "/bin", "/lib", "/lib64", "/etc"]
        .into_iter()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreparedLandlockConfig {
    read_write_paths: Vec<PathBuf>,
    read_only_paths: Vec<PathBuf>,
    denied_paths: Vec<PathBuf>,
}

impl PreparedLandlockConfig {
    fn from_config(config: &LandlockConfig) -> Result<Self, SandboxError> {
        let mut read_write_paths = normalize_existing_paths(&config.read_write_dirs)?;
        let mut read_only_paths = normalize_existing_paths(&config.read_only_dirs)?;
        let denied_paths = normalize_policy_paths(&config.denied_paths, false)?;

        read_write_paths = prune_redundant_paths(read_write_paths);
        read_only_paths = prune_redundant_paths(read_only_paths);

        // A read-only rule nested under a read-write ancestor is ineffective for both
        // Guardian and Landlock semantics, so collapse it early.
        read_only_paths.retain(|candidate| !has_ancestor(&read_write_paths, candidate));

        // Guardian deny rules override positive rules completely. Drop positive paths
        // already shadowed by a denied ancestor before checking for subtractive cases.
        read_write_paths.retain(|candidate| !has_ancestor(&denied_paths, candidate));
        read_only_paths.retain(|candidate| !has_ancestor(&denied_paths, candidate));

        if let Some((allowed_path, denied_path)) =
            find_subtractive_blacklist(&read_write_paths, &read_only_paths, &denied_paths)
        {
            return Err(SandboxError::Landlock {
                operation: "validate policy".into(),
                reason: format!(
                    "deny rule `{}` is nested inside allowed path `{}`; Landlock cannot express subtractive path rules",
                    denied_path.display(),
                    allowed_path.display()
                ),
            });
        }

        if read_write_paths.is_empty() && read_only_paths.is_empty() {
            return Err(SandboxError::UnsupportedPlatform(
                "landlock requires at least one tracked path".into(),
            ));
        }

        Ok(Self {
            read_write_paths,
            read_only_paths,
            denied_paths,
        })
    }
}

fn normalize_existing_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>, SandboxError> {
    normalize_policy_paths(paths, true)
}

fn normalize_policy_paths(
    paths: &[PathBuf],
    require_exists: bool,
) -> Result<Vec<PathBuf>, SandboxError> {
    paths
        .iter()
        .map(|path| normalize_policy_path(path, require_exists))
        .collect()
}

fn normalize_policy_path(path: &Path, require_exists: bool) -> Result<PathBuf, SandboxError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(SandboxError::from)?
            .join(path)
    };

    if require_exists {
        return std::fs::canonicalize(&absolute).map_err(|error| SandboxError::PathAccessDenied {
            path: absolute.clone(),
            reason: error.to_string(),
        });
    }

    let (ancestor, tail) = split_existing_ancestor(&absolute)?;
    let mut normalized =
        std::fs::canonicalize(&ancestor).map_err(|error| SandboxError::PathAccessDenied {
            path: absolute.clone(),
            reason: error.to_string(),
        })?;

    for component in tail {
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
                reason: "path has no existing ancestor".into(),
            })?;
        tail.push(file_name.to_os_string());
        cursor = cursor
            .parent()
            .ok_or_else(|| SandboxError::PathAccessDenied {
                path: path.to_path_buf(),
                reason: "path has no parent directory".into(),
            })?
            .to_path_buf();
    }

    tail.reverse();
    Ok((cursor, tail))
}

fn prune_redundant_paths(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.sort();
    paths.dedup();
    paths.sort_by_key(|path| path.components().count());

    let mut pruned = Vec::new();
    for path in paths {
        if has_ancestor(&pruned, &path) {
            continue;
        }
        pruned.push(path);
    }

    pruned
}

fn has_ancestor(ancestors: &[PathBuf], candidate: &Path) -> bool {
    ancestors.iter().any(|path| candidate.starts_with(path))
}

fn find_subtractive_blacklist<'a>(
    read_write_paths: &'a [PathBuf],
    read_only_paths: &'a [PathBuf],
    denied_paths: &'a [PathBuf],
) -> Option<(&'a Path, &'a Path)> {
    for allowed_path in read_write_paths.iter().chain(read_only_paths.iter()) {
        for denied_path in denied_paths {
            if denied_path.starts_with(allowed_path) {
                return Some((allowed_path.as_path(), denied_path.as_path()));
            }
        }
    }

    None
}

#[cfg(any(target_os = "linux", test))]
fn read_only_access_fs() -> u64 {
    ACCESS_FS_EXECUTE | ACCESS_FS_READ_FILE | ACCESS_FS_READ_DIR
}

#[cfg(any(target_os = "linux", test))]
fn read_write_access_fs_for_abi(abi_version: u32) -> u64 {
    let mut access = read_only_access_fs()
        | ACCESS_FS_WRITE_FILE
        | ACCESS_FS_REMOVE_DIR
        | ACCESS_FS_REMOVE_FILE
        | ACCESS_FS_MAKE_DIR
        | ACCESS_FS_MAKE_REG
        | ACCESS_FS_MAKE_SOCK
        | ACCESS_FS_MAKE_FIFO
        | ACCESS_FS_MAKE_SYM;

    if abi_version >= LANDLOCK_ABI_REFER {
        access |= ACCESS_FS_REFER;
    }
    if abi_version >= LANDLOCK_ABI_TRUNCATE {
        access |= ACCESS_FS_TRUNCATE;
    }

    access
}

#[cfg(any(target_os = "linux", test))]
fn handled_access_fs_for_abi(abi_version: u32) -> u64 {
    let mut access = read_only_access_fs()
        | ACCESS_FS_WRITE_FILE
        | ACCESS_FS_REMOVE_DIR
        | ACCESS_FS_REMOVE_FILE
        | ACCESS_FS_MAKE_CHAR
        | ACCESS_FS_MAKE_DIR
        | ACCESS_FS_MAKE_REG
        | ACCESS_FS_MAKE_SOCK
        | ACCESS_FS_MAKE_FIFO
        | ACCESS_FS_MAKE_BLOCK
        | ACCESS_FS_MAKE_SYM;

    if abi_version >= LANDLOCK_ABI_REFER {
        access |= ACCESS_FS_REFER;
    }
    if abi_version >= LANDLOCK_ABI_TRUNCATE {
        access |= ACCESS_FS_TRUNCATE;
    }
    if abi_version >= LANDLOCK_ABI_IOCTL_DEV {
        access |= ACCESS_FS_IOCTL_DEV;
    }

    access
}

#[cfg(target_os = "linux")]
const LANDLOCK_CREATE_RULESET_VERSION: libc::c_uint = 1;
#[cfg(target_os = "linux")]
const LANDLOCK_RULE_PATH_BENEATH: libc::c_uint = 1;

#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct LandlockRulesetAttr {
    handled_access_fs: u64,
    handled_access_net: u64,
    scoped: u64,
}

#[cfg(target_os = "linux")]
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct LandlockPathBeneathAttr {
    allowed_access: u64,
    parent_fd: i32,
}

#[cfg(target_os = "linux")]
fn query_landlock_abi() -> Result<u32, std::io::Error> {
    let result = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            std::ptr::null::<LandlockRulesetAttr>(),
            0usize,
            LANDLOCK_CREATE_RULESET_VERSION,
        )
    };

    if result < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(result as u32)
    }
}

#[cfg(target_os = "linux")]
fn map_support_error(error: std::io::Error) -> LandlockSupport {
    match error.raw_os_error() {
        Some(code) if code == libc::ENOSYS => {
            LandlockSupport::Unsupported("kernel does not expose Landlock syscalls".into())
        }
        Some(code) if code == libc::EOPNOTSUPP => {
            LandlockSupport::Unsupported("kernel Landlock support is disabled".into())
        }
        Some(code) if code == libc::EINVAL => {
            LandlockSupport::Unsupported("kernel rejected the Landlock ABI query".into())
        }
        _ => LandlockSupport::Unsupported(format!("failed to query Landlock ABI: {error}")),
    }
}

#[cfg(target_os = "linux")]
fn create_ruleset(handled_access_fs: u64) -> Result<OwnedFd, SandboxError> {
    let attr = LandlockRulesetAttr {
        handled_access_fs,
        handled_access_net: 0,
        scoped: 0,
    };

    let result = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            &attr as *const LandlockRulesetAttr,
            size_of::<LandlockRulesetAttr>(),
            0u32,
        )
    };
    let fd = syscall_result(result, "create ruleset")?;

    Ok(unsafe { OwnedFd::from_raw_fd(fd as i32) })
}

#[cfg(target_os = "linux")]
fn add_path_rule(ruleset: &OwnedFd, path: &Path, allowed_access: u64) -> Result<(), SandboxError> {
    let path_fd = open(
        path,
        OFlag::O_PATH | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|error| SandboxError::Landlock {
        operation: "open rule path".into(),
        reason: format!("{}: {error}", path.display()),
    })?;

    let attr = LandlockPathBeneathAttr {
        allowed_access,
        parent_fd: path_fd.as_raw_fd(),
    };

    let result = unsafe {
        libc::syscall(
            libc::SYS_landlock_add_rule,
            ruleset.as_raw_fd(),
            LANDLOCK_RULE_PATH_BENEATH,
            &attr as *const LandlockPathBeneathAttr,
            0u32,
        )
    };
    let _ = syscall_result(result, &format!("add path rule for {}", path.display()))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn set_no_new_privs() -> Result<(), SandboxError> {
    let result = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if result == 0 {
        Ok(())
    } else {
        Err(SandboxError::Landlock {
            operation: "set no_new_privs".into(),
            reason: std::io::Error::last_os_error().to_string(),
        })
    }
}

#[cfg(target_os = "linux")]
fn restrict_self(ruleset: &OwnedFd) -> Result<(), SandboxError> {
    let result =
        unsafe { libc::syscall(libc::SYS_landlock_restrict_self, ruleset.as_raw_fd(), 0u32) };
    let _ = syscall_result(result, "restrict self")?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn syscall_result(result: libc::c_long, operation: &str) -> Result<libc::c_long, SandboxError> {
    if result >= 0 {
        return Ok(result);
    }

    Err(SandboxError::Landlock {
        operation: operation.to_string(),
        reason: std::io::Error::last_os_error().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{detect_landlock_support, handled_access_fs_for_abi, read_write_access_fs_for_abi};
    use super::{LandlockConfig, LandlockSupport};
    use crate::path_restriction::PathRestriction;

    #[test]
    fn landlock_validation_rejects_missing_paths() {
        let config = LandlockConfig {
            read_write_dirs: vec![std::path::PathBuf::from("missing-path")],
            read_only_dirs: Vec::new(),
            denied_paths: Vec::new(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn landlock_validation_rejects_subtractive_blacklist() {
        let temp = tempdir().expect("tempdir");
        let workspace = temp.path().join("workspace");
        let secret = workspace.join("secret");
        std::fs::create_dir_all(&secret).expect("workspace tree");

        let config = LandlockConfig {
            read_write_dirs: vec![workspace],
            read_only_dirs: Vec::new(),
            denied_paths: vec![secret],
        };

        let error = config.validate().expect_err("subtractive deny should fail");
        assert!(error.to_string().contains("subtractive path rules"));
    }

    #[test]
    fn landlock_from_path_restrictions_normalizes_guardian_rules() {
        let temp = tempdir().expect("tempdir");
        let workspace = temp.path().join("workspace");
        let docs = temp.path().join("docs");
        std::fs::create_dir_all(&workspace).expect("workspace");
        std::fs::create_dir_all(&docs).expect("docs");

        let config = LandlockConfig::from_path_restrictions(
            &workspace,
            &[
                PathRestriction::allow(&workspace),
                PathRestriction::read_only(&docs),
                PathRestriction::deny(temp.path().join("outside")),
            ],
        )
        .expect("normalized config");

        assert!(config
            .read_write_dirs
            .iter()
            .any(|path| path.ends_with("workspace")));
        assert!(config
            .denied_paths
            .iter()
            .any(|path| path.ends_with("outside")));
    }

    #[test]
    fn landlock_access_masks_track_abi_features() {
        assert_eq!(read_write_access_fs_for_abi(1) & super::ACCESS_FS_REFER, 0);
        assert_ne!(read_write_access_fs_for_abi(2) & super::ACCESS_FS_REFER, 0);
        assert_eq!(handled_access_fs_for_abi(2) & super::ACCESS_FS_TRUNCATE, 0);
        assert_ne!(handled_access_fs_for_abi(3) & super::ACCESS_FS_TRUNCATE, 0);
        assert_ne!(handled_access_fs_for_abi(5) & super::ACCESS_FS_IOCTL_DEV, 0);
    }

    #[test]
    fn landlock_support_detection_is_total() {
        match detect_landlock_support() {
            LandlockSupport::Supported { .. } | LandlockSupport::Unsupported(_) => {}
        }
    }
}
