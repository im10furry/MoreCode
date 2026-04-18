use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// Strategy applied to child-agent output before the reduce phase.
#[derive(Debug, Clone)]
pub enum FilterStrategy {
    LlmFiltered {
        filter_prompt: String,
        max_retention_ratio: f64,
    },
    RuleBased {
        keep_rules: Vec<FilterRule>,
        discard_rules: Vec<FilterRule>,
    },
    Summarize {
        max_summary_tokens: usize,
    },
    KeepAll,
}

/// Single rule used by the rule-based filter strategy.
#[derive(Debug, Clone)]
pub struct FilterRule {
    pub pattern: String,
    pub rule_type: FilterRuleType,
    pub priority: FilterRulePriority,
}

/// Matching strategy for a filter rule.
#[derive(Debug, Clone)]
pub enum FilterRuleType {
    Regex(String),
    Contains(String),
    StartsWith(String),
    Custom(fn(&str) -> bool),
}

/// Priority assigned to a filter rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterRulePriority {
    Keep,
    Discard,
    Compress,
}

/// Result produced after filtering a child-agent output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilteredResult {
    pub retained: Vec<String>,
    pub discarded_count: usize,
    pub compressed: Vec<CompressedEntry>,
    pub original_tokens: usize,
    pub filtered_tokens: usize,
    /// Retained tokens divided by original tokens. `1.0` means no compression.
    pub compression_ratio: f64,
}

/// Record describing one compressed line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompressedEntry {
    pub original_summary: String,
    pub compressed_to: String,
    pub original_tokens: usize,
    pub compressed_tokens: usize,
}

/// Shared async regex cache used by the rule matcher.
#[derive(Debug, Clone, Default)]
pub struct RegexCache {
    cache: Arc<Mutex<HashMap<String, Regex>>>,
}

impl RegexCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up or compile a regex.
    pub async fn get_or_compile(&self, pattern: &str) -> Result<Regex> {
        let mut cache = self.cache.lock().await;
        if let Some(regex) = cache.get(pattern) {
            return Ok(regex.clone());
        }
        let regex = Regex::new(pattern).with_context(|| format!("无效的正则表达式: {pattern}"))?;
        cache.insert(pattern.to_string(), regex.clone());
        Ok(regex)
    }
}

/// Check whether a line matches a rule.
pub async fn matches_rule(text: &str, rule: &FilterRule, regex_cache: &RegexCache) -> bool {
    match &rule.rule_type {
        FilterRuleType::Regex(pattern) => regex_cache
            .get_or_compile(pattern)
            .await
            .map(|regex| regex.is_match(text))
            .unwrap_or(false),
        FilterRuleType::Contains(pattern) => text.contains(pattern),
        FilterRuleType::StartsWith(pattern) => text.starts_with(pattern),
        FilterRuleType::Custom(function) => function(text),
    }
}

/// Applies the configured filter strategy to raw child-agent output.
#[derive(Debug, Clone)]
pub struct FilterEngine {
    regex_cache: RegexCache,
}

impl FilterEngine {
    pub fn new(regex_cache: RegexCache) -> Self {
        Self { regex_cache }
    }

    /// Filter raw output according to the configured strategy.
    pub async fn apply(
        &self,
        strategy: &FilterStrategy,
        raw_output: &str,
    ) -> Result<FilteredResult> {
        match strategy {
            FilterStrategy::KeepAll => Ok(keep_all(raw_output)),
            FilterStrategy::RuleBased {
                keep_rules,
                discard_rules,
            } => self.rule_based(raw_output, keep_rules, discard_rules).await,
            FilterStrategy::Summarize { max_summary_tokens } => {
                Ok(summarize(raw_output, *max_summary_tokens))
            }
            FilterStrategy::LlmFiltered {
                filter_prompt: _,
                max_retention_ratio,
            } => Ok(llm_filtered(raw_output, *max_retention_ratio)),
        }
    }

    async fn rule_based(
        &self,
        raw_output: &str,
        keep_rules: &[FilterRule],
        discard_rules: &[FilterRule],
    ) -> Result<FilteredResult> {
        let mut retained = Vec::new();
        let mut compressed = Vec::new();
        let mut discarded_count = 0usize;
        let has_explicit_keep_rules = keep_rules
            .iter()
            .any(|rule| rule.priority == FilterRulePriority::Keep);

        for line in raw_output
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            let mut discarded = false;
            for rule in discard_rules {
                if matches_rule(line, rule, &self.regex_cache).await {
                    discarded = true;
                    discarded_count += 1;
                    break;
                }
            }
            if discarded {
                continue;
            }

            let mut compressed_line = None;
            for rule in keep_rules
                .iter()
                .filter(|rule| rule.priority == FilterRulePriority::Compress)
            {
                if matches_rule(line, rule, &self.regex_cache).await {
                    let compressed_to = compress_line(line);
                    compressed.push(CompressedEntry {
                        original_summary: line.to_string(),
                        compressed_to: compressed_to.clone(),
                        original_tokens: estimate_tokens(line),
                        compressed_tokens: estimate_tokens(&compressed_to),
                    });
                    compressed_line = Some(compressed_to);
                    break;
                }
            }
            if let Some(compressed_to) = compressed_line {
                retained.push(compressed_to);
                continue;
            }

            let mut keep = !has_explicit_keep_rules;
            for rule in keep_rules
                .iter()
                .filter(|rule| rule.priority == FilterRulePriority::Keep)
            {
                if matches_rule(line, rule, &self.regex_cache).await {
                    keep = true;
                    break;
                }
            }

            if keep {
                retained.push(line.to_string());
            } else {
                discarded_count += 1;
            }
        }

        Ok(build_filtered_result(
            raw_output,
            retained,
            discarded_count,
            compressed,
        ))
    }
}

/// Estimate the approximate token count for a text blob.
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }

    let char_count = text.chars().count();
    let cjk_count = text.chars().filter(|ch| *ch >= '\u{2E80}').count();
    let other_count = char_count.saturating_sub(cjk_count);

    cjk_count + other_count.div_ceil(4)
}

fn keep_all(raw_output: &str) -> FilteredResult {
    let original_tokens = estimate_tokens(raw_output);
    FilteredResult {
        retained: vec![raw_output.to_string()],
        discarded_count: 0,
        compressed: Vec::new(),
        original_tokens,
        filtered_tokens: original_tokens,
        compression_ratio: 1.0,
    }
}

fn summarize(raw_output: &str, max_summary_tokens: usize) -> FilteredResult {
    let mut retained = Vec::new();
    let mut used_tokens = 0usize;
    let mut discarded_count = 0usize;

    for line in raw_output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let line_tokens = estimate_tokens(line);
        if used_tokens + line_tokens > max_summary_tokens {
            discarded_count += 1;
            continue;
        }
        retained.push(line.to_string());
        used_tokens += line_tokens;
    }

    build_filtered_result(raw_output, retained, discarded_count, Vec::new())
}

fn llm_filtered(raw_output: &str, max_retention_ratio: f64) -> FilteredResult {
    let lines: Vec<_> = raw_output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let clamped_ratio = max_retention_ratio.clamp(0.0, 1.0);
    let keep_count = if lines.is_empty() {
        0
    } else {
        ((lines.len() as f64) * clamped_ratio).ceil() as usize
    };
    let retained = lines
        .into_iter()
        .take(keep_count.max(1))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let discarded_count = raw_output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .count()
        .saturating_sub(retained.len());

    build_filtered_result(raw_output, retained, discarded_count, Vec::new())
}

fn build_filtered_result(
    raw_output: &str,
    retained: Vec<String>,
    discarded_count: usize,
    compressed: Vec<CompressedEntry>,
) -> FilteredResult {
    let original_tokens = estimate_tokens(raw_output);
    let filtered_tokens = estimate_tokens(&retained.join("\n"));
    let compression_ratio = if original_tokens == 0 {
        1.0
    } else {
        filtered_tokens as f64 / original_tokens as f64
    };

    FilteredResult {
        retained,
        discarded_count,
        compressed,
        original_tokens,
        filtered_tokens,
        compression_ratio,
    }
}

fn compress_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.starts_with("use ") {
        return "[import]".to_string();
    }
    if trimmed.contains("#[derive(") {
        return "[derive]".to_string();
    }
    if trimmed.len() > 100 {
        let mut truncated = trimmed.chars().take(97).collect::<String>();
        truncated.push_str("...");
        truncated
    } else {
        trimmed.to_string()
    }
}

/// Default filter strategy tuned for code-reading and structural scanning.
pub fn code_reading_filter_strategy() -> FilterStrategy {
    FilterStrategy::RuleBased {
        keep_rules: vec![
            FilterRule {
                pattern: "pub item".to_string(),
                rule_type: FilterRuleType::Regex(
                    r"pub (struct|enum|trait|type|fn|const|static)".to_string(),
                ),
                priority: FilterRulePriority::Keep,
            },
            FilterRule {
                pattern: "TODO/FIXME".to_string(),
                rule_type: FilterRuleType::Regex(r"TODO|FIXME|SAFETY|UNSAFE|HACK".to_string()),
                priority: FilterRulePriority::Keep,
            },
            FilterRule {
                pattern: "long line".to_string(),
                rule_type: FilterRuleType::Custom(|line| line.len() > 100),
                priority: FilterRulePriority::Compress,
            },
        ],
        discard_rules: vec![
            FilterRule {
                pattern: "use ".to_string(),
                rule_type: FilterRuleType::StartsWith("use ".to_string()),
                priority: FilterRulePriority::Discard,
            },
            FilterRule {
                pattern: "#[cfg(test)]".to_string(),
                rule_type: FilterRuleType::Contains("#[cfg(test)]".to_string()),
                priority: FilterRulePriority::Discard,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::{
        code_reading_filter_strategy, matches_rule, FilterEngine, FilterRule, FilterRulePriority,
        FilterRuleType, FilterStrategy, RegexCache,
    };

    #[tokio::test]
    async fn matches_regex_contains_starts_with_and_custom_rules() -> Result<()> {
        let cache = RegexCache::new();
        assert!(
            matches_rule(
                "pub struct AuthConfig",
                &FilterRule {
                    pattern: "pub".to_string(),
                    rule_type: FilterRuleType::Regex(r"pub struct".to_string()),
                    priority: FilterRulePriority::Keep,
                },
                &cache
            )
            .await
        );
        assert!(
            matches_rule(
                "contains issue",
                &FilterRule {
                    pattern: "issue".to_string(),
                    rule_type: FilterRuleType::Contains("issue".to_string()),
                    priority: FilterRulePriority::Keep,
                },
                &cache
            )
            .await
        );
        assert!(
            matches_rule(
                "use std::fmt",
                &FilterRule {
                    pattern: "use".to_string(),
                    rule_type: FilterRuleType::StartsWith("use ".to_string()),
                    priority: FilterRulePriority::Discard,
                },
                &cache
            )
            .await
        );
        assert!(
            matches_rule(
                "",
                &FilterRule {
                    pattern: "empty".to_string(),
                    rule_type: FilterRuleType::Custom(|line| line.is_empty()),
                    priority: FilterRulePriority::Discard,
                },
                &cache
            )
            .await
        );
        Ok(())
    }

    #[tokio::test]
    async fn keep_all_retains_the_original_output() -> Result<()> {
        let engine = FilterEngine::new(RegexCache::new());
        let filtered = engine
            .apply(&FilterStrategy::KeepAll, "发现: issue\n风险: auth")
            .await?;
        assert_eq!(
            filtered.retained,
            vec!["发现: issue\n风险: auth".to_string()]
        );
        Ok(())
    }

    #[tokio::test]
    async fn rule_based_filter_keeps_relevant_lines() -> Result<()> {
        let engine = FilterEngine::new(RegexCache::new());
        let filtered = engine
            .apply(
                &code_reading_filter_strategy(),
                "use std::fmt;\npub struct Auth;\nTODO: tighten validation",
            )
            .await?;
        assert!(filtered
            .retained
            .iter()
            .any(|line| line.contains("pub struct Auth")));
        assert!(filtered.retained.iter().any(|line| line.contains("TODO")));
        assert!(!filtered
            .retained
            .iter()
            .any(|line| line.starts_with("use ")));
        Ok(())
    }
}
