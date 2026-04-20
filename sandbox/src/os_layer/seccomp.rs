use std::collections::BTreeSet;

#[cfg(target_os = "linux")]
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::SandboxError;

#[cfg(target_os = "linux")]
use std::convert::{TryFrom, TryInto};

#[cfg(target_os = "linux")]
use seccompiler::{
    apply_filter, apply_filter_all_threads, BpfProgram, SeccompAction, SeccompFilter,
};

const DEFAULT_ERRNO_RET: u32 = 1;
const PERMISSIVE_DENIED_SYSCALLS: &[&str] = &[
    "bpf",
    "delete_module",
    "finit_module",
    "init_module",
    "kexec_load",
    "name_to_handle_at",
    "open_by_handle_at",
    "perf_event_open",
    "process_vm_readv",
    "process_vm_writev",
    "ptrace",
    "userfaultfd",
];
const BALANCED_EXTRA_DENIED_SYSCALLS: &[&str] = &[
    "fsconfig",
    "fsmount",
    "fsopen",
    "fspick",
    "mount",
    "move_mount",
    "pivot_root",
    "setns",
    "swapon",
    "swapoff",
    "umount2",
    "unshare",
];
const STRICT_ALLOWED_BASE_SYSCALLS: &[&str] = &[
    "access",
    "brk",
    "clock_gettime",
    "clock_nanosleep",
    "close",
    "dup",
    "dup2",
    "dup3",
    "execve",
    "exit",
    "exit_group",
    "faccessat",
    "faccessat2",
    "fcntl",
    "fstat",
    "futex",
    "getcwd",
    "getdents64",
    "getegid",
    "geteuid",
    "getgid",
    "getpid",
    "getppid",
    "getrandom",
    "gettid",
    "getuid",
    "ioctl",
    "lseek",
    "mmap",
    "mprotect",
    "munmap",
    "newfstatat",
    "openat",
    "pipe2",
    "pread64",
    "prlimit64",
    "read",
    "readlink",
    "readlinkat",
    "readv",
    "rseq",
    "rt_sigaction",
    "rt_sigprocmask",
    "rt_sigreturn",
    "set_robust_list",
    "set_tid_address",
    "sigaltstack",
    "statx",
    "uname",
    "write",
    "writev",
];

/// Security presets for Seccomp-BPF filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeccompMode {
    /// Deny only high-risk kernel attack-surface syscalls.
    Permissive,
    /// Deny high-risk syscalls plus namespace and mount operations.
    Balanced,
    /// Allow only an explicit syscall allowlist and reject everything else.
    Strict,
}

/// Serializable seccomp profile consumed by Guardian and OS-layer runners.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeccompProfile {
    /// Security preset describing whether the profile behaves as a denylist or allowlist.
    pub mode: SeccompMode,
    /// Whether to synchronize the filter across all threads in the current process.
    #[serde(default)]
    pub synchronize_threads: bool,
    /// Allowlist used only by [`SeccompMode::Strict`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_syscalls: Vec<String>,
    /// Denylist used by [`SeccompMode::Permissive`] and [`SeccompMode::Balanced`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub denied_syscalls: Vec<String>,
    /// Errno returned for blocked syscalls. Defaults to `EPERM`.
    #[serde(default = "default_errno_ret")]
    pub errno_ret: u32,
}

/// Whether seccomp is available on the current platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeccompSupport {
    Supported,
    Unsupported(String),
}

/// Result of attempting to install a seccomp filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeccompStatus {
    Installed {
        mode: SeccompMode,
        rule_count: usize,
        synchronized_threads: bool,
    },
    Skipped {
        reason: String,
    },
}

impl SeccompProfile {
    /// Default developer-safe profile: allow common CLI syscalls and deny dangerous kernel hooks.
    pub fn permissive() -> Self {
        Self {
            mode: SeccompMode::Permissive,
            synchronize_threads: false,
            allowed_syscalls: Vec::new(),
            denied_syscalls: PERMISSIVE_DENIED_SYSCALLS
                .iter()
                .map(|syscall| (*syscall).to_string())
                .collect(),
            errno_ret: default_errno_ret(),
        }
    }

    /// Stronger denylist profile suitable for command execution inside a workspace.
    pub fn balanced() -> Self {
        let mut profile = Self::permissive();
        profile.mode = SeccompMode::Balanced;
        profile.with_denied_syscalls(BALANCED_EXTRA_DENIED_SYSCALLS.iter().copied())
    }

    /// Explicit allowlist profile for heavily constrained Linux subprocesses.
    pub fn strict() -> Self {
        Self {
            mode: SeccompMode::Strict,
            synchronize_threads: false,
            allowed_syscalls: strict_allow_syscalls(),
            denied_syscalls: Vec::new(),
            errno_ret: default_errno_ret(),
        }
    }

    /// Add syscall names to the allowlist.
    pub fn with_allowed_syscalls<I, S>(mut self, syscalls: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        extend_unique(&mut self.allowed_syscalls, syscalls);
        self
    }

    /// Add syscall names to the denylist.
    pub fn with_denied_syscalls<I, S>(mut self, syscalls: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        extend_unique(&mut self.denied_syscalls, syscalls);
        self
    }

    /// Control whether installation uses seccomp TSYNC.
    pub fn with_thread_synchronization(mut self, synchronize_threads: bool) -> Self {
        self.synchronize_threads = synchronize_threads;
        self
    }

    /// Validate the profile before compilation or serialization handoff.
    pub fn validate(&self) -> Result<(), SandboxError> {
        match self.mode {
            SeccompMode::Strict => {
                if self.allowed_syscalls.is_empty() {
                    return Err(SandboxError::InvalidSandboxConfig {
                        layer: "seccomp".to_string(),
                        reason: "strict mode requires at least one allowed syscall".to_string(),
                    });
                }
                if !self.denied_syscalls.is_empty() {
                    return Err(SandboxError::InvalidSandboxConfig {
                        layer: "seccomp".to_string(),
                        reason: "strict mode cannot define a denylist".to_string(),
                    });
                }
            }
            SeccompMode::Permissive | SeccompMode::Balanced => {
                if self.denied_syscalls.is_empty() {
                    return Err(SandboxError::InvalidSandboxConfig {
                        layer: "seccomp".to_string(),
                        reason: "denylist modes require at least one blocked syscall".to_string(),
                    });
                }
                if !self.allowed_syscalls.is_empty() {
                    return Err(SandboxError::InvalidSandboxConfig {
                        layer: "seccomp".to_string(),
                        reason: "denylist modes cannot define an allowlist".to_string(),
                    });
                }
            }
        }

        let invalid: Vec<_> = self
            .relevant_syscalls()
            .iter()
            .filter(|syscall| !is_known_syscall_name(syscall))
            .cloned()
            .collect();
        if !invalid.is_empty() {
            return Err(SandboxError::InvalidSandboxConfig {
                layer: "seccomp".to_string(),
                reason: format!("unknown syscall names: {}", invalid.join(", ")),
            });
        }

        Ok(())
    }

    fn relevant_syscalls(&self) -> &[String] {
        match self.mode {
            SeccompMode::Strict => &self.allowed_syscalls,
            SeccompMode::Permissive | SeccompMode::Balanced => &self.denied_syscalls,
        }
    }
}

pub fn safe_profile() -> SeccompProfile {
    SeccompProfile::balanced()
}

pub fn strict_profile() -> SeccompProfile {
    SeccompProfile::strict()
}

pub fn detect_seccomp_support() -> SeccompSupport {
    #[cfg(target_os = "linux")]
    {
        if supported_target_arch().is_err() {
            return SeccompSupport::Unsupported(format!(
                "seccompiler does not support host arch `{}`",
                std::env::consts::ARCH
            ));
        }

        match std::fs::read_to_string("/proc/self/status") {
            Ok(status) if status.lines().any(|line| line.starts_with("Seccomp:")) => {
                SeccompSupport::Supported
            }
            Ok(_) => SeccompSupport::Unsupported(
                "kernel status does not expose a seccomp capability field".to_string(),
            ),
            Err(error) => SeccompSupport::Unsupported(format!(
                "failed to read /proc/self/status for seccomp detection: {error}"
            )),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        SeccompSupport::Unsupported("seccomp is only available on Linux".to_string())
    }
}

pub fn apply_seccomp(profile: &SeccompProfile) -> Result<SeccompStatus, SandboxError> {
    profile.validate()?;

    match detect_seccomp_support() {
        SeccompSupport::Supported => {
            #[cfg(target_os = "linux")]
            {
                let filter = build_filter_program(profile)?;
                if profile.synchronize_threads {
                    apply_filter_all_threads(&filter).map_err(seccomp_backend_error)?;
                } else {
                    apply_filter(&filter).map_err(seccomp_backend_error)?;
                }

                Ok(SeccompStatus::Installed {
                    mode: profile.mode,
                    rule_count: profile.relevant_syscalls().len(),
                    synchronized_threads: profile.synchronize_threads,
                })
            }

            #[cfg(not(target_os = "linux"))]
            {
                Ok(SeccompStatus::Skipped {
                    reason: "seccomp support changed during compilation target selection"
                        .to_string(),
                })
            }
        }
        SeccompSupport::Unsupported(reason) => Ok(SeccompStatus::Skipped { reason }),
    }
}

fn default_errno_ret() -> u32 {
    DEFAULT_ERRNO_RET
}

fn strict_allow_syscalls() -> Vec<String> {
    let mut syscalls: Vec<String> = STRICT_ALLOWED_BASE_SYSCALLS
        .iter()
        .map(|syscall| (*syscall).to_string())
        .collect();

    #[cfg(target_arch = "x86_64")]
    syscalls.push("arch_prctl".to_string());

    syscalls
}

fn extend_unique<I, S>(target: &mut Vec<String>, values: I)
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut seen: BTreeSet<String> = target
        .iter()
        .map(|value| normalize_syscall_name(value.clone()))
        .collect();
    for value in values {
        let value = normalize_syscall_name(value.into());
        if seen.insert(value.clone()) {
            target.push(value);
        }
    }
}

fn normalize_syscall_name(value: String) -> String {
    value.trim().to_ascii_lowercase()
}

fn is_known_syscall_name(name: &str) -> bool {
    match name {
        "access" | "bpf" | "brk" | "clock_gettime" | "clock_nanosleep" | "close"
        | "delete_module" | "dup" | "dup2" | "dup3" | "execve" | "exit" | "exit_group"
        | "faccessat" | "faccessat2" | "fcntl" | "finit_module" | "fstat" | "fsconfig"
        | "fsmount" | "fsopen" | "fspick" | "futex" | "getcwd" | "getdents64" | "getegid"
        | "geteuid" | "getgid" | "getpid" | "getppid" | "getrandom" | "gettid" | "getuid"
        | "init_module" | "ioctl" | "kexec_load" | "lseek" | "mmap" | "mount" | "move_mount"
        | "mprotect" | "munmap" | "name_to_handle_at" | "newfstatat" | "open_by_handle_at"
        | "openat" | "perf_event_open" | "pipe2" | "pivot_root" | "pread64" | "prlimit64"
        | "process_vm_readv" | "process_vm_writev" | "ptrace" | "read" | "readlink"
        | "readlinkat" | "readv" | "rseq" | "rt_sigaction" | "rt_sigprocmask" | "rt_sigreturn"
        | "set_robust_list" | "set_tid_address" | "setns" | "sigaltstack" | "statx" | "swapon"
        | "swapoff" | "umount2" | "uname" | "unshare" | "userfaultfd" | "write" | "writev" => true,
        #[cfg(target_arch = "x86_64")]
        "arch_prctl" => true,
        _ => false,
    }
}

#[cfg(target_os = "linux")]
fn supported_target_arch() -> Result<seccompiler::TargetArch, SandboxError> {
    seccompiler::TargetArch::try_from(std::env::consts::ARCH).map_err(|error| {
        SandboxError::UnsupportedPlatform(format!(
            "seccomp target arch `{}` is unsupported: {error}",
            std::env::consts::ARCH
        ))
    })
}

#[cfg(target_os = "linux")]
fn build_filter_program(profile: &SeccompProfile) -> Result<BpfProgram, SandboxError> {
    let rules = build_syscall_rules(profile.relevant_syscalls())?;
    let arch = supported_target_arch()?;
    let filter = match profile.mode {
        SeccompMode::Strict => SeccompFilter::new(
            rules,
            SeccompAction::Errno(profile.errno_ret),
            SeccompAction::Allow,
            arch,
        ),
        SeccompMode::Permissive | SeccompMode::Balanced => SeccompFilter::new(
            rules,
            SeccompAction::Allow,
            SeccompAction::Errno(profile.errno_ret),
            arch,
        ),
    }
    .map_err(seccomp_backend_error)?;

    filter.try_into().map_err(seccomp_backend_error)
}

#[cfg(target_os = "linux")]
fn build_syscall_rules(
    syscalls: &[String],
) -> Result<BTreeMap<i64, Vec<seccompiler::SeccompRule>>, SandboxError> {
    syscalls
        .iter()
        .try_fold(BTreeMap::new(), |mut rules, syscall| {
            let number =
                syscall_number(syscall).ok_or_else(|| SandboxError::InvalidSandboxConfig {
                    layer: "seccomp".to_string(),
                    reason: format!("no syscall number mapping is available for `{syscall}`"),
                })?;
            rules.entry(number).or_insert_with(Vec::new);
            Ok(rules)
        })
}

#[cfg(target_os = "linux")]
fn seccomp_backend_error(error: impl ToString) -> SandboxError {
    SandboxError::SandboxBackend {
        layer: "seccomp".to_string(),
        reason: error.to_string(),
    }
}

#[cfg(target_os = "linux")]
fn syscall_number(name: &str) -> Option<i64> {
    match name {
        "access" => Some(libc::SYS_access as i64),
        #[cfg(target_arch = "x86_64")]
        "arch_prctl" => Some(libc::SYS_arch_prctl as i64),
        "bpf" => Some(libc::SYS_bpf as i64),
        "brk" => Some(libc::SYS_brk as i64),
        "clock_gettime" => Some(libc::SYS_clock_gettime as i64),
        "clock_nanosleep" => Some(libc::SYS_clock_nanosleep as i64),
        "close" => Some(libc::SYS_close as i64),
        "delete_module" => Some(libc::SYS_delete_module as i64),
        "dup" => Some(libc::SYS_dup as i64),
        "dup2" => Some(libc::SYS_dup2 as i64),
        "dup3" => Some(libc::SYS_dup3 as i64),
        "execve" => Some(libc::SYS_execve as i64),
        "exit" => Some(libc::SYS_exit as i64),
        "exit_group" => Some(libc::SYS_exit_group as i64),
        "faccessat" => Some(libc::SYS_faccessat as i64),
        "faccessat2" => Some(libc::SYS_faccessat2 as i64),
        "fcntl" => Some(libc::SYS_fcntl as i64),
        "finit_module" => Some(libc::SYS_finit_module as i64),
        "fstat" => Some(libc::SYS_fstat as i64),
        "fsconfig" => Some(libc::SYS_fsconfig as i64),
        "fsmount" => Some(libc::SYS_fsmount as i64),
        "fsopen" => Some(libc::SYS_fsopen as i64),
        "fspick" => Some(libc::SYS_fspick as i64),
        "futex" => Some(libc::SYS_futex as i64),
        "getcwd" => Some(libc::SYS_getcwd as i64),
        "getdents64" => Some(libc::SYS_getdents64 as i64),
        "getegid" => Some(libc::SYS_getegid as i64),
        "geteuid" => Some(libc::SYS_geteuid as i64),
        "getgid" => Some(libc::SYS_getgid as i64),
        "getpid" => Some(libc::SYS_getpid as i64),
        "getppid" => Some(libc::SYS_getppid as i64),
        "getrandom" => Some(libc::SYS_getrandom as i64),
        "gettid" => Some(libc::SYS_gettid as i64),
        "getuid" => Some(libc::SYS_getuid as i64),
        "init_module" => Some(libc::SYS_init_module as i64),
        "ioctl" => Some(libc::SYS_ioctl as i64),
        "kexec_load" => Some(libc::SYS_kexec_load as i64),
        "lseek" => Some(libc::SYS_lseek as i64),
        "mmap" => Some(libc::SYS_mmap as i64),
        "mount" => Some(libc::SYS_mount as i64),
        "move_mount" => Some(libc::SYS_move_mount as i64),
        "mprotect" => Some(libc::SYS_mprotect as i64),
        "munmap" => Some(libc::SYS_munmap as i64),
        "name_to_handle_at" => Some(libc::SYS_name_to_handle_at as i64),
        "newfstatat" => Some(libc::SYS_newfstatat as i64),
        "open_by_handle_at" => Some(libc::SYS_open_by_handle_at as i64),
        "openat" => Some(libc::SYS_openat as i64),
        "perf_event_open" => Some(libc::SYS_perf_event_open as i64),
        "pipe2" => Some(libc::SYS_pipe2 as i64),
        "pivot_root" => Some(libc::SYS_pivot_root as i64),
        "pread64" => Some(libc::SYS_pread64 as i64),
        "prlimit64" => Some(libc::SYS_prlimit64 as i64),
        "process_vm_readv" => Some(libc::SYS_process_vm_readv as i64),
        "process_vm_writev" => Some(libc::SYS_process_vm_writev as i64),
        "ptrace" => Some(libc::SYS_ptrace as i64),
        "read" => Some(libc::SYS_read as i64),
        "readlink" => Some(libc::SYS_readlink as i64),
        "readlinkat" => Some(libc::SYS_readlinkat as i64),
        "readv" => Some(libc::SYS_readv as i64),
        "rseq" => Some(libc::SYS_rseq as i64),
        "rt_sigaction" => Some(libc::SYS_rt_sigaction as i64),
        "rt_sigprocmask" => Some(libc::SYS_rt_sigprocmask as i64),
        "rt_sigreturn" => Some(libc::SYS_rt_sigreturn as i64),
        "set_robust_list" => Some(libc::SYS_set_robust_list as i64),
        "set_tid_address" => Some(libc::SYS_set_tid_address as i64),
        "setns" => Some(libc::SYS_setns as i64),
        "sigaltstack" => Some(libc::SYS_sigaltstack as i64),
        "statx" => Some(libc::SYS_statx as i64),
        "swapon" => Some(libc::SYS_swapon as i64),
        "swapoff" => Some(libc::SYS_swapoff as i64),
        "umount2" => Some(libc::SYS_umount2 as i64),
        "uname" => Some(libc::SYS_uname as i64),
        "unshare" => Some(libc::SYS_unshare as i64),
        "userfaultfd" => Some(libc::SYS_userfaultfd as i64),
        "write" => Some(libc::SYS_write as i64),
        "writev" => Some(libc::SYS_writev as i64),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_seccomp, detect_seccomp_support, safe_profile, strict_profile, SeccompMode,
        SeccompProfile, SeccompStatus, SeccompSupport,
    };

    #[test]
    fn safe_profile_uses_balanced_denylist() {
        let profile = safe_profile();
        assert_eq!(profile.mode, SeccompMode::Balanced);
        assert!(profile
            .denied_syscalls
            .iter()
            .any(|syscall| syscall == "ptrace"));
        assert!(profile
            .denied_syscalls
            .iter()
            .any(|syscall| syscall == "mount"));
        assert!(profile.allowed_syscalls.is_empty());
    }

    #[test]
    fn strict_profile_uses_allowlist_only() {
        let profile = strict_profile();
        assert_eq!(profile.mode, SeccompMode::Strict);
        assert!(profile
            .allowed_syscalls
            .iter()
            .any(|syscall| syscall == "read"));
        assert!(profile
            .allowed_syscalls
            .iter()
            .any(|syscall| syscall == "openat"));
        assert!(profile.denied_syscalls.is_empty());
    }

    #[test]
    fn validation_rejects_mixed_profile_shapes() {
        let profile = SeccompProfile::balanced().with_allowed_syscalls(["read"]);
        assert!(profile.validate().is_err());
    }

    #[test]
    fn seccomp_support_detection_is_total() {
        match detect_seccomp_support() {
            SeccompSupport::Supported | SeccompSupport::Unsupported(_) => {}
        }
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn non_linux_apply_seccomp_skips_cleanly() {
        let status = apply_seccomp(&safe_profile()).expect("non-linux seccomp skip");
        assert!(matches!(status, SeccompStatus::Skipped { .. }));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_profiles_compile_into_bpf_programs() {
        assert!(super::build_filter_program(&safe_profile()).is_ok());
        assert!(super::build_filter_program(&strict_profile()).is_ok());
    }
}
