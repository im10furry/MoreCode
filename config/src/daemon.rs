use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DaemonProfile {
    Mvp,
    Fast,
    Medium,
    Full,
    FullExtensible,
}

impl std::str::FromStr for DaemonProfile {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let normalized = input.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "mvp" => Ok(Self::Mvp),
            "fast" => Ok(Self::Fast),
            "medium" => Ok(Self::Medium),
            "full" => Ok(Self::Full),
            "full-extensible" | "full+extensible" | "full-ext" => Ok(Self::FullExtensible),
            _ => Err(format!(
                "未知 daemon profile: {input}（可选：mvp/fast/medium/full/full-extensible）"
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DaemonConfig {
    #[serde(default)]
    pub profile: Option<DaemonProfile>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_pid_file")]
    pub pid_file: String,
    #[serde(default = "default_health_check_interval_secs")]
    pub health_check_interval_secs: u64,
    #[serde(default = "default_auto_update_check_hours")]
    pub auto_update_check_hours: u64,
    #[serde(default)]
    pub quiet_hours: Option<QuietHours>,
    #[serde(default)]
    pub daily_budget_usd: Option<f64>,
    #[serde(default)]
    pub taskpile: TaskPileConfig,
    #[serde(default)]
    pub self_iteration: SelfIterationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelfIterationConfig {
    #[serde(default = "default_self_iteration_enabled")]
    pub enabled: bool,
    #[serde(default = "default_health_check_interval_mins")]
    pub health_check_interval_mins: u64,
    #[serde(default = "default_auto_fix_enabled")]
    pub auto_fix_enabled: bool,
    #[serde(default = "default_auto_commit_enabled")]
    pub auto_commit_enabled: bool,
    #[serde(default = "default_max_daily_tasks")]
    pub max_daily_tasks: u32,
    #[serde(default)]
    pub budget_allocation: BudgetAllocationConfig,
    #[serde(default)]
    pub rules: IterationRulesConfig,
}

impl Default for SelfIterationConfig {
    fn default() -> Self {
        Self {
            enabled: default_self_iteration_enabled(),
            health_check_interval_mins: default_health_check_interval_mins(),
            auto_fix_enabled: default_auto_fix_enabled(),
            auto_commit_enabled: default_auto_commit_enabled(),
            max_daily_tasks: default_max_daily_tasks(),
            budget_allocation: BudgetAllocationConfig::default(),
            rules: IterationRulesConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BudgetAllocationConfig {
    #[serde(default = "default_health_monitoring_budget")]
    pub health_monitoring: f64,
    #[serde(default = "default_bug_fixes_budget")]
    pub bug_fixes: f64,
    #[serde(default = "default_improvements_budget")]
    pub improvements: f64,
    #[serde(default = "default_contingency_budget")]
    pub contingency: f64,
}

impl Default for BudgetAllocationConfig {
    fn default() -> Self {
        Self {
            health_monitoring: default_health_monitoring_budget(),
            bug_fixes: default_bug_fixes_budget(),
            improvements: default_improvements_budget(),
            contingency: default_contingency_budget(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IterationRulesConfig {
    #[serde(default = "default_check_clippy")]
    pub check_clippy: bool,
    #[serde(default = "default_check_tests")]
    pub check_tests: bool,
    #[serde(default = "default_check_docs")]
    pub check_docs: bool,
    #[serde(default = "default_suggest_refactoring")]
    pub suggest_refactoring: bool,
}

impl Default for IterationRulesConfig {
    fn default() -> Self {
        Self {
            check_clippy: default_check_clippy(),
            check_tests: default_check_tests(),
            check_docs: default_check_docs(),
            suggest_refactoring: default_suggest_refactoring(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialSelfIterationConfig {
    pub enabled: Option<bool>,
    pub health_check_interval_mins: Option<u64>,
    pub auto_fix_enabled: Option<bool>,
    pub auto_commit_enabled: Option<bool>,
    pub max_daily_tasks: Option<u32>,
    pub budget_allocation: Option<PartialBudgetAllocationConfig>,
    pub rules: Option<PartialIterationRulesConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialBudgetAllocationConfig {
    pub health_monitoring: Option<f64>,
    pub bug_fixes: Option<f64>,
    pub improvements: Option<f64>,
    pub contingency: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialIterationRulesConfig {
    pub check_clippy: Option<bool>,
    pub check_tests: Option<bool>,
    pub check_docs: Option<bool>,
    pub suggest_refactoring: Option<bool>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            profile: None,
            enabled: false,
            pid_file: default_pid_file(),
            health_check_interval_secs: default_health_check_interval_secs(),
            auto_update_check_hours: default_auto_update_check_hours(),
            quiet_hours: None,
            daily_budget_usd: None,
            taskpile: TaskPileConfig::default(),
            self_iteration: SelfIterationConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskPileConfig {
    #[serde(default = "default_taskpile_enabled")]
    pub enabled: bool,
    #[serde(default = "default_taskpile_storage_dir")]
    pub storage_dir: Option<String>,
    #[serde(default = "default_taskpile_max_running_tasks")]
    pub max_running_tasks: usize,
    #[serde(default = "default_taskpile_dedup_window_secs")]
    pub dedup_window_secs: u64,
    #[serde(default = "default_taskpile_default_token_budget")]
    pub default_token_budget: u32,
    #[serde(default = "default_taskpile_default_isolation_profile")]
    pub default_isolation_profile: String,
    #[serde(default)]
    pub cloud: TaskPileCloudConfig,
}

impl Default for TaskPileConfig {
    fn default() -> Self {
        Self {
            enabled: default_taskpile_enabled(),
            storage_dir: default_taskpile_storage_dir(),
            max_running_tasks: default_taskpile_max_running_tasks(),
            dedup_window_secs: default_taskpile_dedup_window_secs(),
            default_token_budget: default_taskpile_default_token_budget(),
            default_isolation_profile: default_taskpile_default_isolation_profile(),
            cloud: TaskPileCloudConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TaskPileCloudConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuietHours {
    pub start_hour: u8,
    pub end_hour: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialDaemonConfig {
    pub profile: Option<DaemonProfile>,
    pub enabled: Option<bool>,
    pub pid_file: Option<String>,
    pub health_check_interval_secs: Option<u64>,
    pub auto_update_check_hours: Option<u64>,
    pub quiet_hours: Option<PartialQuietHours>,
    pub daily_budget_usd: Option<f64>,
    pub taskpile: Option<PartialTaskPileConfig>,
    pub self_iteration: Option<PartialSelfIterationConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialTaskPileConfig {
    pub enabled: Option<bool>,
    pub storage_dir: Option<Option<String>>,
    pub max_running_tasks: Option<usize>,
    pub dedup_window_secs: Option<u64>,
    pub default_token_budget: Option<u32>,
    pub default_isolation_profile: Option<String>,
    pub cloud: Option<PartialTaskPileCloudConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialTaskPileCloudConfig {
    pub enabled: Option<bool>,
    pub endpoint: Option<Option<String>>,
    pub project_id: Option<Option<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialQuietHours {
    pub start_hour: Option<u8>,
    pub end_hour: Option<u8>,
}

impl DaemonConfig {
    pub(crate) fn apply_partial(
        &mut self,
        partial: PartialDaemonConfig,
    ) -> Result<(), ConfigError> {
        if let Some(profile) = partial.profile {
            self.apply_profile(profile);
            self.profile = Some(profile);
        }
        if let Some(value) = partial.enabled {
            self.enabled = value;
        }
        if let Some(value) = partial.pid_file {
            self.pid_file = value;
        }
        if let Some(value) = partial.health_check_interval_secs {
            self.health_check_interval_secs = value;
        }
        if let Some(value) = partial.auto_update_check_hours {
            self.auto_update_check_hours = value;
        }
        if let Some(value) = partial.quiet_hours {
            let mut quiet_hours = self.quiet_hours.clone().unwrap_or(QuietHours {
                start_hour: u8::MAX,
                end_hour: u8::MAX,
            });
            quiet_hours.apply_partial(value);
            self.quiet_hours = Some(quiet_hours);
        }
        if let Some(value) = partial.daily_budget_usd {
            self.daily_budget_usd = Some(value);
        }
        if let Some(value) = partial.taskpile {
            self.taskpile.apply_partial(value);
        }
        if let Some(value) = partial.self_iteration {
            self.self_iteration.apply_partial(value);
        }
        Ok(())
    }

    fn apply_profile(&mut self, profile: DaemonProfile) {
        match profile {
            DaemonProfile::Mvp => {
                self.health_check_interval_secs = 60;
                self.auto_update_check_hours = 24;
                self.daily_budget_usd = Some(1.0);

                self.taskpile.enabled = false;
                self.taskpile.max_running_tasks = 1;
                self.taskpile.default_token_budget = 6_000;

                self.self_iteration.enabled = false;
            }
            DaemonProfile::Fast => {
                self.health_check_interval_secs = 30;
                self.auto_update_check_hours = 24;
                self.daily_budget_usd = Some(2.0);

                self.taskpile.enabled = true;
                self.taskpile.max_running_tasks = 1;
                self.taskpile.default_token_budget = 8_000;

                self.self_iteration.enabled = false;
            }
            DaemonProfile::Medium => {
                self.health_check_interval_secs = 30;
                self.auto_update_check_hours = 12;
                self.daily_budget_usd = Some(5.0);

                self.taskpile.enabled = true;
                self.taskpile.max_running_tasks = 2;
                self.taskpile.default_token_budget = 12_000;

                self.self_iteration.enabled = true;
                self.self_iteration.health_check_interval_mins = 60;
                self.self_iteration.auto_fix_enabled = false;
                self.self_iteration.max_daily_tasks = 5;
            }
            DaemonProfile::Full => {
                self.health_check_interval_secs = 15;
                self.auto_update_check_hours = 6;
                self.daily_budget_usd = Some(10.0);

                self.taskpile.enabled = true;
                self.taskpile.max_running_tasks = 3;
                self.taskpile.default_token_budget = 20_000;

                self.self_iteration.enabled = true;
                self.self_iteration.health_check_interval_mins = 30;
                self.self_iteration.auto_fix_enabled = true;
                self.self_iteration.max_daily_tasks = 15;
                self.self_iteration.rules.check_clippy = true;
                self.self_iteration.rules.check_tests = true;
            }
            DaemonProfile::FullExtensible => {
                self.health_check_interval_secs = 15;
                self.auto_update_check_hours = 6;
                self.daily_budget_usd = Some(20.0);

                self.taskpile.enabled = true;
                self.taskpile.max_running_tasks = 4;
                self.taskpile.default_token_budget = 25_000;

                self.self_iteration.enabled = true;
                self.self_iteration.health_check_interval_mins = 15;
                self.self_iteration.auto_fix_enabled = true;
                self.self_iteration.auto_commit_enabled = true;
                self.self_iteration.max_daily_tasks = 20;
                self.self_iteration.rules.check_clippy = true;
                self.self_iteration.rules.check_tests = true;
                self.self_iteration.rules.check_docs = true;
                self.self_iteration.rules.suggest_refactoring = true;
            }
        }
    }
}

impl SelfIterationConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialSelfIterationConfig) {
        if let Some(value) = partial.enabled {
            self.enabled = value;
        }
        if let Some(value) = partial.health_check_interval_mins {
            self.health_check_interval_mins = value;
        }
        if let Some(value) = partial.auto_fix_enabled {
            self.auto_fix_enabled = value;
        }
        if let Some(value) = partial.auto_commit_enabled {
            self.auto_commit_enabled = value;
        }
        if let Some(value) = partial.max_daily_tasks {
            self.max_daily_tasks = value;
        }
        if let Some(value) = partial.budget_allocation {
            self.budget_allocation.apply_partial(value);
        }
        if let Some(value) = partial.rules {
            self.rules.apply_partial(value);
        }
    }
}

impl BudgetAllocationConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialBudgetAllocationConfig) {
        if let Some(value) = partial.health_monitoring {
            self.health_monitoring = value;
        }
        if let Some(value) = partial.bug_fixes {
            self.bug_fixes = value;
        }
        if let Some(value) = partial.improvements {
            self.improvements = value;
        }
        if let Some(value) = partial.contingency {
            self.contingency = value;
        }
    }
}

impl IterationRulesConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialIterationRulesConfig) {
        if let Some(value) = partial.check_clippy {
            self.check_clippy = value;
        }
        if let Some(value) = partial.check_tests {
            self.check_tests = value;
        }
        if let Some(value) = partial.check_docs {
            self.check_docs = value;
        }
        if let Some(value) = partial.suggest_refactoring {
            self.suggest_refactoring = value;
        }
    }
}

impl QuietHours {
    pub(crate) fn apply_partial(&mut self, partial: PartialQuietHours) {
        if let Some(value) = partial.start_hour {
            self.start_hour = value;
        }
        if let Some(value) = partial.end_hour {
            self.end_hour = value;
        }
    }
}

impl PartialDaemonConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            profile: other.profile.or(self.profile),
            enabled: other.enabled.or(self.enabled),
            pid_file: other.pid_file.or(self.pid_file),
            health_check_interval_secs: other
                .health_check_interval_secs
                .or(self.health_check_interval_secs),
            auto_update_check_hours: other
                .auto_update_check_hours
                .or(self.auto_update_check_hours),
            quiet_hours: match (self.quiet_hours, other.quiet_hours) {
                (Some(base), Some(overlay)) => Some(base.merge(overlay)),
                (Some(base), None) => Some(base),
                (None, Some(overlay)) => Some(overlay),
                (None, None) => None,
            },
            daily_budget_usd: other.daily_budget_usd.or(self.daily_budget_usd),
            taskpile: match (self.taskpile, other.taskpile) {
                (Some(base), Some(overlay)) => Some(base.merge(overlay)),
                (Some(base), None) => Some(base),
                (None, Some(overlay)) => Some(overlay),
                (None, None) => None,
            },
        }
    }
}

impl PartialQuietHours {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            start_hour: other.start_hour.or(self.start_hour),
            end_hour: other.end_hour.or(self.end_hour),
        }
    }
}

impl TaskPileConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialTaskPileConfig) {
        if let Some(value) = partial.enabled {
            self.enabled = value;
        }
        if let Some(value) = partial.storage_dir {
            self.storage_dir = value;
        }
        if let Some(value) = partial.max_running_tasks {
            self.max_running_tasks = value;
        }
        if let Some(value) = partial.dedup_window_secs {
            self.dedup_window_secs = value;
        }
        if let Some(value) = partial.default_token_budget {
            self.default_token_budget = value;
        }
        if let Some(value) = partial.default_isolation_profile {
            self.default_isolation_profile = value;
        }
        if let Some(value) = partial.cloud {
            self.cloud.apply_partial(value);
        }
    }
}

impl TaskPileCloudConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialTaskPileCloudConfig) {
        if let Some(value) = partial.enabled {
            self.enabled = value;
        }
        if let Some(value) = partial.endpoint {
            self.endpoint = value;
        }
        if let Some(value) = partial.project_id {
            self.project_id = value;
        }
    }
}

impl PartialTaskPileConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            enabled: other.enabled.or(self.enabled),
            storage_dir: other.storage_dir.or(self.storage_dir),
            max_running_tasks: other.max_running_tasks.or(self.max_running_tasks),
            dedup_window_secs: other.dedup_window_secs.or(self.dedup_window_secs),
            default_token_budget: other.default_token_budget.or(self.default_token_budget),
            default_isolation_profile: other
                .default_isolation_profile
                .or(self.default_isolation_profile),
            cloud: match (self.cloud, other.cloud) {
                (Some(base), Some(overlay)) => Some(base.merge(overlay)),
                (Some(base), None) => Some(base),
                (None, Some(overlay)) => Some(overlay),
                (None, None) => None,
            },
        }
    }
}

impl PartialTaskPileCloudConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            enabled: other.enabled.or(self.enabled),
            endpoint: other.endpoint.or(self.endpoint),
            project_id: other.project_id.or(self.project_id),
        }
    }
}

fn default_pid_file() -> String {
    "/tmp/morecode.pid".to_string()
}

fn default_health_check_interval_secs() -> u64 {
    60
}

fn default_auto_update_check_hours() -> u64 {
    24
}

fn default_taskpile_enabled() -> bool {
    false
}

fn default_taskpile_storage_dir() -> Option<String> {
    None
}

fn default_taskpile_max_running_tasks() -> usize {
    2
}

fn default_taskpile_dedup_window_secs() -> u64 {
    900
}

fn default_taskpile_default_token_budget() -> u32 {
    12_000
}

fn default_taskpile_default_isolation_profile() -> String {
    "workspace-write".to_string()
}

fn default_self_iteration_enabled() -> bool {
    false
}

fn default_health_check_interval_mins() -> u64 {
    30
}

fn default_auto_fix_enabled() -> bool {
    false
}

fn default_auto_commit_enabled() -> bool {
    false
}

fn default_max_daily_tasks() -> u32 {
    10
}

fn default_health_monitoring_budget() -> f64 {
    0.10
}

fn default_bug_fixes_budget() -> f64 {
    0.50
}

fn default_improvements_budget() -> f64 {
    0.30
}

fn default_contingency_budget() -> f64 {
    0.10
}

fn default_check_clippy() -> bool {
    true
}

fn default_check_tests() -> bool {
    true
}

fn default_check_docs() -> bool {
    false
}

fn default_suggest_refactoring() -> bool {
    false
}
