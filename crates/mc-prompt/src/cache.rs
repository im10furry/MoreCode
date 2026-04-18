use mc_llm::{ModelInfo, TokenUsage};

use crate::layer::PromptLayer;

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
