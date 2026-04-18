use mc_sandbox::{Capability, CapabilityDeclaration, PermissionLevel};
use serde_json::json;
use tokio::process::Command;

use crate::types::{PermissionScope, Tool, ToolCategory, ToolResult};

const ALLOWED_SUBCOMMANDS: &[&str] = &["status", "log", "diff", "show"];
const DANGEROUS_FLAGS: &[&str] = &["--force", "-f", "--hard", "--no-verify"];

pub struct GitTool;

impl GitTool {
    pub fn new() -> Self {
        Self
    }

    async fn git_command(&self, subcommand: &str, args: &[String], cwd: &str) -> ToolResult {
        let start = std::time::Instant::now();

        if !ALLOWED_SUBCOMMANDS.contains(&subcommand) {
            return ToolResult::error(format!(
                "Git 子命令 `{subcommand}` 不被允许，只允许 status/log/diff/show"
            ));
        }

        if let Some(flag) = args
            .iter()
            .find(|arg| DANGEROUS_FLAGS.iter().any(|flag| contains_flag(arg, flag)))
        {
            return ToolResult::error(format!("检测到危险 Git 参数 `{flag}`，已拒绝执行"));
        }

        let mut command = Command::new("git");
        command.arg(subcommand).args(args).current_dir(cwd);

        match command.output().await {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                if exit_code == 0 {
                    ToolResult::success_with_data(
                        stdout,
                        json!({
                            "subcommand": subcommand,
                            "args": args,
                            "cwd": cwd,
                            "exit_code": exit_code,
                            "stderr": stderr,
                        }),
                    )
                    .with_duration(start.elapsed())
                } else {
                    ToolResult::error(format!(
                        "git {subcommand} 执行失败，退出码 {exit_code}: {stderr}"
                    ))
                    .with_duration(start.elapsed())
                }
            }
            Err(error) => ToolResult::error(format!("启动 git 失败: {error}")),
        }
    }
}

impl Default for GitTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for GitTool {
    fn name(&self) -> &str {
        "git"
    }

    fn description(&self) -> &str {
        "执行安全的 Git 只读操作，仅支持 status、log、diff、show。"
    }

    fn execute(
        &self,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let subcommand = match params.get("subcommand").and_then(serde_json::Value::as_str) {
                Some(subcommand) => subcommand,
                None => return ToolResult::error("缺少必需参数: subcommand"),
            };
            let args: Vec<String> = params
                .get("args")
                .and_then(serde_json::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                        .collect()
                })
                .unwrap_or_default();
            let cwd = params
                .get("cwd")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(".");
            self.git_command(subcommand, &args, cwd).await
        })
    }

    fn required_parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "subcommand": {
                    "type": "string",
                    "description": "Git 子命令",
                    "enum": ALLOWED_SUBCOMMANDS
                },
                "args": {
                    "type": "array",
                    "description": "Git 子命令参数",
                    "items": {
                        "type": "string"
                    }
                },
                "cwd": {
                    "type": "string",
                    "description": "Git 仓库路径",
                    "default": "."
                }
            },
            "required": ["subcommand"]
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Deferred
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Elevated
    }

    fn permission_scope(&self) -> PermissionScope {
        PermissionScope::VersionControl
    }

    fn capability(&self) -> CapabilityDeclaration {
        CapabilityDeclaration::new(
            "git",
            "执行安全的 Git 操作",
            self.permission_level(),
            vec![
                Capability::RunCommand {
                    pattern: "git".to_string(),
                },
                Capability::ReadFile {
                    pattern: "**".to_string(),
                },
            ],
        )
    }
}

fn contains_flag(arg: &str, flag: &str) -> bool {
    arg == flag || arg.starts_with(&format!("{flag}=")) || arg.contains(flag)
}
