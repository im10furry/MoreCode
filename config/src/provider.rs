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

fn default_provider() -> String {
    "openai-compat".to_string()
}

fn default_cache_threshold() -> f32 {
    0.90
}
