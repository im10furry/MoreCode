pub mod agent;
pub mod app;
pub mod context;
pub mod coordinator;
pub mod cost;
pub mod daemon;
pub mod error;
pub mod line_ending;
pub mod loader;
pub mod memory;
pub mod provider;
pub mod recursive;
pub mod sandbox;
pub mod tui;
pub mod validator;

pub use agent::{AgentConfig, PartialAgentConfig};
pub use app::{AppConfig, AppSettings, PartialAppConfig, PartialAppSettings};
pub use context::{ContextConfig, PartialContextConfig};
pub use coordinator::{CoordinatorConfig, PartialCoordinatorConfig};
pub use cost::{CostBudgetConfig, PartialCostBudgetConfig};
pub use daemon::{
    DaemonConfig, DaemonProfile, PartialDaemonConfig, PartialQuietHours, PartialTaskPileCloudConfig,
    PartialTaskPileConfig, QuietHours, TaskPileCloudConfig, TaskPileConfig,
};
pub use error::{ConfigError, Result};
pub use line_ending::{
    auto_fix_line_endings_for_write, LineEndingConfig, LineEndingDefault, LineEndingFixMetadata,
    LineEndingFixOutcome, PartialLineEndingConfig,
};
pub use loader::{ConfigChangeEvent, ConfigLoader, FileChangeType};
pub use memory::{MemoryConfig, PartialMemoryConfig};
pub use provider::{
    BuiltinProviderPreset, PartialProviderConfig, PartialProviderEntry, ProviderConfig,
    ProviderEntry, ResolvedProviderEntry,
};
pub use recursive::{PartialRecursiveConfig, RecursiveConfig};
pub use sandbox::{PartialSandboxConfig, SandboxConfig};
pub use tui::{PartialTuiConfig, TuiConfig};
pub use validator::validate;

#[cfg(test)]
mod tests;
