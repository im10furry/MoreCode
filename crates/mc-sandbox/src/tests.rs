use std::io::Read;
use std::time::Duration;

use tempfile::tempdir;

use crate::capability::{Capability, PermissionLevel};
use crate::command::parse_command;
use crate::guardian::{Guardian, GuardianConfig, GuardianDecision, GuardianMode};
use crate::os_layer::{open_file_no_symlinks, SafeOpenOptions};
use crate::path_restriction::PathRestriction;
use crate::permission::{drop_cleanup_count, TaskPermissionManager};
use crate::tool::{ShellExecTool, ToolCallArgs};

#[test]
fn parse_command_is_structured_and_rejects_shell_operators() {
    let parsed = parse_command(r#"git commit -m "hello world""#).expect("parsed command");
    assert_eq!(parsed.program, "git");
    assert_eq!(parsed.executable_name, "git");
    assert_eq!(parsed.args(), &["commit", "-m", "hello world"]);

    let rejected = parse_command("echo hi && rm -rf /");
    assert!(rejected.is_err());
}

#[tokio::test]
async fn guardian_bypass_still_blocks_destructive_commands_and_logs() {
    let guardian = Guardian::new(GuardianConfig {
        mode: GuardianMode::Bypass,
        ..GuardianConfig::default()
    });
    let capability = ShellExecTool::new(vec!["ls".to_string()]).declaration(
        "terminal",
        "执行终端命令",
        PermissionLevel::Elevated,
    );

    let decision = guardian
        .check_tool_call(
            "tester",
            "terminal",
            &ToolCallArgs::shell_exec("rm -rf /").with_capability(capability),
        )
        .await;

    assert!(guardian.audit_enabled());
    assert!(matches!(decision, GuardianDecision::Deny { .. }));

    let entries = guardian.audit_log().entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].caller, "tester");
    assert_eq!(entries[0].tool_name, "terminal");
    assert_eq!(entries[0].decision_result, "deny");
    assert!(entries[0].parameters.get("command").is_some());
}

#[tokio::test]
async fn guardian_requires_capability_declaration() {
    let guardian = Guardian::default();
    let decision = guardian
        .check_tool_call("tester", "terminal", &ToolCallArgs::shell_exec("ls"))
        .await;

    assert!(matches!(decision, GuardianDecision::Deny { .. }));
}

#[test]
fn path_restriction_blocks_symlink_escape() {
    let temp = tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    let secret = temp.path().join("secret");
    std::fs::create_dir_all(&workspace).expect("workspace dir");
    std::fs::create_dir_all(&secret).expect("secret dir");
    std::fs::write(secret.join("hidden.txt"), "hidden").expect("secret file");

    let link = workspace.join("link");
    if create_directory_symlink(&secret, &link).is_err() {
        return;
    }

    let rules = vec![PathRestriction::allow(&workspace)];
    assert!(!PathRestriction::allows_path(
        &rules,
        &link.join("hidden.txt"),
        false
    ));
}

#[test]
fn safe_open_reads_regular_files_without_symlinks() {
    let temp = tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(&workspace).expect("workspace dir");
    let file_path = workspace.join("hello.txt");
    std::fs::write(&file_path, "hello world").expect("write file");

    let mut file =
        open_file_no_symlinks(&workspace, &file_path, SafeOpenOptions::read_only()).expect("open");
    let mut content = String::new();
    file.read_to_string(&mut content).expect("read file");

    assert_eq!(content, "hello world");
}

#[test]
fn task_permission_manager_drop_runs_cleanup() {
    let before = drop_cleanup_count();
    {
        let mut permissions = TaskPermissionManager::default();
        permissions.grant(
            "task-1",
            vec![Capability::ReadFile {
                pattern: "**".to_string(),
            }],
            Duration::from_millis(1),
            Some(1),
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(drop_cleanup_count() > before);
}

#[test]
fn shell_exec_tool_uses_regex_escape_for_command_patterns() {
    let declaration = ShellExecTool::new(vec!["git+unsafe?".to_string()]).declaration(
        "terminal",
        "执行终端命令",
        PermissionLevel::Elevated,
    );

    let run_command = declaration
        .capabilities
        .iter()
        .find_map(|capability| match capability {
            Capability::RunCommand { pattern } => Some(pattern.clone()),
            _ => None,
        })
        .expect("run command capability");

    assert_eq!(run_command, r"git\+unsafe\?");
}

#[cfg(unix)]
fn create_directory_symlink(
    target: &std::path::Path,
    link: &std::path::Path,
) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_directory_symlink(
    target: &std::path::Path,
    link: &std::path::Path,
) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}
