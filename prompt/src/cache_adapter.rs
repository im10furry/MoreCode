use mc_llm::{CacheCapability, ModelInfo, TokenUsage};

use crate::layer::PromptLayer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheBreakpointPlan {
    pub cacheable_layers: Vec<PromptLayer>,
    pub last_cacheable_layer: Option<PromptLayer>,
}

pub fn should_set_cache_breakpoint<T>(turn_messages: &[T]) -> bool {
    turn_messages.len() >= 3
}

pub fn calculate_cache_savings(
    usage: &TokenUsage,
    model_info: &ModelInfo,
    cache_ttl_secs: u64,
    estimated_requests_per_ttl: u32,
) -> (u64, u64) {
    if cache_ttl_secs == 0 || estimated_requests_per_ttl <= 1 || usage.prompt_tokens == 0 {
        return (0, 0);
    }

    let hit_rate = usage.cache_hit_rate().clamp(0.0, 1.0);
    if hit_rate == 0.0 {
        return (0, 0);
    }

    let ttl_baseline = PromptLayer::Global
        .default_ttl_secs()
        .unwrap_or(cache_ttl_secs) as f64;
    let ttl_factor = (cache_ttl_secs as f64 / ttl_baseline).clamp(0.0, 1.0);
    let reusable_requests = estimated_requests_per_ttl.saturating_sub(1) as f64;
    let effective_hits = reusable_requests * hit_rate * ttl_factor;
    let saved_tokens = (usage.prompt_tokens as f64 * effective_hits).round() as u64;

    let read_discount =
        (model_info.input_price_per_1k - model_info.cache_read_price_per_1k).max(0.0);
    let saved_cost_cents = ((saved_tokens as f64 / 1000.0) * read_discount * 100.0).round() as u64;

    (saved_tokens, saved_cost_cents)
}

pub fn plan_cacheable_layers(
    cache_capability: &CacheCapability,
    layer_token_estimates: &[(PromptLayer, u32)],
) -> CacheBreakpointPlan {
    let mut cacheable_layers = Vec::new();
    let mut accumulated = 0u32;

    for &(layer, tokens) in layer_token_estimates {
        accumulated = accumulated.saturating_add(tokens);
        if layer.should_cache()
            && cache_capability.supports_prompt_caching
            && accumulated >= cache_capability.min_cacheable_tokens
        {
            cacheable_layers.push(layer);
        }
    }

    CacheBreakpointPlan {
        last_cacheable_layer: cacheable_layers.last().copied(),
        cacheable_layers,
    }
}

#[cfg(test)]
mod tests {
    use mc_llm::{CacheCapability, CacheStrategy, ModelInfo, TokenUsage};

    use crate::layer::PromptLayer;

    use super::{calculate_cache_savings, plan_cacheable_layers, should_set_cache_breakpoint};

    #[test]
    fn cache_breakpoint_depends_on_turn_count() {
        assert!(!should_set_cache_breakpoint(&[1, 2]));
        assert!(should_set_cache_breakpoint(&[1, 2, 3]));
    }

    #[test]
    fn cache_savings_respects_hit_rate() {
        let usage = TokenUsage {
            prompt_tokens: 1_000,
            completion_tokens: 100,
            cached_tokens: 500,
            total_tokens: 1_100,
        };
        let mut model = ModelInfo::new("model", "Model", "provider");
        model.input_price_per_1k = 1.0;
        model.cache_read_price_per_1k = 0.1;

        let (tokens, cents) = calculate_cache_savings(&usage, &model, 3600, 4);
        assert!(tokens > 0);
        assert!(cents > 0);
    }

    #[test]
    fn cacheable_layers_follow_capability_threshold() {
        let plan = plan_cacheable_layers(
            &CacheCapability {
                supports_prompt_caching: true,
                max_cache_ttl_secs: Some(3600),
                min_cacheable_tokens: 1_024,
                supported_control_types: Vec::new(),
                strategy: CacheStrategy::None,
            },
            &[
                (PromptLayer::Global, 800),
                (PromptLayer::Organization, 400),
                (PromptLayer::Project, 300),
                (PromptLayer::Session, 200),
            ],
        );

        assert_eq!(
            plan.cacheable_layers,
            vec![PromptLayer::Organization, PromptLayer::Project]
        );
        assert_eq!(plan.last_cacheable_layer, Some(PromptLayer::Project));
    }
}
