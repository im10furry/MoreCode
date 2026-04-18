use std::collections::HashMap;

use crate::{
    filter::{estimate_tokens, FilteredResult},
    result::{AggregatedResult, Contradiction, ContradictionSeverity},
    sub_agent::{SubAgentResult, SubAgentStatus},
};

/// Pure reducer that combines filtered child-agent output into one aggregate result.
#[derive(Debug, Clone)]
pub struct ResultAggregator {
    max_aggregate_tokens: usize,
}

impl ResultAggregator {
    pub fn new(max_aggregate_tokens: usize) -> Self {
        Self {
            max_aggregate_tokens,
        }
    }

    /// Aggregate filtered child-agent results into a final reduced summary.
    pub fn aggregate(
        &self,
        task: &str,
        filtered_results: &[FilteredResult],
        sub_results: &[SubAgentResult],
        depth: usize,
    ) -> AggregatedResult {
        let mut findings = filtered_results
            .iter()
            .flat_map(|result| result.retained.clone())
            .collect::<Vec<_>>();
        findings = truncate_findings_to_budget(&findings, self.max_aggregate_tokens);

        let contradictions = detect_contradictions(sub_results);
        let confidence_scores = sub_results
            .iter()
            .map(|result| (result.sub_agent_id.clone(), confidence_score(result)))
            .collect::<HashMap<_, _>>();
        let total_tokens_used = sub_results
            .iter()
            .map(|result| result.token_usage.total_tokens as usize)
            .sum();

        AggregatedResult {
            summary: format!(
                "任务“{}”的递归编排完成：{} 个子 Agent，保留 {} 条发现，检测到 {} 个矛盾。",
                task,
                sub_results.len(),
                findings.len(),
                contradictions.len()
            ),
            findings,
            confidence_scores,
            contradictions,
            total_tokens_used,
            sub_agents_used: sub_results.len(),
            depth_reached: depth + 1,
        }
    }
}

/// Detect contradictions across successful child-agent outputs.
pub fn detect_contradictions(results: &[SubAgentResult]) -> Vec<Contradiction> {
    let completed = results
        .iter()
        .filter(|result| result.status == SubAgentStatus::Completed)
        .collect::<Vec<_>>();
    let mut contradictions = Vec::new();

    for index in 0..completed.len() {
        for peer in completed.iter().skip(index + 1) {
            let left = completed[index];
            if let Some(severity) = contradiction_severity(&left.raw_output, &peer.raw_output) {
                contradictions.push(Contradiction {
                    description: format!(
                        "Agent {} 与 Agent {} 的结论存在冲突",
                        left.sub_agent_id, peer.sub_agent_id
                    ),
                    agent_a_id: left.sub_agent_id.clone(),
                    agent_a_conclusion: truncate_text(&left.raw_output, 200),
                    agent_b_id: peer.sub_agent_id.clone(),
                    agent_b_conclusion: truncate_text(&peer.raw_output, 200),
                    severity,
                });
            }
        }
    }

    contradictions
}

fn confidence_score(result: &SubAgentResult) -> f64 {
    if result.status != SubAgentStatus::Completed {
        return 0.0;
    }

    let duration_penalty = (result.duration.as_secs_f64() / 300.0).min(0.3);
    let token_bonus = ((result.token_usage.total_tokens as f64) / 10_000.0).min(0.15);
    (1.0 - duration_penalty + token_bonus).clamp(0.0, 1.0)
}

fn contradiction_severity(left: &str, right: &str) -> Option<ContradictionSeverity> {
    let left_polarity = semantic_polarity(left);
    let right_polarity = semantic_polarity(right);

    match (left_polarity, right_polarity) {
        (Some(SemanticPolarity::Positive), Some(SemanticPolarity::Negative))
        | (Some(SemanticPolarity::Negative), Some(SemanticPolarity::Positive)) => {
            Some(ContradictionSeverity::Error)
        }
        (Some(SemanticPolarity::Negative), Some(SemanticPolarity::Cautious))
        | (Some(SemanticPolarity::Cautious), Some(SemanticPolarity::Negative))
        | (Some(SemanticPolarity::Positive), Some(SemanticPolarity::Cautious))
        | (Some(SemanticPolarity::Cautious), Some(SemanticPolarity::Positive)) => {
            Some(ContradictionSeverity::Warning)
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SemanticPolarity {
    Positive,
    Negative,
    Cautious,
}

fn semantic_polarity(text: &str) -> Option<SemanticPolarity> {
    let normalized = text.to_lowercase();

    if contains_any(
        &normalized,
        &["safe", "no issue", "passed", "通过", "无风险", "正常"],
    ) {
        return Some(SemanticPolarity::Positive);
    }

    if contains_any(
        &normalized,
        &[
            "unsafe",
            "has issue",
            "issue found",
            "issue",
            "risk",
            "风险",
            "漏洞",
            "有问题",
            "失败",
        ],
    ) {
        return Some(SemanticPolarity::Negative);
    }

    if contains_any(
        &normalized,
        &["warning", "caution", "需关注", "注意", "可能风险"],
    ) {
        return Some(SemanticPolarity::Cautious);
    }

    None
}

fn contains_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| text.contains(pattern))
}

fn truncate_findings_to_budget(findings: &[String], max_tokens: usize) -> Vec<String> {
    let mut retained = Vec::new();
    let mut used_tokens = 0usize;

    for finding in findings {
        let finding_tokens = estimate_tokens(finding);
        if used_tokens + finding_tokens <= max_tokens {
            retained.push(finding.clone());
            used_tokens += finding_tokens;
            continue;
        }

        let remaining = max_tokens.saturating_sub(used_tokens);
        if remaining == 0 {
            break;
        }

        let max_chars = remaining.saturating_mul(4).max(1);
        retained.push(format!("{}...", truncate_text(finding, max_chars)));
        break;
    }

    retained
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    let mut collected = text.chars().take(max_chars).collect::<String>();
    if text.chars().count() > max_chars {
        collected.push_str(" [截断]");
    }
    collected
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use mc_core::{agent::AgentType, token::TokenUsage};

    use super::{detect_contradictions, ResultAggregator};
    use crate::{
        filter::FilteredResult,
        sub_agent::{SubAgentResult, SubAgentSpec},
    };

    fn sample_spec(id: &str) -> SubAgentSpec {
        SubAgentSpec::new(
            id,
            AgentType::Explorer,
            "inspect auth",
            vec![PathBuf::from("src/auth.rs")],
            4_000,
            30,
            vec!["read_file".to_string()],
        )
    }

    #[test]
    fn contradiction_detection_finds_opposing_conclusions() {
        let left = SubAgentResult::completed(
            sample_spec("left"),
            "auth is safe".to_string(),
            TokenUsage::default(),
            std::time::Duration::from_secs(1),
        );
        let right = SubAgentResult::completed(
            sample_spec("right"),
            "auth has issue and risk".to_string(),
            TokenUsage::default(),
            std::time::Duration::from_secs(1),
        );

        let contradictions = detect_contradictions(&[left, right]);
        assert_eq!(contradictions.len(), 1);
    }

    #[test]
    fn aggregate_truncates_when_findings_exceed_budget() {
        let aggregator = ResultAggregator::new(4);
        let filtered = FilteredResult {
            retained: vec!["very long finding about auth".to_string()],
            discarded_count: 0,
            compressed: Vec::new(),
            original_tokens: 20,
            filtered_tokens: 20,
            compression_ratio: 1.0,
        };
        let result = SubAgentResult::completed(
            sample_spec("a"),
            "auth has issue".to_string(),
            TokenUsage {
                total_tokens: 25,
                ..TokenUsage::default()
            },
            std::time::Duration::from_secs(1),
        );

        let aggregated = aggregator.aggregate("task", &[filtered], &[result], 0);
        assert_eq!(aggregated.sub_agents_used, 1);
        assert_eq!(aggregated.depth_reached, 1);
        assert_eq!(aggregated.total_tokens_used, 25);
        assert_eq!(aggregated.findings.len(), 1);
    }
}
