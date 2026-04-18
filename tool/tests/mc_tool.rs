use std::collections::HashSet;
use std::process::Command;
use std::sync::Arc;

use mc_sandbox::{Guardian, GuardianConfig, GuardianMode};
use mc_tool::{register_all_tools, ToolRegistry, ToolResultStatus, VisibilityLayer};
use serde_json::json;
use tempfile::tempdir;

#[tokio::test]
async fn registry_respects_visibility_and_deferred_loading() {
    let registry = ToolRegistry::new();
    register_all_tools(&registry).await;

    let public_definitions = registry.tool_definitions(VisibilityLayer::Public).await;
    let public_names: Vec<_> = public_definitions
        .iter()
        .map(|definition| definition.name.as_str())
        .collect();
    assert_eq!(public_names, vec!["file_read", "search"]);

    let git_tool = registry
        .get("git")
        .await
        .expect("git tool should load lazily");
    assert_eq!(git_tool.name(), "git");

    let project_definitions = registry.tool_definitions(VisibilityLayer::Project).await;
    let project_names: Vec<_> = project_definitions
        .iter()
        .map(|definition| definition.name.as_str())
        .collect();
    assert_eq!(
        project_names,
        vec!["file_read", "file_write", "git", "search", "terminal"]
    );

    for name in ["file_read", "file_write", "git", "search", "terminal"] {
        let tool = registry.get(name).await.expect("tool should be registered");
        assert!(
            tool.capability().is_complete(),
            "{name} must declare capability"
        );
    }

    assert!(registry.unregister("search").await);
    assert!(registry.get("search").await.is_none());
}

#[tokio::test]
async fn file_read_returns_full_content_for_small_files() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("small.txt");
    std::fs::write(&file_path, "alpha\nbeta\ngamma\n").expect("write small file");

    let registry = ToolRegistry::new();
    register_all_tools(&registry).await;

    let result = registry
        .execute_tool(
            "tester",
            "file_read",
            json!({
                "path": file_path.to_string_lossy().to_string(),
            }),
        )
        .await;

    assert_eq!(result.status, ToolResultStatus::Success);
    assert!(result.content.contains("alpha\nbeta\ngamma"));
    let data = result.data.expect("file_read should return data");
    assert_eq!(data["mode"], "full");
    assert_eq!(data["total_lines"], 3);
}

#[tokio::test]
async fn file_read_returns_summary_for_large_files() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("large.txt");
    let content: String = (0..80_000)
        .map(|index| format!("line-{index:05}-abcdefghijklmnopqrstuvwxyz\n"))
        .collect();
    std::fs::write(&file_path, content).expect("write large file");

    let registry = ToolRegistry::new();
    register_all_tools(&registry).await;

    let result = registry
        .execute_tool(
            "tester",
            "file_read",
            json!({
                "path": file_path.to_string_lossy().to_string(),
                "offset": 10,
                "limit": 3,
            }),
        )
        .await;

    assert_eq!(result.status, ToolResultStatus::Content);
    assert!(result.content.contains("Large file smart segmented read"));
    assert!(result.content.contains("11 | line-00010"));
    let data = result.data.expect("large file read should return data");
    assert_eq!(data["mode"], "partial");
    assert_eq!(data["returned_lines"], 3);
    assert!(data["summary"].is_object());
}

#[tokio::test]
async fn search_uses_regex_and_include_filters() {
    let dir = tempdir().expect("tempdir");
    std::fs::write(dir.path().join("main.rs"), "let foo42 = 1;\nlet bar = 2;\n")
        .expect("write main.rs");
    std::fs::write(dir.path().join("notes.txt"), "foo99\n").expect("write notes.txt");

    let registry = ToolRegistry::new();
    register_all_tools(&registry).await;

    let result = registry
        .execute_tool(
            "tester",
            "search",
            json!({
                "pattern": r"foo\d+",
                "path": dir.path().to_string_lossy().to_string(),
                "include": "*.rs",
            }),
        )
        .await;

    assert_eq!(result.status, ToolResultStatus::Success);
    let data = result.data.expect("search should return structured data");
    let matches = data["matches"]
        .as_array()
        .expect("matches should be an array");
    assert_eq!(matches.len(), 1);
    assert!(matches[0]["file"]
        .as_str()
        .expect("file path")
        .ends_with("main.rs"));
}

#[tokio::test]
async fn terminal_is_blocked_by_guardian() {
    let mut blocked_tools = HashSet::new();
    blocked_tools.insert("terminal".to_string());
    let guardian = Arc::new(Guardian::new(GuardianConfig {
        blocked_tools,
        ..GuardianConfig::default()
    }));

    let registry = ToolRegistry::with_guardian(guardian);
    register_all_tools(&registry).await;

    let result = registry
        .execute_tool("tester", "terminal", json!({ "command": "echo hello" }))
        .await;

    assert_eq!(result.status, ToolResultStatus::Error);
}

#[tokio::test]
async fn terminal_runs_only_after_guardian_check() {
    let git_available = Command::new("git").arg("--version").output().is_ok();
    if !git_available {
        return;
    }

    let guardian = Arc::new(Guardian::new(GuardianConfig {
        mode: GuardianMode::Bypass,
        ..GuardianConfig::default()
    }));
    let registry = ToolRegistry::with_guardian(guardian);
    register_all_tools(&registry).await;

    let result = registry
        .execute_tool("tester", "terminal", json!({ "command": "git --version" }))
        .await;

    assert!(matches!(
        result.status,
        ToolResultStatus::Success | ToolResultStatus::Content
    ));
}

#[tokio::test]
async fn git_tool_allows_safe_operations_only() {
    let git_available = Command::new("git").arg("--version").output().is_ok();
    if !git_available {
        return;
    }

    let dir = tempdir().expect("tempdir");
    let init_status = Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .status()
        .expect("run git init");
    assert!(init_status.success());

    let registry = ToolRegistry::new();
    register_all_tools(&registry).await;

    let status_result = registry
        .execute_tool(
            "tester",
            "git",
            json!({
                "subcommand": "status",
                "cwd": dir.path().to_string_lossy().to_string(),
            }),
        )
        .await;
    assert_eq!(status_result.status, ToolResultStatus::Success);

    let blocked_subcommand = registry
        .execute_tool(
            "tester",
            "git",
            json!({
                "subcommand": "commit",
                "cwd": dir.path().to_string_lossy().to_string(),
            }),
        )
        .await;
    assert_eq!(blocked_subcommand.status, ToolResultStatus::Error);
}
