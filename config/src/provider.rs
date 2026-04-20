use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    #[serde(default = "default_provider")]
    pub default_provider: String,
    #[serde(default)]
    pub providers: HashMap<String, ProviderEntry>,
    #[serde(default)]
    pub semantic_cache_enabled: bool,
    #[serde(default = "default_cache_threshold")]
    pub semantic_cache_threshold: f32,
    #[serde(default)]
    pub precise_token_count: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            default_provider: default_provider(),
            providers: HashMap::new(),
            semantic_cache_enabled: false,
            semantic_cache_threshold: default_cache_threshold(),
            precise_token_count: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderEntry {
    #[serde(default = "default_provider")]
    pub provider_type: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl Default for ProviderEntry {
    fn default() -> Self {
        Self {
            provider_type: default_provider(),
            base_url: None,
            api_key: None,
            api_key_env: None,
            default_model: None,
            headers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum BuiltinProviderPreset {
    OpenAi,
    DeepSeek,
    Zhipu,
    Tongyi,
    Moonshot,
    Ollama,
    Anthropic,
    Google,
    Mock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProviderEntry {
    pub name: String,
    pub provider_type: String,
    pub preset: Option<BuiltinProviderPreset>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub api_key_env: Option<String>,
    pub default_model: Option<String>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialProviderConfig {
    pub default_provider: Option<String>,
    pub providers: Option<HashMap<String, PartialProviderEntry>>,
    pub semantic_cache_enabled: Option<bool>,
    pub semantic_cache_threshold: Option<f32>,
    pub precise_token_count: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialProviderEntry {
    pub provider_type: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub api_key_env: Option<String>,
    pub default_model: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BuiltinProviderDefinition {
    preset: BuiltinProviderPreset,
    implementation_type: &'static str,
    base_url: Option<&'static str>,
    default_model: Option<&'static str>,
    api_key_env: Option<&'static str>,
}

impl ProviderConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialProviderConfig) {
        if let Some(value) = partial.default_provider {
            self.default_provider = value;
        }
        if let Some(value) = partial.providers {
            for (name, provider) in value {
                let entry = self.providers.entry(name).or_default();
                entry.apply_partial(provider);
            }
        }
        if let Some(value) = partial.semantic_cache_enabled {
            self.semantic_cache_enabled = value;
        }
        if let Some(value) = partial.semantic_cache_threshold {
            self.semantic_cache_threshold = value;
        }
        if let Some(value) = partial.precise_token_count {
            self.precise_token_count = value;
        }
    }

    pub fn provider_names(&self) -> Vec<&str> {
        let mut names = self
            .providers
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        names.sort_unstable();
        names
    }

    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name) || BuiltinProviderPreset::from_key(name).is_some()
    }

    pub fn with_builtin_presets_applied(&self) -> Self {
        let mut resolved = self.clone();

        for (name, entry) in &self.providers {
            let normalized_name = normalize_provider_key(name);
            let preset = entry
                .builtin_preset(Some(&normalized_name))
                .or_else(|| BuiltinProviderPreset::from_key(&normalized_name));
            if let Some(preset) = preset {
                resolved
                    .providers
                    .insert(name.clone(), entry.with_builtin_preset(preset));
            }
        }

        if !resolved.providers.contains_key(&resolved.default_provider) {
            if let Some(preset) = BuiltinProviderPreset::from_key(&resolved.default_provider) {
                resolved.providers.insert(
                    resolved.default_provider.clone(),
                    ProviderEntry::from_builtin_preset(preset),
                );
            }
        }

        resolved
    }

    pub fn resolve_provider(&self, name: &str) -> Option<ResolvedProviderEntry> {
        if let Some(entry) = self.providers.get(name) {
            return Some(entry.resolve(name));
        }

        let preset = BuiltinProviderPreset::from_key(name)?;
        Some(ProviderEntry::from_builtin_preset(preset).resolve(name))
    }

    pub fn resolve_default_provider(&self) -> Option<ResolvedProviderEntry> {
        self.resolve_provider(&self.default_provider)
    }
}

impl ProviderEntry {
    pub(crate) fn apply_partial(&mut self, partial: PartialProviderEntry) {
        if let Some(value) = partial.provider_type {
            self.provider_type = value;
        }
        if let Some(value) = partial.base_url {
            self.base_url = Some(value);
        }
        if let Some(value) = partial.api_key {
            self.api_key = Some(value);
        }
        if let Some(value) = partial.api_key_env {
            self.api_key_env = Some(value);
        }
        if let Some(value) = partial.default_model {
            self.default_model = Some(value);
        }
        if let Some(value) = partial.headers {
            for (header, header_value) in value {
                self.headers.insert(header, header_value);
            }
        }
    }

    pub fn from_builtin_preset(preset: BuiltinProviderPreset) -> Self {
        let definition = preset.definition();
        Self {
            provider_type: definition.implementation_type.to_string(),
            base_url: definition.base_url.map(ToOwned::to_owned),
            api_key: None,
            api_key_env: definition.api_key_env.map(ToOwned::to_owned),
            default_model: definition.default_model.map(ToOwned::to_owned),
            headers: HashMap::new(),
        }
    }

    pub fn builtin_preset(
        &self,
        provider_name_hint: Option<&str>,
    ) -> Option<BuiltinProviderPreset> {
        BuiltinProviderPreset::from_key(&self.provider_type)
            .or_else(|| provider_name_hint.and_then(BuiltinProviderPreset::from_key))
    }

    pub fn normalized_provider_type(&self) -> String {
        self.builtin_preset(None)
            .map(|preset| preset.definition().implementation_type.to_string())
            .unwrap_or_else(|| normalize_provider_key(&self.provider_type))
    }

    pub fn resolved_api_key(&self) -> Option<String> {
        self.api_key.clone().or_else(|| {
            self.api_key_env
                .as_deref()
                .and_then(|env_name| std::env::var(env_name).ok())
                .filter(|value| !value.trim().is_empty())
        })
    }

    pub fn with_builtin_preset(&self, preset: BuiltinProviderPreset) -> Self {
        let definition = preset.definition();
        let mut merged = self.clone();
        merged.provider_type = definition.implementation_type.to_string();
        if merged.base_url.is_none() {
            merged.base_url = definition.base_url.map(ToOwned::to_owned);
        }
        if merged.default_model.is_none() {
            merged.default_model = definition.default_model.map(ToOwned::to_owned);
        }
        if merged.api_key.is_none() && merged.api_key_env.is_none() {
            merged.api_key_env = definition.api_key_env.map(ToOwned::to_owned);
        }
        merged
    }

    pub fn resolve(&self, provider_name: impl Into<String>) -> ResolvedProviderEntry {
        let provider_name = provider_name.into();
        let preset = self.builtin_preset(Some(&provider_name));
        let effective = preset
            .map(|value| self.with_builtin_preset(value))
            .unwrap_or_else(|| self.clone());
        let api_key = effective.resolved_api_key();

        ResolvedProviderEntry {
            name: provider_name,
            provider_type: effective.normalized_provider_type(),
            preset,
            base_url: effective.base_url,
            api_key,
            api_key_env: effective.api_key_env,
            default_model: effective.default_model,
            headers: effective.headers,
        }
    }
}

impl PartialProviderConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            default_provider: other.default_provider.or(self.default_provider),
            providers: merge_provider_maps(self.providers, other.providers),
            semantic_cache_enabled: other.semantic_cache_enabled.or(self.semantic_cache_enabled),
            semantic_cache_threshold: other
                .semantic_cache_threshold
                .or(self.semantic_cache_threshold),
            precise_token_count: other.precise_token_count.or(self.precise_token_count),
        }
    }
}

impl PartialProviderEntry {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            provider_type: other.provider_type.or(self.provider_type),
            base_url: other.base_url.or(self.base_url),
            api_key: other.api_key.or(self.api_key),
            api_key_env: other.api_key_env.or(self.api_key_env),
            default_model: other.default_model.or(self.default_model),
            headers: match (self.headers, other.headers) {
                (Some(mut base), Some(overlay)) => {
                    for (name, value) in overlay {
                        base.insert(name, value);
                    }
                    Some(base)
                }
                (Some(base), None) => Some(base),
                (None, Some(overlay)) => Some(overlay),
                (None, None) => None,
            },
        }
    }
}

impl BuiltinProviderPreset {
    pub fn from_key(input: &str) -> Option<Self> {
        match normalize_provider_key(input).as_str() {
            "openai" | "openai-compat" => Some(Self::OpenAi),
            "deepseek" => Some(Self::DeepSeek),
            "zhipu" | "glm" => Some(Self::Zhipu),
            "tongyi" | "qwen" => Some(Self::Tongyi),
            "moonshot" | "kimi" => Some(Self::Moonshot),
            "ollama" => Some(Self::Ollama),
            "anthropic" | "claude" => Some(Self::Anthropic),
            "google" | "gemini" => Some(Self::Google),
            "mock" => Some(Self::Mock),
            _ => None,
        }
    }

    pub(crate) fn definition(self) -> BuiltinProviderDefinition {
        match self {
            Self::OpenAi => BuiltinProviderDefinition {
                preset: self,
                implementation_type: "openai-compat",
                base_url: Some("https://api.openai.com/v1"),
                default_model: Some("gpt-4o-mini"),
                api_key_env: Some("OPENAI_API_KEY"),
            },
            Self::DeepSeek => BuiltinProviderDefinition {
                preset: self,
                implementation_type: "openai-compat",
                base_url: Some("https://api.deepseek.com/v1"),
                default_model: Some("deepseek-chat"),
                api_key_env: Some("DEEPSEEK_API_KEY"),
            },
            Self::Zhipu => BuiltinProviderDefinition {
                preset: self,
                implementation_type: "openai-compat",
                base_url: Some("https://open.bigmodel.cn/api/paas/v4"),
                default_model: Some("glm-4.5"),
                api_key_env: Some("ZHIPU_API_KEY"),
            },
            Self::Tongyi => BuiltinProviderDefinition {
                preset: self,
                implementation_type: "openai-compat",
                base_url: Some("https://dashscope.aliyuncs.com/compatible-mode/v1"),
                default_model: Some("qwen-plus"),
                api_key_env: Some("TONGYI_API_KEY"),
            },
            Self::Moonshot => BuiltinProviderDefinition {
                preset: self,
                implementation_type: "openai-compat",
                base_url: Some("https://api.moonshot.cn/v1"),
                default_model: Some("moonshot-v1-8k"),
                api_key_env: Some("MOONSHOT_API_KEY"),
            },
            Self::Ollama => BuiltinProviderDefinition {
                preset: self,
                implementation_type: "openai-compat",
                base_url: Some("http://localhost:11434/v1"),
                default_model: Some("qwen2.5-coder:7b"),
                api_key_env: None,
            },
            Self::Anthropic => BuiltinProviderDefinition {
                preset: self,
                implementation_type: "anthropic",
                base_url: Some("https://api.anthropic.com/v1"),
                default_model: Some("claude-sonnet-4-20250514"),
                api_key_env: Some("ANTHROPIC_API_KEY"),
            },
            Self::Google => BuiltinProviderDefinition {
                preset: self,
                implementation_type: "google",
                base_url: Some("https://generativelanguage.googleapis.com/v1beta"),
                default_model: Some("gemini-2.5-flash"),
                api_key_env: Some("GOOGLE_API_KEY"),
            },
            Self::Mock => BuiltinProviderDefinition {
                preset: self,
                implementation_type: "mock",
                base_url: None,
                default_model: Some("mock-model"),
                api_key_env: None,
            },
        }
    }
}

fn merge_provider_maps(
    base: Option<HashMap<String, PartialProviderEntry>>,
    overlay: Option<HashMap<String, PartialProviderEntry>>,
) -> Option<HashMap<String, PartialProviderEntry>> {
    match (base, overlay) {
        (Some(mut base), Some(overlay)) => {
            for (name, provider) in overlay {
                match base.remove(&name) {
                    Some(existing) => {
                        base.insert(name, existing.merge(provider));
                    }
                    None => {
                        base.insert(name, provider);
                    }
                }
            }
            Some(base)
        }
        (Some(base), None) => Some(base),
        (None, Some(overlay)) => Some(overlay),
        (None, None) => None,
    }
}

fn normalize_provider_key(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('_', "-")
}

fn default_provider() -> String {
    "openai-compat".to_string()
}

fn default_cache_threshold() -> f32 {
    0.90
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        BuiltinProviderPreset, PartialProviderConfig, PartialProviderEntry, ProviderConfig,
        ProviderEntry,
    };

    #[test]
    fn builtin_preset_fills_missing_fields() {
        let entry = ProviderEntry {
            provider_type: "deepseek".into(),
            ..ProviderEntry::default()
        };

        let resolved = entry.resolve("primary");
        assert_eq!(resolved.provider_type, "openai-compat");
        assert_eq!(
            resolved.base_url.as_deref(),
            Some("https://api.deepseek.com/v1")
        );
        assert_eq!(resolved.default_model.as_deref(), Some("deepseek-chat"));
        assert_eq!(resolved.api_key_env.as_deref(), Some("DEEPSEEK_API_KEY"));
    }

    #[test]
    fn explicit_values_override_builtin_preset_defaults() {
        let entry = ProviderEntry {
            provider_type: "google".into(),
            base_url: Some("https://proxy.example/v1beta".into()),
            default_model: Some("gemini-custom".into()),
            api_key_env: Some("CUSTOM_GOOGLE_API_KEY".into()),
            headers: HashMap::from([("X-Team".into(), "core".into())]),
            ..ProviderEntry::default()
        };

        let resolved = entry.resolve("google");
        assert_eq!(resolved.provider_type, "google");
        assert_eq!(
            resolved.base_url.as_deref(),
            Some("https://proxy.example/v1beta")
        );
        assert_eq!(resolved.default_model.as_deref(), Some("gemini-custom"));
        assert_eq!(
            resolved.api_key_env.as_deref(),
            Some("CUSTOM_GOOGLE_API_KEY")
        );
        assert_eq!(
            resolved.headers.get("X-Team").map(String::as_str),
            Some("core")
        );
    }

    #[test]
    fn resolved_api_key_prefers_explicit_value_then_env() {
        let env_name = "MORECODE_TEST_PROVIDER_KEY";
        std::env::set_var(env_name, "env-secret");

        let from_env = ProviderEntry {
            api_key_env: Some(env_name.into()),
            ..ProviderEntry::default()
        };
        assert_eq!(from_env.resolved_api_key().as_deref(), Some("env-secret"));

        let explicit = ProviderEntry {
            api_key: Some("literal-secret".into()),
            api_key_env: Some(env_name.into()),
            ..ProviderEntry::default()
        };
        assert_eq!(
            explicit.resolved_api_key().as_deref(),
            Some("literal-secret")
        );

        std::env::remove_var(env_name);
    }

    #[test]
    fn config_can_resolve_default_provider_from_builtin_alias() {
        let config = ProviderConfig {
            default_provider: "anthropic".into(),
            ..ProviderConfig::default()
        };

        let resolved = config.resolve_default_provider().unwrap();
        assert_eq!(resolved.preset, Some(BuiltinProviderPreset::Anthropic));
        assert_eq!(resolved.provider_type, "anthropic");
        assert_eq!(
            resolved.default_model.as_deref(),
            Some("claude-sonnet-4-20250514")
        );
    }

    #[test]
    fn applying_builtin_presets_inserts_default_alias_entry() {
        let config = ProviderConfig {
            default_provider: "ollama".into(),
            providers: HashMap::new(),
            ..ProviderConfig::default()
        };

        let resolved = config.with_builtin_presets_applied();
        assert!(resolved.providers.contains_key("ollama"));
        assert_eq!(
            resolved.providers["ollama"].base_url.as_deref(),
            Some("http://localhost:11434/v1")
        );
    }

    #[test]
    fn partial_provider_merge_keeps_overlay_priority() {
        let merged = PartialProviderConfig {
            default_provider: Some("base".into()),
            providers: Some(HashMap::from([(
                "primary".into(),
                PartialProviderEntry {
                    provider_type: Some("openai".into()),
                    base_url: Some("https://a.example".into()),
                    api_key: None,
                    api_key_env: Some("BASE_KEY".into()),
                    default_model: None,
                    headers: Some(HashMap::from([("X-Base".into(), "1".into())])),
                },
            )])),
            semantic_cache_enabled: Some(false),
            semantic_cache_threshold: Some(0.9),
            precise_token_count: Some(false),
        }
        .merge(PartialProviderConfig {
            default_provider: Some("overlay".into()),
            providers: Some(HashMap::from([(
                "primary".into(),
                PartialProviderEntry {
                    provider_type: Some("deepseek".into()),
                    base_url: None,
                    api_key: Some("secret".into()),
                    api_key_env: None,
                    default_model: Some("deepseek-chat".into()),
                    headers: Some(HashMap::from([("X-Overlay".into(), "2".into())])),
                },
            )])),
            semantic_cache_enabled: Some(true),
            semantic_cache_threshold: Some(0.95),
            precise_token_count: Some(true),
        });

        assert_eq!(merged.default_provider.as_deref(), Some("overlay"));
        let provider = &merged.providers.unwrap()["primary"];
        assert_eq!(provider.provider_type.as_deref(), Some("deepseek"));
        assert_eq!(provider.base_url.as_deref(), Some("https://a.example"));
        assert_eq!(provider.api_key.as_deref(), Some("secret"));
        assert_eq!(provider.api_key_env.as_deref(), Some("BASE_KEY"));
        assert_eq!(provider.default_model.as_deref(), Some("deepseek-chat"));
        let headers = provider.headers.as_ref().unwrap();
        assert_eq!(headers.get("X-Base").map(String::as_str), Some("1"));
        assert_eq!(headers.get("X-Overlay").map(String::as_str), Some("2"));
    }
}
