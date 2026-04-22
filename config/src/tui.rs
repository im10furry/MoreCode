use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TuiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub mouse_support: bool,
    #[serde(default = "default_max_log_lines")]
    pub max_log_lines: usize,
    #[serde(default = "default_refresh_rate_ms")]
    pub refresh_rate_ms: u64,
    #[serde(default)]
    pub custom_theme_path: Option<String>,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            language: default_language(),
            mouse_support: false,
            max_log_lines: default_max_log_lines(),
            refresh_rate_ms: default_refresh_rate_ms(),
            custom_theme_path: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialTuiConfig {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub mouse_support: Option<bool>,
    pub max_log_lines: Option<usize>,
    pub refresh_rate_ms: Option<u64>,
    pub custom_theme_path: Option<String>,
}

impl TuiConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialTuiConfig) {
        if let Some(value) = partial.theme {
            self.theme = value;
        }
        if let Some(value) = partial.language {
            self.language = value;
        }
        if let Some(value) = partial.mouse_support {
            self.mouse_support = value;
        }
        if let Some(value) = partial.max_log_lines {
            self.max_log_lines = value;
        }
        if let Some(value) = partial.refresh_rate_ms {
            self.refresh_rate_ms = value;
        }
        if let Some(value) = partial.custom_theme_path {
            self.custom_theme_path = Some(value);
        }
    }
}

impl PartialTuiConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            theme: other.theme.or(self.theme),
            language: other.language.or(self.language),
            mouse_support: other.mouse_support.or(self.mouse_support),
            max_log_lines: other.max_log_lines.or(self.max_log_lines),
            refresh_rate_ms: other.refresh_rate_ms.or(self.refresh_rate_ms),
            custom_theme_path: other.custom_theme_path.or(self.custom_theme_path),
        }
    }
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_language() -> String {
    "auto".to_string()
}

fn default_max_log_lines() -> usize {
    1_000
}

fn default_refresh_rate_ms() -> u64 {
    100
}
