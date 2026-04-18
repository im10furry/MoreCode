pub mod strategy;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Instant;

use async_trait::async_trait;
use chrono::Utc;
use ignore::WalkBuilder;
use mc_core::{
    AgentType, ArchitectureLayer, ArchitecturePattern, CodeConventions, DependencyEdge,
    DependencyGraph, DocumentationConvention, ErrorHandlingPattern, ModuleInfo, NamingConvention,
    ProjectContext, ProjectInfo, ProjectStructure, RiskAreas, ScanMetadata, TaskDescription,
    TechStack, TestingConvention,
};
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use tokio::task;

use crate::explorer::strategy::{CachedFileRecord, ScanCache, ScanMode};
use crate::support::complete_json;
use crate::{Agent, AgentConfig, AgentContext, AgentError, AgentExecutionReport, SharedResources};

#[derive(Debug, Clone)]
pub struct Explorer {
    config: AgentConfig,
}

#[derive(Debug, Clone, Deserialize)]
struct ExplorerLlmSummary {
    project_summary: String,
    architecture_name: String,
    architecture_description: String,
    design_decisions: Vec<String>,
    notable_patterns: Vec<String>,
}

impl Explorer {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    async fn scan_project(&self, root: &Path) -> Result<ProjectContext, AgentError> {
        let root = root.to_path_buf();
        let config = self.config.clone();
        task::spawn_blocking(move || Self::scan_sync(root, config))
            .await
            .map_err(|e| AgentError::ExecutionFailed {
                agent_type: AgentType::Explorer,
                message: e.to_string(),
            })?
    }

    fn scan_sync(root: PathBuf, config: AgentConfig) -> Result<ProjectContext, AgentError> {
        let started = Instant::now();
        let memory_dir = root.join(".assistant-memory");
        fs::create_dir_all(&memory_dir)
            .map_err(|e| AgentError::io(memory_dir.display().to_string(), e))?;
        let cache_path = memory_dir.join("explorer-cache.json");
        let cache = if cache_path.exists() {
            let payload = fs::read_to_string(&cache_path)
                .map_err(|e| AgentError::io(cache_path.display().to_string(), e))?;
            Some(serde_json::from_str::<ScanCache>(&payload).map_err(AgentError::serialization)?)
        } else {
            None
        };

        let files = Self::collect_files(
            &root,
            config.explorer.max_files,
            config.explorer.max_file_size_bytes,
        )?;
        let mut records = BTreeMap::new();
        let mut changed = Vec::new();
        for file in files {
            let rel = Self::norm(
                file.strip_prefix(&root)
                    .map_err(|e| AgentError::io(file.display().to_string(), e))?,
            );
            let hash = Self::hash_file(&file)?;
            if let Some(old) = cache.as_ref().and_then(|cache| cache.records.get(&rel)) {
                if old.hash == hash {
                    records.insert(rel, old.clone());
                    continue;
                }
            }
            changed.push(rel.clone());
            records.insert(rel, Self::analyze_file(&root, &file, hash)?);
        }
        if let Some(cache) = &cache {
            for path in cache.records.keys() {
                if !records.contains_key(path) {
                    changed.push(path.clone());
                }
            }
        }
        changed.sort();
        changed.dedup();

        let cache_fresh = cache
            .as_ref()
            .map(|cache| {
                (Utc::now() - cache.saved_at).num_seconds() <= config.explorer.cache_ttl_secs as i64
            })
            .unwrap_or(false);
        if changed.is_empty() && cache_fresh {
            return cache.map(|cache| cache.project_context).ok_or_else(|| {
                AgentError::ContextBuildFailed {
                    message: "cache missing".to_string(),
                }
            });
        }

        let (facts, framework) = Self::cargo_facts(&root)?;
        let modules = Self::modules(&records, &facts);
        let graph = Self::graph(&modules);
        let primary_language = if records.values().any(|record| record.language == "Rust") {
            "Rust".to_string()
        } else {
            "Unknown".to_string()
        };
        let total_lines = records.values().map(|record| record.line_count).sum();
        let scan_mode = if !changed.is_empty()
            && (changed.len() as f32 / records.len().max(1) as f32)
                <= config.explorer.incremental_change_threshold
        {
            ScanMode::Incremental {
                changed_files: changed.clone(),
            }
        } else {
            ScanMode::Full
        };

        let context = ProjectContext {
            project_info: ProjectInfo {
                name: root
                    .file_name()
                    .map(|v| v.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "workspace".to_string()),
                description: "Scanned project workspace".to_string(),
                version: None,
                language: primary_language.clone(),
                framework: framework.clone(),
                license: None,
                repository_url: None,
            },
            structure: ProjectStructure {
                directory_tree: Self::tree(
                    records.keys().cloned().collect(),
                    config.explorer.max_tree_depth,
                ),
                total_files: records.len(),
                total_lines,
                entry_files: records
                    .values()
                    .filter(|record| record.is_entry)
                    .map(|record| record.relative_path.clone())
                    .collect(),
                config_files: records
                    .values()
                    .filter(|record| record.is_config)
                    .map(|record| record.relative_path.clone())
                    .collect(),
                modules: modules.clone(),
            },
            tech_stack: TechStack {
                language_version: facts
                    .get("rust-toolchain")
                    .cloned()
                    .unwrap_or_else(|| primary_language.clone()),
                rust_edition: facts.get("edition").cloned(),
                framework,
                database: None,
                orm: None,
                auth: None,
                build_tool: Some("cargo".to_string()),
                package_manager: Some("cargo".to_string()),
                dependencies: facts.clone(),
                dev_dependencies: HashMap::new(),
                updated_at: Utc::now(),
            },
            architecture: ArchitecturePattern {
                name: "Multi-crate workspace".to_string(),
                description: format!(
                    "{primary_language} workspace with {} modules",
                    modules.len()
                ),
                layers: vec![ArchitectureLayer {
                    name: "Workspace".to_string(),
                    responsibility: "Cargo workspace modules".to_string(),
                    modules: modules.iter().map(|module| module.path.clone()).collect(),
                    depends_on: graph.edges.iter().map(|edge| edge.to.clone()).collect(),
                }],
                design_decisions: vec!["Respect .gitignore during scanning".to_string()],
            },
            dependency_graph: graph,
            conventions: CodeConventions {
                naming: NamingConvention {
                    function: "snake_case".to_string(),
                    struct_: "PascalCase".to_string(),
                    constant: "SCREAMING_SNAKE_CASE".to_string(),
                    module: "snake_case".to_string(),
                    enum_variant: "PascalCase".to_string(),
                    type_parameter: "T/U".to_string(),
                },
                error_handling: ErrorHandlingPattern {
                    custom_error_type: records.keys().any(|path| path.contains("error")),
                    error_type_name: None,
                    unwrap_policy: if records
                        .values()
                        .any(|record| record.risk_markers.iter().any(|marker| marker == "unwrap"))
                    {
                        "Prefer Result propagation over unwrap()".to_string()
                    } else {
                        "Prefer Result propagation".to_string()
                    },
                    propagation: "Result + ?".to_string(),
                    error_crate: Some("thiserror/anyhow-style".to_string()),
                },
                testing: TestingConvention {
                    naming_pattern: "descriptive snake_case".to_string(),
                    test_attribute: "#[test] / #[tokio::test]".to_string(),
                    test_file_location: "tests/ and inline modules".to_string(),
                    mock_framework: None,
                    coverage_threshold: None,
                },
                documentation: DocumentationConvention {
                    language: "Markdown + Rustdoc".to_string(),
                    format: "Markdown/Rustdoc".to_string(),
                    code_example_style: "inline fenced code blocks".to_string(),
                    api_doc_tool: Some("rustdoc".to_string()),
                },
                custom_rules: vec!["Treat Cargo.toml changes as cross-cutting".to_string()],
            },
            risk_areas: RiskAreas { items: Vec::new() },
            scan_metadata: ScanMetadata {
                scanned_at: Utc::now(),
                files_scanned: records.len(),
                total_lines,
                scan_duration_ms: started.elapsed().as_millis() as u64,
                memory_version: 1,
                scanner_version: "agent.explorer.v1".to_string(),
                scan_mode: scan_mode.label(),
                changed_files: scan_mode.changed_files(),
            },
            root_path: root.to_string_lossy().into_owned(),
        };

        let cache = ScanCache {
            version: 1,
            saved_at: Utc::now(),
            root_path: context.root_path.clone(),
            records,
            project_context: context.clone(),
        };
        let payload = serde_json::to_string_pretty(&cache).map_err(AgentError::serialization)?;
        fs::write(&cache_path, payload)
            .map_err(|e| AgentError::io(cache_path.display().to_string(), e))?;
        Ok(context)
    }

    fn collect_files(
        root: &Path,
        max_files: usize,
        max_size: u64,
    ) -> Result<Vec<PathBuf>, AgentError> {
        let mut builder = WalkBuilder::new(root);
        builder
            .hidden(false)
            .parents(true)
            .git_ignore(true)
            .git_exclude(true)
            .git_global(true)
            .standard_filters(true);
        let mut files = Vec::new();
        for entry in builder.build() {
            let Ok(entry) = entry else { continue };
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let path = entry.into_path();
            let rel = Self::norm(
                path.strip_prefix(root)
                    .map_err(|e| AgentError::io(path.display().to_string(), e))?,
            );
            if rel.starts_with(".git/")
                || rel.starts_with(".assistant-memory/")
                || rel.starts_with("target/")
                || rel.starts_with("node_modules/")
            {
                continue;
            }
            let keep = Self::is_manifest(&rel)
                || matches!(
                    Path::new(&rel)
                        .extension()
                        .and_then(|v| v.to_str())
                        .unwrap_or_default(),
                    "rs" | "toml" | "md" | "json"
                );
            if !keep {
                continue;
            }
            let meta =
                fs::metadata(&path).map_err(|e| AgentError::io(path.display().to_string(), e))?;
            if meta.len() > max_size && !Self::is_manifest(&rel) {
                continue;
            }
            files.push(path);
            if files.len() >= max_files {
                break;
            }
        }
        Ok(files)
    }

    fn analyze_file(root: &Path, file: &Path, hash: u64) -> Result<CachedFileRecord, AgentError> {
        let rel = Self::norm(
            file.strip_prefix(root)
                .map_err(|e| AgentError::io(file.display().to_string(), e))?,
        );
        let content = String::from_utf8_lossy(
            &fs::read(file).map_err(|e| AgentError::io(file.display().to_string(), e))?,
        )
        .into_owned();
        let use_re = Regex::new(r"use\s+([a-zA-Z0-9_]+)").expect("regex");
        let export_re =
            Regex::new(r"pub\s+(?:async\s+)?(?:fn|struct|enum|trait)\s+([A-Za-z0-9_]+)")
                .expect("regex");
        Ok(CachedFileRecord {
            relative_path: rel.clone(),
            hash,
            size_bytes: content.len() as u64,
            line_count: content.lines().count(),
            language: if rel.ends_with(".rs") {
                "Rust".to_string()
            } else {
                "Other".to_string()
            },
            module: Self::module(&rel),
            is_entry: rel.ends_with("/src/main.rs") || rel == "src/main.rs",
            is_config: Self::is_manifest(&rel) || rel.ends_with(".toml"),
            dependencies: use_re
                .captures_iter(&content)
                .filter_map(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            exports: export_re
                .captures_iter(&content)
                .filter_map(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            risk_markers: ["unsafe", "todo", "fixme", "unwrap("]
                .into_iter()
                .filter(|n| content.to_lowercase().contains(*n))
                .map(|n| if n == "unwrap(" { "unwrap" } else { n })
                .map(str::to_string)
                .collect(),
        })
    }

    fn cargo_facts(root: &Path) -> Result<(HashMap<String, String>, Option<String>), AgentError> {
        let root_manifest = root.join("Cargo.toml");
        let payload = fs::read_to_string(&root_manifest)
            .map_err(|e| AgentError::io(root_manifest.display().to_string(), e))?;
        let value: toml::Value = toml::from_str(&payload).map_err(AgentError::serialization)?;
        let members = value
            .get("workspace")
            .and_then(|v| v.get("members"))
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut facts = HashMap::new();
        let mut framework = None;
        if root.join("rust-toolchain.toml").exists() {
            let payload = fs::read_to_string(root.join("rust-toolchain.toml"))
                .map_err(|e| AgentError::io("rust-toolchain.toml", e))?;
            let value: toml::Value = toml::from_str(&payload).map_err(AgentError::serialization)?;
            if let Some(channel) = value
                .get("toolchain")
                .and_then(|t| t.get("channel"))
                .and_then(toml::Value::as_str)
            {
                facts.insert("rust-toolchain".to_string(), channel.to_string());
            }
        }
        for member in members.iter().filter_map(toml::Value::as_str) {
            let manifest = root.join(member).join("Cargo.toml");
            let payload = fs::read_to_string(&manifest)
                .map_err(|e| AgentError::io(manifest.display().to_string(), e))?;
            let value: toml::Value = toml::from_str(&payload).map_err(AgentError::serialization)?;
            if let Some(edition) = value
                .get("package")
                .and_then(|p| p.get("edition"))
                .and_then(toml::Value::as_str)
            {
                facts
                    .entry("edition".to_string())
                    .or_insert_with(|| edition.to_string());
            }
            if let Some(table) = value.get("dependencies").and_then(toml::Value::as_table) {
                for (name, dep) in table {
                    let version = dep
                        .as_str()
                        .map(ToOwned::to_owned)
                        .unwrap_or_else(|| "workspace".to_string());
                    facts.insert(name.clone(), version);
                    if framework.is_none()
                        && ["axum", "actix-web", "ratatui", "tauri", "bevy"]
                            .contains(&name.as_str())
                    {
                        framework = Some(name.clone());
                    }
                }
            }
        }
        Ok((facts, framework))
    }

    fn modules(
        records: &BTreeMap<String, CachedFileRecord>,
        facts: &HashMap<String, String>,
    ) -> Vec<ModuleInfo> {
        let mut grouped: HashMap<String, Vec<&CachedFileRecord>> = HashMap::new();
        for record in records.values() {
            grouped
                .entry(record.module.clone())
                .or_default()
                .push(record);
        }
        let mut modules = grouped
            .into_iter()
            .map(|(path, files)| ModuleInfo {
                name: path.rsplit('/').next().unwrap_or(&path).to_string(),
                path,
                description: "Discovered module".to_string(),
                exports: files
                    .iter()
                    .flat_map(|f| f.exports.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
                dependencies: files
                    .iter()
                    .flat_map(|f| f.dependencies.clone())
                    .filter(|d| facts.contains_key(d))
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
                file_count: files.len(),
                line_count: files.iter().map(|f| f.line_count).sum(),
            })
            .collect::<Vec<_>>();
        modules.sort_by(|a, b| a.path.cmp(&b.path));
        modules
    }

    fn graph(modules: &[ModuleInfo]) -> DependencyGraph {
        let by_name = modules
            .iter()
            .map(|m| (m.name.clone(), m.path.clone()))
            .collect::<HashMap<_, _>>();
        let edges = modules
            .iter()
            .flat_map(|m| {
                m.dependencies.iter().filter_map(|d| {
                    by_name.get(d).map(|to| DependencyEdge {
                        from: m.path.clone(),
                        to: to.clone(),
                        relation_type: "compile".to_string(),
                        weight: Some(1.0),
                    })
                })
            })
            .collect::<Vec<_>>();
        DependencyGraph {
            nodes: modules.iter().map(|m| m.path.clone()).collect(),
            edges,
            circular_dependencies: Vec::new(),
        }
    }

    fn tree(paths: Vec<String>, max_depth: usize) -> String {
        let mut paths = paths;
        paths.sort();
        let mut lines = vec![".".to_string()];
        for path in paths {
            let segs = path.split('/').collect::<Vec<_>>();
            let depth = segs.len().min(max_depth);
            lines.push(format!(
                "{}- {}",
                "  ".repeat(depth.saturating_sub(1)),
                segs[..depth].join("/")
            ));
        }
        lines.join("\n")
    }

    fn hash_file(path: &Path) -> Result<u64, AgentError> {
        let bytes = fs::read(path).map_err(|e| AgentError::io(path.display().to_string(), e))?;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        bytes.hash(&mut hasher);
        Ok(hasher.finish())
    }

    fn module(rel: &str) -> String {
        if let Some(rest) = rel.strip_prefix("crates/") {
            if let Some(name) = rest.split('/').next() {
                return name.to_string();
            }
        }
        rel.split('/').next().unwrap_or(rel).to_string()
    }

    fn is_manifest(rel: &str) -> bool {
        matches!(
            rel,
            "Cargo.toml" | "Cargo.lock" | ".gitignore" | "README.md" | "rust-toolchain.toml"
        ) || rel.ends_with("/Cargo.toml")
    }

    fn norm(path: &Path) -> String {
        path.to_string_lossy().replace('\\', "/")
    }

    async fn enrich(
        &self,
        ctx: &AgentContext,
        project_ctx: &mut ProjectContext,
    ) -> Result<u32, AgentError> {
        let prompt = format!(
            "name: {}\nfiles: {}\nmodules: {}\ntree:\n{}",
            project_ctx.project_info.name,
            project_ctx.structure.total_files,
            project_ctx
                .structure
                .modules
                .iter()
                .map(|m| m.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            project_ctx.structure.directory_tree
        );
        let (summary, tokens): (ExplorerLlmSummary, u32) = complete_json(
            ctx.llm_provider.as_ref(),
            &ctx.config.llm_config.model_id,
            "Summarize the scanned project and return strict JSON.",
            &prompt,
            "explorer_summary",
            json!({"type":"object","additionalProperties":false,"required":["project_summary","architecture_name","architecture_description","design_decisions","notable_patterns"],"properties":{"project_summary":{"type":"string"},"architecture_name":{"type":"string"},"architecture_description":{"type":"string"},"design_decisions":{"type":"array","items":{"type":"string"}},"notable_patterns":{"type":"array","items":{"type":"string"}}}}),
            ctx.config.llm_config.temperature,
            ctx.config.llm_config.max_output_tokens,
            ctx.cancel_token.child_token(),
        ).await?;
        project_ctx.project_info.description = summary.project_summary;
        project_ctx.architecture.name = summary.architecture_name;
        project_ctx.architecture.description = summary.architecture_description;
        project_ctx
            .architecture
            .design_decisions
            .extend(summary.design_decisions);
        project_ctx
            .conventions
            .custom_rules
            .extend(summary.notable_patterns);
        Ok(tokens)
    }
}

#[async_trait]
impl Agent for Explorer {
    fn agent_type(&self) -> AgentType {
        AgentType::Explorer
    }
    fn supports_streaming(&self) -> bool {
        true
    }
    fn default_config(&self) -> AgentConfig {
        AgentConfig::for_agent_type(AgentType::Explorer)
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
        if let Some(project_ctx) = project_ctx {
            return Ok(AgentContext::new(task.clone(), shared, self.config.clone())
                .with_project_ctx(project_ctx));
        }
        let root = task
            .project_root_path()
            .unwrap_or_else(|| shared.project_root.clone());
        let project_ctx = self.scan_project(&root).await?;
        Ok(AgentContext::new(task.clone(), shared, self.config.clone())
            .with_project_ctx(project_ctx))
    }

    async fn execute(&self, ctx: &AgentContext) -> Result<AgentExecutionReport, AgentError> {
        let mut project_ctx =
            ctx.project_ctx
                .as_deref()
                .cloned()
                .ok_or_else(|| AgentError::MissingContextData {
                    data_type: "ProjectContext".to_string(),
                })?;
        let tokens = self.enrich(ctx, &mut project_ctx).await?;
        ctx.handoff.put(project_ctx.clone()).await;
        let result = serde_json::to_value(&project_ctx).map_err(AgentError::serialization)?;
        Ok(AgentExecutionReport::success(
            AgentType::Explorer,
            &ctx.execution_id,
            format!(
                "Explorer scanned {} files in {}",
                project_ctx.structure.total_files, project_ctx.scan_metadata.scan_mode
            ),
            result,
            ctx.elapsed_ms(),
            tokens,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use mc_core::{AgentType, TaskDescription};
    use mc_llm::ResponseFormat;

    use super::Explorer;
    use crate::test_support::{create_test_project, MockLlmProvider};
    use crate::{Agent, AgentConfig, SharedResources};

    #[tokio::test]
    async fn explorer_marks_incremental_scan_and_uses_json_schema() {
        let project = create_test_project();
        let responses = HashMap::from([(
            "explorer_summary".to_string(),
            serde_json::json!({
                "project_summary": "Incremental workspace",
                "architecture_name": "Workspace",
                "architecture_description": "Two crates linked through Cargo dependencies",
                "design_decisions": ["Use cached scan output"],
                "notable_patterns": ["cargo-workspace"]
            })
            .to_string(),
        )]);
        let provider = Arc::new(MockLlmProvider::new(responses));
        let requests = provider.requests();
        let shared = SharedResources::new(project.path(), provider);
        let explorer = Explorer::new(AgentConfig::for_agent_type(AgentType::Explorer));
        let task = TaskDescription::with_root("scan", project.path());

        let first = explorer
            .build_context(&task, None, &shared)
            .await
            .expect("first ctx");
        explorer.execute(&first).await.expect("first run");

        std::fs::write(
            project.path().join("core/src/lib.rs"),
            "pub fn compute() -> usize { 7 }\n",
        )
        .expect("rewrite");

        let second = explorer
            .build_context(&task, None, &shared)
            .await
            .expect("second ctx");
        explorer.execute(&second).await.expect("second run");
        let project_ctx = second
            .handoff
            .get::<mc_core::ProjectContext>()
            .await
            .expect("handoff");
        assert_eq!(project_ctx.scan_metadata.scan_mode, "incremental");
        assert!(project_ctx
            .scan_metadata
            .changed_files
            .iter()
            .any(|path| path == "core/src/lib.rs"));

        let requests = requests.lock().expect("requests");
        assert_eq!(requests.len(), 2);
        assert!(matches!(
            requests[0].response_format,
            Some(ResponseFormat::JsonSchema { strict: true, .. })
        ));
    }
}
