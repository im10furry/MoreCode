use std::collections::BTreeSet;

use mc_core::{ExecutionPlan, ProjectContext, TaskDescription};
use serde::{Deserialize, Serialize};

use crate::coder::codegen::CodeGenerationOutput;
use crate::ImpactReport;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ReviewSeverity {
    Blocker,
    Warning,
    Suggestion,
    Info,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    Approved,
    NeedsChanges,
    Rejected,
}

impl ReviewVerdict {
    pub fn max(left: Self, right: Self) -> Self {
        if left >= right {
            left
        } else {
            right
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewFinding {
    pub severity: ReviewSeverity,
    pub title: String,
    pub detail: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewReport {
    pub verdict: ReviewVerdict,
    pub summary: String,
    pub reviewed_files: Vec<String>,
    pub findings: Vec<ReviewFinding>,
}

impl ReviewReport {
    pub fn recompute_verdict(&mut self) {
        self.verdict = self
            .findings
            .iter()
            .fold(ReviewVerdict::Approved, |acc, finding| {
                ReviewVerdict::max(acc, verdict_for_severity(finding.severity))
            });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewRuleSet {
    pub require_acceptance_checks: bool,
    pub require_target_alignment: bool,
    pub flag_unwrap_usage: bool,
    pub require_validation_steps: bool,
    pub require_high_risk_notes: bool,
}

impl Default for ReviewRuleSet {
    fn default() -> Self {
        Self {
            require_acceptance_checks: true,
            require_target_alignment: true,
            flag_unwrap_usage: true,
            require_validation_steps: true,
            require_high_risk_notes: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ReviewRuleEngine {
    rules: ReviewRuleSet,
}

#[derive(Debug, Clone, Copy)]
pub struct ReviewInput<'a> {
    pub task: &'a TaskDescription,
    pub project_ctx: Option<&'a ProjectContext>,
    pub impact_report: Option<&'a ImpactReport>,
    pub execution_plan: Option<&'a ExecutionPlan>,
    pub codegen: Option<&'a CodeGenerationOutput>,
}

impl ReviewRuleEngine {
    pub fn evaluate(&self, input: &ReviewInput<'_>) -> ReviewReport {
        let expected_files = expected_files(input.task, input.execution_plan);
        let reviewed_files = reviewed_files(input.codegen, &expected_files);
        let mut findings = Vec::new();

        if input.codegen.is_none() {
            findings.push(ReviewFinding {
                severity: ReviewSeverity::Blocker,
                title: "missing code generation output".to_string(),
                detail: "Reviewer did not receive any Coder output from the handoff.".to_string(),
                recommendation: "Run the Coder agent before requesting review.".to_string(),
            });
        }

        if let Some(codegen) = input.codegen {
            if self.rules.require_acceptance_checks {
                for change in &codegen.changes {
                    if change.acceptance_checks.is_empty() {
                        findings.push(ReviewFinding {
                            severity: ReviewSeverity::Warning,
                            title: format!("missing acceptance checks for {}", change.path),
                            detail:
                                "The generated change draft does not include any acceptance checks."
                                    .to_string(),
                            recommendation:
                                "Add at least one focused verification step per changed file."
                                    .to_string(),
                        });
                    }
                }
            }

            if self.rules.require_validation_steps && codegen.validation_steps.is_empty() {
                findings.push(ReviewFinding {
                    severity: ReviewSeverity::Warning,
                    title: "missing validation steps".to_string(),
                    detail: "The code generation output does not define any project-level validation step.".to_string(),
                    recommendation: "Add a focused test or verification command before merge.".to_string(),
                });
            }

            if self.rules.flag_unwrap_usage
                && codegen
                    .changes
                    .iter()
                    .any(|change| change.patch_preview.contains("unwrap("))
            {
                findings.push(ReviewFinding {
                    severity: ReviewSeverity::Warning,
                    title: "unwrap usage introduced in patch preview".to_string(),
                    detail: "Generated patch preview appears to introduce `unwrap()`.".to_string(),
                    recommendation: "Prefer propagating errors with `?` or explicit handling."
                        .to_string(),
                });
            }

            if self.rules.require_target_alignment {
                let changed = codegen
                    .changes
                    .iter()
                    .map(|change| change.path.clone())
                    .collect::<BTreeSet<_>>();
                let missing = expected_files
                    .iter()
                    .filter(|file| !changed.contains(*file))
                    .cloned()
                    .collect::<Vec<_>>();
                if !missing.is_empty() {
                    findings.push(ReviewFinding {
                        severity: ReviewSeverity::Warning,
                        title: "expected files are not covered by the change set".to_string(),
                        detail: format!("Missing review coverage for: {}", missing.join(", ")),
                        recommendation: "Either add explicit change drafts for the missing files or explain why they are untouched.".to_string(),
                    });
                }
            }
        }

        if self.rules.require_high_risk_notes
            && input.impact_report.is_some_and(|impact| {
                impact.overall_risk_level.score() >= mc_core::RiskLevel::High.score()
            })
            && input
                .codegen
                .is_some_and(|codegen| codegen.risks.is_empty())
        {
            findings.push(ReviewFinding {
                severity: ReviewSeverity::Suggestion,
                title: "high-risk change lacks explicit risk notes".to_string(),
                detail: "Impact analysis marked this change as high risk, but the Coder output did not include any risk note.".to_string(),
                recommendation: "Document rollback or validation concerns for the risky area.".to_string(),
            });
        }

        if let Some(project_ctx) = input.project_ctx {
            if project_ctx
                .conventions
                .custom_rules
                .iter()
                .any(|rule| rule.to_lowercase().contains("cargo.toml"))
                && reviewed_files
                    .iter()
                    .any(|file| file.ends_with("Cargo.toml"))
                && input.codegen.is_some_and(|codegen| {
                    !codegen
                        .validation_steps
                        .iter()
                        .any(|step| step.contains("cargo"))
                })
            {
                findings.push(ReviewFinding {
                    severity: ReviewSeverity::Suggestion,
                    title: "manifest change should be followed by cargo validation".to_string(),
                    detail: "Project conventions mention Cargo changes as cross-cutting, but no cargo-oriented validation step was proposed.".to_string(),
                    recommendation: "Add `cargo test` or another cargo-based verification step.".to_string(),
                });
            }
        }

        let verdict = findings
            .iter()
            .fold(ReviewVerdict::Approved, |acc, finding| {
                ReviewVerdict::max(acc, verdict_for_severity(finding.severity))
            });

        ReviewReport {
            verdict,
            summary: if findings.is_empty() {
                "Review completed without actionable findings.".to_string()
            } else {
                format!("Review completed with {} finding(s).", findings.len())
            },
            reviewed_files,
            findings,
        }
    }
}

fn verdict_for_severity(severity: ReviewSeverity) -> ReviewVerdict {
    match severity {
        ReviewSeverity::Blocker => ReviewVerdict::Rejected,
        ReviewSeverity::Warning => ReviewVerdict::NeedsChanges,
        ReviewSeverity::Suggestion | ReviewSeverity::Info => ReviewVerdict::Approved,
    }
}

fn expected_files(task: &TaskDescription, execution_plan: Option<&ExecutionPlan>) -> Vec<String> {
    let mut files = BTreeSet::new();
    if let Some(plan) = execution_plan {
        for sub_task in &plan.sub_tasks {
            for file in &sub_task.target_files {
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
    files.into_iter().collect()
}

fn reviewed_files(codegen: Option<&CodeGenerationOutput>, fallback: &[String]) -> Vec<String> {
    if let Some(codegen) = codegen {
        let files = codegen
            .changes
            .iter()
            .map(|change| change.path.clone())
            .collect::<BTreeSet<_>>();
        if !files.is_empty() {
            return files.into_iter().collect();
        }
    }
    fallback.to_vec()
}

#[cfg(test)]
mod tests {
    use mc_core::TaskDescription;

    use super::{ReviewInput, ReviewRuleEngine, ReviewSeverity, ReviewVerdict};
    use crate::coder::codegen::{CodeChangeDraft, CodeChangeKind, CodeGenerationOutput};

    #[test]
    fn rule_engine_rejects_missing_codegen() {
        let task = TaskDescription::simple("review code");
        let engine = ReviewRuleEngine::default();
        let report = engine.evaluate(&ReviewInput {
            task: &task,
            project_ctx: None,
            impact_report: None,
            execution_plan: None,
            codegen: None,
        });

        assert_eq!(report.verdict, ReviewVerdict::Rejected);
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.severity == ReviewSeverity::Blocker));
    }

    #[test]
    fn rule_engine_warns_when_checks_are_missing() {
        let task = TaskDescription::simple("review code");
        let output = CodeGenerationOutput {
            summary: "change".to_string(),
            implementation_notes: Vec::new(),
            changes: vec![CodeChangeDraft {
                path: "src/lib.rs".to_string(),
                change_kind: CodeChangeKind::Modify,
                rationale: "update implementation".to_string(),
                patch_preview: String::new(),
                acceptance_checks: Vec::new(),
            }],
            validation_steps: Vec::new(),
            risks: Vec::new(),
        };
        let engine = ReviewRuleEngine::default();
        let report = engine.evaluate(&ReviewInput {
            task: &task,
            project_ctx: None,
            impact_report: None,
            execution_plan: None,
            codegen: Some(&output),
        });

        assert_eq!(report.verdict, ReviewVerdict::NeedsChanges);
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.title.contains("acceptance checks")));
    }
}
