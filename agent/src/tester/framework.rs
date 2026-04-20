use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use mc_core::{AgentType, ExecutionPlan, TaskDescription};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestFramework {
    Cargo,
    Pytest,
    Jest,
}

impl Display for TestFramework {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Cargo => "cargo",
            Self::Pytest => "pytest",
            Self::Jest => "jest",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestCommand {
    pub framework: TestFramework,
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

impl TestCommand {
    pub fn render(&self) -> String {
        if self.args.is_empty() {
            return self.program.clone();
        }
        format!("{} {}", self.program, self.args.join(" "))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestRunSummary {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
    pub token_estimate: u32,
    pub stdout_tail: String,
    pub stderr_tail: String,
}

#[derive(Debug, Clone, Copy)]
pub struct FrameworkDetectionContext<'a> {
    pub hint: &'a str,
}

impl TestFramework {
    pub fn build_command(&self, focused_targets: &[String], project_root: &Path) -> TestCommand {
        match self {
            Self::Cargo => {
                let mut args = vec!["test".to_string()];
                if let Some(filter) = focused_targets
                    .iter()
                    .filter_map(|path| cargo_filter_from_path(path))
                    .next()
                {
                    args.push(filter);
                }
                TestCommand {
                    framework: *self,
                    program: "cargo".to_string(),
                    args,
                    cwd: project_root.to_path_buf(),
                }
            }
            Self::Pytest => {
                let mut args = vec!["-q".to_string()];
                args.extend(
                    focused_targets
                        .iter()
                        .filter(|path| path.ends_with(".py"))
                        .take(5)
                        .cloned(),
                );
                TestCommand {
                    framework: *self,
                    program: "pytest".to_string(),
                    args,
                    cwd: project_root.to_path_buf(),
                }
            }
            Self::Jest => {
                let mut args = vec!["jest".to_string(), "--runInBand".to_string()];
                let js_paths = focused_targets
                    .iter()
                    .filter(|path| {
                        path.ends_with(".js")
                            || path.ends_with(".jsx")
                            || path.ends_with(".ts")
                            || path.ends_with(".tsx")
                    })
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>();
                if js_paths.is_empty() {
                    args.push("--passWithNoTests".to_string());
                } else {
                    args.push("--runTestsByPath".to_string());
                    args.extend(js_paths);
                }
                TestCommand {
                    framework: *self,
                    program: "npx".to_string(),
                    args,
                    cwd: project_root.to_path_buf(),
                }
            }
        }
    }
}

pub fn detect_framework(
    project_root: &Path,
    context: &FrameworkDetectionContext<'_>,
) -> TestFramework {
    let hint = context.hint.to_lowercase();
    if hint.contains("pytest") || hint.contains("python test") {
        return TestFramework::Pytest;
    }
    if hint.contains("jest") || hint.contains("npm test") || hint.contains("node test") {
        return TestFramework::Jest;
    }
    if hint.contains("cargo test") || hint.contains("rust test") {
        return TestFramework::Cargo;
    }

    if project_root.join("Cargo.toml").exists() {
        return TestFramework::Cargo;
    }
    if project_root.join("pytest.ini").exists()
        || project_root.join("pyproject.toml").exists()
        || project_root.join("tox.ini").exists()
    {
        return TestFramework::Pytest;
    }
    if project_root.join("package.json").exists()
        || project_root.join("jest.config.js").exists()
        || project_root.join("jest.config.ts").exists()
        || project_root.join("jest.config.cjs").exists()
    {
        return TestFramework::Jest;
    }

    TestFramework::Cargo
}

pub fn derive_focus_filters(
    execution_plan: Option<&ExecutionPlan>,
    task: Option<&TaskDescription>,
) -> Vec<String> {
    let mut focused = Vec::new();
    if let Some(plan) = execution_plan {
        for sub_task in &plan.sub_tasks {
            if matches!(
                sub_task.assigned_agent,
                AgentType::Coder | AgentType::Tester
            ) {
                focused.extend(sub_task.target_files.clone());
            }
        }
    }
    if let Some(task) = task {
        focused.extend(task.affected_files.clone());
    }
    focused.sort();
    focused.dedup();
    focused
}

pub fn parse_test_output(
    framework: TestFramework,
    stdout: &str,
    stderr: &str,
    exit_code: Option<i32>,
    duration_ms: u64,
) -> TestRunSummary {
    let combined = if stderr.trim().is_empty() {
        stdout.to_string()
    } else if stdout.trim().is_empty() {
        stderr.to_string()
    } else {
        format!("{stdout}\n{stderr}")
    };
    let (passed, failed, skipped) = match framework {
        TestFramework::Cargo => parse_cargo_counts(&combined),
        TestFramework::Pytest => parse_pytest_counts(&combined),
        TestFramework::Jest => parse_jest_counts(&combined),
    };

    TestRunSummary {
        success: exit_code.unwrap_or(-1) == 0 && failed == 0,
        exit_code,
        passed,
        failed,
        skipped,
        duration_ms,
        token_estimate: estimate_text_tokens(stdout, stderr),
        stdout_tail: tail_lines(stdout, 20),
        stderr_tail: tail_lines(stderr, 20),
    }
}

fn cargo_filter_from_path(path: &str) -> Option<String> {
    let stem = Path::new(path).file_stem()?.to_string_lossy();
    let filter = stem
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    if filter.is_empty() {
        None
    } else {
        Some(filter)
    }
}

fn parse_cargo_counts(text: &str) -> (usize, usize, usize) {
    let regex = Regex::new(
        r"(?i)test result:\s*(?:ok|FAILED)\.\s*(\d+)\s+passed;\s*(\d+)\s+failed;\s*(\d+)\s+ignored",
    )
    .expect("valid cargo regex");
    if let Some(caps) = regex.captures_iter(text).last() {
        let passed = caps
            .get(1)
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(0);
        let failed = caps
            .get(2)
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(0);
        let skipped = caps
            .get(3)
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(0);
        return (passed, failed, skipped);
    }
    (0, 0, 0)
}

fn parse_pytest_counts(text: &str) -> (usize, usize, usize) {
    let passed = capture_count(text, r"(\d+)\s+passed");
    let failed = capture_count(text, r"(\d+)\s+failed");
    let skipped = capture_count(text, r"(\d+)\s+skipped");
    (passed, failed, skipped)
}

fn parse_jest_counts(text: &str) -> (usize, usize, usize) {
    let regex = Regex::new(
        r"(?im)^Tests:\s*(?:(\d+)\s+failed,?\s*)?(?:(\d+)\s+skipped,?\s*)?(?:(\d+)\s+passed,?\s*)?",
    )
    .expect("valid jest regex");
    if let Some(caps) = regex.captures_iter(text).last() {
        let failed = caps
            .get(1)
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(0);
        let skipped = caps
            .get(2)
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(0);
        let passed = caps
            .get(3)
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(0);
        return (passed, failed, skipped);
    }
    (0, 0, 0)
}

fn capture_count(text: &str, pattern: &str) -> usize {
    let regex = Regex::new(pattern).expect("valid pattern");
    regex
        .captures_iter(text)
        .last()
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<usize>().ok())
        .unwrap_or(0)
}

fn estimate_text_tokens(stdout: &str, stderr: &str) -> u32 {
    ((stdout.len() + stderr.len()).div_ceil(4)) as u32
}

fn tail_lines(text: &str, count: usize) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    text.lines()
        .rev()
        .take(count)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn detect_framework_prefers_hint() {
        let temp = TempDir::new().expect("tempdir");
        std::fs::write(temp.path().join("Cargo.toml"), "[package]\nname=\"x\"").expect("write");
        let framework = detect_framework(
            temp.path(),
            &FrameworkDetectionContext {
                hint: "please run pytest quickly",
            },
        );
        assert_eq!(framework, TestFramework::Pytest);
    }

    #[test]
    fn detect_framework_from_project_files() {
        let temp = TempDir::new().expect("tempdir");
        std::fs::write(temp.path().join("package.json"), "{}").expect("write");
        let framework = detect_framework(temp.path(), &FrameworkDetectionContext { hint: "" });
        assert_eq!(framework, TestFramework::Jest);
    }

    #[test]
    fn build_cargo_command_with_focus() {
        let root = PathBuf::from(".");
        let cmd = TestFramework::Cargo.build_command(&["src/http/server.rs".to_string()], &root);
        assert_eq!(cmd.program, "cargo");
        assert_eq!(cmd.args.first().map(String::as_str), Some("test"));
        assert!(cmd.args.len() >= 2);
    }

    #[test]
    fn build_pytest_command_includes_python_targets() {
        let root = PathBuf::from(".");
        let cmd = TestFramework::Pytest.build_command(
            &[
                "tests/test_api.py".to_string(),
                "src/lib.rs".to_string(),
                "tests/test_auth.py".to_string(),
            ],
            &root,
        );
        assert_eq!(cmd.program, "pytest");
        assert!(cmd.args.contains(&"tests/test_api.py".to_string()));
        assert!(cmd.args.contains(&"tests/test_auth.py".to_string()));
        assert!(!cmd.args.contains(&"src/lib.rs".to_string()));
    }

    #[test]
    fn parse_cargo_output_counts() {
        let text =
            "test result: FAILED. 12 passed; 2 failed; 1 ignored; 0 measured; 0 filtered out";
        let summary = parse_test_output(TestFramework::Cargo, text, "", Some(101), 900);
        assert_eq!(summary.passed, 12);
        assert_eq!(summary.failed, 2);
        assert_eq!(summary.skipped, 1);
        assert!(!summary.success);
    }

    #[test]
    fn parse_pytest_output_counts() {
        let text = "=== 8 passed, 1 failed, 2 skipped in 0.81s ===";
        let summary = parse_test_output(TestFramework::Pytest, text, "", Some(1), 800);
        assert_eq!(summary.passed, 8);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.skipped, 2);
    }

    #[test]
    fn parse_jest_output_counts() {
        let text = "Test Suites: 1 passed, 1 total\nTests: 2 failed, 1 skipped, 5 passed, 8 total";
        let summary = parse_test_output(TestFramework::Jest, text, "", Some(1), 1200);
        assert_eq!(summary.passed, 5);
        assert_eq!(summary.failed, 2);
        assert_eq!(summary.skipped, 1);
    }
}
