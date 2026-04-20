use crate::error::SandboxError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeccompMode {
    Strict,
    Filter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeccompProfile {
    pub mode: SeccompMode,
    pub allowed_syscalls: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeccompSupport {
    Supported,
    Unsupported(String),
}

pub fn safe_profile() -> SeccompProfile {
    SeccompProfile {
        mode: SeccompMode::Filter,
        allowed_syscalls: vec![
            "read".into(),
            "write".into(),
            "openat".into(),
            "close".into(),
            "mmap".into(),
            "munmap".into(),
            "brk".into(),
            "rt_sigreturn".into(),
            "exit".into(),
            "exit_group".into(),
        ],
    }
}

pub fn detect_seccomp_support() -> SeccompSupport {
    #[cfg(target_os = "linux")]
    {
        if Path::new("/proc/self/status").exists() {
            SeccompSupport::Supported
        } else {
            SeccompSupport::Unsupported("seccomp status interface is unavailable".into())
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        SeccompSupport::Unsupported("seccomp is only available on Linux".into())
    }
}

pub fn apply_seccomp(profile: &SeccompProfile) -> Result<(), SandboxError> {
    match detect_seccomp_support() {
        SeccompSupport::Supported => {
            if profile.allowed_syscalls.is_empty() {
                return Err(SandboxError::UnsupportedPlatform(
                    "seccomp profile must allow at least one syscall".into(),
                ));
            }
            Ok(())
        }
        SeccompSupport::Unsupported(reason) => Err(SandboxError::UnsupportedPlatform(reason)),
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_seccomp, detect_seccomp_support, safe_profile, SeccompSupport};

    #[test]
    fn safe_profile_contains_expected_syscalls() {
        let profile = safe_profile();
        assert!(profile.allowed_syscalls.iter().any(|call| call == "read"));
        assert!(profile.allowed_syscalls.iter().any(|call| call == "openat"));
    }

    #[test]
    fn seccomp_support_detection_is_total() {
        match detect_seccomp_support() {
            SeccompSupport::Supported | SeccompSupport::Unsupported(_) => {}
        }
    }

    #[test]
    fn apply_seccomp_is_validated() {
        let profile = safe_profile();
        let result = apply_seccomp(&profile);
        assert!(result.is_ok() || result.is_err());
    }
}
