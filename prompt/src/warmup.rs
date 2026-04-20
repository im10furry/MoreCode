use serde::{Deserialize, Serialize};

use crate::cache_adapter::plan_cacheable_layers;
use crate::layer::PromptLayer;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarmupTarget {
    pub layer: PromptLayer,
    pub estimated_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarmupPlan {
    pub cacheable_layers: Vec<WarmupTarget>,
    pub estimated_requests_per_ttl: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarmupResult {
    pub warmed_layers: Vec<PromptLayer>,
    pub skipped_layers: Vec<PromptLayer>,
}

pub struct CacheWarmupStrategy;

impl CacheWarmupStrategy {
    pub fn build_plan(
        provider: &dyn mc_llm::LlmProvider,
        layer_token_estimates: &[(PromptLayer, u32)],
        estimated_requests_per_ttl: u32,
    ) -> WarmupPlan {
        let plan = plan_cacheable_layers(&provider.cache_capability(), layer_token_estimates);
        let cacheable_layers = layer_token_estimates
            .iter()
            .filter(|(layer, _)| plan.cacheable_layers.contains(layer))
            .map(|(layer, tokens)| WarmupTarget {
                layer: *layer,
                estimated_tokens: *tokens,
            })
            .collect();

        WarmupPlan {
            cacheable_layers,
            estimated_requests_per_ttl,
        }
    }

    pub fn execute_plan(plan: &WarmupPlan) -> WarmupResult {
        WarmupResult {
            warmed_layers: plan
                .cacheable_layers
                .iter()
                .map(|target| target.layer)
                .collect(),
            skipped_layers: PromptLayer::all()
                .into_iter()
                .filter(|layer| {
                    !plan
                        .cacheable_layers
                        .iter()
                        .any(|target| target.layer == *layer)
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;

    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    use mc_llm::{
        CacheCapability, CacheControlType, CacheStrategy, ChatRequest, ChatResponse, LlmError,
        LlmProvider, ModelInfo, StreamEvent,
    };

    use crate::layer::PromptLayer;

    use super::CacheWarmupStrategy;

    struct WarmableProvider {
        model: ModelInfo,
        capability: CacheCapability,
    }

    impl LlmProvider for WarmableProvider {
        fn provider_id(&self) -> &str {
            &self.model.provider_id
        }

        fn model_info(&self) -> &ModelInfo {
            &self.model
        }

        fn chat(
            &self,
            _request: ChatRequest,
            _cancel_token: CancellationToken,
        ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, LlmError>> + Send + '_>> {
            Box::pin(async { Err(LlmError::Internal("unused".into())) })
        }

        fn chat_stream(
            &self,
            _request: ChatRequest,
            _cancel_token: CancellationToken,
        ) -> Pin<Box<dyn Future<Output = Result<mpsc::Receiver<StreamEvent>, LlmError>> + Send + '_>>
        {
            Box::pin(async { Err(LlmError::Internal("unused".into())) })
        }

        fn cache_capability(&self) -> CacheCapability {
            self.capability.clone()
        }

        fn list_models(
            &self,
            _cancel_token: CancellationToken,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<ModelInfo>, LlmError>> + Send + '_>> {
            Box::pin(async { Ok(vec![self.model.clone()]) })
        }

        fn cancel_request(&self, _request_id: &str) -> Result<(), LlmError> {
            Ok(())
        }

        fn estimate_tokens(&self, text: &str) -> usize {
            text.len().div_ceil(4)
        }
    }

    #[test]
    fn builds_plan_from_provider_capability() {
        let provider = WarmableProvider {
            model: ModelInfo::new("model", "Model", "provider"),
            capability: CacheCapability {
                supports_prompt_caching: true,
                max_cache_ttl_secs: Some(3600),
                min_cacheable_tokens: 1024,
                supported_control_types: vec![CacheControlType::CacheBreakpoint],
                strategy: CacheStrategy::None,
            },
        };

        let plan = CacheWarmupStrategy::build_plan(
            &provider,
            &[
                (PromptLayer::Global, 900),
                (PromptLayer::Organization, 200),
                (PromptLayer::Project, 400),
            ],
            5,
        );

        assert_eq!(
            plan.cacheable_layers
                .iter()
                .map(|target| target.layer)
                .collect::<Vec<_>>(),
            vec![PromptLayer::Organization, PromptLayer::Project]
        );
    }

    #[test]
    fn execute_plan_returns_warmed_and_skipped_layers() {
        let plan = super::WarmupPlan {
            cacheable_layers: vec![super::WarmupTarget {
                layer: PromptLayer::Global,
                estimated_tokens: 2_000,
            }],
            estimated_requests_per_ttl: 3,
        };

        let result = CacheWarmupStrategy::execute_plan(&plan);
        assert_eq!(result.warmed_layers, vec![PromptLayer::Global]);
        assert!(result.skipped_layers.contains(&PromptLayer::Turn));
    }
}
