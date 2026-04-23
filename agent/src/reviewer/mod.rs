pub mod rule_engine;

use async_trait::async_trait;
use mc_core::{AgentType, ExecutionPlan, ProjectContext, TaskDescription};
use serde::Deserialize;
use serde_json::json;

use crate::coder::codegen::CodeGenerationOutput;
use crate::reviewer::rule_engine::{
    ReviewFinding, ReviewInput, ReviewReport, ReviewRuleEngine, ReviewSeverity, ReviewVerdict,
};
use crate::support::complete_json;
use crate::{
    Agent, AgentConfig, AgentContext, AgentError, AgentExecutionReport, ImpactReport,
    SharedResources,
};

#[derive(Debug, Clone)]
pub struct Reviewer {
    config: AgentConfig,
    rules: ReviewRuleEngine,
}

#[derive(Debug, Clone, Deserialize)]
struct ReviewerEnrichment {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    verdict: ReviewVerdict,
    #[serde(default)]
    additional_findings: Vec<ReviewFinding>,
}

impl Reviewer {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            rules: ReviewRuleEngine::default(),
        }
    }

    async fn resolve_project(
        &self,
        ctx: &AgentContext,
    ) -> Result<Option<ProjectContext>, AgentError> {
        if let Some(project_ctx) = ctx.project_ctx.as_deref().cloned() {
            return Ok(Some(project_ctx));
        }
        Ok(ctx.handoff.get::<ProjectContext>().await)
    }

    async fn resolve_plan(&self, ctx: &AgentContext) -> Result<Option<ExecutionPlan>, AgentError> {
        if let Some(plan) = ctx.execution_plan.as_deref().cloned() {
            return Ok(Some(plan));
        }
        Ok(ctx.handoff.get::<ExecutionPlan>().await)
    }

    async fn resolve_impact(&self, ctx: &AgentContext) -> Result<Option<ImpactReport>, AgentError> {
        if let Some(report) = ctx.impact_report.as_deref().cloned() {
            return Ok(Some(report));
        }
        Ok(ctx.handoff.get::<ImpactReport>().await)
    }

    async fn resolve_codegen(
        &self,
        ctx: &AgentContext,
    ) -> Result<Option<CodeGenerationOutput>, AgentError> {
        Ok(ctx.handoff.get::<CodeGenerationOutput>().await)
    }

    async fn enrich(
        &self,
        ctx: &AgentContext,
        base: &ReviewReport,
        input: &ReviewInput<'_>,
    ) -> Result<(ReviewerEnrichment, u32), AgentError> {
        let reviewed_files = if base.reviewed_files.is_empty() {
            "none".to_string()
        } else {
            base.reviewed_files.join(", ")
        };
        let existing_findings = if base.findings.is_empty() {
            "none".to_string()
        } else {
            base.findings
                .iter()
                .map(|finding| format!("{:?}: {}", finding.severity, finding.title))
                .collect::<Vec<_>>()
                .join("; ")
        };
        let expected_files = if let Some(plan) = input.execution_plan {
            let mut files = plan
                .sub_tasks
                .iter()
                .flat_map(|sub_task| sub_task.target_files.clone())
                .collect::<Vec<_>>();
            files.sort();
            files.dedup();
            if files.is_empty() {
                ctx.task.affected_files.clone()
            } else {
                files
            }
        } else {
            ctx.task.affected_files.clone()
        };
        let prompt = format!(
            "Task: {}\nReviewed files: {}\nExisting verdict: {:?}\nExisting findings: {}\nExpected files: {}\nHigh-risk impacts: {}\nReturn only JSON.",
            ctx.task.user_input,
            reviewed_files,
            base.verdict,
            existing_findings,
            if expected_files.is_empty() {
                "none".to_string()
            } else {
                expected_files.join(", ")
            },
            input
                .impact_report
                .map(|impact| format!("{:?}", impact.overall_risk_level))
                .unwrap_or_else(|| "unknown".to_string()),
        );

        complete_json(
            ctx.llm_provider.as_ref(),
            &ctx.config.llm_config.model_id,
            "You are a senior code reviewer. Refine the review result and return strict JSON only.",
            &prompt,
            "reviewer_assessment",
            json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["summary", "verdict", "additional_findings"],
                "properties": {
                    "summary": { "type": "string" },
                    "verdict": {
                        "type": "string",
                        "enum": ["approved", "needs_changes", "rejected"]
                    },
                    "additional_findings": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "additionalProperties": false,
                            "required": ["severity", "title", "detail", "recommendation"],
                            "properties": {
                                "severity": {
                                    "type": "string",
                                    "enum": ["blocker", "warning", "suggestion", "info"]
                                },
                                "title": { "type": "string" },
                                "detail": { "type": "string" },
                                "recommendation": { "type": "string" }
                            }
                        }
                    }
                }
            }),
            ctx.config.llm_config.temperature,
            ctx.config.llm_config.max_output_tokens,
            ctx.cancel_token.child_token(),
        )
        .await
    }
}

#[async_trait]
impl Agent for Reviewer {
    fn agent_type(&self) -> AgentType {
        AgentType::Reviewer
    }

    fn default_config(&self) -> AgentConfig {
        AgentConfig::for_agent_type(AgentType::Reviewer)
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
            ctx = ctx.with_project_ctx(project_ctx);
        }
        Ok(ctx)
    }

    async fn execute(&self, ctx: &AgentContext) -> Result<AgentExecutionReport, AgentError> {
        let project_ctx = self.resolve_project(ctx).await?;
        let execution_plan = self.resolve_plan(ctx).await?;
        let impact_report = self.resolve_impact(ctx).await?;
        let codegen = self.resolve_codegen(ctx).await?;

        let input = ReviewInput {
            task: ctx.task.as_ref(),
            project_ctx: project_ctx.as_ref(),
            impact_report: impact_report.as_ref(),
            execution_plan: execution_plan.as_ref(),
            codegen: codegen.as_ref(),
        };
        let mut report = self.rules.evaluate(&input);

        match self.enrich(ctx, &report, &input).await {
            Ok((enrichment, _tokens)) => {
                report.summary = enrichment.summary;
                report.verdict = ReviewVerdict::max(report.verdict, enrichment.verdict);
                report.findings.extend(enrichment.additional_findings);
                report.recompute_verdict();
            }
            Err(AgentError::LlmError { .. }) | Err(AgentError::LlmParseError { .. }) => {}
            Err(error) => return Err(error),
        }

        ctx.handoff.put(report.clone()).await;

        let result = serde_json::to_value(&report).map_err(AgentError::serialization)?;
        let warnings = report
            .findings
            .iter()
            .filter(|finding| {
                matches!(
                    finding.severity,
                    ReviewSeverity::Blocker | ReviewSeverity::Warning
                )
            })
            .map(|finding| finding.title.clone())
            .collect::<Vec<_>>();

        let mut execution = AgentExecutionReport::success(
            AgentType::Reviewer,
            &ctx.execution_id,
            format!(
                "Reviewer completed with {:?} and {} finding(s)",
                report.verdict,
                report.findings.len()
            ),
            result,
            ctx.elapsed_ms(),
            0,
        );
        execution.warnings = warnings;
        Ok(execution)
    }
}

impl Default for Reviewer {
    fn default() -> Self {
        Self::new(AgentConfig::for_agent_type(AgentType::Reviewer))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use mc_core::TaskDescription;

    use super::Reviewer;
    use crate::coder::codegen::{CodeChangeDraft, CodeChangeKind, CodeGenerationOutput};
    use crate::reviewer::rule_engine::{ReviewReport, ReviewVerdict};
    use crate::test_support::{create_test_project, MockLlmProvider};
    use crate::{Agent, AgentHandoff, SharedResources};

    #[tokio::test]
    async fn reviewer_flags_missing_acceptance_checks() {
        let project = create_test_project();
        let provider = Arc::new(MockLlmProvider::new(HashMap::new()));
        let shared = SharedResources::new(project.path(), provider);

        let mut task = TaskDescription::with_root("review implementation", project.path());
        task.affected_files = vec!["core/src/lib.rs".to_string()];

        let handoff = Arc::new(AgentHandoff::new());
        handoff
            .put(CodeGenerationOutput {
                summary: "update core".to_string(),
                implementation_notes: Vec::new(),
                changes: vec![CodeChangeDraft {
                    path: "core/src/lib.rs".to_string(),
                    change_kind: CodeChangeKind::Modify,
                    rationale: "adjust behavior".to_string(),
                    patch_preview: String::new(),
                    acceptance_checks: Vec::new(),
                }],
                validation_steps: Vec::new(),
                risks: Vec::new(),
            })
            .await;

        let reviewer = Reviewer::default();
        let ctx = reviewer
            .build_context(&task, None, &shared)
            .await
            .expect("context")
            .with_handoff(handoff);

        let execution = reviewer.execute(&ctx).await.expect("review");
        let report = ctx
            .handoff
            .get::<ReviewReport>()
            .await
            .expect("review report");

        assert_eq!(execution.agent_type, mc_core::AgentType::Reviewer);
        assert_eq!(report.verdict, ReviewVerdict::NeedsChanges);
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.title.contains("acceptance checks")));
    }
}
