pub mod change_type;

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mc_core::{AgentType, ChangeType, ProjectContext, RiskLevel, TaskDescription};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::impact_analyzer::change_type::infer_change_type;
use crate::support::complete_json;
use crate::{Agent, AgentConfig, AgentContext, AgentError, AgentExecutionReport, SharedResources};

#[derive(Debug, Clone)]
pub struct ImpactAnalyzer {
    config: AgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImpactChange {
    pub file: String,
    pub change_type: ChangeType,
    pub description: String,
    pub affected_symbols: Vec<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskAssessment {
    pub description: String,
    pub level: RiskLevel,
    pub mitigation: String,
    pub affected_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImpactReport {
    pub direct_impacts: Vec<ImpactChange>,
    pub indirect_impacts: Vec<ImpactChange>,
    pub risk_assessment: Vec<RiskAssessment>,
    pub compatibility_notes: Vec<String>,
    pub recommendations: Vec<String>,
    pub overall_risk_level: RiskLevel,
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct ImpactLlmSummary {
    #[serde(default)]
    compatibility_notes: Vec<String>,
    #[serde(default)]
    recommendations: Vec<String>,
    #[serde(default)]
    risk_assessment: Vec<RiskAssessment>,
}

impl ImpactAnalyzer {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    fn resolve_direct_files(
        &self,
        task: &TaskDescription,
        project_ctx: &ProjectContext,
    ) -> Vec<String> {
        if !task.affected_files.is_empty() {
            return task.affected_files.clone();
        }
        let lowered = task.user_input.to_lowercase();
        let mut files = project_ctx
            .structure
            .modules
            .iter()
            .filter(|module| lowered.contains(&module.name.to_lowercase()))
            .map(|module| module.path.clone())
            .collect::<Vec<_>>();
        if files.is_empty() {
            files = project_ctx.structure.entry_files.clone();
        }
        if files.is_empty() {
            files = project_ctx
                .structure
                .modules
                .iter()
                .take(1)
                .map(|module| module.path.clone())
                .collect();
        }
        files
    }

    fn parse_symbols(root: &Path, relative_path: &str) -> Vec<String> {
        let Ok(content) = fs::read_to_string(root.join(relative_path)) else {
            return Vec::new();
        };
        let regex =
            Regex::new(r"(?:pub\s+)?(?:async\s+)?(?:fn|struct|enum|trait)\s+([A-Za-z0-9_]+)")
                .expect("regex");
        regex
            .captures_iter(&content)
            .filter_map(|capture| capture.get(1))
            .map(|capture| capture.as_str().to_string())
            .take(8)
            .collect()
    }

    fn module_for_path(path: &str, project_ctx: &ProjectContext) -> String {
        project_ctx
            .structure
            .modules
            .iter()
            .find(|module| path.starts_with(&module.path))
            .map(|module| module.path.clone())
            .unwrap_or_else(|| path.to_string())
    }

    fn dependents(project_ctx: &ProjectContext) -> HashMap<String, Vec<String>> {
        let mut reverse = HashMap::<String, Vec<String>>::new();
        for edge in &project_ctx.dependency_graph.edges {
            reverse
                .entry(edge.to.clone())
                .or_default()
                .push(edge.from.clone());
        }
        reverse
    }

    fn risk_for_path(path: &str, dependent_count: usize, task: &TaskDescription) -> RiskLevel {
        let mut risk = if path.ends_with(".md") || path.contains("/tests/") {
            RiskLevel::Low
        } else if path.ends_with("Cargo.toml")
            || path.ends_with("Cargo.lock")
            || dependent_count > 0
        {
            RiskLevel::High
        } else {
            RiskLevel::Medium
        };
        if task.requires_new_dependency || task.involves_architecture_change {
            risk = RiskLevel::max(risk, RiskLevel::High);
        }
        risk
    }

    fn deterministic_report(
        &self,
        ctx: &AgentContext,
        project_ctx: &ProjectContext,
    ) -> ImpactReport {
        let root = Path::new(&project_ctx.root_path);
        let direct_files = self.resolve_direct_files(&ctx.task, project_ctx);
        let dependents = Self::dependents(project_ctx);
        let mut direct_impacts = Vec::new();
        let mut indirect_impacts = Vec::new();
        let mut seen = HashSet::new();

        for file in &direct_files {
            let module = Self::module_for_path(file, project_ctx);
            let downstream = dependents.get(&module).cloned().unwrap_or_default();
            let risk = Self::risk_for_path(file, downstream.len(), &ctx.task);
            direct_impacts.push(ImpactChange {
                file: file.clone(),
                change_type: infer_change_type(file),
                description: format!("Directly impacted target `{file}`"),
                affected_symbols: Self::parse_symbols(root, file),
                risk_level: risk,
            });

            let mut queue = VecDeque::from(downstream);
            while let Some(module) = queue.pop_front() {
                if !seen.insert(module.clone()) {
                    continue;
                }
                let next = dependents.get(&module).cloned().unwrap_or_default();
                queue.extend(next.clone());
                indirect_impacts.push(ImpactChange {
                    file: module.clone(),
                    change_type: ChangeType::ModifyFile,
                    description: format!(
                        "Downstream module `{module}` depends on a changed target"
                    ),
                    affected_symbols: Vec::new(),
                    risk_level: if next.is_empty() {
                        RiskLevel::Medium
                    } else {
                        RiskLevel::High
                    },
                });
            }
        }

        let mut overall = RiskLevel::Low;
        for change in direct_impacts.iter().chain(indirect_impacts.iter()) {
            overall = RiskLevel::max(overall, change.risk_level);
        }

        let mut risk_assessment = Vec::new();
        for level in [
            RiskLevel::Low,
            RiskLevel::Medium,
            RiskLevel::High,
            RiskLevel::Critical,
        ] {
            let files = direct_impacts
                .iter()
                .chain(indirect_impacts.iter())
                .filter(|change| change.risk_level == level)
                .map(|change| change.file.clone())
                .collect::<Vec<_>>();
            if files.is_empty() {
                continue;
            }
            risk_assessment.push(RiskAssessment {
                description: format!("{} item(s) classified as {:?} risk", files.len(), level),
                level,
                mitigation: match level {
                    RiskLevel::Low => "Basic regression check is enough".to_string(),
                    RiskLevel::Medium => "Review touched APIs and run focused tests".to_string(),
                    RiskLevel::High => {
                        "Review dependency edges and verify callers before merge".to_string()
                    }
                    RiskLevel::Critical => {
                        "Require explicit approval and staged rollout".to_string()
                    }
                },
                affected_files: files,
            });
        }

        ImpactReport {
            direct_impacts,
            indirect_impacts,
            risk_assessment,
            compatibility_notes: Vec::new(),
            recommendations: Vec::new(),
            overall_risk_level: overall,
            analyzed_at: Utc::now(),
        }
    }

    async fn enrich(
        &self,
        ctx: &AgentContext,
        project_ctx: &ProjectContext,
        report: &mut ImpactReport,
    ) -> Result<u32, AgentError> {
        let prompt = format!(
            "Task: {}\nRoot: {}\nDirect impacts: {}\nIndirect impacts: {}\nOverall risk: {:?}",
            ctx.task.user_input,
            project_ctx.root_path,
            report.direct_impacts.len(),
            report.indirect_impacts.len(),
            report.overall_risk_level
        );
        let (summary, tokens): (ImpactLlmSummary, u32) = complete_json(
            ctx.llm_provider.as_ref(),
            &ctx.config.llm_config.model_id,
            "Analyze compatibility and mitigation risk. Return strict JSON.",
            &prompt,
            "impact_assessment",
            json!({
                "type":"object",
                "additionalProperties":false,
                "required":["compatibility_notes","recommendations","risk_assessment"],
                "properties":{
                    "compatibility_notes":{"type":"array","items":{"type":"string"}},
                    "recommendations":{"type":"array","items":{"type":"string"}},
                    "risk_assessment":{"type":"array","items":{
                        "type":"object",
                        "additionalProperties":false,
                        "required":["description","level","mitigation","affected_files"],
                        "properties":{
                            "description":{"type":"string"},
                            "level":{"type":"string","enum":["Low","Medium","High","Critical"]},
                            "mitigation":{"type":"string"},
                            "affected_files":{"type":"array","items":{"type":"string"}}
                        }
                    }}
                }
            }),
            ctx.config.llm_config.temperature,
            ctx.config.llm_config.max_output_tokens,
            ctx.cancel_token.child_token(),
        )
        .await?;
        report.compatibility_notes = summary.compatibility_notes;
        report.recommendations = summary.recommendations;
        report.risk_assessment.extend(summary.risk_assessment);
        Ok(tokens)
    }
}

#[async_trait]
impl Agent for ImpactAnalyzer {
    fn agent_type(&self) -> AgentType {
        AgentType::ImpactAnalyzer
    }

    fn default_config(&self) -> AgentConfig {
        AgentConfig::for_agent_type(AgentType::ImpactAnalyzer)
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
        let project_ctx = project_ctx.ok_or_else(|| AgentError::MissingContextData {
            data_type: "ProjectContext".to_string(),
        })?;
        Ok(AgentContext::new(task.clone(), shared, self.config.clone())
            .with_project_ctx(project_ctx))
    }

    async fn execute(&self, ctx: &AgentContext) -> Result<AgentExecutionReport, AgentError> {
        let project_ctx = if let Some(project_ctx) = ctx.project_ctx.as_deref().cloned() {
            project_ctx
        } else {
            ctx.handoff.get::<ProjectContext>().await.ok_or_else(|| {
                AgentError::MissingContextData {
                    data_type: "ProjectContext".to_string(),
                }
            })?
        };
        let mut report = self.deterministic_report(ctx, &project_ctx);
        let llm_tokens = self.enrich(ctx, &project_ctx, &mut report).await?;
        ctx.handoff.put(report.clone()).await;
        let result = serde_json::to_value(&report).map_err(AgentError::serialization)?;
        Ok(AgentExecutionReport::success(
            AgentType::ImpactAnalyzer,
            &ctx.execution_id,
            format!(
                "Impact analyzer found {} direct and {} indirect impacts",
                report.direct_impacts.len(),
                report.indirect_impacts.len()
            ),
            result,
            ctx.elapsed_ms(),
            llm_tokens,
        ))
    }
}
