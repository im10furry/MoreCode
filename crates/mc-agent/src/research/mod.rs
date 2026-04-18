use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use mc_core::{AgentType, ProjectContext, TaskDescription};
use serde_json::{json, Value};
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::execution_report::{build_report, serialize_extra};
use crate::{Agent, AgentConfig, AgentContext, AgentError, SharedResources};

pub mod source;
pub use source::{
    ResearchFinding, ResearchReport, ResearchSource, ResearchSourceKind, TechnologyComparison,
};

#[derive(Clone)]
pub struct Research {
    config: AgentConfig,
    web_search_tool: String,
    web_fetch_tool: String,
    api_doc_tool: String,
}

impl Research {
    pub fn new(config: AgentConfig) -> Self {
        Self::with_tools(config, "web_search", "web_fetch", "api_doc_reader")
    }

    pub fn with_tools(
        config: AgentConfig,
        web_search_tool: impl Into<String>,
        web_fetch_tool: impl Into<String>,
        api_doc_tool: impl Into<String>,
    ) -> Self {
        Self {
            config,
            web_search_tool: web_search_tool.into(),
            web_fetch_tool: web_fetch_tool.into(),
            api_doc_tool: api_doc_tool.into(),
        }
    }

    fn execute_topic(
        &self,
        ctx: AgentContext,
    ) -> Pin<Box<dyn Future<Output = Result<ResearchReport, AgentError>> + Send + '_>> {
        Box::pin(async move {
            if ctx.is_cancelled() {
                return Err(AgentError::Cancelled {
                    reason: "research cancelled".to_string(),
                });
            }

            if ctx.recursion_depth > ctx.config.max_recursion_depth {
                return Err(AgentError::RecursionDepthExceeded {
                    current: ctx.recursion_depth,
                    max: ctx.config.max_recursion_depth,
                });
            }

            let subtopics = self.derive_subtopics(&ctx);
            if subtopics.len() > 1 && ctx.recursion_depth < ctx.config.max_recursion_depth {
                let mut join_set = JoinSet::new();
                for topic in subtopics
                    .into_iter()
                    .take(ctx.config.max_parallel_tasks.max(1))
                {
                    let agent = self.clone();
                    let child_task = clone_task_with_input(ctx.task.as_ref(), topic);
                    let mut child_ctx = ctx.create_child_context(child_task);
                    child_ctx.metadata.remove("research_subtopics");
                    join_set.spawn(async move { agent.execute_topic(child_ctx).await });
                }

                let mut reports = Vec::new();
                let mut warnings = Vec::new();
                while let Some(result) = join_set.join_next().await {
                    match result {
                        Ok(Ok(report)) => reports.push(report),
                        Ok(Err(error)) => warnings.push(error.to_string()),
                        Err(error) => warnings.push(error.to_string()),
                    }
                }

                return Ok(self.aggregate_reports(ctx.task.user_input.clone(), reports, warnings));
            }

            self.execute_leaf(ctx).await
        })
    }

    async fn execute_leaf(&self, ctx: AgentContext) -> Result<ResearchReport, AgentError> {
        let search_hits = self.search_web(&ctx).await?;
        let documents = self.fetch_documents(&ctx, &search_hits).await?;
        let api_doc = self.read_api_docs(&ctx, &search_hits).await?;

        let mut sources = Vec::new();
        for hit in &search_hits {
            sources.push(ResearchSource {
                title: hit.title.clone(),
                url: hit.url.clone(),
                relevance: hit.relevance,
                summary: hit.snippet.clone(),
                kind: ResearchSourceKind::Search,
            });
        }

        for document in &documents {
            sources.push(ResearchSource {
                title: document.title.clone(),
                url: document.url.clone(),
                relevance: document.relevance,
                summary: document.summary.clone(),
                kind: ResearchSourceKind::Web,
            });
        }

        if let Some(document) = &api_doc {
            sources.push(ResearchSource {
                title: document.title.clone(),
                url: document.url.clone(),
                relevance: document.relevance,
                summary: document.summary.clone(),
                kind: ResearchSourceKind::ApiDoc,
            });
        }

        dedupe_sources(&mut sources);

        let findings = self.build_findings(
            &ctx.task.user_input,
            &search_hits,
            &documents,
            api_doc.as_ref(),
        );
        let comparisons = self.build_comparisons(&ctx.task.user_input, &sources);
        let recommendations =
            self.build_recommendations(&ctx.task.user_input, &findings, &comparisons);
        let summary = self.build_summary(&ctx.task.user_input, &findings, &sources);

        Ok(ResearchReport {
            topic: ctx.task.user_input.clone(),
            findings,
            recommendations,
            sources,
            summary,
            comparisons,
        })
    }

    async fn search_web(&self, ctx: &AgentContext) -> Result<Vec<SearchHit>, AgentError> {
        let value = ctx
            .call_tool_value(
                AgentType::Research,
                &self.web_search_tool,
                json!({
                    "query": ctx.task.user_input,
                    "topic": ctx.task.user_input,
                    "max_results": ctx.config.max_tool_calls,
                }),
            )
            .await?;

        Ok(parse_search_hits(&value)
            .into_iter()
            .take(ctx.config.max_tool_calls.max(1))
            .collect())
    }

    async fn fetch_documents(
        &self,
        ctx: &AgentContext,
        search_hits: &[SearchHit],
    ) -> Result<Vec<FetchedDocument>, AgentError> {
        if !ctx.has_tool(&self.web_fetch_tool).await {
            return Ok(Vec::new());
        }

        let mut join_set = JoinSet::new();
        for hit in search_hits.iter().filter(|hit| !hit.url.is_empty()).take(3) {
            let tool_name = self.web_fetch_tool.clone();
            let ctx = ctx.clone();
            let url = hit.url.clone();
            let title = hit.title.clone();
            let relevance = hit.relevance;
            join_set.spawn(async move {
                let value = ctx
                    .call_tool_value(
                        AgentType::Research,
                        &tool_name,
                        json!({
                            "url": url,
                            "title": title,
                        }),
                    )
                    .await?;
                Ok::<FetchedDocument, AgentError>(parse_document(&value, relevance))
            });
        }

        let mut documents = Vec::new();
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(document)) => documents.push(document),
                Ok(Err(_)) | Err(_) => {}
            }
        }

        Ok(documents)
    }

    async fn read_api_docs(
        &self,
        ctx: &AgentContext,
        search_hits: &[SearchHit],
    ) -> Result<Option<FetchedDocument>, AgentError> {
        if !ctx.has_tool(&self.api_doc_tool).await
            || !self.should_read_api_docs(&ctx.task.user_input, search_hits)
        {
            return Ok(None);
        }

        let preferred_url = search_hits
            .iter()
            .find(|hit| {
                let haystack = format!("{} {}", hit.title.to_lowercase(), hit.url.to_lowercase());
                haystack.contains("api") || haystack.contains("docs")
            })
            .map(|hit| hit.url.clone());

        let value = ctx
            .call_tool_value(
                AgentType::Research,
                &self.api_doc_tool,
                json!({
                    "query": ctx.task.user_input,
                    "url": preferred_url,
                }),
            )
            .await?;

        Ok(Some(parse_document(&value, 0.95)))
    }

    fn derive_subtopics(&self, ctx: &AgentContext) -> Vec<String> {
        if ctx.recursion_depth == 0 {
            if let Some(topics) = ctx.get_metadata::<Vec<String>>("research_subtopics") {
                let deduped = dedupe_strings(topics);
                if deduped.len() > 1 {
                    return deduped;
                }
            }
        }

        let candidates = self.extract_candidates(&ctx.task.user_input);
        if candidates.len() > 1 {
            return candidates
                .into_iter()
                .map(|candidate| format!("{candidate} evaluation"))
                .collect();
        }

        Vec::new()
    }

    fn extract_candidates(&self, topic: &str) -> Vec<String> {
        let lower = topic.to_lowercase();
        if !(lower.contains(" vs ")
            || lower.contains(" compare ")
            || lower.contains("对比")
            || lower.contains("比较"))
        {
            return Vec::new();
        }

        let replaced = topic
            .replace(" VS ", " vs ")
            .replace(" Vs ", " vs ")
            .replace("对比", " vs ")
            .replace("比较", " vs ")
            .replace(" compare ", " vs ");

        dedupe_strings(
            replaced
                .split(" vs ")
                .flat_map(|segment| segment.split(&[',', '、', '/', '|'][..]))
                .map(str::trim)
                .filter(|segment| segment.len() > 1)
                .map(|segment| {
                    segment
                        .split_whitespace()
                        .take_while(|token| {
                            token
                                .chars()
                                .all(|ch| ch.is_alphanumeric() || ch == '-' || ch == '_')
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .filter(|segment| !segment.is_empty())
                .collect(),
        )
    }

    fn should_read_api_docs(&self, topic: &str, hits: &[SearchHit]) -> bool {
        let topic_lower = topic.to_lowercase();
        if ["api", "sdk", "docs", "文档", "endpoint"]
            .iter()
            .any(|needle| topic_lower.contains(needle))
        {
            return true;
        }

        hits.iter().any(|hit| {
            let haystack = format!("{} {}", hit.title.to_lowercase(), hit.url.to_lowercase());
            haystack.contains("api") || haystack.contains("docs")
        })
    }

    fn build_findings(
        &self,
        topic: &str,
        search_hits: &[SearchHit],
        documents: &[FetchedDocument],
        api_doc: Option<&FetchedDocument>,
    ) -> Vec<ResearchFinding> {
        let mut findings = Vec::new();

        for document in documents.iter().take(3) {
            findings.push(ResearchFinding {
                topic: topic.to_string(),
                description: sentence_or_summary(&document.summary, &document.content),
                confidence: document.relevance.clamp(0.1, 0.99),
                source_titles: vec![document.title.clone()],
            });
        }

        if let Some(document) = api_doc {
            findings.push(ResearchFinding {
                topic: format!("{topic} API"),
                description: format!(
                    "API documentation summary: {}",
                    sentence_or_summary(&document.summary, &document.content)
                ),
                confidence: document.relevance.clamp(0.1, 0.99),
                source_titles: vec![document.title.clone()],
            });
        }

        if findings.is_empty() {
            for hit in search_hits.iter().take(3) {
                findings.push(ResearchFinding {
                    topic: topic.to_string(),
                    description: sentence_or_summary(&hit.snippet, &hit.snippet),
                    confidence: hit.relevance.clamp(0.1, 0.99),
                    source_titles: vec![hit.title.clone()],
                });
            }
        }

        dedupe_findings(findings)
    }

    fn build_comparisons(
        &self,
        topic: &str,
        sources: &[ResearchSource],
    ) -> Vec<TechnologyComparison> {
        let candidates = self.extract_candidates(topic);
        if candidates.len() < 2 {
            return Vec::new();
        }

        candidates
            .into_iter()
            .map(|candidate| {
                let mut strengths = Vec::new();
                let mut weaknesses = Vec::new();
                for source in sources {
                    let haystack = format!(
                        "{} {}",
                        source.title.to_lowercase(),
                        source.summary.to_lowercase()
                    );
                    if !haystack.contains(&candidate.to_lowercase()) {
                        continue;
                    }

                    for sentence in split_sentences(&source.summary) {
                        let sentence_lower = sentence.to_lowercase();
                        if has_positive_signal(&sentence_lower) {
                            strengths.push(sentence.to_string());
                        } else if has_negative_signal(&sentence_lower) {
                            weaknesses.push(sentence.to_string());
                        }
                    }
                }

                let recommendation = if strengths.len() >= weaknesses.len() {
                    format!("{candidate} is a reasonable fit based on the collected sources.")
                } else {
                    format!("{candidate} needs further validation before adoption.")
                };

                TechnologyComparison {
                    candidate,
                    strengths: dedupe_strings(strengths),
                    weaknesses: dedupe_strings(weaknesses),
                    recommendation,
                }
            })
            .collect()
    }

    fn build_recommendations(
        &self,
        topic: &str,
        findings: &[ResearchFinding],
        comparisons: &[TechnologyComparison],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if let Some(best) = comparisons.iter().max_by_key(|comparison| {
            comparison.strengths.len() as isize - comparison.weaknesses.len() as isize
        }) {
            recommendations.push(format!(
                "For {topic}, prioritize {} and validate the decision against the project constraints.",
                best.candidate
            ));
        }

        if !findings.is_empty() {
            recommendations.push("Keep official documentation in the implementation context to avoid stale assumptions.".to_string());
        }

        recommendations.push(
            "Record the chosen sources and rationale in the follow-up design notes.".to_string(),
        );
        dedupe_strings(recommendations)
    }

    fn build_summary(
        &self,
        topic: &str,
        findings: &[ResearchFinding],
        sources: &[ResearchSource],
    ) -> String {
        if findings.is_empty() {
            return format!(
                "No concrete findings were extracted for {topic}, but {} sources were inspected.",
                sources.len()
            );
        }

        let leading = findings
            .iter()
            .take(3)
            .map(|finding| finding.description.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        format!(
            "Research on {topic} produced {} findings from {} sources. {leading}",
            findings.len(),
            sources.len()
        )
    }

    fn aggregate_reports(
        &self,
        topic: String,
        reports: Vec<ResearchReport>,
        warnings: Vec<String>,
    ) -> ResearchReport {
        let mut findings = Vec::new();
        let mut recommendations = warnings;
        let mut sources = Vec::new();
        let mut comparisons = Vec::new();
        let mut summaries = Vec::new();

        for report in reports {
            findings.extend(report.findings);
            recommendations.extend(report.recommendations);
            sources.extend(report.sources);
            comparisons.extend(report.comparisons);
            summaries.push(report.summary);
        }

        dedupe_sources(&mut sources);

        ResearchReport {
            topic: topic.clone(),
            findings: dedupe_findings(findings),
            recommendations: dedupe_strings(recommendations),
            sources,
            summary: format!(
                "Recursive research on {topic} completed across {} subtopics. {}",
                summaries.len(),
                summaries.join(" ")
            ),
            comparisons,
        }
    }
}

#[async_trait]
impl Agent for Research {
    fn agent_type(&self) -> AgentType {
        AgentType::Research
    }

    fn supports_recursion(&self) -> bool {
        true
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn default_config(&self) -> AgentConfig {
        AgentConfig::for_agent_type(AgentType::Research)
    }

    fn update_config(&mut self, config: AgentConfig) {
        self.config = config;
    }

    async fn build_context(
        &self,
        task: &TaskDescription,
        project_ctx: Option<ProjectContext>,
        shared: &SharedResources,
    ) -> Result<AgentContext, AgentError> {
        let mut ctx = AgentContext::new(task.clone(), shared, self.config.clone());
        if let Some(project_ctx) = project_ctx {
            ctx.project_ctx = Some(Arc::new(project_ctx));
        }
        Ok(ctx)
    }

    async fn execute(
        &self,
        ctx: &AgentContext,
    ) -> Result<mc_core::AgentExecutionReport, AgentError> {
        let report = self.execute_topic(ctx.clone()).await?;
        ctx.handoff.put(report.clone()).await;

        Ok(build_report(
            AgentType::Research,
            format!("research completed for {}", report.topic),
            report
                .findings
                .iter()
                .map(|finding| finding.description.clone())
                .collect(),
            Vec::new(),
            report.recommendations.clone(),
            Vec::new(),
            (report.sources.len() + report.findings.len() * 2) as u32,
            Some(serialize_extra(&report)?),
        ))
    }
}

#[derive(Debug, Clone)]
struct SearchHit {
    title: String,
    url: String,
    snippet: String,
    relevance: f64,
}

#[derive(Debug, Clone)]
struct FetchedDocument {
    title: String,
    url: String,
    summary: String,
    content: String,
    relevance: f64,
}

fn parse_search_hits(value: &Value) -> Vec<SearchHit> {
    value
        .get("results")
        .or_else(|| value.get("items"))
        .or_else(|| value.get("matches"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            let title = item
                .get("title")
                .or_else(|| item.get("file"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let url = item
                .get("url")
                .or_else(|| item.get("path"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let snippet = item
                .get("snippet")
                .or_else(|| item.get("summary"))
                .or_else(|| item.get("text"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let relevance = item
                .get("relevance")
                .and_then(Value::as_f64)
                .or_else(|| item.get("score").and_then(Value::as_f64))
                .unwrap_or(0.6);

            if title.is_empty() && url.is_empty() && snippet.is_empty() {
                None
            } else {
                Some(SearchHit {
                    title: if title.is_empty() { url.clone() } else { title },
                    url,
                    snippet,
                    relevance,
                })
            }
        })
        .collect()
}

fn parse_document(value: &Value, fallback_relevance: f64) -> FetchedDocument {
    let derived_summary = value
        .get("content")
        .and_then(Value::as_str)
        .map(first_sentence)
        .unwrap_or_default();
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Untitled source")
        .to_string();
    let url = value
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let summary = value
        .get("summary")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| derived_summary.clone());
    let content = value
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or(derived_summary.as_str())
        .to_string();
    let relevance = value
        .get("relevance")
        .and_then(Value::as_f64)
        .unwrap_or(fallback_relevance);

    FetchedDocument {
        title,
        url,
        summary,
        content,
        relevance,
    }
}

fn clone_task_with_input(task: &TaskDescription, user_input: String) -> TaskDescription {
    let mut cloned = task.clone();
    cloned.id = Uuid::new_v4().to_string();
    cloned.user_input = user_input;
    cloned
}

fn dedupe_sources(sources: &mut Vec<ResearchSource>) {
    let mut seen = HashSet::new();
    sources.retain(|source| {
        let key = if !source.url.is_empty() {
            source.url.clone()
        } else {
            source.title.clone()
        };
        seen.insert(key)
    });
}

fn dedupe_findings(findings: Vec<ResearchFinding>) -> Vec<ResearchFinding> {
    let mut seen = HashSet::new();
    findings
        .into_iter()
        .filter(|finding| seen.insert((finding.topic.clone(), finding.description.clone())))
        .collect()
}

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

fn split_sentences(text: &str) -> Vec<&str> {
    text.split(['.', '!', '?', '。', '！', '？'])
        .map(str::trim)
        .filter(|sentence| !sentence.is_empty())
        .collect()
}

fn first_sentence(text: &str) -> String {
    split_sentences(text)
        .into_iter()
        .next()
        .unwrap_or_default()
        .to_string()
}

fn sentence_or_summary(summary: &str, content: &str) -> String {
    if !summary.trim().is_empty() {
        summary.trim().to_string()
    } else {
        first_sentence(content)
    }
}

fn has_positive_signal(text: &str) -> bool {
    [
        "good",
        "fast",
        "strong",
        "stable",
        "excellent",
        "recommend",
        "native",
        "easy",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn has_negative_signal(text: &str) -> bool {
    [
        "slow",
        "weak",
        "limited",
        "caution",
        "risk",
        "difficult",
        "deprecated",
        "overhead",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}
