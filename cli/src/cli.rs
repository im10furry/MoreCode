use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    Cli,
    Tui,
    Web,
}

impl UiMode {
    fn parse(value: &str) -> Result<Self, CliError> {
        match value.to_ascii_lowercase().as_str() {
            "cli" => Ok(Self::Cli),
            "tui" => Ok(Self::Tui),
            "web" => Ok(Self::Web),
            other => Err(CliError(format!("unknown ui mode: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalMode {
    Auto,
    Prompt,
    Deny,
}

impl ApprovalMode {
    fn parse(value: &str) -> Result<Self, CliError> {
        match value.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "prompt" => Ok(Self::Prompt),
            "deny" => Ok(Self::Deny),
            other => Err(CliError(format!("unknown approval mode: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Md,
    Jsonl,
    Html,
}

impl ExportFormat {
    fn parse(value: &str) -> Result<Self, CliError> {
        match value.to_ascii_lowercase().as_str() {
            "md" | "markdown" => Ok(Self::Md),
            "jsonl" => Ok(Self::Jsonl),
            "html" => Ok(Self::Html),
            other => Err(CliError(format!("unknown export format: {other}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCommand {
    pub request: String,
    pub ui: UiMode,
    pub plan_only: bool,
    pub json: bool,
    pub approval: ApprovalMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewCommand {
    pub run_id: String,
    pub ui: UiMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayCommand {
    pub run_id: String,
    pub json: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportCommand {
    pub run_id: String,
    pub format: ExportFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiCommand {
    pub request: Option<String>,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebCommand {
    pub port: u16,
    pub run_id: Option<String>,
    pub request: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    pub project_root: Option<PathBuf>,
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Run(RunCommand),
    Review(ReviewCommand),
    Replay(ReplayCommand),
    Export(ExportCommand),
    Tui(TuiCommand),
    Web(WebCommand),
    Memory(MemoryCommand),
    Config(ConfigCommand),
    Doctor,
    Daemon(DaemonCommand),
    Taskpile(TaskpileCommand),
    OtherCli,
    OtherCliAutoMigrate,
    Help,
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
pub enum TaskpileCommand {
    List,
    Show { task_id: String },
    Add { instruction: String, options: Vec<String> },
    Claim,
    Complete { task_id: String, summary: Option<String> },
    Fail { task_id: String, reason: Option<String> },
    Pause { task_id: String },
    Resume { task_id: String },
    Cancel { task_id: String },
    Stats,
    CloudStatus,
    CloudPreview { task_id: String },
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
            } else if let Some(value) = arg.strip_prefix("--project-root=") {
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
        [] => Ok(Command::Tui(TuiCommand {
            request: None,
            run_id: None,
        })),
        [cmd] if cmd == "--help" || cmd == "-h" => Ok(Command::Help),
        [cmd, rest @ ..] if cmd == "run" => Ok(Command::Run(parse_run_command(rest)?)),
        [cmd, rest @ ..] if cmd == "review" => Ok(Command::Review(parse_review_command(rest)?)),
        [cmd, rest @ ..] if cmd == "replay" => Ok(Command::Replay(parse_replay_command(rest)?)),
        [cmd, rest @ ..] if cmd == "export" => Ok(Command::Export(parse_export_command(rest)?)),
        [cmd] if cmd == "tui" => Ok(Command::Tui(TuiCommand {
            request: None,
            run_id: None,
        })),
        [cmd, rest @ ..] if cmd == "tui" => Ok(Command::Tui(parse_tui_command(rest)?)),
        [cmd] if cmd == "web" => Ok(Command::Web(WebCommand {
            port: 3000,
            run_id: None,
            request: None,
        })),
        [cmd, rest @ ..] if cmd == "web" => Ok(Command::Web(parse_web_command(rest)?)),
        [cmd] if cmd == "doctor" => Ok(Command::Doctor),
        [cmd] if cmd == "othercli" => Ok(Command::OtherCli),
        [cmd, sub] if cmd == "othercli" && sub == "auto-migrate" => {
            Ok(Command::OtherCliAutoMigrate)
        }
        [cmd, sub] if cmd == "memory" => Ok(Command::Memory(parse_memory_command(sub)?)),
        [cmd, sub] if cmd == "config" => Ok(Command::Config(parse_config_command(sub)?)),
        [cmd, sub] if cmd == "daemon" => Ok(Command::Daemon(parse_daemon_command(sub)?)),
        [cmd, rest @ ..] if cmd == "taskpile" => Ok(Command::Taskpile(parse_taskpile_command(rest)?)),
        _ => Err(CliError(usage())),
    }
}

fn parse_run_command(args: &[String]) -> Result<RunCommand, CliError> {
    let mut ui = UiMode::Cli;
    let mut plan_only = false;
    let mut json = false;
    let mut approval = ApprovalMode::Prompt;
    let mut request_parts = Vec::new();
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--plan-only" => {
                plan_only = true;
            }
            "--json" => {
                json = true;
            }
            "--ui" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError("missing value for --ui".into()))?;
                ui = UiMode::parse(value)?;
            }
            "--approve" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError("missing value for --approve".into()))?;
                approval = ApprovalMode::parse(value)?;
            }
            value if value.starts_with("--ui=") => {
                ui = UiMode::parse(value.trim_start_matches("--ui="))?;
            }
            value if value.starts_with("--approve=") => {
                approval = ApprovalMode::parse(value.trim_start_matches("--approve="))?;
            }
            other => request_parts.push(other.to_string()),
        }
        index += 1;
    }

    let request = request_parts.join(" ").trim().to_string();
    if request.is_empty() {
        return Err(CliError("run command requires a request string".into()));
    }

    Ok(RunCommand {
        request,
        ui,
        plan_only,
        json,
        approval,
    })
}

fn parse_review_command(args: &[String]) -> Result<ReviewCommand, CliError> {
    let mut ui = UiMode::Cli;
    let mut run_id = None;
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--ui" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError("missing value for --ui".into()))?;
                ui = UiMode::parse(value)?;
            }
            value if value.starts_with("--ui=") => {
                ui = UiMode::parse(value.trim_start_matches("--ui="))?;
            }
            value if value.starts_with('-') => {
                return Err(CliError(format!("unknown review option: {value}")));
            }
            value => {
                if run_id.is_some() {
                    return Err(CliError("review accepts exactly one run id".into()));
                }
                run_id = Some(value.to_string());
            }
        }
        index += 1;
    }

    Ok(ReviewCommand {
        run_id: run_id.ok_or_else(|| CliError("review requires a run id".into()))?,
        ui,
    })
}

fn parse_replay_command(args: &[String]) -> Result<ReplayCommand, CliError> {
    let mut json = false;
    let mut run_id = None;

    for arg in args {
        match arg.as_str() {
            "--json" => {
                json = true;
            }
            value if value.starts_with('-') => {
                return Err(CliError(format!("unknown replay option: {value}")));
            }
            value => {
                if run_id.is_some() {
                    return Err(CliError("replay accepts exactly one run id".into()));
                }
                run_id = Some(value.to_string());
            }
        }
    }

    Ok(ReplayCommand {
        run_id: run_id.ok_or_else(|| CliError("replay requires a run id".into()))?,
        json,
    })
}

fn parse_export_command(args: &[String]) -> Result<ExportCommand, CliError> {
    let mut format = ExportFormat::Md;
    let mut run_id = None;
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--format" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError("missing value for --format".into()))?;
                format = ExportFormat::parse(value)?;
            }
            value if value.starts_with("--format=") => {
                format = ExportFormat::parse(value.trim_start_matches("--format="))?;
            }
            value if value.starts_with('-') => {
                return Err(CliError(format!("unknown export option: {value}")));
            }
            value => {
                if run_id.is_some() {
                    return Err(CliError("export accepts exactly one run id".into()));
                }
                run_id = Some(value.to_string());
            }
        }
        index += 1;
    }

    Ok(ExportCommand {
        run_id: run_id.ok_or_else(|| CliError("export requires a run id".into()))?,
        format,
    })
}

fn parse_tui_command(args: &[String]) -> Result<TuiCommand, CliError> {
    let mut request_parts = Vec::new();
    let mut run_id = None;
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--run-id" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError("missing value for --run-id".into()))?;
                run_id = Some(value.to_string());
            }
            value if value.starts_with("--run-id=") => {
                run_id = Some(value.trim_start_matches("--run-id=").to_string());
            }
            value => request_parts.push(value.to_string()),
        }
        index += 1;
    }

    let request = request_parts.join(" ").trim().to_string();
    Ok(TuiCommand {
        request: if request.is_empty() {
            None
        } else {
            Some(request)
        },
        run_id,
    })
}

fn parse_web_command(args: &[String]) -> Result<WebCommand, CliError> {
    let mut port = 3000u16;
    let mut run_id = None;
    let mut request_parts = Vec::new();
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--port" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError("missing value for --port".into()))?;
                port = value
                    .parse::<u16>()
                    .map_err(|_| CliError(format!("invalid port: {value}")))?;
            }
            "--run-id" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError("missing value for --run-id".into()))?;
                run_id = Some(value.to_string());
            }
            value if value.starts_with("--port=") => {
                let raw = value.trim_start_matches("--port=");
                port = raw
                    .parse::<u16>()
                    .map_err(|_| CliError(format!("invalid port: {raw}")))?;
            }
            value if value.starts_with("--run-id=") => {
                run_id = Some(value.trim_start_matches("--run-id=").to_string());
            }
            value => request_parts.push(value.to_string()),
        }
        index += 1;
    }

    let request = request_parts.join(" ").trim().to_string();
    Ok(WebCommand {
        port,
        run_id,
        request: if request.is_empty() {
            None
        } else {
            Some(request)
        },
    })
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

fn parse_taskpile_command(args: &[String]) -> Result<TaskpileCommand, CliError> {
    match args {
        [] => Ok(TaskpileCommand::List),
        [sub] if sub == "list" => Ok(TaskpileCommand::List),
        [sub] if sub == "stats" => Ok(TaskpileCommand::Stats),
        [sub] if sub == "claim" => Ok(TaskpileCommand::Claim),
        [sub] if sub == "cloud-status" => Ok(TaskpileCommand::CloudStatus),
        [sub, task_id] if sub == "show" => Ok(TaskpileCommand::Show { task_id: task_id.clone() }),
        [sub, task_id] if sub == "pause" => Ok(TaskpileCommand::Pause { task_id: task_id.clone() }),
        [sub, task_id] if sub == "resume" => Ok(TaskpileCommand::Resume { task_id: task_id.clone() }),
        [sub, task_id] if sub == "cancel" => Ok(TaskpileCommand::Cancel { task_id: task_id.clone() }),
        [sub, task_id] if sub == "cloud-preview" => Ok(TaskpileCommand::CloudPreview { task_id: task_id.clone() }),
        [sub, task_id, rest @ ..] if sub == "complete" => Ok(TaskpileCommand::Complete {
            task_id: task_id.clone(),
            summary: join_args(rest),
        }),
        [sub, task_id, rest @ ..] if sub == "fail" => Ok(TaskpileCommand::Fail {
            task_id: task_id.clone(),
            reason: join_args(rest),
        }),
        [sub, rest @ ..] if sub == "add" => {
            let mut instruction_parts = Vec::new();
            let mut options = Vec::new();
            for arg in rest {
                if is_option_like(arg) {
                    options.push(arg.clone());
                } else {
                    instruction_parts.push(arg.clone());
                }
            }
            let instruction = instruction_parts.join(" ");
            if instruction.is_empty() {
                return Err(CliError("taskpile add requires an instruction".into()));
            }
            Ok(TaskpileCommand::Add { instruction, options })
        }
        _ => Err(CliError(usage_taskpile())),
    }
}

fn is_option_like(arg: &str) -> bool {
    if arg.starts_with("--") {
        return true;
    }
    if let Some((key, value)) = arg.split_once('=') {
        if value.is_empty() {
            return false;
        }
        let key = key.trim();
        if key.is_empty() {
            return false;
        }
        let first = key.chars().next().unwrap();
        if !first.is_ascii_alphabetic() {
            return false;
        }
        return key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    }
    false
}

fn join_args(args: &[String]) -> Option<String> {
    if args.is_empty() {
        None
    } else {
        Some(args.join(" "))
    }
}

pub fn usage() -> String {
    [
        "Usage:",
        "  morecode [--project-root PATH] run [--ui cli|tui] [--plan-only] [--json] [--approve auto|prompt|deny] <request>",
        "  morecode [--project-root PATH] review [--ui cli|tui] <run_id>",
        "  morecode [--project-root PATH] replay [--json] <run_id>",
        "  morecode [--project-root PATH] export [--format md|jsonl|html] <run_id>",
        "  morecode [--project-root PATH] tui [--run-id <run_id>] [request]",
        "  morecode [--project-root PATH] web [--port <port>] [--run-id <run_id>] [request]",
        "  morecode [--project-root PATH] memory <status|summary|refresh|clear>",
        "  morecode config show",
        "  morecode doctor",
        "  morecode othercli",
        "  morecode othercli auto-migrate",
        "  morecode daemon status",
        "  morecode taskpile [list|add|show|claim|complete|fail|pause|resume|cancel|stats|cloud-status|cloud-preview]",
        "  morecode --help",
    ]
    .join("\n")
}

fn usage_taskpile() -> String {
    [
        "Usage: morecode taskpile [subcommand]",
        "  list                          List all tasks",
        "  add <instruction> [key=val]   Add a new task",
        "  show <task_id>                Show task details",
        "  claim                         Claim next due task",
        "  complete <task_id> [summary]  Mark task completed",
        "  fail <task_id> [reason]       Mark task failed",
        "  pause <task_id>               Pause a task",
        "  resume <task_id>              Resume a paused task",
        "  cancel <task_id>              Cancel a task",
        "  stats                         Show taskpile statistics",
        "  cloud-status                  Show cloud connection status",
        "  cloud-preview <task_id>       Preview cloud payload",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        ApprovalMode, Cli, Command, ConfigCommand, ExportFormat, MemoryCommand, ReplayCommand,
        ReviewCommand, RunCommand, TuiCommand, UiMode, WebCommand,
    };

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
    fn parser_collects_run_request_and_options() {
        let cli = Cli::parse([
            "morecode".to_string(),
            "run".to_string(),
            "--ui=tui".to_string(),
            "--plan-only".to_string(),
            "--approve".to_string(),
            "auto".to_string(),
            "fix".to_string(),
            "auth".to_string(),
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Command::Run(RunCommand {
                request: "fix auth".into(),
                ui: UiMode::Tui,
                plan_only: true,
                json: false,
                approval: ApprovalMode::Auto,
            })
        );
    }

    #[test]
    fn parser_supports_review_replay_and_export() {
        let review = Cli::parse([
            "morecode".to_string(),
            "review".to_string(),
            "--ui".to_string(),
            "tui".to_string(),
            "run-1".to_string(),
        ])
        .unwrap();
        assert_eq!(
            review.command,
            Command::Review(ReviewCommand {
                run_id: "run-1".to_string(),
                ui: UiMode::Tui,
            })
        );

        let replay = Cli::parse([
            "morecode".to_string(),
            "replay".to_string(),
            "--json".to_string(),
            "run-2".to_string(),
        ])
        .unwrap();
        assert_eq!(
            replay.command,
            Command::Replay(ReplayCommand {
                run_id: "run-2".to_string(),
                json: true,
            })
        );

        let export = Cli::parse([
            "morecode".to_string(),
            "export".to_string(),
            "--format=html".to_string(),
            "run-3".to_string(),
        ])
        .unwrap();
        assert_eq!(
            export.command,
            Command::Export(super::ExportCommand {
                run_id: "run-3".to_string(),
                format: ExportFormat::Html,
            })
        );
    }

    #[test]
    fn parser_supports_tui_run_id() {
        let cli = Cli::parse([
            "morecode".to_string(),
            "tui".to_string(),
            "--run-id".to_string(),
            "run-1".to_string(),
        ])
        .unwrap();
        assert_eq!(
            cli.command,
            Command::Tui(TuiCommand {
                request: None,
                run_id: Some("run-1".to_string()),
            })
        );
    }

    #[test]
    fn parser_supports_web_command() {
        let cli = Cli::parse([
            "morecode".to_string(),
            "web".to_string(),
            "--port".to_string(),
            "4100".to_string(),
            "--run-id=run-9".to_string(),
            "ship".to_string(),
            "ui".to_string(),
        ])
        .unwrap();
        assert_eq!(
            cli.command,
            Command::Web(WebCommand {
                port: 4100,
                run_id: Some("run-9".to_string()),
                request: Some("ship ui".to_string()),
            })
        );
    }

    #[test]
    fn parser_defaults_to_tui_without_subcommand() {
        let cli = Cli::parse(["morecode".to_string()]).unwrap();
        assert_eq!(
            cli.command,
            Command::Tui(TuiCommand {
                request: None,
                run_id: None,
            })
        );
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
