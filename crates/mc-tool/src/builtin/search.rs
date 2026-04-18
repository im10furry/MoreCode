use std::io::Read;
use std::path::PathBuf;

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
        let search_root = {
            let requested_root = PathBuf::from(path);
            if requested_root.is_absolute() {
                requested_root
            } else {
                match std::env::current_dir() {
                    Ok(root) => root.join(requested_root),
                    Err(error) => {
                        return ToolResult::error(format!(
                            "failed to get current directory: {error}"
                        ));
                    }
                }
            }
        };
        let regex = match Regex::new(pattern) {
            Ok(regex) => regex,
            Err(error) => return ToolResult::error(format!("invalid regex pattern: {error}")),
        };
        let include_matcher = match build_glob_matcher(include) {
            Ok(matcher) => matcher,
            Err(error) => return ToolResult::error(error),
        };

        let mut matches = Vec::new();
        let mut truncated = false;

        'entries: for entry in WalkDir::new(&search_root)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            if let Some(matcher) = &include_matcher {
                let relative = entry
                    .path()
                    .strip_prefix(&search_root)
                    .ok()
                    .unwrap_or_else(|| entry.path());
                if !matcher.is_match(relative) {
                    continue;
                }
            }

            let file = match open_file_no_symlinks(
                &search_root,
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
            format!("Found {} matches", matches.len()),
            json!({
                "pattern": pattern,
                "search_path": search_root.to_string_lossy(),
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
        "Search for text with a regex and optional glob include filter."
    }

    fn execute(
        &self,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let pattern = match params.get("pattern").and_then(serde_json::Value::as_str) {
                Some(pattern) => pattern,
                None => return ToolResult::error("missing required parameter: pattern"),
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
                    "description": "Regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Root directory to search",
                    "default": "."
                },
                "include": {
                    "type": "string",
                    "description": "Optional glob filter for file names"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of matches to return",
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
            "Search readable files",
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
            .map_err(|error| format!("invalid include glob: {error}")),
        None => Ok(None),
    }
}
