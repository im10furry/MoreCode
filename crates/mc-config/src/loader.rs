use std::{
    env,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use mc_core::{
    CONFIG_EVENT_CHANNEL_CAPACITY, CONFIG_FILE_NAME, GLOBAL_CONFIG_SUBDIR,
    HOT_RELOAD_SETTLE_MILLIS, MORECODE_ENV_PREFIX, PROJECT_CONFIG_SUBDIR,
};
use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
    time::sleep,
};

use crate::{app::PartialAppConfig, validate, AppConfig, ConfigError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigChangeEvent {
    FileChanged {
        path: PathBuf,
        change_type: FileChangeType,
    },
    EnvChanged {
        var_name: String,
    },
    Reloaded {
        config: AppConfig,
    },
    ReloadFailed {
        error: ConfigError,
    },
}

struct HotReloadState {
    _watcher: RecommendedWatcher,
    task: JoinHandle<()>,
}

pub struct ConfigLoader {
    global_config_path: PathBuf,
    project_config_path: PathBuf,
    config_change_tx: broadcast::Sender<ConfigChangeEvent>,
    current_config: Arc<RwLock<AppConfig>>,
    hot_reload_state: Mutex<Option<HotReloadState>>,
}

impl ConfigLoader {
    pub fn new(global_config_dir: PathBuf, project_config_dir: PathBuf) -> Self {
        Self::with_paths(
            normalize_config_path(global_config_dir),
            normalize_config_path(project_config_dir),
        )
    }

    pub fn with_paths(global_config_path: PathBuf, project_config_path: PathBuf) -> Self {
        let (config_change_tx, _) = broadcast::channel(CONFIG_EVENT_CHANNEL_CAPACITY);
        Self {
            global_config_path: absolutize_path(global_config_path),
            project_config_path: absolutize_path(project_config_path),
            config_change_tx,
            current_config: Arc::new(RwLock::new(AppConfig::default())),
            hot_reload_state: Mutex::new(None),
        }
    }

    pub fn with_default_paths() -> Result<Self> {
        Ok(Self::with_paths(
            Self::default_global_config_path()?,
            Self::default_project_config_path()?,
        ))
    }

    pub fn default_global_config_path() -> Result<PathBuf> {
        let home_dir = home_dir().ok_or_else(|| ConfigError::LoadFailed {
            path: "~".to_string(),
            reason: "无法解析 HOME 或 USERPROFILE".to_string(),
        })?;
        Ok(home_dir.join(GLOBAL_CONFIG_SUBDIR).join(CONFIG_FILE_NAME))
    }

    pub fn default_project_config_path() -> Result<PathBuf> {
        env::current_dir()
            .map(|dir| dir.join(PROJECT_CONFIG_SUBDIR).join(CONFIG_FILE_NAME))
            .map_err(|error| ConfigError::LoadFailed {
                path: ".".to_string(),
                reason: error.to_string(),
            })
    }

    pub async fn load(&self) -> Result<AppConfig> {
        let config =
            Self::load_with_paths(&self.global_config_path, &self.project_config_path).await?;
        write_snapshot(&self.current_config, config.clone());
        Ok(config)
    }

    pub async fn start_hot_reload(&self) -> Result<()> {
        let mut hot_reload_state = lock_hot_reload_state(&self.hot_reload_state);
        if hot_reload_state.is_some() {
            return Ok(());
        }

        let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();
        let mut watcher = recommended_watcher(move |result| {
            let _ = notify_tx.send(result);
        })
        .map_err(|error| ConfigError::HotReloadFailed {
            reason: error.to_string(),
        })?;

        for watch_root in self.watch_roots() {
            watcher
                .watch(&watch_root, RecursiveMode::NonRecursive)
                .map_err(|error| ConfigError::HotReloadFailed {
                    reason: format!("监听 {} 失败: {error}", watch_root.display()),
                })?;
        }

        let global_config_path = self.global_config_path.clone();
        let project_config_path = self.project_config_path.clone();
        let config_change_tx = self.config_change_tx.clone();
        let current_config = Arc::clone(&self.current_config);

        let task = tokio::spawn(async move {
            while let Some(result) = notify_rx.recv().await {
                process_notify_result(
                    result,
                    &global_config_path,
                    &project_config_path,
                    &config_change_tx,
                    &current_config,
                )
                .await;
            }
        });

        *hot_reload_state = Some(HotReloadState {
            _watcher: watcher,
            task,
        });

        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.config_change_tx.subscribe()
    }

    pub fn current(&self) -> AppConfig {
        read_snapshot(&self.current_config)
    }

    async fn load_with_paths(
        global_config_path: &Path,
        project_config_path: &Path,
    ) -> Result<AppConfig> {
        let global_config = Self::load_partial_config(global_config_path).await?;
        let project_config = Self::load_partial_config(project_config_path).await?;
        let merged = global_config.merge(project_config);
        let mut config = AppConfig::from_partial(merged)?;
        Self::apply_env_overrides(&mut config, env::vars())?;
        validate(&config)?;
        Ok(config)
    }

    async fn load_partial_config(config_path: &Path) -> Result<PartialAppConfig> {
        if !config_path.exists() {
            return Ok(PartialAppConfig::default());
        }

        let contents = tokio::fs::read_to_string(config_path)
            .await
            .map_err(|error| ConfigError::LoadFailed {
                path: config_path.display().to_string(),
                reason: error.to_string(),
            })?;

        toml::from_str::<PartialAppConfig>(&contents).map_err(|error| ConfigError::ParseFailed {
            path: config_path.display().to_string(),
            reason: error.to_string(),
        })
    }

    pub(crate) fn apply_env_overrides<I>(config: &mut AppConfig, vars: I) -> Result<()>
    where
        I: IntoIterator<Item = (String, String)>,
    {
        for (var_name, raw_value) in vars {
            if !var_name.starts_with(MORECODE_ENV_PREFIX) {
                continue;
            }

            match var_name.as_str() {
                "MORECODE_APP_NAME" => config.app.name = normalize_string(raw_value),
                "MORECODE_APP_VERSION" => config.app.version = optional_string(raw_value),
                "MORECODE_APP_LOG_LEVEL" => config.app.log_level = normalize_string(raw_value),
                "MORECODE_APP_DATA_DIR" => config.app.data_dir = optional_string(raw_value),

                "MORECODE_COORDINATOR_MAX_TOKEN_BUDGET" => {
                    config.coordinator.max_token_budget = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_COORDINATOR_MAX_RECURSION_DEPTH" => {
                    config.coordinator.max_recursion_depth = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_COORDINATOR_AGENT_TIMEOUT_SECS" => {
                    config.coordinator.agent_timeout_secs = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_COORDINATOR_MAX_RETRIES" => {
                    config.coordinator.max_retries = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_COORDINATOR_MEMORY_AWARE_ROUTING" => {
                    config.coordinator.memory_aware_routing = parse_bool(&var_name, &raw_value)?
                }
                "MORECODE_COORDINATOR_RECURSIVE_ORCHESTRATION" => {
                    config.coordinator.recursive_orchestration = parse_bool(&var_name, &raw_value)?
                }
                "MORECODE_COORDINATOR_MEMORY_STALE_THRESHOLD_DAYS" => {
                    config.coordinator.memory_stale_threshold_days =
                        parse_value(&var_name, &raw_value)?
                }
                "MORECODE_COORDINATOR_PREFLIGHT_CHECK" => {
                    config.coordinator.preflight_check = parse_bool(&var_name, &raw_value)?
                }
                "MORECODE_COORDINATOR_LLM_WEIGHT_MULTIPLIER" => {
                    config.coordinator.llm_weight_multiplier = parse_value(&var_name, &raw_value)?
                }

                "MORECODE_AGENT_DEFAULT_MODEL" => {
                    config.agent.default_model = normalize_string(raw_value)
                }
                "MORECODE_AGENT_TEMPERATURE" => {
                    config.agent.temperature = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_AGENT_MAX_OUTPUT_TOKENS" => {
                    config.agent.max_output_tokens = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_AGENT_STREAMING" => {
                    config.agent.streaming = parse_bool(&var_name, &raw_value)?
                }
                "MORECODE_AGENT_TOOL_TIMEOUT_SECS" => {
                    config.agent.tool_timeout_secs = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_AGENT_AUTO_RETRY" => {
                    config.agent.auto_retry = parse_bool(&var_name, &raw_value)?
                }

                "MORECODE_PROVIDER_DEFAULT_PROVIDER" => {
                    config.provider.default_provider = normalize_string(raw_value)
                }
                "MORECODE_PROVIDER_SEMANTIC_CACHE_ENABLED" => {
                    config.provider.semantic_cache_enabled = parse_bool(&var_name, &raw_value)?
                }
                "MORECODE_PROVIDER_SEMANTIC_CACHE_THRESHOLD" => {
                    config.provider.semantic_cache_threshold = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_PROVIDER_PRECISE_TOKEN_COUNT" => {
                    config.provider.precise_token_count = parse_bool(&var_name, &raw_value)?
                }

                "MORECODE_MEMORY_MEMORY_DIR" => {
                    config.memory.memory_dir = normalize_string(raw_value)
                }
                "MORECODE_MEMORY_TTL_DAYS" => {
                    config.memory.ttl_days = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_MEMORY_CORE_MEMORY_LIMIT" => {
                    config.memory.core_memory_limit = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_MEMORY_WORKING_MEMORY_MAX_FILES" => {
                    config.memory.working_memory_max_files = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_MEMORY_WORKING_MEMORY_MAX_MB" => {
                    config.memory.working_memory_max_mb = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_MEMORY_SLEEP_TIME_COMPUTE" => {
                    config.memory.sleep_time_compute = parse_bool(&var_name, &raw_value)?
                }

                "MORECODE_CONTEXT_L1_MICRO_COMPRESS_THRESHOLD" => {
                    config.context.l1_micro_compress_threshold = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_CONTEXT_L2_AUTO_COMPRESS_THRESHOLD" => {
                    config.context.l2_auto_compress_threshold = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_CONTEXT_L3_MEMORY_COMPRESS_THRESHOLD" => {
                    config.context.l3_memory_compress_threshold =
                        parse_value(&var_name, &raw_value)?
                }
                "MORECODE_CONTEXT_REPO_MAP_ENABLED" => {
                    config.context.repo_map_enabled = parse_bool(&var_name, &raw_value)?
                }
                "MORECODE_CONTEXT_LARGE_FILE_THRESHOLD" => {
                    config.context.large_file_threshold = parse_value(&var_name, &raw_value)?
                }

                "MORECODE_SANDBOX_PERMISSION_MODE" => {
                    config.sandbox.permission_mode = normalize_string(raw_value)
                }
                "MORECODE_SANDBOX_LANDLOCK_ENABLED" => {
                    config.sandbox.landlock_enabled = parse_bool(&var_name, &raw_value)?
                }
                "MORECODE_SANDBOX_SECCOMP_ENABLED" => {
                    config.sandbox.seccomp_enabled = parse_bool(&var_name, &raw_value)?
                }
                "MORECODE_SANDBOX_WASM_ENABLED" => {
                    config.sandbox.wasm_enabled = parse_bool(&var_name, &raw_value)?
                }
                "MORECODE_SANDBOX_COMMAND_WHITELIST" => {
                    config.sandbox.command_whitelist = parse_string_list(&var_name, &raw_value)?
                }
                "MORECODE_SANDBOX_READ_ONLY_PATHS" => {
                    config.sandbox.read_only_paths = parse_string_list(&var_name, &raw_value)?
                }
                "MORECODE_SANDBOX_WRITE_PATHS" => {
                    config.sandbox.write_paths = parse_string_list(&var_name, &raw_value)?
                }

                "MORECODE_RECURSIVE_MAX_SUB_AGENTS" => {
                    config.recursive.max_sub_agents = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_RECURSIVE_MAX_DEPTH" => {
                    config.recursive.max_depth = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_RECURSIVE_MAX_TOTAL_SUB_AGENTS" => {
                    config.recursive.max_total_sub_agents = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_RECURSIVE_SUB_AGENT_TIMEOUT_SECS" => {
                    config.recursive.sub_agent_timeout_secs = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_RECURSIVE_ENABLED" => {
                    config.recursive.enabled = parse_bool(&var_name, &raw_value)?
                }

                "MORECODE_DAEMON_ENABLED" => {
                    config.daemon.enabled = parse_bool(&var_name, &raw_value)?
                }
                "MORECODE_DAEMON_PID_FILE" => config.daemon.pid_file = normalize_string(raw_value),
                "MORECODE_DAEMON_HEALTH_CHECK_INTERVAL_SECS" => {
                    config.daemon.health_check_interval_secs = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_DAEMON_AUTO_UPDATE_CHECK_HOURS" => {
                    config.daemon.auto_update_check_hours = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_DAEMON_QUIET_HOURS_START_HOUR" => {
                    let quiet_hours = config.daemon.quiet_hours.get_or_insert(crate::QuietHours {
                        start_hour: u8::MAX,
                        end_hour: u8::MAX,
                    });
                    quiet_hours.start_hour = parse_value(&var_name, &raw_value)?;
                }
                "MORECODE_DAEMON_QUIET_HOURS_END_HOUR" => {
                    let quiet_hours = config.daemon.quiet_hours.get_or_insert(crate::QuietHours {
                        start_hour: u8::MAX,
                        end_hour: u8::MAX,
                    });
                    quiet_hours.end_hour = parse_value(&var_name, &raw_value)?;
                }
                "MORECODE_DAEMON_DAILY_BUDGET_USD" => {
                    config.daemon.daily_budget_usd = parse_optional_f64(&var_name, &raw_value)?
                }

                "MORECODE_TUI_THEME" => config.tui.theme = normalize_string(raw_value),
                "MORECODE_TUI_MOUSE_SUPPORT" => {
                    config.tui.mouse_support = parse_bool(&var_name, &raw_value)?
                }
                "MORECODE_TUI_MAX_LOG_LINES" => {
                    config.tui.max_log_lines = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_TUI_REFRESH_RATE_MS" => {
                    config.tui.refresh_rate_ms = parse_value(&var_name, &raw_value)?
                }
                "MORECODE_TUI_CUSTOM_THEME_PATH" => {
                    config.tui.custom_theme_path = optional_string(raw_value)
                }

                "MORECODE_COST_DAILY_BUDGET_USD" => {
                    config.cost.daily_budget_usd = parse_optional_f64(&var_name, &raw_value)?
                }
                "MORECODE_COST_WEEKLY_BUDGET_USD" => {
                    config.cost.weekly_budget_usd = parse_optional_f64(&var_name, &raw_value)?
                }
                "MORECODE_COST_MONTHLY_BUDGET_USD" => {
                    config.cost.monthly_budget_usd = parse_optional_f64(&var_name, &raw_value)?
                }
                "MORECODE_COST_PER_TASK_BUDGET_USD" => {
                    config.cost.per_task_budget_usd = parse_optional_f64(&var_name, &raw_value)?
                }
                "MORECODE_COST_OVER_BUDGET_ACTION" => {
                    config.cost.over_budget_action = normalize_string(raw_value)
                }
                "MORECODE_COST_COST_LOG_PATH" => {
                    config.cost.cost_log_path = normalize_string(raw_value)
                }

                _ => {}
            }
        }

        Ok(())
    }

    fn watch_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        for path in [&self.global_config_path, &self.project_config_path] {
            let watch_root = existing_watch_root(path);
            if !roots.iter().any(|existing| existing == &watch_root) {
                roots.push(watch_root);
            }
        }
        roots
    }
}

impl Drop for ConfigLoader {
    fn drop(&mut self) {
        let state = match self.hot_reload_state.lock() {
            Ok(mut guard) => guard.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        };

        if let Some(state) = state {
            state.task.abort();
        }
    }
}

async fn process_notify_result(
    result: std::result::Result<Event, notify::Error>,
    global_config_path: &PathBuf,
    project_config_path: &PathBuf,
    config_change_tx: &broadcast::Sender<ConfigChangeEvent>,
    current_config: &Arc<RwLock<AppConfig>>,
) {
    match result {
        Ok(event) => {
            let change_type = classify_change(&event.kind);
            let relevant_paths: Vec<PathBuf> = event
                .paths
                .into_iter()
                .filter(|path| is_target_path(path, global_config_path, project_config_path))
                .collect();

            if relevant_paths.is_empty() {
                return;
            }

            for path in relevant_paths {
                let _ = config_change_tx.send(ConfigChangeEvent::FileChanged { path, change_type });
            }

            sleep(Duration::from_millis(HOT_RELOAD_SETTLE_MILLIS)).await;

            match ConfigLoader::load_with_paths(global_config_path, project_config_path).await {
                Ok(config) => {
                    write_snapshot(current_config, config.clone());
                    let _ = config_change_tx.send(ConfigChangeEvent::Reloaded { config });
                }
                Err(error) => {
                    let _ = config_change_tx.send(ConfigChangeEvent::ReloadFailed { error });
                }
            }
        }
        Err(error) => {
            let _ = config_change_tx.send(ConfigChangeEvent::ReloadFailed {
                error: ConfigError::HotReloadFailed {
                    reason: error.to_string(),
                },
            });
        }
    }
}

fn classify_change(kind: &EventKind) -> FileChangeType {
    match kind {
        EventKind::Create(_) => FileChangeType::Created,
        EventKind::Remove(_) => FileChangeType::Deleted,
        _ => FileChangeType::Modified,
    }
}

fn is_target_path(path: &Path, global_config_path: &Path, project_config_path: &Path) -> bool {
    let normalized = normalize_path_for_compare(path);
    normalized == normalize_path_for_compare(global_config_path)
        || normalized == normalize_path_for_compare(project_config_path)
}

fn normalize_path_for_compare(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn existing_watch_root(path: &Path) -> PathBuf {
    let mut candidate = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| path.to_path_buf());

    while !candidate.exists() {
        if !candidate.pop() {
            return PathBuf::from(".");
        }
    }

    candidate
}

fn normalize_config_path(path: PathBuf) -> PathBuf {
    if path.extension().and_then(|extension| extension.to_str()) == Some("toml") {
        path
    } else {
        path.join(CONFIG_FILE_NAME)
    }
}

fn absolutize_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        match env::current_dir() {
            Ok(current_dir) => current_dir.join(path),
            Err(_) => path,
        }
    }
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn lock_hot_reload_state(
    mutex: &Mutex<Option<HotReloadState>>,
) -> std::sync::MutexGuard<'_, Option<HotReloadState>> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn write_snapshot(snapshot: &Arc<RwLock<AppConfig>>, config: AppConfig) {
    match snapshot.write() {
        Ok(mut guard) => *guard = config,
        Err(poisoned) => *poisoned.into_inner() = config,
    }
}

fn read_snapshot(snapshot: &Arc<RwLock<AppConfig>>) -> AppConfig {
    match snapshot.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

fn normalize_string(value: String) -> String {
    value.trim().to_string()
}

fn optional_string(value: String) -> Option<String> {
    let value = normalize_string(value);
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn parse_bool(var_name: &str, value: &str) -> Result<bool> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(ConfigError::EnvVarParseFailed {
            var_name: var_name.to_string(),
            reason: "期望布尔值(true/false/1/0/yes/no/on/off)".to_string(),
        }),
    }
}

fn parse_value<T>(var_name: &str, value: &str) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value
        .trim()
        .parse::<T>()
        .map_err(|error| ConfigError::EnvVarParseFailed {
            var_name: var_name.to_string(),
            reason: error.to_string(),
        })
}

fn parse_optional_f64(var_name: &str, value: &str) -> Result<Option<f64>> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(parse_value(var_name, value)?))
    }
}

fn parse_string_list(var_name: &str, value: &str) -> Result<Vec<String>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if trimmed.starts_with('[') {
        if let Ok(values) = serde_json::from_str::<Vec<String>>(trimmed) {
            return Ok(values);
        }

        #[derive(Deserialize)]
        struct StringListWrapper {
            value: Vec<String>,
        }

        let wrapped = format!("value = {trimmed}");
        return toml::from_str::<StringListWrapper>(&wrapped)
            .map(|wrapper| wrapper.value)
            .map_err(|error| ConfigError::EnvVarParseFailed {
                var_name: var_name.to_string(),
                reason: error.to_string(),
            });
    }

    Ok(trimmed
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}
