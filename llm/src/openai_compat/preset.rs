use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ModelInfo;

use super::OpenAiProviderConfig;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum OpenAiCompatiblePreset {
    OpenAi,
    DeepSeek,
    Zhipu,
    Tongyi,
    Moonshot,
    Ollama,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiCompatibleProviderPreset {
    pub provider_id: &'static str,
    pub base_url: &'static str,
    pub requires_api_key: bool,
    pub default_headers: HashMap<String, String>,
}

impl OpenAiCompatiblePreset {
    pub fn definition(self) -> OpenAiCompatibleProviderPreset {
        let (provider_id, base_url, requires_api_key) = match self {
            Self::OpenAi => ("openai", "https://api.openai.com/v1", true),
            Self::DeepSeek => ("deepseek", "https://api.deepseek.com/v1", true),
            Self::Zhipu => ("zhipu", "https://open.bigmodel.cn/api/paas/v4", true),
            Self::Tongyi => (
                "tongyi",
                "https://dashscope.aliyuncs.com/compatible-mode/v1",
                true,
            ),
            Self::Moonshot => ("moonshot", "https://api.moonshot.cn/v1", true),
            Self::Ollama => ("ollama", "http://localhost:11434/v1", false),
        };

        OpenAiCompatibleProviderPreset {
            provider_id,
            base_url,
            requires_api_key,
            default_headers: HashMap::new(),
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Self::OpenAi => "gpt-4o-mini",
            Self::DeepSeek => "deepseek-chat",
            Self::Zhipu => "glm-4.5",
            Self::Tongyi => "qwen-plus",
            Self::Moonshot => "moonshot-v1-8k",
            Self::Ollama => "qwen2.5-coder:7b",
        }
    }

    pub fn to_model_info(self) -> ModelInfo {
        let definition = self.definition();
        ModelInfo::new(
            self.default_model(),
            self.default_model(),
            definition.provider_id,
        )
    }

    pub fn to_config(self, api_key: impl Into<String>, model: ModelInfo) -> OpenAiProviderConfig {
        let definition = self.definition();
        OpenAiProviderConfig {
            base_url: definition.base_url.to_string(),
            api_key: api_key.into(),
            model,
            default_headers: definition.default_headers,
            request_timeout: std::time::Duration::from_secs(120),
            stream_buffer_size: 64,
            supports_structured_output: !matches!(self, Self::DeepSeek),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OpenAiCompatiblePreset;

    #[test]
    fn preset_definitions_cover_supported_backends() {
        let deepseek = OpenAiCompatiblePreset::DeepSeek.definition();
        let ollama = OpenAiCompatiblePreset::Ollama.definition();

        assert_eq!(deepseek.base_url, "https://api.deepseek.com/v1");
        assert!(deepseek.requires_api_key);
        assert_eq!(ollama.base_url, "http://localhost:11434/v1");
        assert!(!ollama.requires_api_key);
    }

    #[test]
    fn preset_can_build_model_info() {
        let model = OpenAiCompatiblePreset::Tongyi.to_model_info();
        assert_eq!(model.id, "qwen-plus");
        assert_eq!(model.provider_id, "tongyi");
    }
}
