use std::{
    collections::{BTreeMap, HashMap},
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crossterm::terminal;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{fs, process::Command, sync::RwLock, time::timeout};
use tracing::{info, warn};

type HmacSha256 = Hmac<Sha256>;

const MAX_CONTEXT_BLOCK_LEN: usize = 2_000;
const TOOL_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(60 * 60);
const STATIC_HMAC_KEY: [u8; 32] = *b"morecode-platform-cache-key-v01!";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Dash,
    Sh,
    Cmd,
    PowerShell,
    Pwsh,
    Ash,
    Unknown(String),
}

impl ShellType {
    pub fn from_shell_name(name: &str) -> Self {
        let normalized = name
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(name)
            .to_ascii_lowercase();
        match normalized.as_str() {
            "bash" | "bash.exe" => Self::Bash,
            "zsh" | "zsh.exe" => Self::Zsh,
            "fish" | "fish.exe" => Self::Fish,
            "dash" | "dash.exe" => Self::Dash,
            "sh" | "sh.exe" => Self::Sh,
            "cmd" | "cmd.exe" => Self::Cmd,
            "powershell" | "powershell.exe" => Self::PowerShell,
            "pwsh" | "pwsh.exe" => Self::Pwsh,
            "ash" | "ash.exe" => Self::Ash,
            _ => Self::Unknown(name.to_string()),
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Bash => "Bash",
            Self::Zsh => "Zsh",
            Self::Fish => "Fish",
            Self::Dash => "Dash",
            Self::Sh => "Sh",
            Self::Cmd => "CMD",
            Self::PowerShell => "PowerShell",
            Self::Pwsh => "PowerShell 7+",
            Self::Ash => "Ash",
            Self::Unknown(name) => name.as_str(),
        }
    }

    fn executable_name(&self) -> &str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Fish => "fish",
            Self::Dash => "dash",
            Self::Sh => "sh",
            Self::Cmd => "cmd",
            Self::PowerShell => "powershell",
            Self::Pwsh => "pwsh",
            Self::Ash => "ash",
            Self::Unknown(name) => name.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FileEncoding {
    UTF8,
    UTF8BOM,
    UTF16LE,
    UTF16BE,
    UTF32LE,
    UTF32BE,
    GBK,
    ShiftJIS,
    Big5,
    ASCII,
    Unknown,
}

impl FileEncoding {
    pub fn detect_bytes(bytes: &[u8]) -> Self {
        if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            return Self::UTF8BOM;
        }
        if bytes.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) {
            return Self::UTF32LE;
        }
        if bytes.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
            return Self::UTF32BE;
        }
        if bytes.starts_with(&[0xFF, 0xFE]) {
            return Self::UTF16LE;
        }
        if bytes.starts_with(&[0xFE, 0xFF]) {
            return Self::UTF16BE;
        }
        if bytes.is_ascii() {
            return Self::ASCII;
        }
        if std::str::from_utf8(bytes).is_ok() {
            return Self::UTF8;
        }
        if looks_like_shift_jis(bytes) {
            return Self::ShiftJIS;
        }
        if looks_like_big5(bytes) {
            return Self::Big5;
        }
        if looks_like_gbk(bytes) {
            return Self::GBK;
        }
        Self::Unknown
    }

    fn from_locale(locale: &str) -> Self {
        let locale = locale.to_ascii_lowercase();
        if locale.contains("utf-8") || locale.contains("utf8") {
            Self::UTF8
        } else if locale.contains("gbk") || locale.contains("gb2312") {
            Self::GBK
        } else if locale.contains("shift_jis") || locale.contains("sjis") {
            Self::ShiftJIS
        } else if locale.contains("big5") {
            Self::Big5
        } else {
            Self::UTF8
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::UTF8 => "UTF-8",
            Self::UTF8BOM => "UTF-8 BOM",
            Self::UTF16LE => "UTF-16 LE",
            Self::UTF16BE => "UTF-16 BE",
            Self::UTF32LE => "UTF-32 LE",
            Self::UTF32BE => "UTF-32 BE",
            Self::GBK => "GBK",
            Self::ShiftJIS => "Shift-JIS",
            Self::Big5 => "Big5",
            Self::ASCII => "ASCII",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LineEnding {
    LF,
    CRLF,
}

impl LineEnding {
    fn system_default() -> Self {
        if cfg!(windows) {
            Self::CRLF
        } else {
            Self::LF
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::LF => "LF (\\n)",
            Self::CRLF => "CRLF (\\r\\n)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PathStyle {
    Unix,
    Windows,
    Unc,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PathHandler {
    pub style: PathStyle,
    pub separator: char,
    pub max_length: usize,
    pub case_sensitive: bool,
}

impl PathHandler {
    pub fn new(os: &str) -> Self {
        match os {
            "windows" => Self {
                style: PathStyle::Windows,
                separator: '\\',
                max_length: 260,
                case_sensitive: false,
            },
            "macos" => Self {
                style: PathStyle::Unix,
                separator: '/',
                max_length: 1024,
                case_sensitive: false,
            },
            _ => Self {
                style: PathStyle::Unix,
                separator: '/',
                max_length: 4096,
                case_sensitive: true,
            },
        }
    }

    pub fn normalize(&self, path: &str) -> String {
        let path = path.replace('\\', "/");
        let (prefix, remainder, absolute) = if path.starts_with("//") {
            ("//", path.trim_start_matches('/'), true)
        } else if path.len() >= 2 && path.as_bytes()[1] == b':' {
            (&path[..2], path[2..].trim_start_matches('/'), true)
        } else if path.starts_with('/') {
            ("/", path.trim_start_matches('/'), true)
        } else {
            ("", path.as_str(), false)
        };

        let mut stack = Vec::new();
        for segment in remainder.split('/') {
            match segment {
                "" | "." => {}
                ".." => {
                    stack.pop();
                }
                other => stack.push(other),
            }
        }

        let joined = stack.join("/");
        match prefix {
            "//" => {
                if joined.is_empty() {
                    "//".to_string()
                } else {
                    format!("//{joined}")
                }
            }
            "/" if absolute => {
                if joined.is_empty() {
                    "/".to_string()
                } else {
                    format!("/{joined}")
                }
            }
            _ if prefix.ends_with(':') => {
                if joined.is_empty() {
                    format!("{prefix}/")
                } else {
                    format!("{prefix}/{joined}")
                }
            }
            _ => joined,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShellConfig {
    pub shell_type: ShellType,
    pub executable_path: String,
    pub version: String,
    pub is_login_shell: bool,
    pub is_interactive: bool,
    pub available_shells: Vec<ShellType>,
}

impl ShellConfig {
    pub fn set_env_var(&self, key: &str, value: &str) -> String {
        match self.shell_type {
            ShellType::Bash | ShellType::Zsh | ShellType::Dash | ShellType::Sh | ShellType::Ash => {
                format!("export {}={}", key, quote_for_posix(value))
            }
            ShellType::Fish => format!("set -gx {} {}", key, quote_for_posix(value)),
            ShellType::PowerShell | ShellType::Pwsh => {
                format!("$env:{} = '{}'", key, value.replace('\'', "''"))
            }
            ShellType::Cmd => format!("set {}=\"{}\"", key, value.replace('"', "\"\"")),
            ShellType::Unknown(_) => format!("export {}={}", key, quote_for_posix(value)),
        }
    }

    pub fn maybe_wrap_command(
        &self,
        command: &str,
        preferred_shell: &ShellType,
    ) -> Option<(ShellType, String)> {
        if &self.shell_type == preferred_shell {
            return None;
        }
        let wrapped = match preferred_shell {
            ShellType::Bash => format!("bash -c {}", quote_for_posix(command)),
            ShellType::Zsh => format!("zsh -c {}", quote_for_posix(command)),
            ShellType::Fish => format!("fish -c {}", quote_for_posix(command)),
            ShellType::Dash => format!("dash -c {}", quote_for_posix(command)),
            ShellType::Sh => format!("sh -c {}", quote_for_posix(command)),
            ShellType::Ash => format!("ash -c {}", quote_for_posix(command)),
            ShellType::PowerShell => {
                format!("powershell -Command {}", quote_for_powershell(command))
            }
            ShellType::Pwsh => format!("pwsh -Command {}", quote_for_powershell(command)),
            ShellType::Cmd => format!("cmd /c \"{}\"", command.replace('"', "\"\"")),
            ShellType::Unknown(_) => return None,
        };
        Some((preferred_shell.clone(), wrapped))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalInfo {
    pub term: String,
    pub term_program: String,
    pub cols: usize,
    pub rows: usize,
    pub color_support: bool,
    pub true_color: bool,
}

impl Default for TerminalInfo {
    fn default() -> Self {
        Self {
            term: "unknown".into(),
            term_program: "unknown".into(),
            cols: 80,
            rows: 24,
            color_support: false,
            true_color: false,
        }
    }
}

impl TerminalInfo {
    pub fn detect() -> Self {
        let term = std::env::var("TERM").unwrap_or_else(|_| "unknown".into());
        let term_program = std::env::var("TERM_PROGRAM")
            .or_else(|_| std::env::var("WT_SESSION").map(|_| "WindowsTerminal".into()))
            .unwrap_or_else(|_| "unknown".into());
        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        let color_term = std::env::var("COLORTERM").unwrap_or_default();
        let true_color = color_term.contains("truecolor") || color_term.contains("24bit");
        let color_support = true_color || term.contains("256color") || term.contains("color");
        Self {
            term,
            term_program,
            cols: cols as usize,
            rows: rows as usize,
            color_support,
            true_color,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageManager {
    pub name: String,
    pub command: String,
    pub version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DevToolChain {
    pub git: Option<String>,
    pub rustc: Option<String>,
    pub cargo: Option<String>,
    pub node: Option<String>,
    pub npm: Option<String>,
    pub python: Option<String>,
    pub pip: Option<String>,
    pub docker: Option<String>,
    pub go: Option<String>,
    pub java: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub shell: ShellType,
    pub shell_path: String,
    pub shell_version: String,
    pub home_dir: PathBuf,
    pub pwd: PathBuf,
    pub env_vars: HashMap<String, String>,
    pub encoding: FileEncoding,
    pub line_ending: LineEnding,
    pub path_separator: String,
    pub case_sensitive: bool,
    pub terminal: TerminalInfo,
    pub package_managers: Vec<PackageManager>,
    pub dev_tools: DevToolChain,
    pub is_wsl: bool,
    pub is_container: bool,
    pub distribution: Option<String>,
    pub detected_at: DateTime<Utc>,
}

impl Default for PlatformInfo {
    fn default() -> Self {
        Self {
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            shell: ShellType::Sh,
            shell_path: "sh".into(),
            shell_version: String::new(),
            home_dir: PathBuf::from("/home/agent"),
            pwd: PathBuf::from("."),
            env_vars: HashMap::new(),
            encoding: FileEncoding::UTF8,
            line_ending: LineEnding::system_default(),
            path_separator: std::path::MAIN_SEPARATOR.to_string(),
            case_sensitive: !cfg!(windows) && !cfg!(target_os = "macos"),
            terminal: TerminalInfo::default(),
            package_managers: Vec::new(),
            dev_tools: DevToolChain::default(),
            is_wsl: false,
            is_container: false,
            distribution: None,
            detected_at: Utc::now(),
        }
    }
}

impl PlatformInfo {
    pub async fn detect() -> Result<Self> {
        DefaultPlatformDetector::detect().await
    }

    pub fn to_context_block(&self) -> String {
        let mut env_lines = self
            .env_vars
            .iter()
            .filter(|(key, _)| !is_sensitive_env_key(key))
            .map(|(key, value)| {
                format!(
                    "- {}: {}",
                    sanitize_prompt_text(key),
                    sanitize_prompt_text(value)
                )
            })
            .collect::<Vec<_>>();
        env_lines.sort();

        let pkg_managers = if self.package_managers.is_empty() {
            "none detected".into()
        } else {
            self.package_managers
                .iter()
                .map(|pkg| format!("{} ({})", pkg.name, sanitize_prompt_text(&pkg.version)))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let block = format!(
            "## Platform Environment\n\n- OS: {}{} ({})\n- Shell: {} {}\n- File Encoding: {}\n- Line Ending: {}\n- Path Separator: {}\n- Case Sensitive Paths: {}\n- Terminal: {} ({}x{})\n- Color Support: {}\n- Package Managers: {}\n- Development Toolchain:\n{}\n{}\n### Command Guidance\n- Use {} syntax when composing shell commands.\n- Quote paths with spaces.\n- Respect {} line endings when generating files.",
            sanitize_prompt_text(&self.os),
            self.distribution
                .as_ref()
                .map(|distribution| format!(" / {}", sanitize_prompt_text(distribution)))
                .unwrap_or_default(),
            sanitize_prompt_text(&self.arch),
            self.shell.display_name(),
            sanitize_prompt_text(&self.shell_version),
            self.encoding.display_name(),
            self.line_ending.display_name(),
            sanitize_prompt_text(&self.path_separator),
            if self.case_sensitive { "yes" } else { "no" },
            sanitize_prompt_text(&self.terminal.term_program),
            self.terminal.cols,
            self.terminal.rows,
            if self.terminal.true_color {
                "true color"
            } else if self.terminal.color_support {
                "256 colors"
            } else {
                "basic"
            },
            pkg_managers,
            self.format_dev_tools(),
            if env_lines.is_empty() {
                String::new()
            } else {
                format!("### Safe Environment Hints\n{}\n", env_lines.join("\n"))
            },
            self.shell.display_name(),
            self.line_ending.display_name(),
        );

        truncate_chars(&sanitize_prompt_text(&block), MAX_CONTEXT_BLOCK_LEN)
    }

    fn format_dev_tools(&self) -> String {
        let mut tools = BTreeMap::new();
        if let Some(version) = &self.dev_tools.git {
            tools.insert("Git", version.as_str());
        }
        if let Some(version) = &self.dev_tools.rustc {
            tools.insert("Rust", version.as_str());
        }
        if let Some(version) = &self.dev_tools.cargo {
            tools.insert("Cargo", version.as_str());
        }
        if let Some(version) = &self.dev_tools.node {
            tools.insert("Node.js", version.as_str());
        }
        if let Some(version) = &self.dev_tools.python {
            tools.insert("Python", version.as_str());
        }
        if let Some(version) = &self.dev_tools.docker {
            tools.insert("Docker", version.as_str());
        }
        if let Some(version) = &self.dev_tools.go {
            tools.insert("Go", version.as_str());
        }

        if tools.is_empty() {
            "  - none detected".into()
        } else {
            tools
                .into_iter()
                .map(|(name, version)| format!("  - {}: {}", name, sanitize_prompt_text(version)))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

impl DevToolChain {
    pub async fn detect() -> Self {
        let python = match detect_command_version("python", &["--version"]).await {
            some @ Some(_) => some,
            None => detect_command_version("python3", &["--version"]).await,
        };
        let pip = match detect_command_version("pip", &["--version"]).await {
            some @ Some(_) => some,
            None => detect_command_version("pip3", &["--version"]).await,
        };
        Self {
            git: detect_command_version("git", &["--version"]).await,
            rustc: detect_command_version("rustc", &["--version"]).await,
            cargo: detect_command_version("cargo", &["--version"]).await,
            node: detect_command_version("node", &["--version"]).await,
            npm: detect_command_version("npm", &["--version"]).await,
            python,
            pip,
            docker: detect_command_version("docker", &["--version"]).await,
            go: detect_command_version("go", &["version"]).await,
            java: detect_command_version("java", &["--version"]).await,
        }
    }
}

fn detect_home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn collect_safe_env_vars() -> HashMap<String, String> {
    let mut vars = HashMap::new();
    for key in [
        "LANG",
        "LC_ALL",
        "TERM",
        "TERM_PROGRAM",
        "COLORTERM",
        "ComSpec",
        "SHELL",
    ] {
        if let Ok(value) = std::env::var(key) {
            if !is_sensitive_env_key(key) {
                vars.insert(key.to_string(), sanitize_prompt_text(&value));
            }
        }
    }
    vars
}

fn is_sensitive_env_key(key: &str) -> bool {
    let key = key.to_ascii_uppercase();
    [
        "API_KEY",
        "TOKEN",
        "SECRET",
        "PASSWORD",
        "PASSWD",
        "PRIVATE",
        "CREDENTIAL",
        "COOKIE",
        "SESSION",
        "AUTH",
    ]
    .iter()
    .any(|needle| key.contains(needle))
}

fn sanitize_prompt_text(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !matches!(ch, '\0'..='\u{001F}' | '\u{007F}') || matches!(ch, '\n' | '\t'))
        .collect::<String>()
        .trim()
        .to_string()
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        value.chars().take(max_chars).collect()
    }
}

fn quote_for_posix(value: &str) -> String {
    let safe = value
        .chars()
        .filter(|ch| !ch.is_control() || matches!(ch, '\n' | '\t'))
        .collect::<String>();
    shlex::try_quote(&safe)
        .map(|quoted| quoted.into_owned())
        .unwrap_or_else(|_| "''".into())
}

fn quote_for_powershell(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

async fn detect_command_version(cmd: &str, args: &[&str]) -> Option<String> {
    let mut command = Command::new(cmd);
    command.args(args);

    let output = timeout(TOOL_TIMEOUT, command.output()).await.ok()?.ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let value = if stdout.is_empty() { stderr } else { stdout };
    value.lines().next().map(str::to_string)
}

async fn detect_shell_version(shell: &ShellType) -> Option<String> {
    match shell {
        ShellType::Cmd => detect_command_version("cmd", &["/c", "ver"]).await,
        ShellType::PowerShell => {
            detect_command_version(
                "powershell",
                &[
                    "-NoProfile",
                    "-Command",
                    "$PSVersionTable.PSVersion.ToString()",
                ],
            )
            .await
        }
        ShellType::Pwsh => {
            detect_command_version(
                "pwsh",
                &[
                    "-NoProfile",
                    "-Command",
                    "$PSVersionTable.PSVersion.ToString()",
                ],
            )
            .await
        }
        other => detect_command_version(other.executable_name(), &["--version"]).await,
    }
}

async fn detect_os_name() -> String {
    match std::env::consts::OS {
        "windows" => "windows".into(),
        "macos" => "macos".into(),
        "linux" => {
            if std::env::var("ANDROID_ROOT").is_ok() || std::env::var("ANDROID_DATA").is_ok() {
                "android".into()
            } else {
                "linux".into()
            }
        }
        other => {
            if let Some(name) = detect_command_version("uname", &["-s"]).await {
                return name.to_ascii_lowercase();
            }
            other.into()
        }
    }
}

async fn detect_distribution() -> Option<String> {
    let content = fs::read_to_string("/etc/os-release").await.ok()?;
    content.lines().find_map(|line| {
        line.strip_prefix("ID=")
            .map(|value| value.trim_matches('"').to_string())
    })
}

async fn detect_wsl() -> bool {
    if std::env::var("WSL_DISTRO_NAME").is_ok() {
        return true;
    }
    fs::read_to_string("/proc/version")
        .await
        .map(|content| {
            let lower = content.to_ascii_lowercase();
            lower.contains("microsoft") || lower.contains("wsl")
        })
        .unwrap_or(false)
}

async fn detect_container() -> bool {
    if fs::metadata("/.dockerenv").await.is_ok() {
        return true;
    }
    fs::read_to_string("/proc/1/cgroup")
        .await
        .map(|content| content.contains("docker") || content.contains("kubepods"))
        .unwrap_or(false)
}

async fn detect_package_managers(os: &str) -> Vec<PackageManager> {
    let candidates: &[(&str, &[&str])] = match os {
        "windows" => &[
            ("winget", &["--version"]),
            ("choco", &["--version"]),
            ("scoop", &["--version"]),
        ],
        "macos" => &[("brew", &["--version"])],
        _ => &[
            ("apt", &["--version"]),
            ("dnf", &["--version"]),
            ("yum", &["--version"]),
            ("pacman", &["--version"]),
            ("apk", &["--version"]),
        ],
    };

    let mut managers = Vec::new();
    for (cmd, args) in candidates {
        if let Some(version) = detect_command_version(cmd, args).await {
            managers.push(PackageManager {
                name: (*cmd).to_string(),
                command: format!("{} {}", cmd, args.join(" ")),
                version,
            });
        }
    }
    managers
}

fn sanitize_path_env(path_value: &OsStr, home_dir: Option<&Path>) -> OsString {
    let filtered = std::env::split_paths(path_value)
        .filter(|path| !is_untrusted_path(path, home_dir))
        .collect::<Vec<_>>();

    std::env::join_paths(filtered).unwrap_or_else(|_| path_value.to_os_string())
}

fn is_untrusted_path(path: &Path, home_dir: Option<&Path>) -> bool {
    let text = path.to_string_lossy();
    if text.is_empty() || text == "." {
        return true;
    }

    let lower = text.to_ascii_lowercase();
    if lower.starts_with("/tmp") || lower.starts_with("/var/tmp") || lower.contains("\\temp") {
        return true;
    }

    if let Some(home_dir) = home_dir {
        if path.starts_with(home_dir) {
            return true;
        }
    }

    false
}

async fn detect_executable_path_inner(
    cmd_name: &str,
    raw_path: &OsStr,
    home_dir: Option<&Path>,
    pathext: &str,
) -> Option<String> {
    let safe_path = sanitize_path_env(raw_path, home_dir);
    let pathext = parse_pathext(pathext);
    let candidate = find_executable_in_path(cmd_name, &safe_path, &pathext).await?;
    let hash = hash_file_sha256(&candidate).await.ok()?;
    info!(
        command = cmd_name,
        path = %candidate.display(),
        sha256 = hash,
        "Resolved executable from sanitized PATH"
    );
    Some(candidate.display().to_string())
}

fn parse_pathext(pathext: &str) -> Vec<String> {
    let mut items = pathext
        .split(';')
        .filter(|item| !item.trim().is_empty())
        .map(|item| item.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    if items.is_empty() {
        items = vec![".exe".into(), ".cmd".into(), ".bat".into(), ".com".into()];
    }
    items
}

async fn find_executable_in_path(
    cmd_name: &str,
    sanitized_path: &OsStr,
    pathext: &[String],
) -> Option<PathBuf> {
    let has_extension = Path::new(cmd_name)
        .extension()
        .map(|extension| !extension.is_empty())
        .unwrap_or(false);

    for directory in std::env::split_paths(sanitized_path) {
        let direct = directory.join(cmd_name);
        if is_regular_file(&direct).await {
            return Some(direct);
        }

        if cfg!(windows) && !has_extension {
            for extension in pathext {
                let candidate = directory.join(format!("{cmd_name}{extension}"));
                if is_regular_file(&candidate).await {
                    return Some(candidate);
                }
            }
        }
    }

    warn!(
        command = cmd_name,
        "Unable to resolve executable from sanitized PATH"
    );
    None
}

async fn is_regular_file(path: &Path) -> bool {
    fs::metadata(path)
        .await
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
}

async fn hash_file_sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path).await?;
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(""))
}

fn looks_like_gbk(bytes: &[u8]) -> bool {
    if bytes
        .windows(2)
        .any(|pair| matches!(pair, [0xD6, 0xD0] | [0xCE, 0xC4]))
    {
        return true;
    }
    count_double_byte_matches(bytes, |lead, trail| {
        (0x81..=0xFE).contains(&lead) && (0x40..=0xFE).contains(&trail) && trail != 0x7F
    }) > 0
}

fn looks_like_shift_jis(bytes: &[u8]) -> bool {
    count_double_byte_matches(bytes, |lead, trail| {
        ((0x81..=0x9F).contains(&lead) || (0xE0..=0xFC).contains(&lead))
            && ((0x40..=0x7E).contains(&trail) || (0x80..=0xFC).contains(&trail))
    }) > 0
}

fn looks_like_big5(bytes: &[u8]) -> bool {
    bytes.windows(2).any(|pair| {
        let lead = pair[0];
        let trail = pair[1];
        (0xA1..=0xC6).contains(&lead)
            && ((0x40..=0x7E).contains(&trail) || (0xA1..=0xFE).contains(&trail))
    })
}

fn count_double_byte_matches(bytes: &[u8], predicate: impl Fn(u8, u8) -> bool) -> usize {
    bytes
        .windows(2)
        .filter(|pair| predicate(pair[0], pair[1]))
        .count()
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use tempfile::tempdir;
    use tokio::time::sleep;

    use super::{
        detect_executable_path_inner, sanitize_path_env, DefaultPlatformDetector, FileEncoding,
        PathHandler, PlatformCache, PlatformDetector, PlatformInfo, ShellConfig, ShellType,
    };

    #[test]
    fn set_env_var_uses_try_quote_for_unix_shells() {
        let config = ShellConfig {
            shell_type: ShellType::Bash,
            executable_path: "bash".into(),
            version: "5.2".into(),
            is_login_shell: false,
            is_interactive: false,
            available_shells: vec![ShellType::Bash],
        };

        let command = config.set_env_var("NAME", "hello world");
        assert!(command.contains("'hello world'"));
    }

    #[test]
    fn maybe_wrap_command_switches_shell() {
        let config = ShellConfig {
            shell_type: ShellType::PowerShell,
            executable_path: "powershell".into(),
            version: "7".into(),
            is_login_shell: false,
            is_interactive: true,
            available_shells: vec![ShellType::PowerShell, ShellType::Bash],
        };

        let wrapped = config
            .maybe_wrap_command("echo hi", &ShellType::Bash)
            .unwrap();
        assert_eq!(wrapped.0, ShellType::Bash);
        assert!(wrapped.1.contains("bash -c"));
    }

    #[test]
    fn normalize_removes_dot_segments() {
        let handler = PathHandler::new("linux");
        assert_eq!(handler.normalize("/a/./b/../c"), "/a/c");
    }

    #[test]
    fn normalize_preserves_windows_drive_prefix() {
        let handler = PathHandler::new("windows");
        assert_eq!(handler.normalize(r"C:\temp\..\app"), "C:/app");
    }

    #[test]
    fn detect_bom_and_multibyte_encodings() {
        assert_eq!(
            FileEncoding::detect_bytes(&[0xEF, 0xBB, 0xBF, b'a']),
            FileEncoding::UTF8BOM
        );
        assert_eq!(
            FileEncoding::detect_bytes(&[0xFF, 0xFE, 0x41, 0x00]),
            FileEncoding::UTF16LE
        );
        assert_eq!(
            FileEncoding::detect_bytes(&[0xD6, 0xD0, 0xCE, 0xC4]),
            FileEncoding::GBK
        );
        assert_eq!(
            FileEncoding::detect_bytes(&[0x83, 0x65, 0x83, 0x58]),
            FileEncoding::ShiftJIS
        );
        assert_eq!(
            FileEncoding::detect_bytes(&[0xA4, 0xA4, 0xA4, 0xE5]),
            FileEncoding::Big5
        );
    }

    #[test]
    fn context_block_filters_sensitive_env_and_caps_length() {
        let mut info = PlatformInfo::default();
        info.env_vars.insert("API_KEY".into(), "secret".into());
        info.env_vars.insert("LANG".into(), "en_US.UTF-8".into());

        let block = info.to_context_block();
        assert!(!block.contains("secret"));
        assert!(block.contains("LANG"));
        assert!(block.chars().count() <= 2_000);
    }

    #[test]
    fn sanitize_path_env_removes_tmp_and_home_paths() {
        let home = PathBuf::from("/home/alice");
        let raw = std::env::join_paths([
            PathBuf::from("/usr/bin"),
            PathBuf::from("/tmp"),
            PathBuf::from("/home/alice/bin"),
            PathBuf::from("/bin"),
        ])
        .unwrap();
        let safe = sanitize_path_env(&raw, Some(&home));
        let safe = safe.to_string_lossy();

        assert!(safe.contains("/usr/bin"));
        assert!(safe.contains("/bin"));
        assert!(!safe.contains("/tmp"));
        assert!(!safe.contains("/home/alice/bin"));
    }

    #[tokio::test]
    async fn detect_executable_path_prefers_safe_entries() {
        let home_dir = tempdir().unwrap();
        let safe_dir = std::env::current_dir()
            .unwrap()
            .join("target-mc-context-safe-bin")
            .join(uuid::Uuid::new_v4().to_string());
        tokio::fs::create_dir_all(&safe_dir).await.unwrap();
        let safe_file = safe_dir.join("tool");
        let unsafe_dir = home_dir.path().join("bin");
        tokio::fs::create_dir_all(&unsafe_dir).await.unwrap();
        tokio::fs::write(unsafe_dir.join("tool"), b"unsafe")
            .await
            .unwrap();
        tokio::fs::write(&safe_file, b"safe").await.unwrap();

        let path = std::env::join_paths([unsafe_dir.clone(), safe_dir.clone()]).unwrap();
        let resolved = detect_executable_path_inner("tool", &path, Some(home_dir.path()), "")
            .await
            .unwrap();

        assert_eq!(resolved, safe_file.display().to_string());
    }

    #[tokio::test]
    async fn platform_cache_reloads_on_signature_mismatch() {
        let cache = PlatformCache::with_key_and_ttl(
            *b"12345678901234567890123456789012",
            Duration::from_secs(60),
        );
        let first = cache
            .get_or_detect(|| async { Ok(PlatformInfo::default()) })
            .await
            .unwrap();

        *cache.cached_signature.write().await = Some(vec![0u8; 32]);
        let second = cache
            .get_or_detect(|| async {
                let mut info = PlatformInfo::default();
                info.os = "fallback".into();
                Ok(info)
            })
            .await
            .unwrap();

        assert_ne!(first.os, second.os);
    }

    #[tokio::test]
    async fn platform_cache_respects_ttl_and_invalidate() {
        let cache = PlatformCache::with_key_and_ttl(
            *b"12345678901234567890123456789012",
            Duration::from_millis(10),
        );

        let first = cache
            .get_or_detect(|| async { Ok(PlatformInfo::default()) })
            .await
            .unwrap();
        sleep(Duration::from_millis(25)).await;
        let second = cache
            .get_or_detect(|| async {
                let mut info = PlatformInfo::default();
                info.os = "second".into();
                Ok(info)
            })
            .await
            .unwrap();
        assert_ne!(first.os, second.os);

        cache.invalidate().await;
        let third = cache
            .get_or_detect(|| async {
                let mut info = PlatformInfo::default();
                info.os = "third".into();
                Ok(info)
            })
            .await
            .unwrap();
        assert_eq!(third.os, "third");
    }

    #[test]
    fn detector_shell_type_smoke_test() {
        let _ = DefaultPlatformDetector::detect_shell_type();
    }
}

#[derive(Debug)]
pub struct PlatformCache {
    cached_info: RwLock<Option<Arc<PlatformInfo>>>,
    cached_signature: RwLock<Option<Vec<u8>>>,
    cached_at: RwLock<Option<Instant>>,
    hmac_key: [u8; 32],
    ttl: Duration,
}

impl Default for PlatformCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformCache {
    pub fn new() -> Self {
        Self::with_key_and_ttl(STATIC_HMAC_KEY, DEFAULT_CACHE_TTL)
    }

    pub fn with_key_and_ttl(hmac_key: [u8; 32], ttl: Duration) -> Self {
        Self {
            cached_info: RwLock::new(None),
            cached_signature: RwLock::new(None),
            cached_at: RwLock::new(None),
            hmac_key,
            ttl,
        }
    }

    pub async fn get_or_detect<F, Fut>(&self, detect_fn: F) -> Result<Arc<PlatformInfo>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<PlatformInfo>>,
    {
        {
            let cached = self.cached_info.read().await;
            let signature = self.cached_signature.read().await;
            let cached_at = self.cached_at.read().await;

            if let (Some(info), Some(signature), Some(cached_at)) =
                (cached.as_ref(), signature.as_ref(), cached_at.as_ref())
            {
                if cached_at.elapsed() < self.ttl && self.verify_signature(info, signature)? {
                    return Ok(Arc::clone(info));
                }
            }
        }

        let info = Arc::new(detect_fn().await?);
        let signature = self.sign(&info)?;
        *self.cached_info.write().await = Some(Arc::clone(&info));
        *self.cached_signature.write().await = Some(signature);
        *self.cached_at.write().await = Some(Instant::now());
        Ok(info)
    }

    pub async fn invalidate(&self) {
        *self.cached_info.write().await = None;
        *self.cached_signature.write().await = None;
        *self.cached_at.write().await = None;
    }

    fn sign(&self, info: &PlatformInfo) -> Result<Vec<u8>> {
        let payload = serde_json::to_vec(info)?;
        let mut mac =
            HmacSha256::new_from_slice(&self.hmac_key).expect("HMAC accepts keys of any size");
        mac.update(&payload);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    fn verify_signature(&self, info: &PlatformInfo, signature: &[u8]) -> Result<bool> {
        let payload = serde_json::to_vec(info)?;
        let mut mac =
            HmacSha256::new_from_slice(&self.hmac_key).expect("HMAC accepts keys of any size");
        mac.update(&payload);
        Ok(mac.verify_slice(signature).is_ok())
    }
}

#[async_trait]
pub trait PlatformDetector: Send + Sync {
    async fn detect() -> Result<PlatformInfo>
    where
        Self: Sized;

    fn detect_shell_type() -> ShellType
    where
        Self: Sized;

    async fn detect_executable_path(cmd_name: &str) -> String
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultPlatformDetector;

#[async_trait]
impl PlatformDetector for DefaultPlatformDetector {
    async fn detect() -> Result<PlatformInfo> {
        let os = detect_os_name().await;
        let shell = Self::detect_shell_type();
        let shell_path = Self::detect_executable_path(shell.executable_name()).await;
        let shell_version = detect_shell_version(&shell).await.unwrap_or_default();
        let home_dir = detect_home_dir();
        let pwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let locale = std::env::var("LC_ALL")
            .or_else(|_| std::env::var("LANG"))
            .unwrap_or_default();
        let env_vars = collect_safe_env_vars();
        let encoding = FileEncoding::from_locale(&locale);
        let path_handler = PathHandler::new(&os);
        let is_wsl = detect_wsl().await;
        let is_container = detect_container().await;
        let distribution = detect_distribution().await;
        let package_managers = detect_package_managers(&os).await;
        let dev_tools = DevToolChain::detect().await;

        info!(
            os = os,
            arch = std::env::consts::ARCH,
            shell = shell.display_name(),
            "Platform detection completed"
        );

        Ok(PlatformInfo {
            os,
            arch: std::env::consts::ARCH.into(),
            shell,
            shell_path,
            shell_version,
            home_dir,
            pwd,
            env_vars,
            encoding,
            line_ending: LineEnding::system_default(),
            path_separator: path_handler.separator.to_string(),
            case_sensitive: path_handler.case_sensitive,
            terminal: TerminalInfo::detect(),
            package_managers,
            dev_tools,
            is_wsl,
            is_container,
            distribution,
            detected_at: Utc::now(),
        })
    }

    fn detect_shell_type() -> ShellType {
        std::env::var("SHELL")
            .map(|shell| ShellType::from_shell_name(&shell))
            .or_else(|_| std::env::var("ComSpec").map(|shell| ShellType::from_shell_name(&shell)))
            .or_else(|_| {
                std::env::var("TERM_PROGRAM").map(|shell| ShellType::from_shell_name(&shell))
            })
            .unwrap_or_else(|_| {
                if cfg!(windows) {
                    ShellType::PowerShell
                } else {
                    ShellType::Sh
                }
            })
    }

    async fn detect_executable_path(cmd_name: &str) -> String {
        detect_executable_path_inner(
            cmd_name,
            &std::env::var_os("PATH").unwrap_or_default(),
            Some(&detect_home_dir()),
            &std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".into()),
        )
        .await
        .unwrap_or_else(|| cmd_name.to_string())
    }
}
