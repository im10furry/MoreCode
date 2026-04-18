use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryConfig {
    #[serde(default = "default_memory_dir")]
    pub memory_dir: String,
    #[serde(default = "default_ttl_days")]
    pub ttl_days: i64,
    #[serde(default = "default_core_memory_limit")]
    pub core_memory_limit: usize,
    #[serde(default = "default_working_memory_max_files")]
    pub working_memory_max_files: usize,
    #[serde(default = "default_working_memory_max_mb")]
    pub working_memory_max_mb: usize,
    #[serde(default)]
    pub sleep_time_compute: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            memory_dir: default_memory_dir(),
            ttl_days: default_ttl_days(),
            core_memory_limit: default_core_memory_limit(),
            working_memory_max_files: default_working_memory_max_files(),
            working_memory_max_mb: default_working_memory_max_mb(),
            sleep_time_compute: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialMemoryConfig {
    pub memory_dir: Option<String>,
    pub ttl_days: Option<i64>,
    pub core_memory_limit: Option<usize>,
    pub working_memory_max_files: Option<usize>,
    pub working_memory_max_mb: Option<usize>,
    pub sleep_time_compute: Option<bool>,
}

impl MemoryConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialMemoryConfig) {
        if let Some(value) = partial.memory_dir {
            self.memory_dir = value;
        }
        if let Some(value) = partial.ttl_days {
            self.ttl_days = value;
        }
        if let Some(value) = partial.core_memory_limit {
            self.core_memory_limit = value;
        }
        if let Some(value) = partial.working_memory_max_files {
            self.working_memory_max_files = value;
        }
        if let Some(value) = partial.working_memory_max_mb {
            self.working_memory_max_mb = value;
        }
        if let Some(value) = partial.sleep_time_compute {
            self.sleep_time_compute = value;
        }
    }
}

impl PartialMemoryConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            memory_dir: other.memory_dir.or(self.memory_dir),
            ttl_days: other.ttl_days.or(self.ttl_days),
            core_memory_limit: other.core_memory_limit.or(self.core_memory_limit),
            working_memory_max_files: other
                .working_memory_max_files
                .or(self.working_memory_max_files),
            working_memory_max_mb: other.working_memory_max_mb.or(self.working_memory_max_mb),
            sleep_time_compute: other.sleep_time_compute.or(self.sleep_time_compute),
        }
    }
}

fn default_memory_dir() -> String {
    ".assistant-memory".to_string()
}

fn default_ttl_days() -> i64 {
    30
}

fn default_core_memory_limit() -> usize {
    100
}

fn default_working_memory_max_files() -> usize {
    100
}

fn default_working_memory_max_mb() -> usize {
    25
}
