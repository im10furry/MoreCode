use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use mc_sandbox::{
    parse_command, CapabilityDeclaration, Guardian, GuardianDecision, PermissionLevel,
    ShellExecTool, ToolCallArgs,
};
use serde_json::json;
use tokio::process::Command;

use crate::types::{PermissionScope, Tool, ToolCategory, ToolResult};

const DEFAULT_TIMEOUT_SECS: u64 = 30;

pub struct TerminalTool {
    guardian: Option<Arc<Guardian>>,
    timeout: Duration,
}

impl TerminalTool {
    pub fn new(guardian: Option<Arc<Guardian>>) -> Self {
        Self {
            guardian,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    async fn execute_command(&self, caller: &str, command: &str, cwd: Option<&str>) -> ToolResult {
        let start = std::time::Instant::now();
        let parsed = match parse_command(command) {
            Ok(parsed) => parsed,
            Err(error) => {
                return ToolResult::error(error.to_string()).with_duration(start.elapsed())
            }
        };

        if let Some(guardian) = &self.guardian {
            let mut args = ToolCallArgs::shell_exec(command).with_capability(self.capability());
            if let Some(cwd) = cwd {
                args = args.with_target_path(PathBuf::from(cwd));
            }

            match guardian.check_tool_call(caller, self.name(), &args).await {
                GuardianDecision::Allow => {}
                GuardianDecision::Simulate { mock_result } => {
                    return ToolResult::content(mock_result).with_duration(start.elapsed())
                }
                GuardianDecision::Deny { reason }
                | GuardianDecision::ConfirmRequired { reason } => {
                    return ToolResult::error(reason).with_duration(start.elapsed())
                }
            }
        }

        let mut process = Command::new(&parsed.program);
        process.args(parsed.args());
        if let Some(cwd) = cwd {
            process.current_dir(cwd);
        }

        match tokio::time::timeout(self.timeout, process.output()).await {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                if exit_code == 0 {
                    ToolResult::success_with_data(
                        stdout,
                        json!({
                            "exit_code": exit_code,
                            "stderr": stderr,
                        }),
                    )
                    .with_duration(start.elapsed())
                } else {
                    let content = if stdout.is_empty() {
                        stderr.clone()
                    } else if stderr.is_empty() {
                        stdout.clone()
                    } else {
                        format!("{stdout}\n{stderr}")
                    };
                    ToolResult::content_with_data(
                        format!("命令返回非零退出码 {exit_code}\n{content}"),
                        json!({
                            "exit_code": exit_code,
                            "stdout": stdout,
                            "stderr": stderr,
                        }),
                    )
                    .with_duration(start.elapsed())
                }
            }
            Ok(Err(error)) => ToolResult::error(format!("执行命令失败: {error}")),
            Err(_) => {
                ToolResult::error(format!("命令执行超时，超过 {} 秒", self.timeout.as_secs()))
            }
        }
    }
}

impl Tool for TerminalTool {
    fn name(&self) -> &str {
        "terminal"
    }

    fn description(&self) -> &str {
        "在终端执行命令，执行前必须经过 Guardian 检查。"
    }

    fn execute(
        &self,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let command = match params.get("command").and_then(serde_json::Value::as_str) {
                Some(command) => command,
                None => return ToolResult::error("缺少必需参数: command"),
            };
            let cwd = params.get("cwd").and_then(serde_json::Value::as_str);
            let caller = params
                .get("caller")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            self.execute_command(caller, command, cwd).await
        })
    }

    fn required_parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "要执行的终端命令"
                },
                "cwd": {
                    "type": "string",
                    "description": "命令的工作目录"
                }
            },
            "required": ["command"]
        })
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Extended
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Elevated
    }

    fn permission_scope(&self) -> PermissionScope {
        PermissionScope::Process
    }

    fn capability(&self) -> CapabilityDeclaration {
        ShellExecTool::new(vec![
            "ls".to_string(),
            "pwd".to_string(),
            "cat".to_string(),
            "echo".to_string(),
            "rg".to_string(),
            "git".to_string(),
        ])
        .with_read_patterns(vec!["**".to_string()])
        .with_write_patterns(vec!["**".to_string()])
        .declaration("terminal", "执行终端命令", self.permission_level())
    }
}
