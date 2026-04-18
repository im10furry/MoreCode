use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use mc_core::{AgentType, ProjectContext, TaskDescription};
use regex::Regex;
use serde_json::{json, Value};
use tokio::task::JoinSet;

use crate::execution_report::{build_report, serialize_extra};
use crate::{Agent, AgentConfig, AgentContext, AgentError, SharedResources};

pub mod format;
pub use format::{DocumentType, Documentation, GeneratedDocument};

#[async_trait]
pub trait TemplateEngine: Send + Sync {
    async fn render(&self, template_name: &str, context: &Value) -> Result<String, AgentError>;
}

#[derive(Debug, Clone)]
pub struct SimpleTemplateEngine {
    templates: HashMap<String, String>,
}

impl SimpleTemplateEngine {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        templates.insert(
            "readme".to_string(),
            "# {{project_name}}\n\n{{project_description}}\n\n## Overview\n- Language: {{language}}\n- Framework: {{framework}}\n- Version: {{project_version}}\n\n## Structure\n{{directory_tree}}\n\n## Key Modules\n{{module_summaries}}\n\n## Development Notes\n{{development_notes}}\n".to_string(),
        );
        templates.insert(
            "api".to_string(),
            "# API Reference\n\n## Project\n{{project_name}}\n\n## Public Modules\n{{module_exports}}\n\n## Interfaces\n{{interface_summary}}\n\n## Notes\n{{api_notes}}\n".to_string(),
        );
        templates.insert(
            "changelog".to_string(),
            "# Changelog\n\n## {{current_date}}\n- {{change_summary}}\n".to_string(),
        );
        templates.insert(
            "contributing".to_string(),
            "# Contributing\n\n## Workflow\n{{development_notes}}\n\n## Code Conventions\n{{conventions_summary}}\n\n## Testing\n{{testing_summary}}\n".to_string(),
        );
        Self { templates }
    }

    pub fn with_template(mut self, name: impl Into<String>, content: impl Into<String>) -> Self {
        self.templates.insert(name.into(), content.into());
        self
    }
}

impl Default for SimpleTemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TemplateEngine for SimpleTemplateEngine {
    async fn render(&self, template_name: &str, context: &Value) -> Result<String, AgentError> {
        let template =
            self.templates
                .get(template_name)
                .ok_or_else(|| AgentError::TemplateError {
                    message: format!("template `{template_name}` not found"),
                })?;
        render_template(template, context)
    }
}

#[derive(Clone)]
pub struct DocWriter {
    config: AgentConfig,
    template_engine: Arc<dyn TemplateEngine>,
    file_write_tool: String,
}

impl DocWriter {
    pub fn new(config: AgentConfig, template_engine: Arc<dyn TemplateEngine>) -> Self {
        Self {
            config,
            template_engine,
            file_write_tool: "file_write".to_string(),
        }
    }

    pub fn with_file_write_tool(mut self, file_write_tool: impl Into<String>) -> Self {
        self.file_write_tool = file_write_tool.into();
        self
    }

    async fn generate_document(
        &self,
        ctx: AgentContext,
        doc_type: DocumentType,
    ) -> Result<GeneratedDocument, AgentError> {
        let context = self.build_template_context(&ctx, doc_type);
        let template_name = doc_type.template_name();
        let content =
            if let Some(override_template) = ctx.config.template_overrides.get(template_name) {
                render_template(override_template, &context)?
            } else {
                self.template_engine.render(template_name, &context).await?
            };

        let path = destination_path(&ctx, doc_type);
        self.write_document(&ctx, &path, &content).await?;

        Ok(GeneratedDocument {
            doc_type,
            path: path.to_string_lossy().to_string(),
            summary: summarize_document(&content),
        })
    }

    async fn write_document(
        &self,
        ctx: &AgentContext,
        path: &std::path::Path,
        content: &str,
    ) -> Result<(), AgentError> {
        if ctx.has_tool(&self.file_write_tool).await {
            ctx.call_tool(
                AgentType::DocWriter,
                &self.file_write_tool,
                json!({
                    "path": path.to_string_lossy(),
                    "content": content,
                    "create_dirs": true,
                }),
            )
            .await?;
            return Ok(());
        }

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|error| AgentError::Io {
                    path: parent.to_string_lossy().to_string(),
                    reason: error.to_string(),
                })?;
        }

        tokio::fs::write(path, content)
            .await
            .map_err(|error| AgentError::Io {
                path: path.to_string_lossy().to_string(),
                reason: error.to_string(),
            })?;
        Ok(())
    }

    fn build_template_context(&self, ctx: &AgentContext, doc_type: DocumentType) -> Value {
        let project = ctx.project_ctx.as_ref();
        let project_name = project
            .map(|project| project.project_info.name.clone())
            .unwrap_or_else(|| "MoreCode Project".to_string());
        let project_description = project
            .map(|project| project.project_info.description.clone())
            .unwrap_or_else(|| ctx.task.user_input.clone());
        let project_version = project
            .and_then(|project| project.project_info.version.clone())
            .unwrap_or_else(|| "0.1.0".to_string());
        let language = project
            .map(|project| project.project_info.language.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let framework = project
            .and_then(|project| project.project_info.framework.clone())
            .unwrap_or_else(|| "none".to_string());
        let directory_tree = project
            .map(|project| project.structure.directory_tree.clone())
            .unwrap_or_else(|| "- project structure unavailable".to_string());
        let module_summaries = project
            .map(|project| {
                project
                    .structure
                    .modules
                    .iter()
                    .map(|module| format!("- {}: {}", module.name, module.description))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "- no modules discovered".to_string());
        let module_exports = project
            .map(|project| {
                project
                    .structure
                    .modules
                    .iter()
                    .map(|module| {
                        format!(
                            "- {}: {}",
                            module.name,
                            if module.exports.is_empty() {
                                "no explicit exports".to_string()
                            } else {
                                module.exports.join(", ")
                            }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "- no exported interfaces found".to_string());
        let development_notes = build_development_notes(project, &ctx.task.user_input);
        let conventions_summary = project
            .map(|project| {
                let rules = &project.conventions.custom_rules;
                if rules.is_empty() {
                    "Follow the project naming, error-handling, and documentation conventions."
                        .to_string()
                } else {
                    rules.join("; ")
                }
            })
            .unwrap_or_else(|| {
                "Document changes together with implementation changes.".to_string()
            });
        let testing_summary = project
            .map(|project| {
                format!(
                    "Tests use `{}` and follow `{}` naming.",
                    project.conventions.testing.test_attribute,
                    project.conventions.testing.naming_pattern
                )
            })
            .unwrap_or_else(|| {
                "Add or update regression tests for every user-visible change.".to_string()
            });
        let api_notes = match doc_type {
            DocumentType::Api => {
                "Keep this document aligned with public module exports and configuration changes."
            }
            _ => "Keep documentation aligned with the codebase.",
        };
        let change_summary = build_change_summary(ctx);
        let interface_summary = project
            .map(|project| {
                project
                    .structure
                    .modules
                    .iter()
                    .map(|module| format!("- {} interfaces: {}", module.name, module.exports.len()))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "- interface inventory unavailable".to_string());

        json!({
            "project_name": project_name,
            "project_description": project_description,
            "project_version": project_version,
            "language": language,
            "framework": framework,
            "directory_tree": directory_tree,
            "module_summaries": module_summaries,
            "module_exports": module_exports,
            "development_notes": development_notes,
            "conventions_summary": conventions_summary,
            "testing_summary": testing_summary,
            "current_date": chrono::Utc::now().format("%Y-%m-%d").to_string(),
            "change_summary": change_summary,
            "api_notes": api_notes,
            "interface_summary": interface_summary,
        })
    }

    fn resolve_document_types(&self, ctx: &AgentContext) -> Vec<DocumentType> {
        if let Some(hints) = ctx.get_metadata::<Vec<String>>("doc_types") {
            let resolved = hints
                .into_iter()
                .filter_map(|hint| DocumentType::from_hint(&hint))
                .collect::<Vec<_>>();
            if !resolved.is_empty() {
                return dedupe_doc_types(resolved);
            }
        }

        let task_lower = ctx.task.user_input.to_lowercase();
        let mut resolved = Vec::new();
        if task_lower.contains("readme") {
            resolved.push(DocumentType::Readme);
        }
        if task_lower.contains("api") {
            resolved.push(DocumentType::Api);
        }
        if task_lower.contains("changelog") {
            resolved.push(DocumentType::Changelog);
        }
        if task_lower.contains("contributing")
            || task_lower.contains("贡献")
            || task_lower.contains("contribution")
        {
            resolved.push(DocumentType::Contributing);
        }
        if resolved.is_empty() {
            resolved.extend([
                DocumentType::Readme,
                DocumentType::Api,
                DocumentType::Contributing,
            ]);
        }
        dedupe_doc_types(resolved)
    }
}

#[async_trait]
impl Agent for DocWriter {
    fn agent_type(&self) -> AgentType {
        AgentType::DocWriter
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn default_config(&self) -> AgentConfig {
        AgentConfig::for_agent_type(AgentType::DocWriter)
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
        let doc_types = self.resolve_document_types(ctx);
        let mut join_set = JoinSet::new();
        for doc_type in doc_types
            .into_iter()
            .take(ctx.config.max_parallel_tasks.max(1))
        {
            let agent = self.clone();
            let ctx = ctx.clone();
            join_set.spawn(async move { agent.generate_document(ctx, doc_type).await });
        }

        let mut generated = Vec::new();
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(document)) => generated.push(document),
                Ok(Err(error)) => {
                    return Err(error);
                }
                Err(error) => {
                    return Err(AgentError::Internal {
                        message: error.to_string(),
                    });
                }
            }
        }

        generated.sort_by(|left, right| left.path.cmp(&right.path));

        let documentation = Documentation {
            files_created: generated.iter().map(|doc| doc.path.clone()).collect(),
            content_summary: generated
                .iter()
                .map(|doc| format!("{} -> {}", doc.doc_type.as_str(), doc.summary))
                .collect::<Vec<_>>()
                .join("\n"),
            generated_documents: generated,
        };

        ctx.handoff.put(documentation.clone()).await;

        Ok(build_report(
            AgentType::DocWriter,
            "documentation generated",
            documentation
                .generated_documents
                .iter()
                .map(|doc| format!("{} written to {}", doc.doc_type.as_str(), doc.path))
                .collect(),
            documentation.files_created.clone(),
            vec!["Review generated documents before publishing.".to_string()],
            Vec::new(),
            documentation.files_created.len() as u32,
            Some(serialize_extra(&documentation)?),
        ))
    }
}

fn render_template(template: &str, context: &Value) -> Result<String, AgentError> {
    let regex = Regex::new(r"\{\{\s*([a-zA-Z0-9_]+)\s*\}\}").map_err(|error| {
        AgentError::TemplateError {
            message: error.to_string(),
        }
    })?;

    Ok(regex
        .replace_all(template, |captures: &regex::Captures<'_>| {
            let key = captures
                .get(1)
                .map(|capture| capture.as_str())
                .unwrap_or_default();
            context
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        })
        .to_string())
}

fn destination_path(ctx: &AgentContext, doc_type: DocumentType) -> std::path::PathBuf {
    let root = ctx
        .project_root()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    match doc_type {
        DocumentType::Readme => root.join(doc_type.default_file_name()),
        DocumentType::Changelog => root.join(doc_type.default_file_name()),
        DocumentType::Contributing => root.join(doc_type.default_file_name()),
        DocumentType::Api => ctx.output_root().join(doc_type.default_file_name()),
    }
}

fn summarize_document(content: &str) -> String {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" | ")
}

fn build_change_summary(ctx: &AgentContext) -> String {
    if ctx.task.affected_files.is_empty() {
        return ctx.task.user_input.clone();
    }

    format!(
        "{}; affected files: {}",
        ctx.task.user_input,
        ctx.task.affected_files.join(", ")
    )
}

fn build_development_notes(
    project: Option<&Arc<mc_core::ProjectContext>>,
    user_input: &str,
) -> String {
    if let Some(project) = project {
        let framework = project
            .project_info
            .framework
            .clone()
            .unwrap_or_else(|| "project defaults".to_string());
        return format!(
            "Work in `{framework}` style and keep docs aligned with task: {user_input}"
        );
    }

    format!("Describe how to develop and validate the change requested by: {user_input}")
}

fn dedupe_doc_types(values: Vec<DocumentType>) -> Vec<DocumentType> {
    let mut seen = std::collections::HashSet::new();
    values
        .into_iter()
        .filter(|item| seen.insert(*item))
        .collect()
}
