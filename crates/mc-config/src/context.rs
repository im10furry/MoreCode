use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextConfig {
    #[serde(default = "default_l1_micro_compress_threshold")]
    pub l1_micro_compress_threshold: usize,
    #[serde(default = "default_l2_auto_compress_threshold")]
    pub l2_auto_compress_threshold: f32,
    #[serde(default = "default_l3_memory_compress_threshold")]
    pub l3_memory_compress_threshold: f32,
    #[serde(default = "default_true")]
    pub repo_map_enabled: bool,
    #[serde(default = "default_large_file_threshold")]
    pub large_file_threshold: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            l1_micro_compress_threshold: default_l1_micro_compress_threshold(),
            l2_auto_compress_threshold: default_l2_auto_compress_threshold(),
            l3_memory_compress_threshold: default_l3_memory_compress_threshold(),
            repo_map_enabled: default_true(),
            large_file_threshold: default_large_file_threshold(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialContextConfig {
    pub l1_micro_compress_threshold: Option<usize>,
    pub l2_auto_compress_threshold: Option<f32>,
    pub l3_memory_compress_threshold: Option<f32>,
    pub repo_map_enabled: Option<bool>,
    pub large_file_threshold: Option<usize>,
}

impl ContextConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialContextConfig) {
        if let Some(value) = partial.l1_micro_compress_threshold {
            self.l1_micro_compress_threshold = value;
        }
        if let Some(value) = partial.l2_auto_compress_threshold {
            self.l2_auto_compress_threshold = value;
        }
        if let Some(value) = partial.l3_memory_compress_threshold {
            self.l3_memory_compress_threshold = value;
        }
        if let Some(value) = partial.repo_map_enabled {
            self.repo_map_enabled = value;
        }
        if let Some(value) = partial.large_file_threshold {
            self.large_file_threshold = value;
        }
    }
}

impl PartialContextConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            l1_micro_compress_threshold: other
                .l1_micro_compress_threshold
                .or(self.l1_micro_compress_threshold),
            l2_auto_compress_threshold: other
                .l2_auto_compress_threshold
                .or(self.l2_auto_compress_threshold),
            l3_memory_compress_threshold: other
                .l3_memory_compress_threshold
                .or(self.l3_memory_compress_threshold),
            repo_map_enabled: other.repo_map_enabled.or(self.repo_map_enabled),
            large_file_threshold: other.large_file_threshold.or(self.large_file_threshold),
        }
    }
}

fn default_l1_micro_compress_threshold() -> usize {
    10_000
}

fn default_l2_auto_compress_threshold() -> f32 {
    0.85
}

fn default_l3_memory_compress_threshold() -> f32 {
    0.90
}

fn default_true() -> bool {
    true
}

fn default_large_file_threshold() -> usize {
    2_000
}
