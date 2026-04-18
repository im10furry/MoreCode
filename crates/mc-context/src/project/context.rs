use serde::{Deserialize, Serialize};

use crate::project::{
    CodeConventions, ImpactReport, ProjectInfo, RiskArea, ScanMetadata, TechStack,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectContext {
    pub info: ProjectInfo,
    pub tech_stack: TechStack,
    pub conventions: CodeConventions,
    pub risk_areas: Vec<RiskArea>,
    pub scan_metadata: ScanMetadata,
    pub impact_report: Option<ImpactReport>,
    pub notes: Vec<String>,
}

impl ProjectContext {
    pub fn to_markdown(&self) -> String {
        let mut sections = vec![format!("# {}", self.info.name)];

        if let Some(summary) = &self.info.summary {
            sections.push(summary.clone());
        }

        if !self.tech_stack.languages.is_empty() {
            sections.push(format!(
                "## Tech Stack\n- {}",
                self.tech_stack.languages.join("\n- ")
            ));
        }

        if !self.notes.is_empty() {
            sections.push(format!("## Notes\n- {}", self.notes.join("\n- ")));
        }

        sections.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectContext;
    use crate::project::{ProjectInfo, TechStack};

    #[test]
    fn markdown_contains_core_sections() {
        let context = ProjectContext {
            info: ProjectInfo {
                name: "MoreCode".into(),
                summary: Some("Agent workspace".into()),
                ..ProjectInfo::default()
            },
            tech_stack: TechStack {
                languages: vec!["Rust".into()],
                ..TechStack::default()
            },
            notes: vec!["Keep platform info in prompt".into()],
            ..ProjectContext::default()
        };

        let markdown = context.to_markdown();
        assert!(markdown.contains("# MoreCode"));
        assert!(markdown.contains("## Tech Stack"));
        assert!(markdown.contains("## Notes"));
    }
}
