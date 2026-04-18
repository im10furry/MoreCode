use std::sync::Arc;

use mc_core::{ExecutionPlan, ProjectContext, TaskDescription};

use crate::{
    Agent, AgentError, AgentExecutionReport, AgentHandoff, Explorer, ImpactAnalyzer, ImpactReport,
    Planner, SharedResources,
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

pub struct CognitivePipeline {
    explorer: Explorer,
    impact_analyzer: ImpactAnalyzer,
    planner: Planner,
}

impl CognitivePipeline {
    pub fn new(explorer: Explorer, impact_analyzer: ImpactAnalyzer, planner: Planner) -> Self {
        Self {
            explorer,
            impact_analyzer,
            planner,
        }
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use mc_core::{AgentType, TaskDescription};
    use mc_llm::ResponseFormat;

    use super::CognitivePipeline;
    use crate::test_support::{create_test_project, MockLlmProvider};
    use crate::{AgentConfig, Explorer, ImpactAnalyzer, Planner, SharedResources};

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
        task.affected_files = vec!["crates/core/src/lib.rs".to_string()];
        task.requires_testing = true;

        let result = pipeline.execute(&task, &shared).await.expect("pipeline");
        assert!(result.project_context.structure.total_files > 0);
        assert_eq!(
            result.impact_report.direct_impacts[0].file,
            "crates/core/src/lib.rs"
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
}
