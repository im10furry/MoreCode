use std::{
    fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::time::{sleep, timeout, Duration};

use crate::{
    loader::ConfigLoader, validate, AgentConfig, AppConfig, AppSettings, ConfigChangeEvent,
    ContextConfig, CoordinatorConfig, CostBudgetConfig, DaemonConfig, MemoryConfig, ProviderConfig,
    RecursiveConfig, SandboxConfig, TuiConfig,
};

struct TempWorkspace {
    root: PathBuf,
}

impl TempWorkspace {
    fn new() -> Self {
        let unique = format!(
            "config-test-{}-{}",
            process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn global_config_path(&self) -> PathBuf {
        self.root.join("global").join("config.toml")
    }

    fn project_config_path(&self) -> PathBuf {
        self.root.join("project").join("config.toml")
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn each_section_supports_toml_parsing() {
    let app: AppSettings = toml::from_str(
        r#"
name = "demo"
log_level = "debug"
version = "0.2.0"
data_dir = ".data"
"#,
    )
    .unwrap();
    assert_eq!(app.name, "demo");
    assert_eq!(app.log_level, "debug");

    let coordinator: CoordinatorConfig = toml::from_str(
        r#"
max_token_budget = 123456
max_recursion_depth = 3
agent_timeout_secs = 99
max_retries = 5
memory_aware_routing = false
recursive_orchestration = false
memory_stale_threshold_days = 14
preflight_check = false
llm_weight_multiplier = 1.5
"#,
    )
    .unwrap();
    assert_eq!(coordinator.max_token_budget, 123456);
    assert_eq!(coordinator.max_recursion_depth, 3);

    let agent: AgentConfig = toml::from_str(
        r#"
default_model = "gpt-4.1-mini"
temperature = 0.2
max_output_tokens = 2048
streaming = false
tool_timeout_secs = 5
auto_retry = false
"#,
    )
    .unwrap();
    assert_eq!(agent.default_model, "gpt-4.1-mini");
    assert_eq!(agent.temperature, 0.2);

    let provider: ProviderConfig = toml::from_str(
        r#"
default_provider = "primary"
semantic_cache_enabled = true
semantic_cache_threshold = 0.95
precise_token_count = true

[providers.primary]
provider_type = "openai-compat"
base_url = "https://example.com/v1"
api_key_env = "OPENAI_API_KEY"
default_model = "gpt-4o"
"#,
    )
    .unwrap();
    assert_eq!(provider.default_provider, "primary");
    assert_eq!(
        provider.providers["primary"].base_url.as_deref(),
        Some("https://example.com/v1")
    );

    let memory: MemoryConfig = toml::from_str(
        r#"
memory_dir = ".memory"
ttl_days = 7
core_memory_limit = 50
working_memory_max_files = 20
working_memory_max_mb = 8
sleep_time_compute = true
"#,
    )
    .unwrap();
    assert_eq!(memory.memory_dir, ".memory");

    let context: ContextConfig = toml::from_str(
        r#"
l1_micro_compress_threshold = 5000
l2_auto_compress_threshold = 0.8
l3_memory_compress_threshold = 0.9
repo_map_enabled = false
large_file_threshold = 1000
"#,
    )
    .unwrap();
    assert_eq!(context.large_file_threshold, 1000);

    let sandbox: SandboxConfig = toml::from_str(
        r#"
permission_mode = "readonly"
landlock_enabled = true
seccomp_enabled = true
wasm_enabled = true
command_whitelist = ["git", "cargo"]
read_only_paths = ["/repo"]
write_paths = ["/tmp"]
"#,
    )
    .unwrap();
    assert_eq!(sandbox.permission_mode, "readonly");
    assert_eq!(sandbox.command_whitelist.len(), 2);

    let recursive: RecursiveConfig = toml::from_str(
        r#"
max_sub_agents = 3
max_depth = 2
max_total_sub_agents = 10
sub_agent_timeout_secs = 45
enabled = false
"#,
    )
    .unwrap();
    assert_eq!(recursive.max_sub_agents, 3);
    assert!(!recursive.enabled);

    let daemon: DaemonConfig = toml::from_str(
        r#"
enabled = true
pid_file = "/tmp/demo.pid"
health_check_interval_secs = 15
auto_update_check_hours = 12
daily_budget_usd = 4.5

[quiet_hours]
start_hour = 22
end_hour = 8
"#,
    )
    .unwrap();
    assert!(daemon.enabled);
    assert_eq!(daemon.quiet_hours.unwrap().start_hour, 22);

    let tui: TuiConfig = toml::from_str(
        r#"
theme = "light"
mouse_support = true
max_log_lines = 200
refresh_rate_ms = 33
custom_theme_path = "theme.toml"
"#,
    )
    .unwrap();
    assert_eq!(tui.theme, "light");

    let cost: CostBudgetConfig = toml::from_str(
        r#"
daily_budget_usd = 5.0
weekly_budget_usd = 20.0
monthly_budget_usd = 60.0
per_task_budget_usd = 1.0
over_budget_action = "pause"
cost_log_path = ".memory/cost.json"
"#,
    )
    .unwrap();
    assert_eq!(cost.over_budget_action, "pause");
}

#[test]
fn empty_toml_uses_defaults() {
    let config: AppConfig = toml::from_str("").unwrap();
    assert_eq!(config.app.name, "morecode");
    assert_eq!(config.coordinator.max_token_budget, 200_000);
    assert_eq!(config.agent.default_model, "gpt-4o");
    assert_eq!(config.sandbox.permission_mode, "default");
    assert_eq!(config.tui.theme, "dark");
}

#[test]
fn env_mapping_applies_expected_override() {
    let mut config = AppConfig::default();
    ConfigLoader::apply_env_overrides(
        &mut config,
        vec![(
            "MORECODE_COORDINATOR_MAX_TOKEN_BUDGET".to_string(),
            "100000".to_string(),
        )],
    )
    .unwrap();
    assert_eq!(config.coordinator.max_token_budget, 100_000);
}

#[test]
fn validate_rejects_mutually_exclusive_modes() {
    let mut config = AppConfig::default();
    config.sandbox.permission_mode = "bypass".to_string();
    config.daemon.enabled = true;

    let error = validate(&config).unwrap_err();
    assert_eq!(
        error.to_string(),
        "配置验证失败: sandbox.permission_mode: bypass 模式与 daemon.enabled=true 互斥"
    );
}

#[tokio::test]
async fn loader_merges_global_and_project_config() {
    let workspace = TempWorkspace::new();

    write_file(
        &workspace.global_config_path(),
        r#"
[coordinator]
max_token_budget = 50000

[agent]
temperature = 0.1

[provider.providers.primary]
provider_type = "openai-compat"
base_url = "https://global.example/v1"
headers = { Authorization = "global" }
"#,
    );

    write_file(
        &workspace.project_config_path(),
        r#"
[agent]
temperature = 0.9

[provider]
default_provider = "primary"

[provider.providers.primary]
headers = { Authorization = "project", X-Team = "core" }
"#,
    );

    let loader = ConfigLoader::with_paths(
        workspace.global_config_path(),
        workspace.project_config_path(),
    );

    let mut loaded = None;
    for _ in 0..5 {
        match loader.load().await {
            Ok(config) => {
                loaded = Some(config);
                break;
            }
            Err(_) => sleep(Duration::from_millis(50)).await,
        }
    }
    let config = loaded.expect("config should load after retries");

    assert_eq!(config.coordinator.max_token_budget, 50_000);
    assert_eq!(config.agent.temperature, 0.9);
    assert_eq!(config.provider.default_provider, "primary");
    assert_eq!(
        config.provider.providers["primary"].base_url.as_deref(),
        Some("https://global.example/v1")
    );
    assert_eq!(
        config.provider.providers["primary"].headers["Authorization"],
        "project"
    );
    assert_eq!(
        config.provider.providers["primary"].headers["X-Team"],
        "core"
    );
}

#[test]
fn validation_rejects_invalid_config() {
    let mut config = AppConfig::default();
    config.coordinator.max_token_budget = 0;

    let error = validate(&config).unwrap_err();
    assert_eq!(
        error,
        crate::ConfigError::ValidationFailed {
            field: "coordinator.max_token_budget".to_string(),
            reason: "必须大于 0".to_string(),
        }
    );
}

#[tokio::test]
async fn missing_config_files_fall_back_to_defaults() {
    let workspace = TempWorkspace::new();
    let loader = ConfigLoader::with_paths(
        workspace.global_config_path(),
        workspace.project_config_path(),
    );

    let config = loader.load().await.unwrap();
    assert_eq!(config, AppConfig::default());
}

#[tokio::test]
async fn hot_reload_updates_snapshot_and_emits_event() {
    let workspace = TempWorkspace::new();
    write_file(
        &workspace.project_config_path(),
        r#"
[agent]
temperature = 0.2
"#,
    );

    let loader = ConfigLoader::with_paths(
        workspace.global_config_path(),
        workspace.project_config_path(),
    );
    loader.load().await.unwrap();
    let mut receiver = loader.subscribe();
    loader.start_hot_reload().await.unwrap();

    let reloaded = write_until_reloaded(
        &workspace.project_config_path(),
        &mut receiver,
        r#"
[agent]
temperature = 1.1
"#,
        1.1,
    )
    .await
    .expect("hot reload should emit a matching reload event");

    assert_eq!(reloaded.agent.temperature, 1.1);
    assert_eq!(loader.current().agent.temperature, 1.1);
}

async fn write_until_reloaded(
    path: &Path,
    receiver: &mut tokio::sync::broadcast::Receiver<ConfigChangeEvent>,
    contents: &str,
    expected_temperature: f32,
) -> Option<AppConfig> {
    for _ in 0..5 {
        sleep(Duration::from_millis(250)).await;
        write_file(path, contents);

        let result = timeout(Duration::from_secs(6), async {
            loop {
                match receiver.recv().await {
                    Ok(ConfigChangeEvent::Reloaded { config })
                        if (config.agent.temperature - expected_temperature).abs()
                            < f32::EPSILON =>
                    {
                        break Some(*config)
                    }
                    Ok(_) => continue,
                    Err(error) => panic!("unexpected broadcast error: {error}"),
                }
            }
        })
        .await;

        if let Ok(Some(config)) = result {
            return Some(config);
        }
    }

    None
}

#[test]
fn app_config_round_trips_with_json() {
    let config = AppConfig::default();
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: AppConfig = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, config);
}
