use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use mc_sandbox::{CapabilityDeclaration, PermissionLevel};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    Core,
    Extended,
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityLayer {
    Public,
    Project,
    Admin,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionScope {
    Workspace,
    Filesystem,
    Search,
    Process,
    VersionControl,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub strict: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolResultStatus {
    Success,
    Error,
    Content,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub status: ToolResultStatus,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
    pub duration_ms: u64,
}

impl ToolResult {
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            status: ToolResultStatus::Success,
            content: content.into(),
            data: None,
            metadata: HashMap::new(),
            duration_ms: 0,
        }
    }

    pub fn success_with_data(content: impl Into<String>, data: Value) -> Self {
        Self {
            status: ToolResultStatus::Success,
            content: content.into(),
            data: Some(data),
            metadata: HashMap::new(),
            duration_ms: 0,
        }
    }

    pub fn error(content: impl Into<String>) -> Self {
        Self {
            status: ToolResultStatus::Error,
            content: content.into(),
            data: None,
            metadata: HashMap::new(),
            duration_ms: 0,
        }
    }

    pub fn content(content: impl Into<String>) -> Self {
        Self {
            status: ToolResultStatus::Content,
            content: content.into(),
            data: None,
            metadata: HashMap::new(),
            duration_ms: 0,
        }
    }

    pub fn content_with_data(content: impl Into<String>, data: Value) -> Self {
        Self {
            status: ToolResultStatus::Content,
            content: content.into(),
            data: Some(data),
            metadata: HashMap::new(),
            duration_ms: 0,
        }
    }

    pub fn partial(content: impl Into<String>) -> Self {
        Self::content(content)
    }

    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.duration_ms = duration.as_millis() as u64;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn is_success(&self) -> bool {
        matches!(
            self.status,
            ToolResultStatus::Success | ToolResultStatus::Content
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermission {
    pub tool_name: String,
    pub level: PermissionLevel,
    pub scope: PermissionScope,
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;

    fn description(&self) -> &str;

    fn execute(&self, params: Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>>;

    fn required_parameters(&self) -> Value;

    fn is_read_only(&self) -> bool;

    fn category(&self) -> ToolCategory {
        ToolCategory::Core
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Standard
    }

    fn permission_scope(&self) -> PermissionScope {
        PermissionScope::Workspace
    }

    fn capability(&self) -> CapabilityDeclaration;

    fn permission(&self) -> ToolPermission {
        ToolPermission {
            tool_name: self.name().to_string(),
            level: self.permission_level(),
            scope: self.permission_scope(),
        }
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: self.required_parameters(),
            strict: false,
        }
    }
}
