use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::CommandParseError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedCommand {
    pub program: String,
    pub executable_name: String,
    pub argv: Vec<String>,
}

impl ParsedCommand {
    pub fn args(&self) -> &[String] {
        self.argv.get(1..).unwrap_or(&[])
    }
}

pub fn parse_command(input: &str) -> Result<ParsedCommand, CommandParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(CommandParseError::EmptyCommand);
    }

    if contains_shell_control_operators(trimmed) {
        return Err(CommandParseError::ShellControlOperator(trimmed.to_string()));
    }

    let argv = shlex::split(trimmed)
        .ok_or_else(|| CommandParseError::InvalidQuoting(trimmed.to_string()))?;
    let program = argv
        .first()
        .cloned()
        .ok_or(CommandParseError::MissingExecutable)?;
    let executable_name = Path::new(&program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(program.as_str())
        .to_ascii_lowercase();

    Ok(ParsedCommand {
        program,
        executable_name,
        argv,
    })
}

pub fn render_command(program: &str, args: &[String]) -> String {
    let mut command = quote_for_display(program);
    for arg in args {
        command.push(' ');
        command.push_str(&quote_for_display(arg));
    }
    command
}

pub fn contains_shell_control_operators(command: &str) -> bool {
    let mut chars = command.chars().peekable();
    let mut escaped = false;
    let mut in_single = false;
    let mut in_double = false;

    while let Some(ch) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if !in_single => escaped = true,
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '`' if !in_single && !in_double => return true,
            '$' if !in_single && !in_double && chars.peek() == Some(&'(') => return true,
            ';' | '\n' if !in_single && !in_double => return true,
            '|' if !in_single && !in_double => return true,
            '&' if !in_single && !in_double && chars.peek() == Some(&'&') => return true,
            _ => {}
        }
    }

    false
}

pub fn is_destructive_command(command: &str) -> bool {
    let command_lower = command.to_ascii_lowercase();
    let raw_patterns = [
        "curl | sh",
        "curl | bash",
        "wget | sh",
        "wget | bash",
        ":(){ :|:& };:",
        "fork bomb",
    ];

    if raw_patterns
        .iter()
        .any(|pattern| command_lower.contains(pattern))
    {
        return true;
    }

    parse_command(command)
        .ok()
        .and_then(|parsed| check_destructive_patterns(&parsed))
        .is_some()
}

pub fn check_destructive_patterns(parsed: &ParsedCommand) -> Option<String> {
    let args = parsed.args();

    match parsed.executable_name.as_str() {
        "format" => Some("检测到 format 命令".to_string()),
        command if command.starts_with("mkfs") => Some(format!("检测到破坏性命令 `{command}`")),
        "fdisk" | "parted" | "dd" | "shred" => {
            Some(format!("检测到破坏性命令 `{}`", parsed.executable_name))
        }
        "shutdown" | "reboot" | "halt" | "poweroff" => {
            Some(format!("检测到系统控制命令 `{}`", parsed.executable_name))
        }
        "sudo" | "su" | "doas" | "run0" => {
            Some(format!("检测到权限提升命令 `{}`", parsed.executable_name))
        }
        "rm" => {
            let has_recursive = args.iter().any(|arg| {
                matches!(
                    arg.as_str(),
                    "-r" | "-R" | "-rf" | "-fr" | "--recursive" | "-Rf" | "-rRf"
                )
            });
            let has_force = args
                .iter()
                .any(|arg| matches!(arg.as_str(), "-f" | "--force"));
            let dangerous_targets = args
                .iter()
                .filter(|arg| !arg.starts_with('-'))
                .filter(|arg| {
                    matches!(
                        arg.as_str(),
                        "/" | "/*" | "*" | ".*" | "~" | "~/" | "/root" | "/home"
                    )
                })
                .cloned()
                .collect::<Vec<_>>();

            if has_recursive && has_force && !dangerous_targets.is_empty() {
                return Some(format!(
                    "检测到递归强制删除危险目标: {:?}",
                    dangerous_targets
                ));
            }

            None
        }
        "chmod" => {
            let touches_system = args
                .iter()
                .filter(|arg| !arg.starts_with('-'))
                .any(|arg| arg == "/" || arg.starts_with("/etc") || arg.starts_with("/usr"));
            let recursive = args
                .iter()
                .any(|arg| matches!(arg.as_str(), "-r" | "-R" | "--recursive"));
            let permissive = args.iter().any(|arg| arg == "777");
            if touches_system && recursive && permissive {
                return Some("检测到对系统路径的递归 chmod 777".to_string());
            }
            None
        }
        _ => None,
    }
}

fn quote_for_display(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }

    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | '\\' | ':'))
    {
        return value.to_string();
    }

    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
