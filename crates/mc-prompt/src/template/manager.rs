use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::error::PromptCacheError;
use crate::layer::PromptLayer;
use crate::template::definition::{PromptTemplate, TemplateVariable};
use crate::template::renderer::{extract_template_variables, TemplateRenderer};
use crate::watcher::is_supported_prompt_file;

pub struct TemplateManager {
    root_dir: PathBuf,
    templates: Arc<RwLock<HashMap<String, PromptTemplate>>>,
    renderer: TemplateRenderer,
}

impl TemplateManager {
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
            templates: Arc::new(RwLock::new(HashMap::new())),
            renderer: TemplateRenderer::new(),
        }
    }

    pub async fn load_all(&self) -> Result<usize, PromptCacheError> {
        let mut templates = self.templates.write().await;
        templates.clear();

        for directory in ["system", "tools", "org", "project"] {
            self.load_templates_from_dir(&self.root_dir.join(directory), &mut templates)
                .await?;
        }

        Ok(templates.len())
    }

    pub async fn render_template(
        &self,
        template_id: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, PromptCacheError> {
        let templates = self.templates.read().await;
        let template = templates
            .get(template_id)
            .ok_or_else(|| PromptCacheError::TemplateNotFound(template_id.to_string()))?;
        self.renderer.render(&template.raw_content, context)
    }

    pub async fn get_template(&self, template_id: &str) -> Option<PromptTemplate> {
        let templates = self.templates.read().await;
        templates.get(template_id).cloned()
    }

    async fn load_templates_from_dir(
        &self,
        dir: &Path,
        templates: &mut HashMap<String, PromptTemplate>,
    ) -> Result<(), PromptCacheError> {
        if !dir.exists() {
            return Ok(());
        }

        let mut stack = vec![dir.to_path_buf()];
        while let Some(current_dir) = stack.pop() {
            let mut entries = tokio::fs::read_dir(&current_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let metadata = entry.metadata().await?;

                if metadata.is_dir() {
                    stack.push(path);
                    continue;
                }

                if !is_supported_prompt_file(&path) {
                    continue;
                }

                let content = tokio::fs::read_to_string(&path).await?;
                let variable_names = extract_template_variables(&content)?;
                let relative_path = path
                    .strip_prefix(&self.root_dir)
                    .unwrap_or(&path)
                    .to_path_buf();
                let id = relative_path
                    .with_extension("")
                    .to_string_lossy()
                    .replace('\\', "/");

                let variables = variable_names
                    .into_iter()
                    .map(|name| TemplateVariable {
                        description: format!("template variable '{name}'"),
                        name,
                        required: true,
                        default_value: None,
                    })
                    .collect::<Vec<_>>();

                templates.insert(
                    id.clone(),
                    PromptTemplate::new(
                        id,
                        infer_template_layer(&relative_path),
                        relative_path.to_string_lossy().replace('\\', "/"),
                        content,
                        variables,
                    ),
                );
            }
        }

        Ok(())
    }
}

fn infer_template_layer(path: &Path) -> PromptLayer {
    let components = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_ascii_lowercase())
        .collect::<Vec<_>>();

    if components
        .iter()
        .any(|part| part == "system" || part == "tools")
    {
        PromptLayer::Global
    } else if components
        .iter()
        .any(|part| part == "org" || part == "organization")
    {
        PromptLayer::Organization
    } else if components.iter().any(|part| part == "project") {
        PromptLayer::Project
    } else {
        PromptLayer::Session
    }
}
