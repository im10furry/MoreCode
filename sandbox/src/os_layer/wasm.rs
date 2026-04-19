use std::path::PathBuf;
use std::time::Duration;

use crate::error::SandboxError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmSandboxLimits {
    pub fuel: u64,
    pub epoch_deadline_ticks: u64,
    pub epoch_interval: Duration,
}

impl Default for WasmSandboxLimits {
    fn default() -> Self {
        Self {
            fuel: 10_000_000_000,
            epoch_deadline_ticks: 1,
            epoch_interval: Duration::from_millis(100),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasiAccessPlan {
    pub allowed_dirs: Vec<PathBuf>,
    pub network_enabled: bool,
    pub env_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmSandbox {
    pub limits: WasmSandboxLimits,
}

impl WasmSandbox {
    pub fn new(limits: WasmSandboxLimits) -> Result<Self, SandboxError> {
        if limits.fuel == 0 || limits.epoch_deadline_ticks == 0 {
            return Err(SandboxError::UnsupportedPlatform(
                "wasm sandbox limits must be non-zero".into(),
            ));
        }
        Ok(Self { limits })
    }

    pub fn create_restricted_wasi(&self, allowed_dirs: &[PathBuf]) -> Result<WasiAccessPlan, SandboxError> {
        if allowed_dirs.iter().any(|path| !path.exists()) {
            return Err(SandboxError::UnsupportedPlatform(
                "all WASI preopened directories must exist".into(),
            ));
        }

        Ok(WasiAccessPlan {
            allowed_dirs: allowed_dirs.to_vec(),
            network_enabled: false,
            env_enabled: false,
        })
    }

    pub fn validate_wasm_module(&self, bytes: &[u8]) -> Result<(), SandboxError> {
        const WASM_MAGIC: &[u8; 4] = b"\0asm";
        if bytes.len() < 4 || &bytes[..4] != WASM_MAGIC {
            return Err(SandboxError::UnsupportedPlatform(
                "input is not a valid WebAssembly binary".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{WasmSandbox, WasmSandboxLimits};

    #[test]
    fn wasm_sandbox_rejects_invalid_limits() {
        let result = WasmSandbox::new(WasmSandboxLimits {
            fuel: 0,
            ..WasmSandboxLimits::default()
        });
        assert!(result.is_err());
    }

    #[test]
    fn wasm_module_validation_checks_magic_header() {
        let sandbox = WasmSandbox::new(WasmSandboxLimits::default()).unwrap();
        assert!(sandbox.validate_wasm_module(b"\0asm\x01\x00\x00\x00").is_ok());
        assert!(sandbox.validate_wasm_module(b"not-wasm").is_err());
    }

    #[test]
    fn restricted_wasi_requires_existing_paths() {
        let sandbox = WasmSandbox::new(WasmSandboxLimits::default()).unwrap();
        let temp = tempdir().unwrap();
        assert!(sandbox
            .create_restricted_wasi(&[temp.path().to_path_buf()])
            .is_ok());
    }
}
