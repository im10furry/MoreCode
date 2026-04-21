use serde::{Deserialize, Serialize};

use crate::{
    agent::{AgentConfig, PartialAgentConfig},
    context::{ContextConfig, PartialContextConfig},
    coordinator::{CoordinatorConfig, PartialCoordinatorConfig},
    cost::{CostBudgetConfig, PartialCostBudgetConfig},
    daemon::{DaemonConfig, PartialDaemonConfig},
    error::ConfigError,
    line_ending::{LineEndingConfig, PartialLineEndingConfig},
    memory::{MemoryConfig, PartialMemoryConfig},
    provider::{PartialProviderConfig, ProviderConfig},
    recursive::{PartialRecursiveConfig, RecursiveConfig},
    sandbox::{PartialSandboxConfig, SandboxConfig},
    tui::{PartialTuiConfig, TuiConfig},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub app: AppSettings,
    #[serde(default)]
    pub coordinator: CoordinatorConfig,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub provider: ProviderConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(default)]
    pub context: ContextConfig,
    #[serde(default)]
    pub sandbox: SandboxConfig,
    #[serde(default)]
    pub recursive: RecursiveConfig,
    #[serde(default)]
    pub daemon: DaemonConfig,
    #[serde(default)]
    pub tui: TuiConfig,
    #[serde(default)]
    pub cost: CostBudgetConfig,
    #[serde(default)]
    pub line_ending: LineEndingConfig,
}

impl AppConfig {
    pub(crate) fn from_partial(partial: PartialAppConfig) -> Result<Self, ConfigError> {
        let mut config = Self::default();
        config.apply_partial(partial)?;
        Ok(config)
    }

    pub(crate) fn apply_partial(&mut self, partial: PartialAppConfig) -> Result<(), ConfigError> {
        if let Some(value) = partial.app {
            self.app.apply_partial(value);
        }
        if let Some(value) = partial.coordinator {
            self.coordinator.apply_partial(value);
        }
        if let Some(value) = partial.agent {
            self.agent.apply_partial(value);
        }
        if let Some(value) = partial.provider {
            self.provider.apply_partial(value);
        }
        if let Some(value) = partial.memory {
            self.memory.apply_partial(value);
        }
        if let Some(value) = partial.context {
            self.context.apply_partial(value);
        }
        if let Some(value) = partial.sandbox {
            self.sandbox.apply_partial(value);
        }
        if let Some(value) = partial.recursive {
            self.recursive.apply_partial(value);
        }
        if let Some(value) = partial.daemon {
            self.daemon.apply_partial(value)?;
        }
        if let Some(value) = partial.tui {
            self.tui.apply_partial(value);
        }
        if let Some(value) = partial.cost {
            self.cost.apply_partial(value);
        }
        if let Some(value) = partial.line_ending {
            self.line_ending.apply_partial(value);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    #[serde(default = "default_app_name")]
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub data_dir: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            version: None,
            log_level: default_log_level(),
            data_dir: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialAppConfig {
    pub app: Option<PartialAppSettings>,
    pub coordinator: Option<PartialCoordinatorConfig>,
    pub agent: Option<PartialAgentConfig>,
    pub provider: Option<PartialProviderConfig>,
    pub memory: Option<PartialMemoryConfig>,
    pub context: Option<PartialContextConfig>,
    pub sandbox: Option<PartialSandboxConfig>,
    pub recursive: Option<PartialRecursiveConfig>,
    pub daemon: Option<PartialDaemonConfig>,
    pub tui: Option<PartialTuiConfig>,
    pub cost: Option<PartialCostBudgetConfig>,
    pub line_ending: Option<PartialLineEndingConfig>,
}

impl PartialAppConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            app: merge_optional(self.app, other.app, PartialAppSettings::merge),
            coordinator: merge_optional(
                self.coordinator,
                other.coordinator,
                PartialCoordinatorConfig::merge,
            ),
            agent: merge_optional(self.agent, other.agent, PartialAgentConfig::merge),
            provider: merge_optional(self.provider, other.provider, PartialProviderConfig::merge),
            memory: merge_optional(self.memory, other.memory, PartialMemoryConfig::merge),
            context: merge_optional(self.context, other.context, PartialContextConfig::merge),
            sandbox: merge_optional(self.sandbox, other.sandbox, PartialSandboxConfig::merge),
            recursive: merge_optional(
                self.recursive,
                other.recursive,
                PartialRecursiveConfig::merge,
            ),
            daemon: merge_optional(self.daemon, other.daemon, PartialDaemonConfig::merge),
            tui: merge_optional(self.tui, other.tui, PartialTuiConfig::merge),
            cost: merge_optional(self.cost, other.cost, PartialCostBudgetConfig::merge),
            line_ending: merge_optional(
                self.line_ending,
                other.line_ending,
                PartialLineEndingConfig::merge,
            ),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialAppSettings {
    pub name: Option<String>,
    pub version: Option<String>,
    pub log_level: Option<String>,
    pub data_dir: Option<String>,
}

impl AppSettings {
    pub(crate) fn apply_partial(&mut self, partial: PartialAppSettings) {
        if let Some(value) = partial.name {
            self.name = value;
        }
        if let Some(value) = partial.version {
            self.version = Some(value);
        }
        if let Some(value) = partial.log_level {
            self.log_level = value;
        }
        if let Some(value) = partial.data_dir {
            self.data_dir = Some(value);
        }
    }
}

impl PartialAppSettings {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            name: other.name.or(self.name),
            version: other.version.or(self.version),
            log_level: other.log_level.or(self.log_level),
            data_dir: other.data_dir.or(self.data_dir),
        }
    }
}

fn merge_optional<T>(base: Option<T>, overlay: Option<T>, merge: fn(T, T) -> T) -> Option<T> {
    match (base, overlay) {
        (Some(base), Some(overlay)) => Some(merge(base, overlay)),
        (Some(base), None) => Some(base),
        (None, Some(overlay)) => Some(overlay),
        (None, None) => None,
    }
}

fn default_app_name() -> String {
    "morecode".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}
