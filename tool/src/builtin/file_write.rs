use mc_config::{auto_fix_line_endings_for_write, ConfigLoader};
use mc_sandbox::os_layer::{open_file_no_symlinks, SafeOpenOptions};
use mc_sandbox::{Capability, CapabilityDeclaration, PermissionLevel};
use serde_json::json;
use tokio::io::AsyncWriteExt;

use crate::types::{PermissionScope, Tool, ToolCategory, ToolResult};

pub struct FileWriteTool;

impl FileWriteTool {
    pub fn new() -> Self {
        Self
    }

    async fn write_file(&self, path: &str, content: &str, create_dirs: bool) -> ToolResult {
        let start = std::time::Instant::now();
        let path = std::path::PathBuf::from(path);
        let workspace_root = match std::env::current_dir() {
            Ok(root) => root,
            Err(error) => return ToolResult::error(format!("无法获取工作目录: {error}")),
        };

        let (content, line_ending_data) =
            auto_fix_line_endings(&workspace_root, &path, content).await;

        if create_dirs {
            if let Some(parent) = path.parent() {
                if let Err(error) = tokio::fs::create_dir_all(parent).await {
                    return ToolResult::error(format!("创建父目录失败: {error}"));
                }
            }
        }

        let file =
            match open_file_no_symlinks(&workspace_root, &path, SafeOpenOptions::write_only()) {
                Ok(file) => file,
                Err(error) => return ToolResult::error(format!("写入文件失败: {error}")),
            };
        let mut file = tokio::fs::File::from_std(file);

        if let Err(error) = file.write_all(content.as_bytes()).await {
            return ToolResult::error(format!("写入文件失败: {error}"));
        }
        if let Err(error) = file.flush().await {
            return ToolResult::error(format!("刷新文件失败: {error}"));
        }

        ToolResult::success_with_data(
            format!("已写入文件 {}", path.display()),
            json!({
                "path": path.to_string_lossy(),
                "bytes_written": content.len(),
                "lines_written": content.lines().count(),
                "line_ending": line_ending_data,
            }),
        )
        .with_duration(start.elapsed())
    }
}

async fn auto_fix_line_endings(
    workspace_root: &std::path::Path,
    path: &std::path::Path,
    content: &str,
) -> (String, serde_json::Value) {
    let cfg = match ConfigLoader::with_default_paths_for(workspace_root) {
        Ok(loader) => loader.load().await.ok().map(|c| c.line_ending),
        Err(_) => None,
    }
    .unwrap_or_default();
    let outcome = auto_fix_line_endings_for_write(workspace_root, path, content, &cfg).await;
    let mut data = serde_json::json!({
        "enabled": outcome.metadata.enabled,
        "applied": outcome.metadata.applied,
        "skipped": outcome.metadata.skipped,
        "reason": outcome.metadata.reason,
    });
    if let Some(stats) = outcome.metadata.input {
        data["input"] = serde_json::json!({
            "lf": stats.lf_count,
            "crlf": stats.crlf_count,
            "mixed": stats.mixed,
        });
    }
    if let Some(target) = outcome.metadata.target {
        data["target"] = serde_json::Value::String(target.as_str().to_string());
    }
    (outcome.content, data)
}

impl Default for FileWriteTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "写入文件内容，可选自动创建父目录。"
    }

    fn execute(
        &self,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let path = match params.get("path").and_then(serde_json::Value::as_str) {
                Some(path) => path,
                None => return ToolResult::error("缺少必需参数: path"),
            };
            let content = match params.get("content").and_then(serde_json::Value::as_str) {
                Some(content) => content,
                None => return ToolResult::error("缺少必需参数: content"),
            };
            let create_dirs = params
                .get("create_dirs")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            self.write_file(path, content, create_dirs).await
        })
    }

    fn required_parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要写入的文件路径"
                },
                "content": {
                    "type": "string",
                    "description": "写入的文件内容"
                },
                "create_dirs": {
                    "type": "boolean",
                    "description": "是否自动创建父目录",
                    "default": false
                }
            },
            "required": ["path", "content"]
        })
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Core
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Elevated
    }

    fn permission_scope(&self) -> PermissionScope {
        PermissionScope::Filesystem
    }

    fn capability(&self) -> CapabilityDeclaration {
        CapabilityDeclaration::new(
            "file_write",
            "写入工作区文件内容",
            self.permission_level(),
            vec![Capability::WriteFile {
                pattern: "**".to_string(),
            }],
        )
    }
}
