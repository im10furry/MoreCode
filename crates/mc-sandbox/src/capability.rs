use globset::Glob;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionLevel {
    Public,
    Standard,
    Elevated,
    Admin,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Capability {
    ReadFile { pattern: String },
    WriteFile { pattern: String },
    RunCommand { pattern: String },
    NetworkAccess { pattern: String },
}

impl Capability {
    pub fn matches(&self, required: &Capability) -> bool {
        match (self, required) {
            (Capability::ReadFile { pattern }, Capability::ReadFile { pattern: required })
            | (Capability::WriteFile { pattern }, Capability::WriteFile { pattern: required }) => {
                glob_matches(pattern, required)
            }
            (Capability::RunCommand { pattern }, Capability::RunCommand { pattern: required })
            | (
                Capability::NetworkAccess { pattern },
                Capability::NetworkAccess { pattern: required },
            ) => regex_matches(pattern, required),
            _ => false,
        }
    }

    pub fn description(&self) -> String {
        match self {
            Capability::ReadFile { pattern } => format!("读取文件: {pattern}"),
            Capability::WriteFile { pattern } => format!("写入文件: {pattern}"),
            Capability::RunCommand { pattern } => format!("执行命令: {pattern}"),
            Capability::NetworkAccess { pattern } => format!("访问网络: {pattern}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDeclaration {
    pub name: String,
    pub description: String,
    pub permission_level: PermissionLevel,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<Capability>,
}

impl CapabilityDeclaration {
    pub fn new<T>(
        name: impl Into<String>,
        description: impl Into<String>,
        permission_level: PermissionLevel,
        capabilities: T,
    ) -> Self
    where
        T: IntoCapabilitySpec,
    {
        Self {
            name: name.into(),
            description: description.into(),
            permission_level,
            capabilities: capabilities.into_capabilities(),
        }
    }

    pub fn is_read_only(&self) -> bool {
        !self.capabilities.is_empty()
            && self
                .capabilities
                .iter()
                .all(|capability| matches!(capability, Capability::ReadFile { .. }))
    }

    pub fn is_complete(&self) -> bool {
        !self.capabilities.is_empty()
    }

    pub fn permission_description(&self) -> String {
        if self.capabilities.is_empty() {
            return self.description.clone();
        }

        let details = self
            .capabilities
            .iter()
            .map(Capability::description)
            .collect::<Vec<_>>()
            .join("，");
        format!("{}（{}）", self.description, details)
    }
}

pub trait IntoCapabilitySpec {
    fn into_capabilities(self) -> Vec<Capability>;
}

impl IntoCapabilitySpec for Vec<Capability> {
    fn into_capabilities(self) -> Vec<Capability> {
        self
    }
}

impl IntoCapabilitySpec for bool {
    fn into_capabilities(self) -> Vec<Capability> {
        if self {
            vec![Capability::ReadFile {
                pattern: "**".to_string(),
            }]
        } else {
            vec![Capability::WriteFile {
                pattern: "**".to_string(),
            }]
        }
    }
}

fn glob_matches(pattern: &str, candidate: &str) -> bool {
    let pattern = normalize_path_like(pattern);
    let candidate = normalize_path_like(candidate);

    match Glob::new(&pattern) {
        Ok(glob) => glob.compile_matcher().is_match(candidate),
        Err(_) => pattern == candidate,
    }
}

fn regex_matches(pattern: &str, candidate: &str) -> bool {
    Regex::new(&format!("^(?:{})$", pattern))
        .map(|regex| regex.is_match(candidate))
        .unwrap_or_else(|_| pattern == candidate)
}

fn normalize_path_like(value: &str) -> String {
    value.replace('\\', "/")
}
