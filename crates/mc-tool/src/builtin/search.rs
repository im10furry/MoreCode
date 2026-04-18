use std::io::Read;

use globset::{Glob, GlobMatcher};
use mc_sandbox::os_layer::{open_file_no_symlinks, SafeOpenOptions};
use mc_sandbox::{Capability, CapabilityDeclaration, PermissionLevel};
use regex::Regex;
use serde_json::json;
use walkdir::WalkDir;

use crate::types::{PermissionScope, Tool, ToolCategory, ToolResult};

const DEFAULT_MAX_RESULTS: usize = 100;

pub struct SearchTool;

impl SearchTool {
    pub fn new() -> Self {
        Self
    }

    async fn search(
        &self,
        pattern: &str,
        path: &str,
        include: Option<&str>,
        max_results: usize,
    ) -> ToolResult {
        let start = std::time::Instant::now();
        let workspace_root = match std::env::current_dir() {
            Ok(root) => root,
            Err(error) => return ToolResult::error(format!("无法获取工作目录: {error}")),
        };
        let regex = match Regex::new(pattern) {
            Ok(regex) => regex,
            Err(error) => return ToolResult::error(format!("无效的正则表达式: {error}")),
        };
        let include_matcher = match build_glob_matcher(include) {
            Ok(matcher) => matcher,
            Err(error) => return ToolResult::error(error),
        };

        let mut matches = Vec::new();
        let mut truncated = false;

        'entries: for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            if let Some(matcher) = &include_matcher {
                let relative = entry
                    .path()
                    .strip_prefix(path)
                    .ok()
                    .unwrap_or_else(|| entry.path());
                if !matcher.is_match(relative) {
                    continue;
                }
            }

            let file = match open_file_no_symlinks(
                &workspace_root,
                entry.path(),
                SafeOpenOptions::read_only(),
            ) {
                Ok(file) => file,
                Err(_) => continue,
            };
            let mut reader = std::io::BufReader::new(file);
            let mut content = String::new();
            if reader.read_to_string(&mut content).is_err() {
                continue;
            }

            for (line_number, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    matches.push(json!({
                        "file": entry.path().to_string_lossy(),
                        "line": line_number + 1,
                        "text": line.trim(),
                    }));
                    if matches.len() >= max_results {
                        truncated = true;
                        break 'entries;
                    }
                }
            }
        }

        ToolResult::success_with_data(
            format!("共找到 {} 条匹配", matches.len()),
            json!({
                "pattern": pattern,
                "search_path": path,
                "matches": matches,
                "total_matches": matches.len(),
                "truncated": truncated,
            }),
        )
        .with_duration(start.elapsed())
    }
}

impl Default for SearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "使用正则表达式搜索目录中的文本内容，支持文件名过滤。"
    }

    fn execute(
        &self,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let pattern = match params.get("pattern").and_then(serde_json::Value::as_str) {
                Some(pattern) => pattern,
                None => return ToolResult::error("缺少必需参数: pattern"),
            };
            let path = params
                .get("path")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(".");
            let include = params.get("include").and_then(serde_json::Value::as_str);
            let max_results = params
                .get("max_results")
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(DEFAULT_MAX_RESULTS);

            self.search(pattern, path, include, max_results).await
        })
    }

    fn required_parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "用于搜索内容的正则表达式"
                },
                "path": {
                    "type": "string",
                    "description": "搜索起始目录",
                    "default": "."
                },
                "include": {
                    "type": "string",
                    "description": "文件名过滤模式，使用 glob 语法"
                },
                "max_results": {
                    "type": "integer",
                    "description": "最大返回结果数",
                    "default": DEFAULT_MAX_RESULTS
                }
            },
            "required": ["pattern"]
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
        PermissionScope::Search
    }

    fn capability(&self) -> CapabilityDeclaration {
        CapabilityDeclaration::new(
            "search",
            "在工作区中进行正则搜索",
            self.permission_level(),
            vec![Capability::ReadFile {
                pattern: "**".to_string(),
            }],
        )
    }
}

fn build_glob_matcher(include: Option<&str>) -> Result<Option<GlobMatcher>, String> {
    match include {
        Some(pattern) => Glob::new(pattern)
            .map(|glob| Some(glob.compile_matcher()))
            .map_err(|error| format!("无效的 include 过滤模式: {error}")),
        None => Ok(None),
    }
}
