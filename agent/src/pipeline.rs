use std::sync::Arc;

use mc_core::{ExecutionPlan, ProjectContext, TaskDescription};

use crate::{
    Agent, AgentError, AgentExecutionReport, AgentHandoff, CodeGenerationOutput, Coder, Explorer,
    ImpactAnalyzer, ImpactReport, Planner, ReviewReport, Reviewer, SharedResources, Tester,
    TesterExecutionReport,
};

#[derive(Debug, Clone)]
pub struct CognitivePipelineResult {
    pub project_context: ProjectContext,
    pub impact_report: ImpactReport,
    pub execution_plan: ExecutionPlan,
    pub explorer_report: AgentExecutionReport,
    pub impact_report_execution: AgentExecutionReport,
    pub planner_report: AgentExecutionReport,
}

#[derive(Debug, Clone)]
pub struct CognitiveExecutionResult {
    pub planning: CognitivePipelineResult,
    pub coder_output: CodeGenerationOutput,
    pub review_report: ReviewReport,
    pub tester_report: TesterExecutionReport,
    pub coder_report: AgentExecutionReport,
    pub reviewer_execution: AgentExecutionReport,
    pub tester_execution: AgentExecutionReport,
}

pub struct CognitivePipeline {
    explorer: Explorer,
    impact_analyzer: ImpactAnalyzer,
    planner: Planner,
    coder: Option<Coder>,
    reviewer: Option<Reviewer>,
    tester: Option<Tester>,
}

impl CognitivePipeline {
    pub fn new(explorer: Explorer, impact_analyzer: ImpactAnalyzer, planner: Planner) -> Self {
        Self {
            explorer,
            impact_analyzer,
            planner,
            coder: None,
            reviewer: None,
            tester: None,
        }
    }

    pub fn with_execution_agents(
        mut self,
        coder: Coder,
        reviewer: Reviewer,
        tester: Tester,
    ) -> Self {
        self.coder = Some(coder);
        self.reviewer = Some(reviewer);
        self.tester = Some(tester);
        self
    }

    pub async fn execute(
        &self,
        task: &TaskDescription,
        shared: &SharedResources,
    ) -> Result<CognitivePipelineResult, AgentError> {
        let handoff = Arc::new(AgentHandoff::new());

        let explorer_ctx = self
            .explorer
            .build_context(task, None, shared)
            .await?
            .with_handoff(Arc::clone(&handoff));
        let explorer_report = self.explorer.execute(&explorer_ctx).await?;
        let project_context = handoff.get::<ProjectContext>().await.ok_or_else(|| {
            AgentError::MissingContextData {
                data_type: "ProjectContext".to_string(),
            }
        })?;

        let impact_ctx = self
            .impact_analyzer
            .build_context(task, Some(project_context.clone()), shared)
            .await?
            .with_handoff(Arc::clone(&handoff));
        let impact_report_execution = self.impact_analyzer.execute(&impact_ctx).await?;
        let impact_report =
            handoff
                .get::<ImpactReport>()
                .await
                .ok_or_else(|| AgentError::MissingContextData {
                    data_type: "ImpactReport".to_string(),
                })?;

        let planner_ctx = self
            .planner
            .build_context(task, Some(project_context.clone()), shared)
            .await?
            .with_handoff(handoff)
            .with_impact_report(impact_report.clone());
        let planner_report = self.planner.execute(&planner_ctx).await?;
        let execution_plan = planner_ctx
            .handoff
            .get::<ExecutionPlan>()
            .await
            .ok_or_else(|| AgentError::MissingContextData {
                data_type: "ExecutionPlan".to_string(),
            })?;

        Ok(CognitivePipelineResult {
            project_context,
            impact_report,
            execution_plan,
            explorer_report,
            impact_report_execution,
            planner_report,
        })
    }

    pub async fn execute_full(
        &self,
        task: &TaskDescription,
        shared: &SharedResources,
    ) -> Result<CognitiveExecutionResult, AgentError> {
        let (coder, reviewer, tester) = self.execution_agents()?;
        let planning = self.execute(task, shared).await?;
        let handoff = Arc::new(AgentHandoff::new());
        handoff.put(planning.project_context.clone()).await;
        handoff.put(planning.impact_report.clone()).await;
        handoff.put(planning.execution_plan.clone()).await;

        let coder_ctx = coder
            .build_context(task, Some(planning.project_context.clone()), shared)
            .await?
            .with_handoff(Arc::clone(&handoff))
            .with_impact_report(planning.impact_report.clone())
            .with_execution_plan(planning.execution_plan.clone());
        let coder_report = coder.execute(&coder_ctx).await?;
        let coder_output = handoff.get::<CodeGenerationOutput>().await.ok_or_else(|| {
            AgentError::MissingContextData {
                data_type: "CodeGenerationOutput".to_string(),
            }
        })?;

        let reviewer_ctx = reviewer
            .build_context(task, Some(planning.project_context.clone()), shared)
            .await?
            .with_handoff(Arc::clone(&handoff))
            .with_impact_report(planning.impact_report.clone())
            .with_execution_plan(planning.execution_plan.clone());
        let reviewer_execution = reviewer.execute(&reviewer_ctx).await?;
        let review_report =
            handoff
                .get::<ReviewReport>()
                .await
                .ok_or_else(|| AgentError::MissingContextData {
                    data_type: "ReviewReport".to_string(),
                })?;

        let tester_ctx = tester
            .build_context(task, Some(planning.project_context.clone()), shared)
            .await?
            .with_handoff(handoff)
            .with_impact_report(planning.impact_report.clone())
            .with_execution_plan(planning.execution_plan.clone());
        let tester_execution = tester.execute(&tester_ctx).await?;
        let tester_report = tester_ctx
            .handoff
            .get::<TesterExecutionReport>()
            .await
            .ok_or_else(|| AgentError::MissingContextData {
                data_type: "TesterExecutionReport".to_string(),
            })?;

        Ok(CognitiveExecutionResult {
            planning,
            coder_output,
            review_report,
            tester_report,
            coder_report,
            reviewer_execution,
            tester_execution,
        })
    }

    fn execution_agents(&self) -> Result<(&Coder, &Reviewer, &Tester), AgentError> {
        let coder = self.coder.as_ref().ok_or_else(|| AgentError::Validation {
            message: "Coder is not configured on the cognitive pipeline".to_string(),
        })?;
        let reviewer = self
            .reviewer
            .as_ref()
            .ok_or_else(|| AgentError::Validation {
                message: "Reviewer is not configured on the cognitive pipeline".to_string(),
            })?;
        let tester = self.tester.as_ref().ok_or_else(|| AgentError::Validation {
            message: "Tester is not configured on the cognitive pipeline".to_string(),
        })?;
        Ok((coder, reviewer, tester))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use mc_core::{AgentType, TaskDescription};
    use mc_llm::ResponseFormat;

    use super::CognitivePipeline;
    use crate::test_support::{create_test_project, MockLlmProvider};
    use crate::{
        AgentConfig, Coder, Explorer, ImpactAnalyzer, Planner, Reviewer, SharedResources, Tester,
    };

    #[tokio::test]
    async fn pipeline_runs_end_to_end_with_json_schema_requests() {
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
        ])));
        let requests = provider.requests();
        let shared = SharedResources::new(project.path(), provider);
        let pipeline = CognitivePipeline::new(
            Explorer::new(AgentConfig::for_agent_type(AgentType::Explorer)),
            ImpactAnalyzer::new(AgentConfig::for_agent_type(AgentType::ImpactAnalyzer)),
            Planner::new(AgentConfig::for_agent_type(AgentType::Planner)),
        );

        let mut task = TaskDescription::with_root("update core behavior", project.path());
        task.affected_files = vec!["core/src/lib.rs".to_string()];
        task.requires_testing = true;

        let result = pipeline.execute(&task, &shared).await.expect("pipeline");
        assert!(result.project_context.structure.total_files > 0);
        assert_eq!(
            result.impact_report.direct_impacts[0].file,
            "core/src/lib.rs"
        );
        assert!(!result.execution_plan.parallel_groups.is_empty());

        let requests = requests.lock().expect("requests");
        assert_eq!(requests.len(), 3);
        for request in requests.iter() {
            assert!(matches!(
                request.response_format,
                Some(ResponseFormat::JsonSchema { strict: true, .. })
            ));
        }
    }

    #[tokio::test]
    async fn pipeline_execute_full_runs_coder_reviewer_and_tester() {
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
                    "summary": "Adjust compute behavior in core",
                    "implementation_notes": ["Change the implementation in core", "Keep downstream contract stable"],
                    "changes": [{
                        "path": "core/src/lib.rs",
                        "change_kind": "modify",
                        "rationale": "The task directly targets the compute implementation",
                        "patch_preview": "",
                        "acceptance_checks": ["cargo test"]
                    }],
                    "validation_steps": ["cargo test"],
                    "risks": ["Verify the app crate still compiles against the updated interface"]
                })
                .to_string(),
            ),
            (
                "reviewer_assessment".to_string(),
                serde_json::json!({
                    "summary": "Review completed with attention on core-to-app compatibility.",
                    "verdict": "approved",
                    "additional_findings": []
                })
                .to_string(),
            ),
        ])));
        let requests = provider.requests();
        let shared = SharedResources::new(project.path(), provider);
        let pipeline = CognitivePipeline::new(
            Explorer::new(AgentConfig::for_agent_type(AgentType::Explorer)),
            ImpactAnalyzer::new(AgentConfig::for_agent_type(AgentType::ImpactAnalyzer)),
            Planner::new(AgentConfig::for_agent_type(AgentType::Planner)),
        )
        .with_execution_agents(
            Coder::new(AgentConfig::for_agent_type(AgentType::Coder)),
            Reviewer::new(AgentConfig::for_agent_type(AgentType::Reviewer)),
            Tester::new(AgentConfig::for_agent_type(AgentType::Tester)),
        );

        let mut task = TaskDescription::with_root("update core behavior", project.path());
        task.affected_files = vec!["core/src/lib.rs".to_string()];
        task.requires_testing = true;

        let result = pipeline
            .execute_full(&task, &shared)
            .await
            .expect("full pipeline");
        assert_eq!(result.coder_output.changes[0].path, "core/src/lib.rs");
        assert!(matches!(
            result.review_report.verdict,
            crate::ReviewVerdict::Approved | crate::ReviewVerdict::NeedsChanges
        ));
        assert_eq!(result.tester_report.framework, crate::TestFramework::Cargo);
        assert!(result.tester_report.summary.success);
        assert!(result.tester_execution.summary.contains("Tester passed"));

        let requests = requests.lock().expect("requests");
        assert_eq!(requests.len(), 5);
    }
}
