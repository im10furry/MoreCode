use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    pub project_root: Option<PathBuf>,
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Run { request: String },
    Tui { request: Option<String> },
    Memory(MemoryCommand),
    Config(ConfigCommand),
    Doctor,
    Daemon(DaemonCommand),
    OtherCli,
    OtherCliAutoMigrate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryCommand {
    Status,
    Summary,
    Refresh,
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigCommand {
    Show,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonCommand {
    Status,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError(pub String);

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CliError {}

impl Cli {
    pub fn parse<I>(args: I) -> Result<Self, CliError>
    where
        I: IntoIterator<Item = String>,
    {
        let mut args = args.into_iter();
        let _program = args.next();
        let mut project_root = None;
        let mut remaining = Vec::new();

        while let Some(arg) = args.next() {
            if arg == "--project-root" {
                let value = args
                    .next()
                    .ok_or_else(|| CliError("missing value for --project-root".into()))?;
                project_root = Some(PathBuf::from(value));
            } else {
                remaining.push(arg);
                remaining.extend(args);
                break;
            }
        }

        let command = parse_command(&remaining)?;
        Ok(Self {
            project_root,
            command,
        })
    }
}

fn parse_command(args: &[String]) -> Result<Command, CliError> {
    match args {
        [] => Ok(Command::Tui { request: None }),
        [cmd, rest @ ..] if cmd == "run" => {
            let request = rest.join(" ").trim().to_string();
            if request.is_empty() {
                return Err(CliError("run command requires a request string".into()));
            }
            Ok(Command::Run { request })
        }
        [cmd] if cmd == "tui" => Ok(Command::Tui { request: None }),
        [cmd, rest @ ..] if cmd == "tui" => {
            let request = rest.join(" ").trim().to_string();
            Ok(Command::Tui {
                request: if request.is_empty() { None } else { Some(request) },
            })
        }
        [cmd] if cmd == "doctor" => Ok(Command::Doctor),
        [cmd] if cmd == "othercli" => Ok(Command::OtherCli),
        [cmd, sub] if cmd == "othercli" && sub == "auto-migrate" => Ok(Command::OtherCliAutoMigrate),
        [cmd, sub] if cmd == "memory" => Ok(Command::Memory(parse_memory_command(sub)?)),
        [cmd, sub] if cmd == "config" => Ok(Command::Config(parse_config_command(sub)?)),
        [cmd, sub] if cmd == "daemon" => Ok(Command::Daemon(parse_daemon_command(sub)?)),
        _ => Err(CliError(usage())),
    }
}

fn parse_memory_command(sub: &str) -> Result<MemoryCommand, CliError> {
    match sub {
        "status" => Ok(MemoryCommand::Status),
        "summary" => Ok(MemoryCommand::Summary),
        "refresh" => Ok(MemoryCommand::Refresh),
        "clear" => Ok(MemoryCommand::Clear),
        _ => Err(CliError(format!("unknown memory subcommand: {sub}"))),
    }
}

fn parse_config_command(sub: &str) -> Result<ConfigCommand, CliError> {
    match sub {
        "show" => Ok(ConfigCommand::Show),
        _ => Err(CliError(format!("unknown config subcommand: {sub}"))),
    }
}

fn parse_daemon_command(sub: &str) -> Result<DaemonCommand, CliError> {
    match sub {
        "status" => Ok(DaemonCommand::Status),
        _ => Err(CliError(format!("unknown daemon subcommand: {sub}"))),
    }
}

fn usage() -> String {
    [
        "Usage:",
        "  morecode [--project-root PATH] run <request>",
        "  morecode [--project-root PATH] tui",
        "  morecode [--project-root PATH] memory <status|summary|refresh|clear>",
        "  morecode config show",
        "  morecode doctor",
        "  morecode othercli",
        "  morecode othercli auto-migrate",
        "  morecode daemon status",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Cli, Command, ConfigCommand, MemoryCommand};

    #[test]
    fn parser_handles_project_root_and_subcommands() {
        let cli = Cli::parse([
            "morecode".to_string(),
            "--project-root".to_string(),
            "C:/repo".to_string(),
            "memory".to_string(),
            "status".to_string(),
        ])
        .unwrap();

        assert_eq!(cli.project_root, Some(PathBuf::from("C:/repo")));
        assert_eq!(cli.command, Command::Memory(MemoryCommand::Status));
    }

    #[test]
    fn parser_collects_run_request() {
        let cli = Cli::parse([
            "morecode".to_string(),
            "run".to_string(),
            "fix".to_string(),
            "auth".to_string(),
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Command::Run {
                request: "fix auth".into()
            }
        );
    }

    #[test]
    fn parser_supports_tui_command() {
        let cli = Cli::parse(["morecode".to_string(), "tui".to_string()]).unwrap();
        assert_eq!(cli.command, Command::Tui { request: None });
    }

    #[test]
    fn parser_defaults_to_tui_without_subcommand() {
        let cli = Cli::parse(["morecode".to_string()]).unwrap();
        assert_eq!(cli.command, Command::Tui { request: None });
    }

    #[test]
    fn parser_rejects_unknown_commands() {
        let error = Cli::parse([
            "morecode".to_string(),
            "config".to_string(),
            "unknown".to_string(),
        ])
        .unwrap_err();
        assert!(error.to_string().contains("unknown config subcommand"));
        assert_eq!(
            Command::Config(ConfigCommand::Show),
            Command::Config(ConfigCommand::Show)
        );
    }
}
