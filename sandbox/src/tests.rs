use std::io::Read;
use std::time::Duration;

use tempfile::tempdir;

use crate::capability::{Capability, PermissionLevel};
use crate::command::parse_command;
use crate::guardian::{Guardian, GuardianConfig, GuardianDecision, GuardianMode};
use crate::os_layer::{open_file_no_symlinks, safe_profile, SafeOpenOptions, SeccompMode};
use crate::path_restriction::PathRestriction;
use crate::permission::{drop_cleanup_count, TaskPermissionManager};
use crate::tool::{ShellExecTool, ToolCallArgs};

#[cfg(target_os = "linux")]
use crate::os_layer::{apply_landlock, detect_landlock_support, LandlockConfig, LandlockSupport};
#[cfg(target_os = "linux")]
use std::process::Command;

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

#[test]
fn guardian_exposes_configured_seccomp_profile() {
    let guardian = Guardian::new(GuardianConfig {
        seccomp_profile: Some(safe_profile()),
        ..GuardianConfig::default()
    });

    let profile = guardian.seccomp_profile().expect("seccomp profile");
    assert_eq!(profile.mode, SeccompMode::Balanced);
    assert!(profile
        .denied_syscalls
        .iter()
        .any(|syscall| syscall == "ptrace"));
}

#[tokio::test]
async fn guardian_builds_landlock_config_from_restrictions() {
    let temp = tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    let docs = temp.path().join("docs");
    std::fs::create_dir_all(&workspace).expect("workspace");
    std::fs::create_dir_all(&docs).expect("docs");
    let guardian = Guardian::new(GuardianConfig {
        path_restrictions: vec![PathRestriction::read_only(&docs)],
        ..GuardianConfig::default()
    });

    let config = guardian
        .landlock_config_for_workspace(&workspace)
        .await
        .expect("landlock config");

    assert!(config
        .read_write_dirs
        .iter()
        .any(|path| path.ends_with("workspace")));
    assert!(config
        .read_only_dirs
        .iter()
        .any(|path| path.ends_with("docs")));
}

#[cfg(target_os = "linux")]
#[test]
fn landlock_child_process_enforces_workspace_boundaries() {
    const CHILD_ENV: &str = "MC_SANDBOX_LANDLOCK_CHILD";
    const WORKSPACE_ENV: &str = "MC_SANDBOX_LANDLOCK_WORKSPACE";
    const INSIDE_ENV: &str = "MC_SANDBOX_LANDLOCK_INSIDE";
    const OUTSIDE_ENV: &str = "MC_SANDBOX_LANDLOCK_OUTSIDE";
    const TEST_NAME: &str = "tests::landlock_child_process_enforces_workspace_boundaries";

    if std::env::var_os(CHILD_ENV).is_some() {
        let workspace = std::env::var_os(WORKSPACE_ENV).expect("workspace path");
        let inside_file = std::env::var_os(INSIDE_ENV).expect("inside file");
        let outside_file = std::env::var_os(OUTSIDE_ENV).expect("outside file");

        let config = LandlockConfig {
            read_write_dirs: vec![workspace.into()],
            read_only_dirs: Vec::new(),
            denied_paths: Vec::new(),
        };

        apply_landlock(&config).expect("apply landlock");
        std::fs::write(inside_file, "inside").expect("write inside workspace");

        let outside_result = std::fs::write(outside_file, "outside");
        assert!(outside_result.is_err(), "outside write should be denied");
        return;
    }

    let LandlockSupport::Supported { .. } = detect_landlock_support() else {
        return;
    };

    let workspace = tempdir().expect("workspace tempdir");
    let outside = tempdir().expect("outside tempdir");
    let inside_file = workspace.path().join("inside.txt");
    let outside_file = outside.path().join("outside.txt");
    std::fs::write(&inside_file, "before").expect("seed inside file");
    std::fs::write(&outside_file, "before").expect("seed outside file");

    let status = Command::new(std::env::current_exe().expect("current test binary"))
        .arg("--exact")
        .arg(TEST_NAME)
        .arg("--nocapture")
        .env(CHILD_ENV, "1")
        .env(WORKSPACE_ENV, workspace.path())
        .env(INSIDE_ENV, &inside_file)
        .env(OUTSIDE_ENV, &outside_file)
        .status()
        .expect("spawn child test process");

    assert!(status.success(), "child test should succeed");
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
