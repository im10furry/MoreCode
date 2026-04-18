use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SandboxConfig {
    #[serde(default = "default_permission_mode")]
    pub permission_mode: String,
    #[serde(default)]
    pub landlock_enabled: bool,
    #[serde(default)]
    pub seccomp_enabled: bool,
    #[serde(default)]
    pub wasm_enabled: bool,
    #[serde(default)]
    pub command_whitelist: Vec<String>,
    #[serde(default)]
    pub read_only_paths: Vec<String>,
    #[serde(default)]
    pub write_paths: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            permission_mode: default_permission_mode(),
            landlock_enabled: false,
            seccomp_enabled: false,
            wasm_enabled: false,
            command_whitelist: Vec::new(),
            read_only_paths: Vec::new(),
            write_paths: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialSandboxConfig {
    pub permission_mode: Option<String>,
    pub landlock_enabled: Option<bool>,
    pub seccomp_enabled: Option<bool>,
    pub wasm_enabled: Option<bool>,
    pub command_whitelist: Option<Vec<String>>,
    pub read_only_paths: Option<Vec<String>>,
    pub write_paths: Option<Vec<String>>,
}

impl SandboxConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialSandboxConfig) {
        if let Some(value) = partial.permission_mode {
            self.permission_mode = value;
        }
        if let Some(value) = partial.landlock_enabled {
            self.landlock_enabled = value;
        }
        if let Some(value) = partial.seccomp_enabled {
            self.seccomp_enabled = value;
        }
        if let Some(value) = partial.wasm_enabled {
            self.wasm_enabled = value;
        }
        if let Some(value) = partial.command_whitelist {
            self.command_whitelist = value;
        }
        if let Some(value) = partial.read_only_paths {
            self.read_only_paths = value;
        }
        if let Some(value) = partial.write_paths {
            self.write_paths = value;
        }
    }
}

impl PartialSandboxConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            permission_mode: other.permission_mode.or(self.permission_mode),
            landlock_enabled: other.landlock_enabled.or(self.landlock_enabled),
            seccomp_enabled: other.seccomp_enabled.or(self.seccomp_enabled),
            wasm_enabled: other.wasm_enabled.or(self.wasm_enabled),
            command_whitelist: other.command_whitelist.or(self.command_whitelist),
            read_only_paths: other.read_only_paths.or(self.read_only_paths),
            write_paths: other.write_paths.or(self.write_paths),
        }
    }
}

fn default_permission_mode() -> String {
    "default".to_string()
}
