use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    fs,
    io::AsyncWriteExt,
    process::Command,
    sync::{mpsc, oneshot, RwLock},
};
use tracing::info;

use crate::error::MemoryError;

use super::types::{
    ApiEndpoints, DataModels, DependencyGraph, FileChange, MemoryUpdate, MemoryWriteRequest,
    MetaJson, ModuleInfo, ModuleMap, ProjectMemory, ProjectMemorySnapshot, RiskAreas, RiskInfo,
    TechStack,
};

const HISTORY_FILE: &str = "change-history.jsonl";

#[async_trait]
pub trait MemoryManagerTrait: Send + Sync {
    async fn load_memory(&self) -> anyhow::Result<ProjectMemory>;
    async fn incremental_update(&self) -> anyhow::Result<ProjectMemory>;
    async fn agent_update(&self, update: MemoryUpdate) -> anyhow::Result<()>;
    async fn get_memory_summary(&self) -> anyhow::Result<String>;
}

#[derive(Debug)]
pub struct MemoryManager {
    memory_dir: PathBuf,
    project_dir: PathBuf,
    write_tx: mpsc::Sender<MemoryWriteRequest>,
    current_memory: RwLock<Option<ProjectMemory>>,
    stale_threshold_days: i64,
}

#[derive(Debug, Default)]
struct ProjectStats {
    total_files: usize,
    total_lines: usize,
    project_hash: String,
}

impl MemoryManager {
    pub fn new(project_dir: &Path, write_tx: mpsc::Sender<MemoryWriteRequest>) -> Self {
        Self {
            memory_dir: project_dir.join(".assistant-memory"),
            project_dir: project_dir.to_path_buf(),
            write_tx,
            current_memory: RwLock::new(None),
            stale_threshold_days: 7,
        }
    }

    pub fn with_channel(
        project_dir: &Path,
        channel_capacity: usize,
    ) -> (Self, mpsc::Receiver<MemoryWriteRequest>) {
        let (write_tx, write_rx) = mpsc::channel(channel_capacity);
        (Self::new(project_dir, write_tx), write_rx)
    }

    pub fn memory_dir(&self) -> &Path {
        &self.memory_dir
    }

    pub fn write_sender(&self) -> mpsc::Sender<MemoryWriteRequest> {
        self.write_tx.clone()
    }

    pub async fn run_write_loop(self: Arc<Self>, mut rx: mpsc::Receiver<MemoryWriteRequest>) {
        while let Some(request) = rx.recv().await {
            let result = self
                .apply_agent_update(request.update)
                .await
                .map_err(anyhow::Error::from);
            let _ = request.ack.send(result);
        }
    }

    pub async fn is_memory_stale(&self, meta: &MetaJson) -> anyhow::Result<bool> {
        let age = Utc::now() - meta.last_updated;
        if age > Duration::days(self.stale_threshold_days) {
            return Ok(true);
        }

        if !meta.git_commit.is_empty() {
            if let Ok(current_commit) = self.get_current_git_commit().await {
                if current_commit != meta.git_commit {
                    return Ok(true);
                }
            }
        }

        if !meta.git_branch.is_empty() {
            if let Ok(current_branch) = self.get_current_git_branch().await {
                if current_branch != meta.git_branch {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    async fn ensure_layout(&self) -> Result<(), MemoryError> {
        fs::create_dir_all(&self.memory_dir).await?;
        fs::create_dir_all(self.memory_dir.join("agent-notes")).await?;
        fs::create_dir_all(self.memory_dir.join("letta")).await?;
        fs::create_dir_all(self.memory_dir.join("letta").join("archival")).await?;

        self.ensure_file_if_missing("project-overview.md", "")
            .await?;
        self.ensure_file_if_missing("conventions.md", "").await?;
        self.ensure_json_if_missing("tech-stack.json", &TechStack::default())
            .await?;
        self.ensure_json_if_missing("module-map.json", &ModuleMap::default())
            .await?;
        self.ensure_json_if_missing("api-endpoints.json", &ApiEndpoints::default())
            .await?;
        self.ensure_json_if_missing("data-models.json", &DataModels::default())
            .await?;
        self.ensure_json_if_missing("risk-areas.json", &RiskAreas::default())
            .await?;
        self.ensure_json_if_missing("dependency-graph.json", &DependencyGraph::default())
            .await?;
        self.ensure_json_if_missing(
            "user-rules.json",
            &serde_json::json!({ "version": 1, "rules": [] }),
        )
        .await?;
        self.ensure_json_if_missing(
            "user-preferences.json",
            &serde_json::json!({ "version": 1, "preferences": [], "rules": [] }),
        )
        .await?;
        self.ensure_file_if_missing(HISTORY_FILE, "").await?;

        Ok(())
    }

    async fn ensure_file_if_missing(
        &self,
        relative: &str,
        contents: &str,
    ) -> Result<(), MemoryError> {
        let path = self.memory_dir.join(relative);
        if !fs::try_exists(&path).await? {
            fs::write(path, contents).await?;
        }
        Ok(())
    }

    async fn ensure_json_if_missing<T: Serialize>(
        &self,
        relative: &str,
        value: &T,
    ) -> Result<(), MemoryError> {
        let path = self.memory_dir.join(relative);
        if !fs::try_exists(&path).await? {
            let serialized = serde_json::to_vec_pretty(value)?;
            fs::write(path, serialized).await?;
        }
        Ok(())
    }

    async fn load_valid_memory(
        &self,
        meta: &MetaJson,
    ) -> Result<ProjectMemorySnapshot, MemoryError> {
        Ok(ProjectMemorySnapshot {
            meta: meta.clone(),
            overview: self.read_string_or_default("project-overview.md").await?,
            tech_stack: self.read_json_or_default("tech-stack.json").await?,
            module_map: self.read_json_or_default("module-map.json").await?,
            api_endpoints: self.read_json_or_default("api-endpoints.json").await?,
            data_models: self.read_json_or_default("data-models.json").await?,
            conventions: self.read_string_or_default("conventions.md").await?,
            risk_areas: self.read_json_or_default("risk-areas.json").await?,
            dependency_graph: self.read_json_or_default("dependency-graph.json").await?,
        })
    }

    async fn read_string_or_default(&self, relative: &str) -> Result<String, MemoryError> {
        let path = self.memory_dir.join(relative);
        if fs::try_exists(&path).await? {
            Ok(fs::read_to_string(path).await?)
        } else {
            Ok(String::new())
        }
    }

    async fn read_json_or_default<T>(&self, relative: &str) -> Result<T, MemoryError>
    where
        T: DeserializeOwned + Default,
    {
        let path = self.memory_dir.join(relative);
        if fs::try_exists(&path).await? {
            let contents = fs::read_to_string(path).await?;
            Ok(serde_json::from_str(&contents)?)
        } else {
            Ok(T::default())
        }
    }

    async fn write_json<T: Serialize>(&self, relative: &str, value: &T) -> Result<(), MemoryError> {
        let path = self.memory_dir.join(relative);
        let contents = serde_json::to_vec_pretty(value)?;
        fs::write(path, contents).await?;
        Ok(())
    }

    async fn load_existing_meta(&self) -> Result<Option<MetaJson>, MemoryError> {
        let path = self.memory_dir.join("META.json");
        if !fs::try_exists(&path).await? {
            return Ok(None);
        }
        let contents = fs::read_to_string(path).await?;
        Ok(Some(serde_json::from_str(&contents)?))
    }

    async fn get_current_git_commit(&self) -> Result<String, MemoryError> {
        self.run_git(["rev-parse", "HEAD"]).await
    }

    async fn get_current_git_branch(&self) -> Result<String, MemoryError> {
        self.run_git(["rev-parse", "--abbrev-ref", "HEAD"]).await
    }

    async fn run_git<const N: usize>(&self, args: [&str; N]) -> Result<String, MemoryError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.project_dir)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(MemoryError::Command(stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn detect_file_changes(&self) -> Result<Vec<FileChange>, MemoryError> {
        let output = Command::new("git")
            .args(["status", "--porcelain", "--untracked-files=all"])
            .current_dir(&self.project_dir)
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(Self::parse_git_status_line)
            .collect())
    }

    pub fn parse_git_status_line(line: &str) -> Option<FileChange> {
        if line.len() < 3 {
            return None;
        }

        let status = &line[..2];
        let path = line[3..].trim();

        if status.contains('R') {
            let mut parts = path.splitn(2, " -> ");
            let old_path = parts.next()?.trim().to_string();
            let new_path = parts.next()?.trim().to_string();
            return Some(FileChange::Renamed { old_path, new_path });
        }

        if status == "??" || status.contains('A') {
            return Some(FileChange::Added {
                path: path.to_string(),
            });
        }

        if status.contains('D') {
            return Some(FileChange::Deleted {
                path: path.to_string(),
            });
        }

        if status.contains('M') {
            return Some(FileChange::Modified {
                path: path.to_string(),
            });
        }

        None
    }

    async fn classify_changes(&self, changes: &[FileChange]) -> Vec<MemoryUpdate> {
        let mut updates = Vec::with_capacity(changes.len());

        for change in changes {
            match change {
                FileChange::Added { path } => updates.push(MemoryUpdate::FileAdded {
                    path: path.clone(),
                    module_name: self.infer_module(path),
                }),
                FileChange::Modified { path } => updates.push(MemoryUpdate::FileModified {
                    path: path.clone(),
                    summary: format!("文件 {path} 已修改"),
                }),
                FileChange::Deleted { path } => updates.push(MemoryUpdate::FileDeleted {
                    path: path.clone(),
                    module_name: self.infer_module(path),
                }),
                FileChange::Renamed { old_path, new_path } => {
                    updates.push(MemoryUpdate::FileDeleted {
                        path: old_path.clone(),
                        module_name: self.infer_module(old_path),
                    });
                    updates.push(MemoryUpdate::FileAdded {
                        path: new_path.clone(),
                        module_name: self.infer_module(new_path),
                    });
                }
            }
        }

        updates
    }

    fn infer_module(&self, path: &str) -> Option<String> {
        let normalized = path.replace('\\', "/");
        let parts: Vec<&str> = normalized
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();
        match parts.as_slice() {
            ["src", module, ..] => Some((*module).to_string()),
            [module, ..] => Some((*module).to_string()),
            _ => None,
        }
    }

    async fn apply_agent_update(&self, update: MemoryUpdate) -> Result<(), MemoryError> {
        self.ensure_layout().await?;
        self.persist_update(&update, "agent").await?;
        self.update_meta("valid").await?;
        Ok(())
    }

    async fn persist_update(&self, update: &MemoryUpdate, actor: &str) -> Result<(), MemoryError> {
        match update {
            MemoryUpdate::FileModified { path, summary } => {
                self.append_change_history(&serde_json::json!({
                    "timestamp": Utc::now().to_rfc3339(),
                    "type": "file_modified",
                    "path": path,
                    "summary": summary,
                    "agent": actor,
                }))
                .await?;
            }
            MemoryUpdate::FileAdded { path, module_name } => {
                self.append_change_history(&serde_json::json!({
                    "timestamp": Utc::now().to_rfc3339(),
                    "type": "file_added",
                    "path": path,
                    "module": module_name,
                    "agent": actor,
                }))
                .await?;

                if let Some(module_name) = module_name {
                    let mut module_map: ModuleMap =
                        self.read_json_or_default("module-map.json").await?;
                    if !module_map
                        .modules
                        .iter()
                        .any(|module| module.name == *module_name)
                    {
                        module_map.modules.push(ModuleInfo {
                            name: module_name.clone(),
                            path: format!("src/{module_name}"),
                            responsibility: "待补充".to_string(),
                            key_files: vec![path.clone()],
                            public_api: Vec::new(),
                            dependencies: Vec::new(),
                            dependents: Vec::new(),
                        });
                        self.write_json("module-map.json", &module_map).await?;
                    }

                    let mut graph: DependencyGraph =
                        self.read_json_or_default("dependency-graph.json").await?;
                    if !graph.nodes.iter().any(|node| node == module_name) {
                        graph.nodes.push(module_name.clone());
                        self.write_json("dependency-graph.json", &graph).await?;
                    }
                }
            }
            MemoryUpdate::FileDeleted { path, module_name } => {
                self.append_change_history(&serde_json::json!({
                    "timestamp": Utc::now().to_rfc3339(),
                    "type": "file_deleted",
                    "path": path,
                    "module": module_name,
                    "agent": actor,
                }))
                .await?;
            }
            MemoryUpdate::ApiAdded { endpoint } => {
                let mut apis: ApiEndpoints =
                    self.read_json_or_default("api-endpoints.json").await?;
                apis.endpoints.retain(|existing| {
                    !(existing.method == endpoint.method && existing.path == endpoint.path)
                });
                apis.endpoints.push(endpoint.clone());
                self.write_json("api-endpoints.json", &apis).await?;
            }
            MemoryUpdate::ApiRemoved { method, path } => {
                let mut apis: ApiEndpoints =
                    self.read_json_or_default("api-endpoints.json").await?;
                apis.endpoints
                    .retain(|endpoint| !(endpoint.method == *method && endpoint.path == *path));
                self.write_json("api-endpoints.json", &apis).await?;
            }
            MemoryUpdate::DataModelChanged { model, change_type } => {
                self.append_change_history(&serde_json::json!({
                    "timestamp": Utc::now().to_rfc3339(),
                    "type": "data_model_changed",
                    "model": model,
                    "change_type": change_type,
                    "agent": actor,
                }))
                .await?;
            }
            MemoryUpdate::RiskDiscovered {
                area,
                r#type,
                description,
                severity,
            } => {
                let mut risks: RiskAreas = self.read_json_or_default("risk-areas.json").await?;
                risks.risks.push(RiskInfo {
                    area: area.clone(),
                    r#type: r#type.clone(),
                    description: description.clone(),
                    severity: severity.clone(),
                    discovered_at: Utc::now(),
                    discovered_by: actor.to_string(),
                });
                self.write_json("risk-areas.json", &risks).await?;
            }
            MemoryUpdate::RiskResolved { area } => {
                let mut risks: RiskAreas = self.read_json_or_default("risk-areas.json").await?;
                risks.risks.retain(|risk| risk.area != *area);
                self.write_json("risk-areas.json", &risks).await?;
            }
            MemoryUpdate::AgentNote {
                agent,
                topic,
                content,
            } => {
                let dir = self.memory_dir.join("agent-notes").join(agent);
                fs::create_dir_all(&dir).await?;
                let path = dir.join(format!("{topic}.md"));
                fs::write(path, content).await?;
            }
        }

        Ok(())
    }

    async fn append_change_history(&self, entry: &serde_json::Value) -> Result<(), MemoryError> {
        let path = self.memory_dir.join(HISTORY_FILE);
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;
        let line = serde_json::to_string(entry)? + "\n";
        file.write_all(line.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }

    async fn update_meta(&self, status: &str) -> Result<(), MemoryError> {
        let existing = self.load_existing_meta().await?;
        let stats = self.scan_project_stats().await?;
        let created_at = existing
            .as_ref()
            .map(|meta| meta.created_at)
            .unwrap_or_else(Utc::now);

        let meta = MetaJson {
            version: "1.0".to_string(),
            created_at,
            last_updated: Utc::now(),
            project_hash: stats.project_hash,
            git_branch: self.get_current_git_branch().await.unwrap_or_default(),
            git_commit: self.get_current_git_commit().await.unwrap_or_default(),
            total_files: stats.total_files,
            total_lines: stats.total_lines,
            memory_status: status.to_string(),
            stale_threshold_days: self.stale_threshold_days,
        };

        self.write_json("META.json", &meta).await?;
        Ok(())
    }

    async fn scan_project_stats(&self) -> Result<ProjectStats, MemoryError> {
        let mut stack = vec![self.project_dir.clone()];
        let mut total_files = 0usize;
        let mut total_lines = 0usize;
        let mut hasher = DefaultHasher::new();

        while let Some(dir) = stack.pop() {
            let mut entries = fs::read_dir(&dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();

                if file_name == ".git" || file_name == ".assistant-memory" {
                    continue;
                }

                let file_type = entry.file_type().await?;
                if file_type.is_dir() {
                    stack.push(path);
                    continue;
                }

                if !file_type.is_file() {
                    continue;
                }

                total_files += 1;
                let relative = path
                    .strip_prefix(&self.project_dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                relative.hash(&mut hasher);

                let bytes = fs::read(&path).await?;
                bytes.len().hash(&mut hasher);
                total_lines += count_lines(&bytes);
            }
        }

        Ok(ProjectStats {
            total_files,
            total_lines,
            project_hash: format!("{:016x}", hasher.finish()),
        })
    }
}

#[async_trait]
impl MemoryManagerTrait for MemoryManager {
    async fn load_memory(&self) -> anyhow::Result<ProjectMemory> {
        self.ensure_layout().await?;

        let meta_path = self.memory_dir.join("META.json");
        if !fs::try_exists(&meta_path).await? {
            info!("记忆目录已初始化，但尚未生成 META.json，返回 Empty");
            let memory = ProjectMemory::Empty;
            *self.current_memory.write().await = Some(memory.clone());
            return Ok(memory);
        }

        let meta_contents = fs::read_to_string(meta_path).await?;
        let meta: MetaJson = serde_json::from_str(&meta_contents)?;

        if self.is_memory_stale(&meta).await? {
            info!(last_updated = %meta.last_updated, "检测到项目记忆已过期");
            let memory = ProjectMemory::Stale(meta);
            *self.current_memory.write().await = Some(memory.clone());
            return Ok(memory);
        }

        let snapshot = self.load_valid_memory(&meta).await?;
        info!(
            modules = snapshot.module_map.modules.len(),
            "项目记忆加载完成"
        );
        let memory = ProjectMemory::Valid(Box::new(snapshot));
        *self.current_memory.write().await = Some(memory.clone());
        Ok(memory)
    }

    async fn incremental_update(&self) -> anyhow::Result<ProjectMemory> {
        self.ensure_layout().await?;
        let changes = self.detect_file_changes().await?;
        if changes.is_empty() {
            if let Some(memory) = self.current_memory.read().await.clone() {
                return Ok(memory);
            }
            return self.load_memory().await;
        }

        info!(changes = changes.len(), "检测到项目文件变更，开始增量更新");
        let updates = self.classify_changes(&changes).await;
        for update in updates {
            self.persist_update(&update, "coordinator").await?;
        }

        self.update_meta("valid").await?;
        self.load_memory().await
    }

    async fn agent_update(&self, update: MemoryUpdate) -> anyhow::Result<()> {
        let (ack_tx, ack_rx) = oneshot::channel();
        self.write_tx
            .send(MemoryWriteRequest {
                update,
                ack: ack_tx,
            })
            .await
            .context("提交记忆更新事件失败")?;
        ack_rx.await.context("等待记忆写入结果失败")?
    }

    async fn get_memory_summary(&self) -> anyhow::Result<String> {
        let memory = if let Some(memory) = self.current_memory.read().await.clone() {
            memory
        } else {
            self.load_memory().await?
        };

        let summary = match memory {
            ProjectMemory::Empty => "[无项目记忆，首次扫描后会生成 .assistant-memory/]".to_string(),
            ProjectMemory::Stale(meta) => format!(
                "[项目记忆已过期，最近更新于 {}，建议先刷新记忆]",
                meta.last_updated.to_rfc3339()
            ),
            ProjectMemory::Valid(snapshot) => {
                let mut summary = String::new();
                summary.push_str("## 项目记忆摘要\n");
                if !snapshot.overview.trim().is_empty() {
                    summary.push_str(&snapshot.overview);
                    summary.push('\n');
                }
                summary.push_str("\n## 技术栈\n");
                summary.push_str(&format!("- 语言: {}\n", snapshot.tech_stack.language));
                summary.push_str(&format!("- 版本: {}\n", snapshot.tech_stack.edition));
                for (name, version) in &snapshot.tech_stack.framework {
                    summary.push_str(&format!("- {name}: {version}\n"));
                }
                summary.push_str("\n## 模块\n");
                for module in &snapshot.module_map.modules {
                    summary.push_str(&format!(
                        "- {} ({}) {}\n",
                        module.name, module.path, module.responsibility
                    ));
                }
                summary.push_str(&format!(
                    "\n## 风险数\n- {}\n",
                    snapshot.risk_areas.risks.len()
                ));
                summary
            }
        };

        Ok(summary)
    }
}

fn count_lines(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        0
    } else {
        bytes.iter().filter(|&&byte| byte == b'\n').count() + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_git_status_line_handles_rename() {
        let parsed = MemoryManager::parse_git_status_line("R  old/path.rs -> new/path.rs");
        assert_eq!(
            parsed,
            Some(FileChange::Renamed {
                old_path: "old/path.rs".to_string(),
                new_path: "new/path.rs".to_string(),
            })
        );
    }

    #[test]
    fn count_lines_handles_empty_and_non_empty() {
        assert_eq!(count_lines(&[]), 0);
        assert_eq!(count_lines(b"one"), 1);
        assert_eq!(count_lines(b"one\ntwo\n"), 3);
    }
}
