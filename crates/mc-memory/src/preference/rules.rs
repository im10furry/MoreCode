use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::MemoryError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserRule {
    pub id: String,
    pub description: String,
    pub rule_type: RuleType,
    pub scope: RuleScope,
    pub created_at: DateTime<Utc>,
    pub source: RuleSource,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleType {
    ForbiddenWords {
        words: Vec<String>,
        case_sensitive: bool,
    },
    FileFilter {
        patterns: Vec<String>,
    },
    CodeConstraint {
        forbidden_patterns: Vec<String>,
        required_patterns: Vec<String>,
    },
    OutputConstraint {
        max_length: Option<u32>,
        language: Option<String>,
        forbidden_content: Vec<String>,
    },
    NamingConstraint {
        target: String,
        style: String,
    },
    FreeText {
        instruction: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuleScope {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "project")]
    Project,
    #[serde(rename = "project_local")]
    ProjectLocal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleSource {
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "llm_extracted")]
    LlmExtracted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RuleValidationResult {
    pub passed: bool,
    pub violations: Vec<RuleViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuleViolation {
    pub rule_id: String,
    pub rule_description: String,
    pub matched_content: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleBundle {
    pub rules: Vec<UserRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RuleDocument {
    version: u32,
    rules: Vec<UserRule>,
}

pub trait RuleValidatorTrait: Send + Sync {
    fn validate_output(&self, output: &str, rules: &[UserRule]) -> RuleValidationResult;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RuleValidator;

impl RuleValidator {
    fn regex(pattern: &str) -> Result<Regex, MemoryError> {
        Regex::new(pattern).map_err(|source| MemoryError::InvalidRuleRegex {
            pattern: pattern.to_string(),
            source,
        })
    }

    pub fn validate_paths<P: AsRef<Path>>(paths: &[P], rules: &[UserRule]) -> RuleValidationResult {
        let mut violations = Vec::new();
        for rule in rules.iter().filter(|rule| rule.enabled) {
            if let RuleType::FileFilter { patterns } = &rule.rule_type {
                for path in paths {
                    let path = path.as_ref().to_string_lossy().replace('\\', "/");
                    if patterns.iter().any(|pattern| path.contains(pattern)) {
                        violations.push(RuleViolation {
                            rule_id: rule.id.clone(),
                            rule_description: rule.description.clone(),
                            matched_content: path.clone(),
                            location: "path".to_string(),
                        });
                    }
                }
            }
        }
        RuleValidationResult {
            passed: violations.is_empty(),
            violations,
        }
    }
}

impl RuleValidatorTrait for RuleValidator {
    fn validate_output(&self, output: &str, rules: &[UserRule]) -> RuleValidationResult {
        let mut violations = Vec::new();

        for rule in rules.iter().filter(|rule| rule.enabled) {
            match &rule.rule_type {
                RuleType::ForbiddenWords {
                    words,
                    case_sensitive,
                } => {
                    let haystack = if *case_sensitive {
                        output.to_string()
                    } else {
                        output.to_lowercase()
                    };
                    for word in words {
                        let needle = if *case_sensitive {
                            word.clone()
                        } else {
                            word.to_lowercase()
                        };
                        if haystack.contains(&needle) {
                            violations.push(RuleViolation {
                                rule_id: rule.id.clone(),
                                rule_description: rule.description.clone(),
                                matched_content: word.clone(),
                                location: "output".to_string(),
                            });
                        }
                    }
                }
                RuleType::CodeConstraint {
                    forbidden_patterns,
                    required_patterns,
                } => {
                    for pattern in forbidden_patterns {
                        if let Ok(regex) = Self::regex(pattern) {
                            for capture in regex.find_iter(output) {
                                violations.push(RuleViolation {
                                    rule_id: rule.id.clone(),
                                    rule_description: rule.description.clone(),
                                    matched_content: capture.as_str().to_string(),
                                    location: "code".to_string(),
                                });
                            }
                        }
                    }

                    for pattern in required_patterns {
                        if let Ok(regex) = Self::regex(pattern) {
                            if !regex.is_match(output) {
                                violations.push(RuleViolation {
                                    rule_id: rule.id.clone(),
                                    rule_description: rule.description.clone(),
                                    matched_content: pattern.clone(),
                                    location: "code-missing".to_string(),
                                });
                            }
                        }
                    }
                }
                RuleType::OutputConstraint {
                    max_length,
                    language,
                    forbidden_content,
                } => {
                    if let Some(max_length) = max_length {
                        let actual_length = output.chars().count() as u32;
                        if actual_length > *max_length {
                            violations.push(RuleViolation {
                                rule_id: rule.id.clone(),
                                rule_description: rule.description.clone(),
                                matched_content: format!("{actual_length}>{max_length}"),
                                location: "output-length".to_string(),
                            });
                        }
                    }

                    if let Some(language) = language {
                        if language.eq_ignore_ascii_case("json")
                            && !(output.trim_start().starts_with('{')
                                || output.trim_start().starts_with('['))
                        {
                            violations.push(RuleViolation {
                                rule_id: rule.id.clone(),
                                rule_description: rule.description.clone(),
                                matched_content: "language=json".to_string(),
                                location: "output-format".to_string(),
                            });
                        }
                    }

                    for pattern in forbidden_content {
                        if let Ok(regex) = Self::regex(pattern) {
                            if let Some(capture) = regex.find(output) {
                                violations.push(RuleViolation {
                                    rule_id: rule.id.clone(),
                                    rule_description: rule.description.clone(),
                                    matched_content: capture.as_str().to_string(),
                                    location: "output".to_string(),
                                });
                            }
                        }
                    }
                }
                RuleType::NamingConstraint { style, .. } => {
                    let matched = match style.as_str() {
                        "snake_case" => output.contains('-') || output.contains(char::is_uppercase),
                        "kebab-case" => output.contains('_') || output.contains(char::is_uppercase),
                        "PascalCase" => output.contains('_') || output.contains('-'),
                        _ => false,
                    };
                    if matched {
                        violations.push(RuleViolation {
                            rule_id: rule.id.clone(),
                            rule_description: rule.description.clone(),
                            matched_content: style.clone(),
                            location: "naming".to_string(),
                        });
                    }
                }
                RuleType::FileFilter { .. } | RuleType::FreeText { .. } => {}
            }
        }

        RuleValidationResult {
            passed: violations.is_empty(),
            violations,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuleLoader {
    project_root: PathBuf,
}

impl RuleLoader {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    pub async fn load(&self) -> Result<RuleBundle, MemoryError> {
        let mut merged = Vec::<UserRule>::new();

        for (scope, path) in self.rule_paths() {
            if let Some(document) = load_rule_document(&path).await? {
                merge_rules(&mut merged, document.rules, scope);
            }
        }

        Ok(RuleBundle { rules: merged })
    }

    fn rule_paths(&self) -> Vec<(RuleScope, PathBuf)> {
        let mut paths = Vec::new();
        if let Some(data_dir) = dirs::data_dir() {
            paths.push((
                RuleScope::User,
                data_dir.join("morecode").join("user-rules.json"),
            ));
        }
        paths.push((
            RuleScope::Project,
            self.project_root.join(".morecode-rules.json"),
        ));
        paths.push((
            RuleScope::ProjectLocal,
            self.project_root
                .join(".assistant-memory")
                .join("user-rules.json"),
        ));
        paths
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RuleEnforcer;

impl RuleEnforcer {
    pub fn system_prompt_block(rules: &[UserRule]) -> String {
        if rules.is_empty() {
            return String::new();
        }

        let mut lines = Vec::with_capacity(rules.len() + 2);
        lines.push("=== 用户规则（必须遵守） ===".to_string());
        for rule in rules.iter().filter(|rule| rule.enabled) {
            lines.push(format!(
                "- [{}] {}",
                render_rule_type(&rule.rule_type),
                rule.description
            ));
        }
        lines.push("=== 输出后仍需通过规则校验 ===".to_string());
        lines.join("\n")
    }

    pub fn validate_output(output: &str, rules: &[UserRule]) -> RuleValidationResult {
        RuleValidator.validate_output(output, rules)
    }

    pub fn apply_dual_guard(
        base_system_prompt: &str,
        output: &str,
        rules: &[UserRule],
    ) -> (String, RuleValidationResult) {
        let prompt = if rules.is_empty() {
            base_system_prompt.to_string()
        } else {
            format!(
                "{}\n\n{}",
                base_system_prompt,
                Self::system_prompt_block(rules)
            )
        };
        let validation = Self::validate_output(output, rules);
        (prompt, validation)
    }
}

async fn load_rule_document(path: &Path) -> Result<Option<RuleDocument>, MemoryError> {
    if !fs::try_exists(path).await? {
        return Ok(None);
    }

    let contents = fs::read_to_string(path).await?;
    Ok(Some(serde_json::from_str(&contents)?))
}

fn merge_rules(target: &mut Vec<UserRule>, rules: Vec<UserRule>, scope: RuleScope) {
    let mut by_id = HashMap::<String, usize>::new();
    for (index, existing) in target.iter().enumerate() {
        by_id.insert(existing.id.clone(), index);
    }

    for mut rule in rules {
        rule.scope = scope.clone();
        if let Some(index) = by_id.get(&rule.id).copied() {
            target[index] = rule;
        } else {
            by_id.insert(rule.id.clone(), target.len());
            target.push(rule);
        }
    }

    target.sort_by(|left, right| left.scope.cmp(&right.scope));
}

fn render_rule_type(rule_type: &RuleType) -> &'static str {
    match rule_type {
        RuleType::ForbiddenWords { .. } => "禁用词",
        RuleType::FileFilter { .. } => "文件过滤",
        RuleType::CodeConstraint { .. } => "代码约束",
        RuleType::OutputConstraint { .. } => "输出约束",
        RuleType::NamingConstraint { .. } => "命名约束",
        RuleType::FreeText { .. } => "自由文本",
    }
}
