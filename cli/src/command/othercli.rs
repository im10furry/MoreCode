use std::{
    collections::HashSet,
    io::{self, Write},
    path::{Path, PathBuf},
};

use mc_config::ConfigLoader;
use serde_json::Value as JsonValue;

use crate::init::AppContext;

#[derive(Debug, Clone, PartialEq, Eq)]
enum SourceKind {
    Env,
    UserFile,
    ProjectEnvFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DetectedSource {
    kind: SourceKind,
    label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DetectedProvider {
    source: DetectedSource,
    product: String,
    provider_type: String,
    api_key_present: bool,
    api_key_env: Option<String>,
    base_url: Option<String>,
    default_model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteTarget {
    User,
    Project,
    GenerateOnly,
}

pub async fn execute(context: &AppContext) -> Result<String, String> {
    println!("OtherCliScanStarted");

    let scan = scan_sources(&context.project_root).await?;

    for source in &scan.sources {
        println!("OtherCliSourceDetected kind={:?} label={}", source.kind, source.label);
    }
    for provider in &scan.providers {
        println!(
            "OtherCliProviderDetected provider_type={} product={} source={}",
            provider.provider_type, provider.product, provider.source.label
        );
    }

    if scan.providers.is_empty() {
        return Ok("未检测到可迁移的 Provider 配置线索（项目 .env 仅检测环境变量键名，不会导入任何明文值）".into());
    }

    println!();
    println!("检测到的 Provider 候选项（不会展示任何密钥明文）：");
    for (idx, provider) in scan.providers.iter().enumerate() {
        println!(
            "  [{}] type={} product={} api_key={} api_key_env={} base_url={} model={} source={}",
            idx + 1,
            provider.provider_type,
            provider.product,
            yes_no(provider.api_key_present),
            provider
                .api_key_env
                .as_deref()
                .unwrap_or("-"),
            provider.base_url.as_deref().unwrap_or("-"),
            provider.default_model.as_deref().unwrap_or("-"),
            provider.source.label
        );
    }

    println!();
    let selected = prompt_indices("选择要导入的项（例如 1,3 或 all，默认 all）: ", scan.providers.len())?;
    if selected.is_empty() {
        return Ok("已取消导入".into());
    }

    let target = prompt_target()?;

    let target_path = match target {
        WriteTarget::User => ConfigLoader::default_global_config_path()
            .map_err(|error| error.to_string())?,
        WriteTarget::Project => context
            .project_root
            .join(mc_core::PROJECT_CONFIG_SUBDIR)
            .join(mc_core::CONFIG_FILE_NAME),
        WriteTarget::GenerateOnly => context
            .project_root
            .join(mc_core::PROJECT_CONFIG_SUBDIR)
            .join("othercli-import.toml"),
    };

    let existing = existing_provider_names(&target_path).await;
    let mut provider_names: HashSet<String> = context
        .config
        .provider
        .provider_names()
        .into_iter()
        .map(str::to_string)
        .collect();
    provider_names.extend(existing);

    let plan = build_import_plan(&scan.providers, &selected, &provider_names)?;

    println!();
    println!("OtherCliPlanGenerated");
    println!("将追加写入：{}", target_path.display());
    println!();
    println!("{}", plan.preview_snippet);

    if !prompt_confirm("确认写入？（y/N）: ")? {
        return Ok("已取消写入".into());
    }

    apply_snippet(&target_path, &plan.preview_snippet).await?;
    println!("OtherCliApplied");

    Ok(format!(
        "导入完成：写入 {} 个 provider 到 {}",
        plan.imported_count,
        target_path.display()
    ))
}

struct ScanResult {
    sources: Vec<DetectedSource>,
    providers: Vec<DetectedProvider>,
}

async fn scan_sources(project_root: &Path) -> Result<ScanResult, String> {
    let mut sources = Vec::new();
    let mut providers = Vec::new();

    let (env_sources, env_providers) = scan_env();
    sources.extend(env_sources);
    providers.extend(env_providers);

    let (user_sources, user_providers) = scan_user_files().await;
    sources.extend(user_sources);
    providers.extend(user_providers);

    let (project_sources, project_providers) = scan_project_env_files(project_root).await;
    sources.extend(project_sources);
    providers.extend(project_providers);

    Ok(ScanResult { sources, providers })
}

fn scan_env() -> (Vec<DetectedSource>, Vec<DetectedProvider>) {
    let mut sources = Vec::new();
    let mut providers = Vec::new();

    for spec in provider_specs() {
        let (present, used_env) = find_present_env(&spec.api_key_env_candidates);
        if present {
            sources.push(DetectedSource {
                kind: SourceKind::Env,
                label: format!("env:{}", used_env.as_deref().unwrap_or(spec.canonical_api_key_env)),
            });
        }

        let base_url = first_non_empty_env(&spec.base_url_env_candidates);
        let default_model = first_non_empty_env(&spec.model_env_candidates);

        if present || base_url.is_some() || default_model.is_some() {
            providers.push(DetectedProvider {
                source: DetectedSource {
                    kind: SourceKind::Env,
                    label: "env".into(),
                },
                product: spec.product.into(),
                provider_type: spec.provider_type.into(),
                api_key_present: present,
                api_key_env: used_env.or_else(|| Some(spec.canonical_api_key_env.into())),
                base_url,
                default_model,
            });
        }
    }

    (sources, providers)
}

async fn scan_user_files() -> (Vec<DetectedSource>, Vec<DetectedProvider>) {
    let mut sources = Vec::new();
    let mut providers = Vec::new();

    let mut candidates = Vec::new();
    if let Some(home) = dirs::home_dir() {
        candidates.extend([
            home.join(".codex").join("config.json"),
            home.join(".claude").join("config.json"),
            home.join(".gemini").join("config.json"),
            home.join(".config").join("codex").join("config.json"),
            home.join(".config").join("claude").join("config.json"),
            home.join(".config").join("claude").join("settings.json"),
            home.join(".config").join("gemini").join("config.json"),
        ]);
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let roaming = PathBuf::from(appdata);
        candidates.extend([
            roaming.join("codex").join("config.json"),
            roaming.join("claude").join("config.json"),
            roaming.join("gemini").join("config.json"),
        ]);
    }

    for path in candidates {
        let Some(file_info) = read_small_file(&path, 256 * 1024).await else {
            continue;
        };
        sources.push(DetectedSource {
            kind: SourceKind::UserFile,
            label: path.display().to_string(),
        });

        let (product, provider_type) = infer_product_and_provider_type(&path);
        let extracted = extract_provider_hints(&path, &file_info.contents);
        if !extracted.any_present() {
            continue;
        }

        let api_key_env = extracted
            .api_key_env
            .or_else(|| canonical_api_key_env_for_provider(&provider_type).map(str::to_string));

        providers.push(DetectedProvider {
            source: DetectedSource {
                kind: SourceKind::UserFile,
                label: path.display().to_string(),
            },
            product,
            provider_type,
            api_key_present: extracted.api_key_present,
            api_key_env,
            base_url: extracted.base_url,
            default_model: extracted.default_model,
        });
    }

    (sources, providers)
}

async fn scan_project_env_files(project_root: &Path) -> (Vec<DetectedSource>, Vec<DetectedProvider>) {
    let mut sources = Vec::new();
    let mut providers = Vec::new();

    let files = [
        ".env",
        ".env.local",
        ".env.development",
        ".env.production",
        ".env.test",
    ];

    let mut present_keys = HashSet::new();
    for name in files {
        let path = project_root.join(name);
        let Some(file_info) = read_small_file(&path, 256 * 1024).await else {
            continue;
        };
        sources.push(DetectedSource {
            kind: SourceKind::ProjectEnvFile,
            label: path.display().to_string(),
        });

        for key in extract_env_keys(&file_info.contents) {
            present_keys.insert(key);
        }
    }

    for spec in provider_specs() {
        let api_key_present = present_keys
            .iter()
            .any(|key| spec.api_key_env_candidates.iter().any(|candidate| candidate == key));
        if !api_key_present {
            continue;
        }

        providers.push(DetectedProvider {
            source: DetectedSource {
                kind: SourceKind::ProjectEnvFile,
                label: "project:.env*".into(),
            },
            product: spec.product.into(),
            provider_type: spec.provider_type.into(),
            api_key_present: true,
            api_key_env: Some(spec.canonical_api_key_env.into()),
            base_url: None,
            default_model: None,
        });
    }

    (sources, providers)
}

struct ProviderHints {
    api_key_present: bool,
    api_key_env: Option<String>,
    base_url: Option<String>,
    default_model: Option<String>,
}

impl ProviderHints {
    fn any_present(&self) -> bool {
        self.api_key_present || self.api_key_env.is_some() || self.base_url.is_some() || self.default_model.is_some()
    }
}

fn extract_provider_hints(path: &Path, contents: &str) -> ProviderHints {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "json" => extract_from_json(contents),
        "toml" => extract_from_toml(contents),
        _ => ProviderHints {
            api_key_present: false,
            api_key_env: None,
            base_url: None,
            default_model: None,
        },
    }
}

fn extract_from_json(contents: &str) -> ProviderHints {
    let value: JsonValue = match serde_json::from_str(contents) {
        Ok(value) => value,
        Err(_) => {
            return ProviderHints {
                api_key_present: false,
                api_key_env: None,
                base_url: None,
                default_model: None,
            }
        }
    };

    let api_key_present = find_any_string_key(&value, &["api_key", "apikey", "apiKey", "key", "token"])
        .is_some();
    let api_key_env =
        find_any_string_key(&value, &["api_key_env", "apiKeyEnv", "api_key_env_var", "key_env"]);
    let base_url = find_any_string_key(&value, &["base_url", "baseUrl", "api_base", "apiBaseUrl", "endpoint", "url"]);
    let default_model = find_any_string_key(&value, &["default_model", "defaultModel", "model", "model_id", "modelId"]);

    ProviderHints {
        api_key_present,
        api_key_env,
        base_url,
        default_model,
    }
}

fn extract_from_toml(contents: &str) -> ProviderHints {
    let value: toml::Value = match toml::from_str(contents) {
        Ok(value) => value,
        Err(_) => {
            return ProviderHints {
                api_key_present: false,
                api_key_env: None,
                base_url: None,
                default_model: None,
            }
        }
    };

    let api_key_present =
        find_any_toml_string_key(&value, &["api_key", "apikey", "key", "token"]).is_some();
    let api_key_env =
        find_any_toml_string_key(&value, &["api_key_env", "key_env", "api_key_env_var"]);
    let base_url = find_any_toml_string_key(&value, &["base_url", "api_base", "endpoint", "url"]);
    let default_model = find_any_toml_string_key(&value, &["default_model", "model", "model_id"]);

    ProviderHints {
        api_key_present,
        api_key_env,
        base_url,
        default_model,
    }
}

fn find_any_string_key(value: &JsonValue, keys: &[&str]) -> Option<String> {
    match value {
        JsonValue::Object(map) => {
            for (k, v) in map {
                if keys.iter().any(|target| k.eq_ignore_ascii_case(target)) {
                    if let Some(s) = v.as_str().map(str::trim).filter(|v| !v.is_empty()) {
                        return Some(s.to_string());
                    }
                }
                if let Some(found) = find_any_string_key(v, keys) {
                    return Some(found);
                }
            }
            None
        }
        JsonValue::Array(items) => {
            for item in items {
                if let Some(found) = find_any_string_key(item, keys) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

fn find_any_toml_string_key(value: &toml::Value, keys: &[&str]) -> Option<String> {
    match value {
        toml::Value::Table(map) => {
            for (k, v) in map {
                if keys.iter().any(|target| k.eq_ignore_ascii_case(target)) {
                    if let toml::Value::String(s) = v {
                        let s = s.trim();
                        if !s.is_empty() {
                            return Some(s.to_string());
                        }
                    }
                }
                if let Some(found) = find_any_toml_string_key(v, keys) {
                    return Some(found);
                }
            }
            None
        }
        toml::Value::Array(items) => {
            for item in items {
                if let Some(found) = find_any_toml_string_key(item, keys) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

fn extract_env_keys(contents: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for raw in contents.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, _)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        keys.push(key.to_string());
    }
    keys
}

struct SmallFile {
    contents: String,
}

async fn read_small_file(path: &Path, max_bytes: u64) -> Option<SmallFile> {
    let meta = tokio::fs::metadata(path).await.ok()?;
    if !meta.is_file() || meta.len() > max_bytes {
        return None;
    }
    let contents = tokio::fs::read_to_string(path).await.ok()?;
    Some(SmallFile { contents })
}

struct ProviderSpec {
    product: &'static str,
    provider_type: &'static str,
    canonical_api_key_env: &'static str,
    api_key_env_candidates: Vec<&'static str>,
    base_url_env_candidates: Vec<&'static str>,
    model_env_candidates: Vec<&'static str>,
}

fn provider_specs() -> Vec<ProviderSpec> {
    vec![
        ProviderSpec {
            product: "codex",
            provider_type: "openai-compat",
            canonical_api_key_env: "OPENAI_API_KEY",
            api_key_env_candidates: vec!["OPENAI_API_KEY", "OPENAI_KEY", "OPENAI_APIKEY"],
            base_url_env_candidates: vec!["OPENAI_BASE_URL", "OPENAI_API_BASE"],
            model_env_candidates: vec!["OPENAI_MODEL", "OPENAI_DEFAULT_MODEL"],
        },
        ProviderSpec {
            product: "claude",
            provider_type: "anthropic",
            canonical_api_key_env: "ANTHROPIC_API_KEY",
            api_key_env_candidates: vec!["ANTHROPIC_API_KEY", "ANTHROPIC_KEY", "CLAUDE_API_KEY"],
            base_url_env_candidates: vec!["ANTHROPIC_BASE_URL"],
            model_env_candidates: vec!["ANTHROPIC_MODEL", "CLAUDE_MODEL"],
        },
        ProviderSpec {
            product: "gemini",
            provider_type: "google",
            canonical_api_key_env: "GOOGLE_API_KEY",
            api_key_env_candidates: vec!["GOOGLE_API_KEY", "GEMINI_API_KEY", "GOOGLE_GENAI_API_KEY"],
            base_url_env_candidates: vec!["GOOGLE_BASE_URL", "GEMINI_BASE_URL"],
            model_env_candidates: vec!["GOOGLE_MODEL", "GEMINI_MODEL"],
        },
    ]
}

fn canonical_api_key_env_for_provider(provider_type: &str) -> Option<&'static str> {
    for spec in provider_specs() {
        if spec.provider_type == provider_type {
            return Some(spec.canonical_api_key_env);
        }
    }
    None
}

fn find_present_env(candidates: &[&'static str]) -> (bool, Option<String>) {
    for name in candidates {
        if let Ok(value) = std::env::var(name) {
            if !value.trim().is_empty() {
                return (true, Some((*name).to_string()));
            }
        }
    }
    (false, None)
}

fn first_non_empty_env(candidates: &[&'static str]) -> Option<String> {
    for name in candidates {
        if let Ok(value) = std::env::var(name) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn infer_product_and_provider_type(path: &Path) -> (String, String) {
    let lower = path.display().to_string().to_ascii_lowercase();
    if lower.contains("claude") {
        return ("claude".into(), "anthropic".into());
    }
    if lower.contains("gemini") {
        return ("gemini".into(), "google".into());
    }
    if lower.contains("codex") || lower.contains("openai") {
        return ("codex".into(), "openai-compat".into());
    }
    ("unknown".into(), "openai-compat".into())
}

struct ImportPlan {
    imported_count: usize,
    preview_snippet: String,
}

fn build_import_plan(
    providers: &[DetectedProvider],
    selected: &[usize],
    existing_names: &HashSet<String>,
) -> Result<ImportPlan, String> {
    let mut imported = Vec::new();
    let mut used_names = existing_names.clone();

    for idx in selected {
        let provider = providers
            .get(*idx)
            .ok_or_else(|| format!("非法选择项：{}", idx + 1))?;
        let base_name = match provider.provider_type.as_str() {
            "anthropic" => "imported-claude",
            "google" => "imported-gemini",
            _ => "imported-codex",
        };
        let name = unique_name(base_name, &used_names);
        used_names.insert(name.clone());

        imported.push((name, provider.clone()));
    }

    let snippet = render_provider_append_snippet(&imported);

    Ok(ImportPlan {
        imported_count: imported.len(),
        preview_snippet: snippet,
    })
}

fn unique_name(base: &str, existing: &HashSet<String>) -> String {
    if !existing.contains(base) {
        return base.to_string();
    }
    for idx in 2..=999 {
        let candidate = format!("{base}-{idx}");
        if !existing.contains(&candidate) {
            return candidate;
        }
    }
    format!("{base}-imported")
}

fn render_provider_append_snippet(imports: &[(String, DetectedProvider)]) -> String {
    let mut out = String::new();
    out.push('\n');

    for (name, provider) in imports {
        out.push('\n');
        out.push_str(&format!(
            "[provider.providers.\"{}\"]\n",
            escape_toml_key(name)
        ));
        out.push_str(&format!(
            "provider_type = \"{}\"\n",
            escape_toml_string(&provider.provider_type)
        ));
        if let Some(base_url) = provider.base_url.as_deref().filter(|v| !v.trim().is_empty()) {
            out.push_str(&format!(
                "base_url = \"{}\"\n",
                escape_toml_string(base_url)
            ));
        }
        if let Some(env) = provider.api_key_env.as_deref().filter(|v| !v.trim().is_empty()) {
            out.push_str(&format!(
                "api_key_env = \"{}\"\n",
                escape_toml_string(env)
            ));
        }
        if let Some(model) = provider
            .default_model
            .as_deref()
            .filter(|v| !v.trim().is_empty())
        {
            out.push_str(&format!(
                "default_model = \"{}\"\n",
                escape_toml_string(model)
            ));
        }
    }

    out
}

fn escape_toml_key(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\"', "\\\"")
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\"', "\\\"")
}

async fn apply_snippet(target_path: &Path, snippet: &str) -> Result<(), String> {
    if let Some(parent) = target_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }

    let mut existing = String::new();
    if target_path.exists() {
        existing = tokio::fs::read_to_string(target_path)
            .await
            .unwrap_or_default();
    }

    let mut merged = existing;
    if !merged.ends_with('\n') {
        merged.push('\n');
    }
    merged.push_str(snippet);

    tokio::fs::write(target_path, merged)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn existing_provider_names(path: &Path) -> HashSet<String> {
    let Ok(contents) = tokio::fs::read_to_string(path).await else {
        return HashSet::new();
    };
    let Ok(partial) = mc_config::loader::parse_partial_app_config(&contents) else {
        return HashSet::new();
    };
    let Some(provider) = partial.provider else {
        return HashSet::new();
    };
    provider
        .providers
        .unwrap_or_default()
        .keys()
        .cloned()
        .collect()
}

fn prompt_indices(prompt: &str, max: usize) -> Result<Vec<usize>, String> {
    let input = prompt_line(prompt)?;
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") {
        return Ok((0..max).collect());
    }
    if trimmed.eq_ignore_ascii_case("none") {
        return Ok(Vec::new());
    }

    let mut indices = Vec::new();
    for part in trimmed.split(',') {
        let value = part.trim();
        if value.is_empty() {
            continue;
        }
        let parsed: usize = value
            .parse()
            .map_err(|_| format!("无法解析选择项：{value}"))?;
        if parsed == 0 || parsed > max {
            return Err(format!("选择项超出范围：{parsed}"));
        }
        indices.push(parsed - 1);
    }
    indices.sort_unstable();
    indices.dedup();
    Ok(indices)
}

fn prompt_target() -> Result<WriteTarget, String> {
    println!("写入目标：");
    println!("  [1] 用户级配置（推荐，不进入仓库）");
    println!("  [2] 项目级配置（写入 .morecode/config.toml）");
    println!("  [3] 仅生成导入文件（.morecode/othercli-import.toml）");

    let input = prompt_line("选择 1/2/3（默认 1）: ")?;
    match input.trim() {
        "" | "1" => Ok(WriteTarget::User),
        "2" => Ok(WriteTarget::Project),
        "3" => Ok(WriteTarget::GenerateOnly),
        other => Err(format!("未知选择：{other}")),
    }
}

fn prompt_confirm(prompt: &str) -> Result<bool, String> {
    let input = prompt_line(prompt)?;
    Ok(matches!(input.trim(), "y" | "Y" | "yes" | "YES"))
}

fn prompt_line(prompt: &str) -> Result<String, String> {
    print!("{prompt}");
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .map_err(|error| error.to_string())?;
    Ok(buf)
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_key_parser_extracts_keys() {
        let keys = extract_env_keys(
            r#"
            # comment
            OPENAI_API_KEY=abc
            ANTHROPIC_API_KEY = def
            INVALID
            "#,
        );
        assert!(keys.contains(&"OPENAI_API_KEY".to_string()));
        assert!(keys.contains(&"ANTHROPIC_API_KEY".to_string()));
        assert!(!keys.contains(&"INVALID".to_string()));
    }

    #[test]
    fn snippet_quotes_provider_name() {
        let snippet = render_provider_append_snippet(&[(
            "imported-a".into(),
            DetectedProvider {
                source: DetectedSource {
                    kind: SourceKind::Env,
                    label: "env".into(),
                },
                product: "codex".into(),
                provider_type: "openai-compat".into(),
                api_key_present: true,
                api_key_env: Some("OPENAI_API_KEY".into()),
                base_url: None,
                default_model: None,
            },
        )]);
        assert!(snippet.contains("[provider.providers.\"imported-a\"]"));
        assert!(snippet.contains("api_key_env = \"OPENAI_API_KEY\""));
        assert!(!snippet.contains("api_key ="));
    }
}
