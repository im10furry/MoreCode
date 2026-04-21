use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use globset::GlobBuilder;
use mc_core::{
    detect_line_endings, is_probably_binary, is_safe_newline_rewrite, normalize_line_endings,
    EolStats, LineEnding,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LineEndingDefault {
    Lf,
    Crlf,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineEndingConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_line_ending_default")]
    pub default: LineEndingDefault,
    #[serde(default = "default_true")]
    pub respect_gitattributes: bool,
    #[serde(default = "default_true")]
    pub respect_editorconfig: bool,
    #[serde(default = "default_true")]
    pub fix_on_file_write: bool,
    #[serde(default)]
    pub fix_before_git_commit: bool,
    #[serde(default = "default_true")]
    pub skip_binary: bool,
    #[serde(default = "default_max_file_size_kb")]
    pub max_file_size_kb: usize,
}

impl Default for LineEndingConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            default: default_line_ending_default(),
            respect_gitattributes: default_true(),
            respect_editorconfig: default_true(),
            fix_on_file_write: default_true(),
            fix_before_git_commit: false,
            skip_binary: default_true(),
            max_file_size_kb: default_max_file_size_kb(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PartialLineEndingConfig {
    pub enabled: Option<bool>,
    pub default: Option<LineEndingDefault>,
    pub respect_gitattributes: Option<bool>,
    pub respect_editorconfig: Option<bool>,
    pub fix_on_file_write: Option<bool>,
    pub fix_before_git_commit: Option<bool>,
    pub skip_binary: Option<bool>,
    pub max_file_size_kb: Option<usize>,
}

impl LineEndingConfig {
    pub(crate) fn apply_partial(&mut self, partial: PartialLineEndingConfig) {
        if let Some(value) = partial.enabled {
            self.enabled = value;
        }
        if let Some(value) = partial.default {
            self.default = value;
        }
        if let Some(value) = partial.respect_gitattributes {
            self.respect_gitattributes = value;
        }
        if let Some(value) = partial.respect_editorconfig {
            self.respect_editorconfig = value;
        }
        if let Some(value) = partial.fix_on_file_write {
            self.fix_on_file_write = value;
        }
        if let Some(value) = partial.fix_before_git_commit {
            self.fix_before_git_commit = value;
        }
        if let Some(value) = partial.skip_binary {
            self.skip_binary = value;
        }
        if let Some(value) = partial.max_file_size_kb {
            self.max_file_size_kb = value;
        }
    }
}

impl PartialLineEndingConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            enabled: other.enabled.or(self.enabled),
            default: other.default.or(self.default),
            respect_gitattributes: other.respect_gitattributes.or(self.respect_gitattributes),
            respect_editorconfig: other.respect_editorconfig.or(self.respect_editorconfig),
            fix_on_file_write: other.fix_on_file_write.or(self.fix_on_file_write),
            fix_before_git_commit: other.fix_before_git_commit.or(self.fix_before_git_commit),
            skip_binary: other.skip_binary.or(self.skip_binary),
            max_file_size_kb: other.max_file_size_kb.or(self.max_file_size_kb),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_line_ending_default() -> LineEndingDefault {
    LineEndingDefault::Lf
}

fn default_max_file_size_kb() -> usize {
    2048
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineEndingFixMetadata {
    pub enabled: bool,
    pub applied: bool,
    pub skipped: bool,
    pub reason: Option<&'static str>,
    pub input: Option<EolStats>,
    pub target: Option<LineEnding>,
}

impl Default for LineEndingFixMetadata {
    fn default() -> Self {
        Self {
            enabled: false,
            applied: false,
            skipped: false,
            reason: None,
            input: None,
            target: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineEndingFixOutcome {
    pub content: String,
    pub metadata: LineEndingFixMetadata,
}

pub async fn auto_fix_line_endings_for_write(
    workspace_root: &Path,
    path: &Path,
    content: &str,
    cfg: &LineEndingConfig,
) -> LineEndingFixOutcome {
    let mut metadata = LineEndingFixMetadata::default();

    if !cfg.enabled || !cfg.fix_on_file_write {
        return LineEndingFixOutcome {
            content: content.to_string(),
            metadata,
        };
    }
    metadata.enabled = true;

    let max_bytes = cfg.max_file_size_kb.saturating_mul(1024);
    if max_bytes > 0 && content.len() > max_bytes {
        metadata.skipped = true;
        metadata.reason = Some("too_large");
        return LineEndingFixOutcome {
            content: content.to_string(),
            metadata,
        };
    }

    if cfg.skip_binary && (content.contains('\0') || is_probably_binary(content.as_bytes())) {
        metadata.skipped = true;
        metadata.reason = Some("binary");
        return LineEndingFixOutcome {
            content: content.to_string(),
            metadata,
        };
    }

    let input_stats = detect_line_endings(content.as_bytes());
    metadata.input = Some(input_stats);

    let target = resolve_target_line_ending(workspace_root, path, cfg, &input_stats).await;
    metadata.target = Some(target);

    let normalized = normalize_line_endings(content, target);
    if normalized == content {
        metadata.reason = Some("no_change");
        return LineEndingFixOutcome {
            content: content.to_string(),
            metadata,
        };
    }

    if !is_safe_newline_rewrite(content, &normalized) {
        metadata.skipped = true;
        metadata.reason = Some("unsafe_rewrite");
        return LineEndingFixOutcome {
            content: content.to_string(),
            metadata,
        };
    }

    metadata.applied = true;
    metadata.reason = Some("normalized");
    LineEndingFixOutcome {
        content: normalized,
        metadata,
    }
}

async fn resolve_target_line_ending(
    workspace_root: &Path,
    path: &Path,
    cfg: &LineEndingConfig,
    input_stats: &EolStats,
) -> LineEnding {
    if cfg.respect_gitattributes {
        if let Some(eol) = resolve_gitattributes_eol(workspace_root, path).await {
            return eol;
        }
    }
    if cfg.respect_editorconfig {
        if let Some(eol) = resolve_editorconfig_eol(workspace_root, path).await {
            return eol;
        }
    }

    match cfg.default {
        LineEndingDefault::Lf => LineEnding::Lf,
        LineEndingDefault::Crlf => LineEnding::Crlf,
        LineEndingDefault::Auto => {
            if let Some(eol) = resolve_existing_file_majority_eol(workspace_root, path).await {
                return eol;
            }
            if input_stats.crlf_count > input_stats.lf_count {
                LineEnding::Crlf
            } else if input_stats.lf_count > 0 || input_stats.crlf_count > 0 {
                LineEnding::Lf
            } else {
                LineEnding::system_default()
            }
        }
    }
}

async fn resolve_existing_file_majority_eol(workspace_root: &Path, path: &Path) -> Option<LineEnding> {
    let (_, target) = normalize_workspace_and_target(workspace_root, path)?;
    let bytes = tokio::fs::read(&target).await.ok()?;
    let stats = detect_line_endings(&bytes);
    if stats.is_binary {
        return None;
    }
    if stats.crlf_count > stats.lf_count {
        Some(LineEnding::Crlf)
    } else if stats.lf_count > 0 || stats.crlf_count > 0 {
        Some(LineEnding::Lf)
    } else {
        None
    }
}

async fn resolve_gitattributes_eol(workspace_root: &Path, path: &Path) -> Option<LineEnding> {
    let (root, target) = normalize_workspace_and_target(workspace_root, path)?;
    let mut resolved = None;
    for dir in ancestor_dirs_from_root(&root, target.parent().unwrap_or(&root)) {
        let attrs_path = dir.join(".gitattributes");
        let Ok(content) = tokio::fs::read_to_string(&attrs_path).await else {
            continue;
        };
        let Ok(relative) = target.strip_prefix(&dir) else {
            continue;
        };
        for raw in content.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split_whitespace();
            let Some(pattern) = parts.next() else {
                continue;
            };
            if !path_pattern_matches(pattern, relative) {
                continue;
            }
            for attr in parts {
                if attr.eq_ignore_ascii_case("eol=lf") {
                    resolved = Some(LineEnding::Lf);
                } else if attr.eq_ignore_ascii_case("eol=crlf") {
                    resolved = Some(LineEnding::Crlf);
                }
            }
        }
    }
    resolved
}

async fn resolve_editorconfig_eol(workspace_root: &Path, path: &Path) -> Option<LineEnding> {
    let (root, target) = normalize_workspace_and_target(workspace_root, path)?;
    let mut config_files = Vec::new();
    let mut cursor = Some(target.parent().unwrap_or(&root));
    while let Some(dir) = cursor {
        if !dir.starts_with(&root) {
            break;
        }
        let candidate = dir.join(".editorconfig");
        if let Ok(content) = tokio::fs::read_to_string(&candidate).await {
            let has_root = editorconfig_declares_root(&content);
            config_files.push((dir.to_path_buf(), content));
            if has_root {
                break;
            }
        }
        if dir == root {
            break;
        }
        cursor = dir.parent();
    }

    config_files.reverse();
    let mut resolved = None;
    for (dir, content) in config_files {
        let Ok(relative) = target.strip_prefix(&dir) else {
            continue;
        };
        if let Some(eol) = resolve_editorconfig_eol_in_file(&content, relative) {
            resolved = Some(eol);
        }
    }
    resolved
}

fn editorconfig_declares_root(content: &str) -> bool {
    let mut in_section = false;
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_section = true;
            continue;
        }
        if in_section {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("root") && value.trim().eq_ignore_ascii_case("true") {
            return true;
        }
    }
    false
}

fn resolve_editorconfig_eol_in_file(content: &str, relative: &Path) -> Option<LineEnding> {
    let mut resolved = None;
    let mut section_matches = false;
    let mut in_section = false;

    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let section = line[1..line.len() - 1].trim();
            section_matches = path_pattern_matches(section, relative);
            in_section = true;
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if !key.trim().eq_ignore_ascii_case("end_of_line") {
            continue;
        }
        if in_section && !section_matches {
            continue;
        }
        resolved = match value.trim().to_ascii_lowercase().as_str() {
            "lf" => Some(LineEnding::Lf),
            "crlf" => Some(LineEnding::Crlf),
            _ => resolved,
        };
    }

    resolved
}

fn path_pattern_matches(pattern: &str, relative: &Path) -> bool {
    let normalized_pattern = pattern.trim().replace('\\', "/");
    if normalized_pattern.is_empty() {
        return false;
    }

    let relative_text = normalize_relative_path(relative);
    let file_name = relative
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let (glob_pattern, candidate) = if normalized_pattern.contains('/') {
        (normalized_pattern.trim_start_matches('/').to_string(), relative_text)
    } else {
        (normalized_pattern, file_name.to_string())
    };

    GlobBuilder::new(&glob_pattern)
        .literal_separator(true)
        .build()
        .map(|glob| glob.compile_matcher().is_match(&candidate))
        .unwrap_or(false)
}

fn normalize_relative_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => part.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn ancestor_dirs_from_root(root: &Path, dir: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut cursor = Some(dir);
    while let Some(current) = cursor {
        if !current.starts_with(root) {
            break;
        }
        dirs.push(current.to_path_buf());
        if current == root {
            break;
        }
        cursor = current.parent();
    }
    dirs.reverse();
    dirs
}

fn normalize_workspace_and_target(workspace_root: &Path, path: &Path) -> Option<(PathBuf, PathBuf)> {
    let root = normalize_path_no_symlink_escape(workspace_root)?;
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let target = normalize_path_no_symlink_escape(&candidate)?;
    if !target.starts_with(&root) {
        return None;
    }
    Some((root, target))
}

fn normalize_path_no_symlink_escape(path: &Path) -> Option<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };

    ensure_no_symlink_components(&absolute)?;

    let (existing_ancestor, missing_tail) = split_existing_ancestor(&absolute)?;
    let mut normalized = std::fs::canonicalize(&existing_ancestor).ok()?;
    for component in missing_tail {
        normalized.push(component);
    }
    Some(normalized)
}

fn split_existing_ancestor(path: &Path) -> Option<(PathBuf, Vec<OsString>)> {
    let mut cursor = path.to_path_buf();
    let mut tail = Vec::new();

    while !cursor.exists() {
        let file_name = cursor.file_name()?.to_os_string();
        tail.push(file_name);
        cursor = cursor.parent()?.to_path_buf();
    }

    tail.reverse();
    Some((cursor, tail))
}

fn ensure_no_symlink_components(path: &Path) -> Option<()> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return None;
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => break,
            Err(_) => return None,
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_workspace(name: &str) -> PathBuf {
        let unique = format!(
            "line-ending-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&root).expect("workspace");
        root
    }

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("parent");
        }
        std::fs::write(path, contents).expect("write");
    }

    #[tokio::test]
    async fn auto_fix_matches_nested_gitattributes_by_target_path() {
        let root = temp_workspace("gitattributes");
        write_file(&root.join(".gitattributes"), "*.md eol=lf\n");
        write_file(&root.join("docs").join(".gitattributes"), "*.md eol=crlf\n");

        let cfg = LineEndingConfig::default();
        let output = auto_fix_line_endings_for_write(
            &root,
            &root.join("docs").join("guide.md"),
            "alpha\nbeta\n",
            &cfg,
        )
        .await;

        assert_eq!(output.content, "alpha\r\nbeta\r\n");
        assert_eq!(output.metadata.target, Some(LineEnding::Crlf));
        assert!(output.metadata.applied);
        std::fs::remove_dir_all(root).ok();
    }

    #[tokio::test]
    async fn auto_fix_matches_editorconfig_sections_by_target_path() {
        let root = temp_workspace("editorconfig");
        write_file(
            &root.join(".editorconfig"),
            "root = true\n[*]\nend_of_line = lf\n[docs/*.md]\nend_of_line = crlf\n",
        );

        let cfg = LineEndingConfig::default();
        let output = auto_fix_line_endings_for_write(
            &root,
            &root.join("docs").join("guide.md"),
            "alpha\nbeta\n",
            &cfg,
        )
        .await;

        assert_eq!(output.content, "alpha\r\nbeta\r\n");
        assert_eq!(output.metadata.target, Some(LineEnding::Crlf));
        assert!(output.metadata.applied);
        std::fs::remove_dir_all(root).ok();
    }
}
