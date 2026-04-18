use std::collections::HashMap;

use mc_core::{AgentType, Complexity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    FeatureDevelopment,
    BugFix,
    Refactoring,
    Documentation,
    Testing,
    Configuration,
    CodeReview,
    Debugging,
    Other(String),
}

impl TaskType {
    pub fn as_key(&self) -> String {
        match self {
            Self::FeatureDevelopment => "FeatureDevelopment".into(),
            Self::BugFix => "BugFix".into(),
            Self::Refactoring => "Refactoring".into(),
            Self::Documentation => "Documentation".into(),
            Self::Testing => "Testing".into(),
            Self::Configuration => "Configuration".into(),
            Self::CodeReview => "CodeReview".into(),
            Self::Debugging => "Debugging".into(),
            Self::Other(value) => format!("Other:{value}"),
        }
    }

    pub fn preferred_agent(&self) -> AgentType {
        match self {
            Self::Documentation => AgentType::DocWriter,
            Self::Testing => AgentType::Tester,
            Self::CodeReview => AgentType::Reviewer,
            Self::Debugging => AgentType::Debugger,
            Self::Other(kind) if kind.eq_ignore_ascii_case("research") => AgentType::Research,
            _ => AgentType::Coder,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserIntent {
    pub raw_request: String,
    pub task_type: TaskType,
    pub target_files: Vec<String>,
    pub domains: Vec<String>,
    pub estimated_complexity: Complexity,
    pub needs_project_context: bool,
    pub needs_research: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Question {
    pub key: String,
    pub prompt: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Clarification {
    pub questions: Vec<Question>,
    #[serde(default)]
    pub suggestions: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntentAnalysis {
    pub intent: UserIntent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clarifications: Option<Clarification>,
}

pub fn keyword_fast_path(request: &str) -> Option<UserIntent> {
    let normalized = request.to_lowercase();

    if normalized.starts_with("/full-analysis") {
        return Some(UserIntent {
            raw_request: request.to_string(),
            task_type: TaskType::CodeReview,
            target_files: extract_target_files(request),
            domains: infer_domains(request),
            estimated_complexity: Complexity::Complex,
            needs_project_context: true,
            needs_research: false,
        });
    }

    let patterns: &[(&[&str], TaskType, Complexity)] = &[
        (
            &["修复 typo", "fix typo", "typo"],
            TaskType::BugFix,
            Complexity::Simple,
        ),
        (
            &["格式化代码", "format code", "lint"],
            TaskType::Refactoring,
            Complexity::Simple,
        ),
        (
            &["添加注释", "write docs", "update docs"],
            TaskType::Documentation,
            Complexity::Simple,
        ),
        (
            &["运行测试", "add test", "write test"],
            TaskType::Testing,
            Complexity::Simple,
        ),
        (
            &["深入排查", "deep debug", "root cause"],
            TaskType::Debugging,
            Complexity::Complex,
        ),
        (
            &["先调研", "research first"],
            TaskType::Other("research".into()),
            Complexity::Research,
        ),
    ];

    for (keywords, task_type, complexity) in patterns {
        if keywords.iter().any(|keyword| normalized.contains(keyword)) {
            let target_files = extract_target_files(request);
            return Some(UserIntent {
                raw_request: request.to_string(),
                task_type: task_type.clone(),
                target_files,
                domains: infer_domains(request),
                estimated_complexity: *complexity,
                needs_project_context: matches!(
                    complexity,
                    Complexity::Medium | Complexity::Complex | Complexity::Research
                ),
                needs_research: matches!(complexity, Complexity::Research),
            });
        }
    }

    None
}

pub fn keyword_fallback(request: &str) -> UserIntent {
    let normalized = request.to_lowercase();
    let task_type =
        if normalized.contains("bug") || normalized.contains("修复") || normalized.contains("fix")
        {
            TaskType::BugFix
        } else if normalized.contains("重构") || normalized.contains("refactor") {
            TaskType::Refactoring
        } else if normalized.contains("测试") || normalized.contains("test") {
            TaskType::Testing
        } else if normalized.contains("文档") || normalized.contains("doc") {
            TaskType::Documentation
        } else if normalized.contains("review") || normalized.contains("审查") {
            TaskType::CodeReview
        } else if normalized.contains("debug") || normalized.contains("排查") {
            TaskType::Debugging
        } else {
            TaskType::FeatureDevelopment
        };

    let needs_research = normalized.contains("research") || normalized.contains("调研");
    let estimated_complexity = if needs_research {
        Complexity::Research
    } else if normalized.contains("架构")
        || normalized.contains("跨模块")
        || normalized.contains("multi-module")
    {
        Complexity::Complex
    } else if normalized.contains("新增")
        || normalized.contains("add")
        || normalized.contains("implement")
    {
        Complexity::Medium
    } else {
        Complexity::Simple
    };

    let target_files = extract_target_files(request);
    UserIntent {
        raw_request: request.to_string(),
        task_type,
        target_files: target_files.clone(),
        domains: infer_domains(request),
        estimated_complexity,
        needs_project_context: !target_files.is_empty()
            || !matches!(estimated_complexity, Complexity::Simple),
        needs_research,
    }
}

pub fn extract_target_files(request: &str) -> Vec<String> {
    request
        .split_whitespace()
        .map(|token| {
            token.trim_matches(|ch: char| {
                matches!(ch, ',' | '.' | '"' | '\'' | '(' | ')' | '[' | ']')
            })
        })
        .filter(|token| {
            token.contains('/')
                || token.ends_with(".rs")
                || token.ends_with(".md")
                || token.ends_with(".toml")
                || token.ends_with(".json")
        })
        .map(ToOwned::to_owned)
        .collect()
}

pub fn infer_domains(request: &str) -> Vec<String> {
    let normalized = request.to_lowercase();
    let mut domains = Vec::new();

    let candidates = [
        ("api", "api"),
        ("router", "routing"),
        ("database", "database"),
        ("sql", "database"),
        ("auth", "authentication"),
        ("测试", "testing"),
        ("test", "testing"),
        ("文档", "documentation"),
        ("doc", "documentation"),
        ("配置", "configuration"),
        ("config", "configuration"),
        ("性能", "performance"),
        ("performance", "performance"),
    ];

    for (needle, domain) in candidates {
        if normalized.contains(needle) && !domains.iter().any(|item| item == domain) {
            domains.push(domain.to_string());
        }
    }

    domains
}

pub fn intent_analysis_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["intent"],
        "properties": {
            "intent": {
                "type": "object",
                "required": [
                    "raw_request",
                    "task_type",
                    "target_files",
                    "domains",
                    "estimated_complexity",
                    "needs_project_context",
                    "needs_research"
                ],
                "properties": {
                    "raw_request": { "type": "string" },
                    "task_type": { "type": ["string", "object"] },
                    "target_files": { "type": "array", "items": { "type": "string" } },
                    "domains": { "type": "array", "items": { "type": "string" } },
                    "estimated_complexity": { "type": "string" },
                    "needs_project_context": { "type": "boolean" },
                    "needs_research": { "type": "boolean" }
                }
            },
            "clarifications": {
                "type": ["object", "null"],
                "properties": {
                    "questions": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["key", "prompt", "reason"],
                            "properties": {
                                "key": { "type": "string" },
                                "prompt": { "type": "string" },
                                "reason": { "type": "string" }
                            }
                        }
                    },
                    "suggestions": {
                        "type": "object",
                        "additionalProperties": { "type": "string" }
                    }
                }
            }
        }
    })
}
