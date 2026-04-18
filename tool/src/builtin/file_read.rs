use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use mc_sandbox::os_layer::{open_file_no_symlinks, SafeOpenOptions};
use mc_sandbox::{Capability, CapabilityDeclaration, PermissionLevel};
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

use crate::types::{PermissionScope, Tool, ToolCategory, ToolResult};

const DEFAULT_LARGE_FILE_THRESHOLD: u64 = 1024 * 1024;
const DEFAULT_LINE_COUNT: usize = 200;
const SUMMARY_PREVIEW_LINES: usize = 5;

pub struct FileReadTool {
    large_file_threshold: u64,
    default_line_count: usize,
}

impl FileReadTool {
    pub fn new() -> Self {
        Self {
            large_file_threshold: DEFAULT_LARGE_FILE_THRESHOLD,
            default_line_count: DEFAULT_LINE_COUNT,
        }
    }

    pub fn with_config(large_file_threshold: u64, default_line_count: usize) -> Self {
        Self {
            large_file_threshold,
            default_line_count,
        }
    }

    async fn read_file(
        &self,
        path: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> ToolResult {
        let start = std::time::Instant::now();
        let path = PathBuf::from(path);
        let access_root = if path.is_absolute() {
            path.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| path.clone())
        } else {
            match std::env::current_dir() {
                Ok(root) => root,
                Err(error) => {
                    return ToolResult::error(format!("failed to get current directory: {error}"));
                }
            }
        };

        let file = match open_file_no_symlinks(&access_root, &path, SafeOpenOptions::read_only()) {
            Ok(file) => file,
            Err(error) => return ToolResult::error(format!("failed to open file: {error}")),
        };
        let metadata = match file.metadata() {
            Ok(metadata) => metadata,
            Err(error) => return ToolResult::error(format!("failed to read metadata: {error}")),
        };

        if !metadata.is_file() {
            return ToolResult::error("target path is not a regular file");
        }

        if metadata.len() >= self.large_file_threshold {
            return self
                .read_large_file(file, &path, metadata.len(), offset, limit)
                .await
                .with_duration(start.elapsed());
        }

        let mut file = tokio::fs::File::from_std(file);
        let mut bytes = Vec::new();
        if let Err(error) = file.read_to_end(&mut bytes).await {
            return ToolResult::error(format!("failed to read file: {error}"));
        }

        let content = match String::from_utf8(bytes) {
            Ok(content) => content,
            Err(_) => return ToolResult::error("binary files are not supported"),
        };

        let total_lines = content.lines().count();
        let mode = if offset.is_some() || limit.is_some() {
            "range"
        } else {
            "full"
        };
        let selected_content = if mode == "range" {
            select_lines(
                &content,
                offset.unwrap_or(0),
                limit.unwrap_or(self.default_line_count),
            )
        } else {
            content.clone()
        };
        let returned_lines = selected_content.lines().count();

        ToolResult::success_with_data(
            selected_content,
            json!({
                "path": path.to_string_lossy(),
                "size_bytes": metadata.len(),
                "total_lines": total_lines,
                "offset": offset.unwrap_or(0),
                "limit": limit.unwrap_or(self.default_line_count),
                "returned_lines": returned_lines,
                "mode": mode,
            }),
        )
        .with_duration(start.elapsed())
    }

    async fn read_large_file(
        &self,
        file: std::fs::File,
        path: &Path,
        file_size: u64,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> ToolResult {
        let start_line = offset.unwrap_or(0);
        let line_limit = limit.unwrap_or(self.default_line_count);

        if line_limit == 0 {
            return ToolResult::error("limit must be greater than 0");
        }

        let mut lines = BufReader::new(tokio::fs::File::from_std(file)).lines();
        let mut total_lines = 0usize;
        let mut selected_lines = Vec::new();
        let mut head_preview = Vec::new();
        let mut tail_preview = VecDeque::with_capacity(SUMMARY_PREVIEW_LINES);

        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    if head_preview.len() < SUMMARY_PREVIEW_LINES {
                        head_preview.push(line.clone());
                    }
                    if tail_preview.len() == SUMMARY_PREVIEW_LINES {
                        tail_preview.pop_front();
                    }
                    tail_preview.push_back(line.clone());
                    if total_lines >= start_line && selected_lines.len() < line_limit {
                        selected_lines.push(format!("{:>6} | {line}", total_lines + 1));
                    }
                    total_lines += 1;
                }
                Ok(None) => break,
                Err(_) => return ToolResult::error("large file contains non UTF-8 text"),
            }
        }

        let returned_lines = selected_lines.len();
        let line_end = if returned_lines == 0 {
            start_line
        } else {
            start_line + returned_lines - 1
        };
        let summary = json!({
            "size_bytes": file_size,
            "total_lines": total_lines,
            "head_preview": head_preview,
            "tail_preview": tail_preview.into_iter().collect::<Vec<_>>(),
        });
        let summary_text = format!(
            "Large file smart segmented read\nFile: {}\nSize: {} bytes\nTotal lines: {}\nRequested range: {} - {}\nReturned lines: {}",
            path.display(),
            file_size,
            total_lines,
            start_line + 1,
            line_end + 1,
            returned_lines,
        );
        let body = if selected_lines.is_empty() {
            format!("{summary_text}\n\nNo lines were returned for the requested range.")
        } else {
            format!("{summary_text}\n\n{}", selected_lines.join("\n"))
        };

        ToolResult::content_with_data(
            body,
            json!({
                "path": path.to_string_lossy(),
                "size_bytes": file_size,
                "total_lines": total_lines,
                "offset": start_line,
                "limit": line_limit,
                "returned_lines": returned_lines,
                "line_start": start_line + 1,
                "line_end": line_end + 1,
                "mode": "partial",
                "summary": summary,
            }),
        )
        .with_metadata("notice", "large file segmented read enabled")
    }
}

impl Default for FileReadTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read file content. Files larger than 1MB return a structured partial view."
    }

    fn execute(
        &self,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let path = match params.get("path").and_then(serde_json::Value::as_str) {
                Some(path) => path,
                None => return ToolResult::error("missing required parameter: path"),
            };
            let offset = params
                .get("offset")
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as usize);
            let limit = params
                .get("limit")
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as usize);
            self.read_file(path, offset, limit).await
        })
    }

    fn required_parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the target file"
                },
                "offset": {
                    "type": "integer",
                    "description": "Zero-based starting line",
                    "default": 0
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to return",
                    "default": self.default_line_count
                }
            },
            "required": ["path"]
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Core
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Public
    }

    fn permission_scope(&self) -> PermissionScope {
        PermissionScope::Filesystem
    }

    fn capability(&self) -> CapabilityDeclaration {
        CapabilityDeclaration::new(
            "file_read",
            "Read file content",
            self.permission_level(),
            vec![Capability::ReadFile {
                pattern: "**".to_string(),
            }],
        )
    }
}

fn select_lines(content: &str, offset: usize, limit: usize) -> String {
    content
        .lines()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>()
        .join("\n")
}
