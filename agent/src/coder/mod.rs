pub mod codegen;

use async_trait::async_trait;
use mc_core::{AgentType, ExecutionPlan, ProjectContext, TaskDescription};
use mc_llm::StreamForwarder;
use serde_json::json;

use crate::coder::codegen::{build_prompt, fallback_output, CodeGenerationOutput};
use crate::support::complete_json;
use crate::{
    Agent, AgentConfig, AgentContext, AgentError, AgentExecutionReport, ImpactReport,
    SharedResources,
};

#[derive(Debug, Clone)]
pub struct Coder {
    config: AgentConfig,
}

impl Coder {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    async fn resolve_plan(&self, ctx: &AgentContext) -> Result<Option<ExecutionPlan>, AgentError> {
        if let Some(plan) = ctx.execution_plan.as_deref().cloned() {
            return Ok(Some(plan));
        }
        Ok(ctx.handoff.get::<ExecutionPlan>().await)
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

    async fn resolve_impact(&self, ctx: &AgentContext) -> Result<Option<ImpactReport>, AgentError> {
        if let Some(report) = ctx.impact_report.as_deref().cloned() {
            return Ok(Some(report));
        }
        Ok(ctx.handoff.get::<ImpactReport>().await)
    }

    async fn generate_output(
        &self,
        ctx: &AgentContext,
        project_ctx: Option<&ProjectContext>,
        impact_report: Option<&ImpactReport>,
        execution_plan: Option<&ExecutionPlan>,
    ) -> Result<(CodeGenerationOutput, u32), AgentError> {
        let prompt = build_prompt(
            ctx.task.as_ref(),
            project_ctx,
            impact_report,
            execution_plan,
        );
        complete_json(
            ctx.llm_provider.as_ref(),
            &ctx.config.llm_config.model_id,
            "You are the implementation agent. Return strict JSON describing the concrete code changes to make. Do not include markdown fences.",
            &prompt,
            "coder_generation",
            json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["summary", "implementation_notes", "changes", "validation_steps", "risks"],
                "properties": {
                    "summary": { "type": "string" },
                    "implementation_notes": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "changes": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "additionalProperties": false,
                            "required": ["path", "change_kind", "rationale", "patch_preview", "acceptance_checks"],
                            "properties": {
                                "path": { "type": "string" },
                                "change_kind": { "type": "string", "enum": ["add", "modify", "delete"] },
                                "rationale": { "type": "string" },
                                "patch_preview": { "type": "string" },
                                "acceptance_checks": {
                                    "type": "array",
                                    "items": { "type": "string" }
                                }
                            }
                        }
                    },
                    "validation_steps": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "risks": {
                        "type": "array",
                        "items": { "type": "string" }
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
impl Agent for Coder {
    fn agent_type(&self) -> AgentType {
        AgentType::Coder
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn default_config(&self) -> AgentConfig {
        AgentConfig::for_agent_type(AgentType::Coder)
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
        let execution_plan = self.resolve_plan(ctx).await?;
        let project_ctx = self.resolve_project(ctx).await?;
        let impact_report = self.resolve_impact(ctx).await?;

        let (mut output, tokens) = match self
            .generate_output(
                ctx,
                project_ctx.as_ref(),
                impact_report.as_ref(),
                execution_plan.as_ref(),
            )
            .await
        {
            Ok(result) => result,
            Err(AgentError::LlmError { .. }) | Err(AgentError::LlmParseError { .. }) => (
                fallback_output(ctx.task.as_ref(), execution_plan.as_ref()),
                0,
            ),
            Err(error) => return Err(error),
        };
        output.ensure_consistency(ctx.task.as_ref(), execution_plan.as_ref());

        ctx.handoff.put(output.clone()).await;
        let result = serde_json::to_value(&output).map_err(AgentError::serialization)?;
        let mut report = AgentExecutionReport::success(
            AgentType::Coder,
            &ctx.execution_id,
            format!(
                "Coder prepared {} concrete change drafts",
                output.changes.len()
            ),
            result,
            ctx.elapsed_ms(),
            tokens,
        );
        report.warnings = output.risks.clone();
        Ok(report)
    }

    async fn execute_streaming(
        &self,
        ctx: &AgentContext,
        _forwarder: &mut StreamForwarder,
    ) -> Result<AgentExecutionReport, AgentError> {
        self.execute(ctx).await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use mc_core::{AgentType, TaskDescription};

    use super::Coder;
    use crate::coder::codegen::CodeGenerationOutput;
    use crate::test_support::{create_test_project, MockLlmProvider};
    use crate::{
        Agent, AgentConfig, AgentHandoff, CognitivePipeline, Explorer, ImpactAnalyzer, Planner,
        SharedResources,
    };

    #[tokio::test]
    async fn coder_generates_structured_change_drafts_and_stores_them_in_handoff() {
        let project = create_test_project();
        let provider = Arc::new(MockLlmProvider::new(HashMap::from([
            (
                "explorer_summary".to_string(),
                serde_json::json!({
                    "project_summary": "Workspace summary",
                    "architecture_name": "Workspace",
                    "architecture_description": "Two Rust crates",
                    "design_decisions": ["Core crate shared by app"],
                    "notable_patterns": ["pipeline"]
                })
                .to_string(),
            ),
            (
                "impact_assessment".to_string(),
                serde_json::json!({
                    "compatibility_notes": ["Check downstream callers in app"],
                    "recommendations": ["Run focused verification on app"],
                    "risk_assessment": []
                })
                .to_string(),
            ),
            (
                "planner_summary".to_string(),
                serde_json::json!({
                    "summary": "Implement core change, then review and test",
                    "review_focus": ["core to app dependency"]
                })
                .to_string(),
            ),
            (
                "coder_generation".to_string(),
                serde_json::json!({
                    "summary": "Adjust the core compute logic and keep downstream behavior aligned",
                    "implementation_notes": ["Update the core crate", "Preserve the public contract used by app"],
                    "changes": [{
                        "path": "core/src/lib.rs",
                        "change_kind": "modify",
                        "rationale": "The task directly targets the compute implementation",
                        "patch_preview": "",
                        "acceptance_checks": ["cargo test"]
                    }],
                    "validation_steps": ["cargo test"],
                    "risks": ["app depends on the return value of `compute`"]
                })
                .to_string(),
            ),
        ])));
        let shared = SharedResources::new(project.path(), provider);
        let pipeline = CognitivePipeline::new(
            Explorer::new(AgentConfig::for_agent_type(AgentType::Explorer)),
            ImpactAnalyzer::new(AgentConfig::for_agent_type(AgentType::ImpactAnalyzer)),
            Planner::new(AgentConfig::for_agent_type(AgentType::Planner)),
        );

        let mut task = TaskDescription::with_root("update core behavior", project.path());
        task.affected_files = vec!["core/src/lib.rs".to_string()];
        task.requires_testing = true;

        let planning = pipeline
            .execute(&task, &shared)
            .await
            .expect("planning pipeline");

        let handoff = Arc::new(AgentHandoff::new());
        let coder = Coder::new(AgentConfig::for_agent_type(AgentType::Coder));
        let ctx = coder
            .build_context(&task, Some(planning.project_context.clone()), &shared)
            .await
            .expect("coder context")
            .with_handoff(Arc::clone(&handoff))
            .with_impact_report(planning.impact_report.clone())
            .with_execution_plan(planning.execution_plan.clone());

        let report = coder.execute(&ctx).await.expect("coder execution");
        let generated = ctx
            .handoff
            .get::<CodeGenerationOutput>()
            .await
            .expect("generated output should exist");

        assert_eq!(report.agent_type, AgentType::Coder);
        assert_eq!(generated.changes.len(), 1);
        assert_eq!(generated.changes[0].path, "core/src/lib.rs");
        assert!(!generated.changes[0].patch_preview.is_empty());
        assert_eq!(report.warnings.len(), 1);
    }
}
