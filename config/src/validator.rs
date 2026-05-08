use crate::{AppConfig, ConfigError, Result};

pub fn validate(config: &AppConfig) -> Result<()> {
    validate_app(config)?;
    validate_coordinator(config)?;
    validate_agent(config)?;
    validate_provider(config)?;
    validate_provider_default_exists(config)?;
    validate_memory(config)?;
    validate_context(config)?;
    validate_sandbox(config)?;
    validate_recursive(config)?;
    validate_daemon(config)?;
    validate_tui(config)?;
    validate_cost(config)?;
    validate_line_ending(config)?;

    if config.daemon.enabled
        && config
            .sandbox
            .permission_mode
            .eq_ignore_ascii_case("bypass")
    {
        return Err(invalid(
            "sandbox.permission_mode",
            "bypass 模式与 daemon.enabled=true 互斥",
        ));
    }

    Ok(())
}

fn validate_app(config: &AppConfig) -> Result<()> {
    require_non_empty("app.name", &config.app.name)?;
    require_non_empty("app.log_level", &config.app.log_level)?;
    let log_level = config.app.log_level.to_ascii_lowercase();
    ensure(
        matches!(
            log_level.as_str(),
            "trace" | "debug" | "info" | "warn" | "error"
        ),
        "app.log_level",
        "log_level 必须是 trace/debug/info/warn/error 之一",
    )?;
    if let Some(version) = &config.app.version {
        require_non_empty("app.version", version)?;
    }
    if let Some(data_dir) = &config.app.data_dir {
        require_non_empty("app.data_dir", data_dir)?;
    }
    Ok(())
}

fn validate_coordinator(config: &AppConfig) -> Result<()> {
    ensure(
        config.coordinator.max_token_budget > 0,
        "coordinator.max_token_budget",
        "必须大于 0",
    )?;
    ensure(
        config.coordinator.max_recursion_depth <= 3,
        "coordinator.max_recursion_depth",
        "必须小于等于 3",
    )?;
    ensure(
        config.coordinator.agent_timeout_secs > 0,
        "coordinator.agent_timeout_secs",
        "必须大于 0",
    )?;
    ensure(
        config.coordinator.max_retries > 0,
        "coordinator.max_retries",
        "必须大于 0",
    )?;
    ensure(
        config.coordinator.memory_stale_threshold_days > 0,
        "coordinator.memory_stale_threshold_days",
        "必须大于 0",
    )?;
    ensure(
        config.coordinator.llm_weight_multiplier > 0.0,
        "coordinator.llm_weight_multiplier",
        "必须大于 0",
    )?;
    Ok(())
}

fn validate_agent(config: &AppConfig) -> Result<()> {
    require_non_empty("agent.default_model", &config.agent.default_model)?;
    ensure(
        (0.0..=2.0).contains(&config.agent.temperature),
        "agent.temperature",
        "temperature 必须在 0.0 到 2.0 之间",
    )?;
    ensure(
        config.agent.max_output_tokens > 0,
        "agent.max_output_tokens",
        "必须大于 0",
    )?;
    ensure(
        config.agent.tool_timeout_secs > 0,
        "agent.tool_timeout_secs",
        "必须大于 0",
    )?;
    Ok(())
}

fn validate_provider(config: &AppConfig) -> Result<()> {
    require_non_empty(
        "provider.default_provider",
        &config.provider.default_provider,
    )?;
    ensure(
        (0.0..=1.0).contains(&config.provider.semantic_cache_threshold),
        "provider.semantic_cache_threshold",
        "必须在 0.0 到 1.0 之间",
    )?;

    for (name, provider) in &config.provider.providers {
        require_non_empty(
            &format!("provider.providers.{name}.provider_type"),
            &provider.provider_type,
        )?;
        if let Some(value) = &provider.base_url {
            require_non_empty(&format!("provider.providers.{name}.base_url"), value)?;
        }
        if let Some(value) = &provider.default_model {
            require_non_empty(&format!("provider.providers.{name}.default_model"), value)?;
        }
        ensure(
            !(provider.api_key.is_some() && provider.api_key_env.is_some()),
            &format!("provider.providers.{name}.api_key"),
            "api_key 与 api_key_env 互斥",
        )?;
        for (header, value) in &provider.headers {
            require_non_empty(&format!("provider.providers.{name}.headers.key"), header)?;
            require_non_empty(
                &format!("provider.providers.{name}.headers.{header}"),
                value,
            )?;
        }
    }

    Ok(())
}

fn validate_provider_default_exists(config: &AppConfig) -> Result<()> {
    ensure(
        config
            .provider
            .has_provider(&config.provider.default_provider),
        "provider.default_provider",
        "default_provider 必须指向已定义的 provider 或内置 preset",
    )
}

fn validate_memory(config: &AppConfig) -> Result<()> {
    require_non_empty("memory.memory_dir", &config.memory.memory_dir)?;
    ensure(config.memory.ttl_days > 0, "memory.ttl_days", "必须大于 0")?;
    ensure(
        config.memory.core_memory_limit > 0,
        "memory.core_memory_limit",
        "必须大于 0",
    )?;
    ensure(
        config.memory.working_memory_max_files > 0,
        "memory.working_memory_max_files",
        "必须大于 0",
    )?;
    ensure(
        config.memory.working_memory_max_mb > 0,
        "memory.working_memory_max_mb",
        "必须大于 0",
    )?;
    Ok(())
}

fn validate_context(config: &AppConfig) -> Result<()> {
    ensure(
        config.context.l1_micro_compress_threshold > 0,
        "context.l1_micro_compress_threshold",
        "必须大于 0",
    )?;
    ensure(
        (0.0..=1.0).contains(&config.context.l2_auto_compress_threshold),
        "context.l2_auto_compress_threshold",
        "必须在 0.0 到 1.0 之间",
    )?;
    ensure(
        (0.0..=1.0).contains(&config.context.l3_memory_compress_threshold),
        "context.l3_memory_compress_threshold",
        "必须在 0.0 到 1.0 之间",
    )?;
    ensure(
        config.context.l3_memory_compress_threshold >= config.context.l2_auto_compress_threshold,
        "context.l3_memory_compress_threshold",
        "必须大于等于 l2_auto_compress_threshold",
    )?;
    ensure(
        config.context.large_file_threshold > 0,
        "context.large_file_threshold",
        "必须大于 0",
    )?;
    Ok(())
}

fn validate_sandbox(config: &AppConfig) -> Result<()> {
    require_non_empty("sandbox.permission_mode", &config.sandbox.permission_mode)?;
    let permission_mode = config.sandbox.permission_mode.to_ascii_lowercase();
    ensure(
        matches!(
            permission_mode.as_str(),
            "default" | "auto" | "readonly" | "plan" | "bypass"
        ),
        "sandbox.permission_mode",
        "permission_mode 必须是 default/auto/readonly/plan/bypass 之一",
    )?;

    for command in &config.sandbox.command_whitelist {
        require_non_empty("sandbox.command_whitelist", command)?;
    }
    for path in &config.sandbox.read_only_paths {
        require_non_empty("sandbox.read_only_paths", path)?;
    }
    for path in &config.sandbox.write_paths {
        require_non_empty("sandbox.write_paths", path)?;
    }

    Ok(())
}

fn validate_recursive(config: &AppConfig) -> Result<()> {
    ensure(
        config.recursive.max_sub_agents > 0,
        "recursive.max_sub_agents",
        "必须大于 0",
    )?;
    ensure(
        config.recursive.max_depth > 0,
        "recursive.max_depth",
        "必须大于 0",
    )?;
    ensure(
        config.recursive.max_total_sub_agents >= config.recursive.max_sub_agents,
        "recursive.max_total_sub_agents",
        "必须大于等于 max_sub_agents",
    )?;
    ensure(
        config.recursive.sub_agent_timeout_secs > 0,
        "recursive.sub_agent_timeout_secs",
        "必须大于 0",
    )?;
    ensure(
        config.recursive.max_depth <= usize::from(config.coordinator.max_recursion_depth),
        "recursive.max_depth",
        "不能超过 coordinator.max_recursion_depth",
    )?;
    Ok(())
}

fn validate_daemon(config: &AppConfig) -> Result<()> {
    require_non_empty("daemon.pid_file", &config.daemon.pid_file)?;
    ensure(
        config.daemon.health_check_interval_secs > 0,
        "daemon.health_check_interval_secs",
        "必须大于 0",
    )?;
    ensure(
        config.daemon.auto_update_check_hours > 0,
        "daemon.auto_update_check_hours",
        "必须大于 0",
    )?;
    if let Some(quiet_hours) = &config.daemon.quiet_hours {
        let start_set = quiet_hours.start_hour != u8::MAX;
        let end_set = quiet_hours.end_hour != u8::MAX;
        if start_set {
            ensure(
                quiet_hours.start_hour < 24,
                "daemon.quiet_hours.start_hour",
                "必须在 0 到 23 之间",
            )?;
        }
        if end_set {
            ensure(
                quiet_hours.end_hour < 24,
                "daemon.quiet_hours.end_hour",
                "必须在 0 到 23 之间",
            )?;
        }
        if start_set && end_set {
            ensure(
                quiet_hours.start_hour != quiet_hours.end_hour,
                "daemon.quiet_hours",
                "start_hour 与 end_hour 不能相同",
            )?;
        }
    }
    if let Some(daily_budget_usd) = config.daemon.daily_budget_usd {
        ensure(
            daily_budget_usd > 0.0,
            "daemon.daily_budget_usd",
            "必须大于 0",
        )?;
    }
    Ok(())
}

fn validate_tui(config: &AppConfig) -> Result<()> {
    require_non_empty("tui.theme", &config.tui.theme)?;
    require_non_empty("tui.language", &config.tui.language)?;
    let language = config.tui.language.to_ascii_lowercase();
    ensure(
        matches!(
            language.as_str(),
            "auto" | "en" | "en-us" | "en_us" | "zh" | "zh-cn" | "zh_cn"
        ),
        "tui.language",
        "language 必须是 auto/en/zh/zh-cn 之一",
    )?;
    ensure(
        config.tui.max_log_lines > 0,
        "tui.max_log_lines",
        "必须大于 0",
    )?;
    ensure(
        config.tui.refresh_rate_ms > 0,
        "tui.refresh_rate_ms",
        "必须大于 0",
    )?;
    if let Some(custom_theme_path) = &config.tui.custom_theme_path {
        require_non_empty("tui.custom_theme_path", custom_theme_path)?;
    }
    Ok(())
}

fn validate_cost(config: &AppConfig) -> Result<()> {
    validate_optional_positive(config.cost.daily_budget_usd, "cost.daily_budget_usd")?;
    validate_optional_positive(config.cost.weekly_budget_usd, "cost.weekly_budget_usd")?;
    validate_optional_positive(config.cost.monthly_budget_usd, "cost.monthly_budget_usd")?;
    validate_optional_positive(config.cost.per_task_budget_usd, "cost.per_task_budget_usd")?;

    if let (Some(daily), Some(weekly)) =
        (config.cost.daily_budget_usd, config.cost.weekly_budget_usd)
    {
        ensure(
            weekly >= daily,
            "cost.weekly_budget_usd",
            "weekly_budget_usd 不能小于 daily_budget_usd",
        )?;
    }
    if let (Some(weekly), Some(monthly)) = (
        config.cost.weekly_budget_usd,
        config.cost.monthly_budget_usd,
    ) {
        ensure(
            monthly >= weekly,
            "cost.monthly_budget_usd",
            "monthly_budget_usd 不能小于 weekly_budget_usd",
        )?;
    }
    if let (Some(per_task), Some(daily)) = (
        config.cost.per_task_budget_usd,
        config.cost.daily_budget_usd,
    ) {
        ensure(
            per_task <= daily,
            "cost.per_task_budget_usd",
            "per_task_budget_usd 不能大于 daily_budget_usd",
        )?;
    }

    require_non_empty("cost.over_budget_action", &config.cost.over_budget_action)?;
    let over_budget_action = config.cost.over_budget_action.to_ascii_lowercase();
    ensure(
        matches!(over_budget_action.as_str(), "warn" | "pause" | "block"),
        "cost.over_budget_action",
        "over_budget_action 必须是 warn/pause/block 之一",
    )?;
    require_non_empty("cost.cost_log_path", &config.cost.cost_log_path)?;
    Ok(())
}

fn validate_line_ending(config: &AppConfig) -> Result<()> {
    ensure(
        config.line_ending.max_file_size_kb > 0,
        "line_ending.max_file_size_kb",
        "必须大于 0",
    )?;
    Ok(())
}

fn validate_optional_positive(value: Option<f64>, field: &str) -> Result<()> {
    if let Some(value) = value {
        ensure(value > 0.0, field, "必须大于 0")?;
    }
    Ok(())
}

fn require_non_empty(field: &str, value: &str) -> Result<()> {
    ensure(!value.trim().is_empty(), field, "不能为空")
}

fn ensure(condition: bool, field: &str, reason: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid(field, reason))
    }
}

fn invalid(field: &str, reason: &str) -> ConfigError {
    ConfigError::ValidationFailed {
        field: field.to_string(),
        reason: reason.to_string(),
    }
}
