use std::collections::BTreeSet;

use mc_core::{ExecutionPlan, ProjectContext, TaskDescription};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodeChangeKind {
    Add,
    Modify,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeChangeDraft {
    pub path: String,
    pub change_kind: CodeChangeKind,
    pub rationale: String,
    #[serde(default)]
    pub patch_preview: String,
    #[serde(default)]
    pub acceptance_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeGenerationOutput {
    pub summary: String,
    #[serde(default)]
    pub implementation_notes: Vec<String>,
    #[serde(default)]
    pub changes: Vec<CodeChangeDraft>,
    #[serde(default)]
    pub validation_steps: Vec<String>,
    #[serde(default)]
    pub risks: Vec<String>,
}

impl CodeGenerationOutput {
    pub fn ensure_consistency(
        &mut self,
        task: &TaskDescription,
        execution_plan: Option<&ExecutionPlan>,
    ) {
        if self.summary.trim().is_empty() {
            self.summary = format!("Implement task: {}", task.user_input);
        }

        if self.changes.is_empty() {
            self.changes = fallback_output(task, execution_plan).changes;
        }

        if self.validation_steps.is_empty() {
            self.validation_steps = self
                .changes
                .iter()
                .flat_map(|change| change.acceptance_checks.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
        }
        if self.validation_steps.is_empty() {
            self.validation_steps
                .push("Run focused regression checks for the impacted files".to_string());
        }

        for change in &mut self.changes {
            if change.acceptance_checks.is_empty() {
                change.acceptance_checks = self.validation_steps.clone();
            }
            if change.patch_preview.trim().is_empty() {
                change.patch_preview =
                    render_patch_preview(&change.path, change.change_kind, &change.rationale);
            }
        }
    }
}

pub(crate) fn build_prompt(
    task: &TaskDescription,
    project_ctx: Option<&ProjectContext>,
    impact_report: Option<&crate::ImpactReport>,
    execution_plan: Option<&ExecutionPlan>,
) -> String {
    let project_summary = project_ctx
        .map(|project| {
            format!(
                "project={} language={} modules={} conventions={}",
                project.project_info.name,
                project.project_info.language,
                project.structure.modules.len(),
                project.conventions.custom_rules.join("; ")
            )
        })
        .unwrap_or_else(|| "project context unavailable".to_string());
    let plan_summary = execution_plan
        .map(|plan| {
            format!(
                "plan_id={} summary={} subtasks={} groups={}",
                plan.plan_id,
                plan.summary,
                plan.sub_tasks.len(),
                plan.parallel_groups.len()
            )
        })
        .unwrap_or_else(|| "execution plan unavailable".to_string());
    let impact_summary = impact_report
        .map(|impact| {
            format!(
                "overall_risk={:?}; direct_impacts={}; indirect_impacts={}",
                impact.overall_risk_level,
                impact.direct_impacts.len(),
                impact.indirect_impacts.len()
            )
        })
        .unwrap_or_else(|| "impact report unavailable".to_string());
    let target_files = infer_target_files(task, execution_plan);

    format!(
        "Task: {}\nIntent: {:?}\nComplexity: {:?}\nAffected files: {}\nConstraints: {}\nProject: {}\nPlan: {}\nImpact: {}\nReturn concrete code change drafts with acceptance checks.",
        task.user_input,
        task.intent,
        task.complexity,
        target_files.join(", "),
        if task.constraints.is_empty() {
            "none".to_string()
        } else {
            task.constraints.join("; ")
        },
        project_summary,
        plan_summary,
        impact_summary,
    )
}

pub(crate) fn fallback_output(
    task: &TaskDescription,
    execution_plan: Option<&ExecutionPlan>,
) -> CodeGenerationOutput {
    let files = infer_target_files(task, execution_plan);
    let validation_steps = default_validation_steps(&files);
    let changes = files
        .into_iter()
        .map(|path| {
            let change_kind = infer_change_kind(&path);
            let rationale = format!("Implement the requested behavior in `{path}`");
            CodeChangeDraft {
                patch_preview: render_patch_preview(&path, change_kind, &rationale),
                acceptance_checks: validation_steps.clone(),
                path,
                change_kind,
                rationale,
            }
        })
        .collect();

    CodeGenerationOutput {
        summary: format!("Prepare implementation drafts for {}", task.user_input),
        implementation_notes: vec![
            "Keep the change scoped to the impacted files".to_string(),
            "Preserve public interfaces expected by downstream modules".to_string(),
        ],
        changes,
        validation_steps,
        risks: Vec::new(),
    }
}

pub(crate) fn infer_target_files(
    task: &TaskDescription,
    execution_plan: Option<&ExecutionPlan>,
) -> Vec<String> {
    let mut files = BTreeSet::new();
    if let Some(plan) = execution_plan {
        for task in &plan.sub_tasks {
            for file in &task.target_files {
                if !file.trim().is_empty() {
                    files.insert(file.clone());
                }
            }
        }
    }

    if files.is_empty() {
        for file in &task.affected_files {
            if !file.trim().is_empty() {
                files.insert(file.clone());
            }
        }
    }

    if files.is_empty() {
        files.insert("src/lib.rs".to_string());
    }

    files.into_iter().collect()
}

pub(crate) fn infer_change_kind(path: &str) -> CodeChangeKind {
    let lower = path.to_lowercase();
    if lower.ends_with("mod.rs") || lower.ends_with("lib.rs") || lower.ends_with("main.rs") {
        CodeChangeKind::Modify
    } else if lower.ends_with(".md") || lower.ends_with(".toml") || lower.ends_with(".json") {
        CodeChangeKind::Modify
    } else {
        CodeChangeKind::Modify
    }
}

fn default_validation_steps(files: &[String]) -> Vec<String> {
    if files.iter().any(|path| path.ends_with(".rs") || path.ends_with("Cargo.toml")) {
        return vec!["cargo test".to_string()];
    }
    if files.iter().any(|path| path.ends_with(".py")) {
        return vec!["pytest".to_string()];
    }
    if files
        .iter()
        .any(|path| path.ends_with(".ts") || path.ends_with(".tsx") || path.ends_with("package.json"))
    {
        return vec!["npm test".to_string()];
    }
    vec!["Run focused regression checks for the touched files".to_string()]
}

fn comment_prefix(path: &str) -> &'static str {
    if path.ends_with(".py") || path.ends_with(".sh") {
        "#"
    } else {
        "//"
    }
}

fn render_patch_preview(path: &str, change_kind: CodeChangeKind, rationale: &str) -> String {
    let prefix = comment_prefix(path);
    let action = match change_kind {
        CodeChangeKind::Add => "add",
        CodeChangeKind::Modify => "modify",
        CodeChangeKind::Delete => "delete",
    };
    format!(
        "--- a/{path}\n+++ b/{path}\n@@\n- {prefix} existing implementation\n+ {prefix} {action}: {rationale}"
    )
}

#[cfg(test)]
mod tests {
    use mc_core::TaskDescription;

    use super::{fallback_output, infer_target_files, CodeGenerationOutput};

    #[test]
    fn infer_target_files_prefers_plan_then_task_then_default() {
        let task = TaskDescription::simple("implement feature");
        let files = infer_target_files(&task, None);
        assert_eq!(files, vec!["src/lib.rs".to_string()]);
    }

    #[test]
    fn ensure_consistency_backfills_missing_previews_and_validation() {
        let task = TaskDescription::simple("update implementation");
        let mut output = CodeGenerationOutput {
            summary: String::new(),
            implementation_notes: Vec::new(),
            changes: fallback_output(&task, None).changes,
            validation_steps: Vec::new(),
            risks: Vec::new(),
        };

        output.ensure_consistency(&task, None);
        assert!(!output.summary.is_empty());
        assert!(!output.validation_steps.is_empty());
        assert!(output
            .changes
            .iter()
            .all(|change| !change.patch_preview.is_empty()));
    }
}
