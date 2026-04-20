pub mod landlock;
pub mod seccomp;
pub mod wasm;

use std::fs::File;
use std::path::Path;

#[cfg(not(target_os = "linux"))]
use std::fs::OpenOptions;

use crate::error::SandboxError;
use crate::path_restriction::normalize_path_no_symlink_escape;

pub use landlock::{
    apply_landlock, detect_landlock_support, LandlockConfig, LandlockStatus, LandlockSupport,
};
pub use seccomp::{
    apply_seccomp, detect_seccomp_support, safe_profile, strict_profile, SeccompMode,
    SeccompProfile, SeccompStatus, SeccompSupport,
};
pub use wasm::{
    WasiAccessPlan, WasiDirectoryAccess, WasmExecutionRequest, WasmExecutionResult, WasmModule,
    WasmSandbox, WasmSandboxLimits,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafeOpenOptions {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub truncate: bool,
}

impl SafeOpenOptions {
    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            create: false,
            truncate: false,
        }
    }

    pub fn write_only() -> Self {
        Self {
            read: false,
            write: true,
            create: true,
            truncate: true,
        }
    }
}

pub fn open_file_no_symlinks(
    root: &Path,
    target: &Path,
    options: SafeOpenOptions,
) -> Result<File, SandboxError> {
    let normalized_root = normalize_path_no_symlink_escape(root)?;
    let candidate = if target.is_absolute() {
        target.to_path_buf()
    } else {
        normalized_root.join(target)
    };
    let normalized_target = normalize_path_no_symlink_escape(&candidate)?;

    if !normalized_target.starts_with(&normalized_root) {
        return Err(SandboxError::PathAccessDenied {
            path: normalized_target,
            reason: format!("目标路径不在允许根目录 `{}` 内", normalized_root.display()),
        });
    }

    #[cfg(target_os = "linux")]
    {
        linux_open_file_no_symlinks(&normalized_root, &normalized_target, options)
    }

    #[cfg(not(target_os = "linux"))]
    {
        portable_open_file_no_symlinks(&normalized_target, options)
    }
}

#[cfg(target_os = "linux")]
fn linux_open_file_no_symlinks(
    root: &Path,
    target: &Path,
    options: SafeOpenOptions,
) -> Result<File, SandboxError> {
    use nix::fcntl::{openat2, OFlag, OpenHow, ResolveFlag};
    use nix::sys::stat::Mode;

    let parent = target
        .parent()
        .ok_or_else(|| SandboxError::PathAccessDenied {
            path: target.to_path_buf(),
            reason: "目标路径缺少父目录".to_string(),
        })?;
    let relative_parent =
        parent
            .strip_prefix(root)
            .map_err(|_| SandboxError::PathAccessDenied {
                path: target.to_path_buf(),
                reason: "目标父目录不在允许根目录内".to_string(),
            })?;
    let leaf = target
        .file_name()
        .ok_or_else(|| SandboxError::PathAccessDenied {
            path: target.to_path_buf(),
            reason: "目标路径缺少文件名".to_string(),
        })?;

    let parent_fd = std::fs::File::open(root.join(relative_parent)).map_err(SandboxError::from)?;

    let mut flags = OFlag::O_CLOEXEC;
    flags |= match (options.read, options.write) {
        (true, true) => OFlag::O_RDWR,
        (false, true) => OFlag::O_WRONLY,
        _ => OFlag::O_RDONLY,
    };
    if options.create {
        flags |= OFlag::O_CREAT;
    }
    if options.truncate {
        flags |= OFlag::O_TRUNC;
    }

    let how = OpenHow::new()
        .flags(flags)
        .mode(Mode::from_bits_truncate(0o600))
        .resolve(
            ResolveFlag::RESOLVE_BENEATH
                | ResolveFlag::RESOLVE_NO_MAGICLINKS
                | ResolveFlag::RESOLVE_NO_SYMLINKS,
        );

    openat2(&parent_fd, leaf, how)
        .map(File::from)
        .map_err(|error| SandboxError::PathAccessDenied {
            path: target.to_path_buf(),
            reason: format!("openat2 拒绝访问: {error}"),
        })
}

#[cfg(not(target_os = "linux"))]
fn portable_open_file_no_symlinks(
    target: &Path,
    options: SafeOpenOptions,
) -> Result<File, SandboxError> {
    let mut open_options = OpenOptions::new();
    open_options.read(options.read);
    open_options.write(options.write);
    open_options.create(options.create);
    open_options.truncate(options.truncate);

    open_options.open(target).map_err(SandboxError::from)
}
