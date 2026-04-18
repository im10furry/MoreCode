use std::collections::HashMap;
use std::path::PathBuf;

use regex::escape;
use serde::{Deserialize, Serialize};

use crate::capability::{Capability, CapabilityDeclaration, PermissionLevel};
use crate::command::parse_command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCallArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_path: Option<PathBuf>,
    pub is_write: bool,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<CapabilityDeclaration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
}

impl ToolCallArgs {
    pub fn shell_exec(command: impl Into<String>) -> Self {
        Self {
            command: Some(command.into()),
            target_path: None,
            is_write: true,
            extra: HashMap::new(),
            capability: None,
            task_id: None,
        }
    }

    pub fn file_read(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            command: None,
            target_path: Some(path.clone()),
            is_write: false,
            extra: HashMap::new(),
            capability: Some(CapabilityDeclaration::new(
                "file_read",
                "读取文件内容",
                PermissionLevel::Public,
                vec![Capability::ReadFile {
                    pattern: path.to_string_lossy().to_string(),
                }],
            )),
            task_id: None,
        }
    }

    pub fn file_write(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            command: None,
            target_path: Some(path.clone()),
            is_write: true,
            extra: HashMap::new(),
            capability: Some(CapabilityDeclaration::new(
                "file_write",
                "写入文件内容",
                PermissionLevel::Elevated,
                vec![Capability::WriteFile {
                    pattern: path.to_string_lossy().to_string(),
                }],
            )),
            task_id: None,
        }
    }

    pub fn with_target_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.target_path = Some(path.into());
        self
    }

    pub fn with_capability(mut self, capability: CapabilityDeclaration) -> Self {
        self.capability = Some(capability);
        self
    }

    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct ShellExecTool {
    allowed_commands: Vec<String>,
    read_patterns: Vec<String>,
    write_patterns: Vec<String>,
}

impl ShellExecTool {
    pub fn new(allowed_commands: Vec<String>) -> Self {
        Self {
            allowed_commands,
            read_patterns: Vec::new(),
            write_patterns: Vec::new(),
        }
    }

    pub fn with_read_patterns(mut self, patterns: Vec<String>) -> Self {
        self.read_patterns = patterns;
        self
    }

    pub fn with_write_patterns(mut self, patterns: Vec<String>) -> Self {
        self.write_patterns = patterns;
        self
    }

    pub fn declaration(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        permission_level: PermissionLevel,
    ) -> CapabilityDeclaration {
        let mut capabilities = Vec::new();

        if !self.allowed_commands.is_empty() {
            capabilities.push(Capability::RunCommand {
                pattern: self
                    .allowed_commands
                    .iter()
                    .map(|command| escape(command))
                    .collect::<Vec<_>>()
                    .join("|"),
            });
        }

        capabilities.extend(
            self.read_patterns
                .iter()
                .cloned()
                .map(|pattern| Capability::ReadFile { pattern }),
        );
        capabilities.extend(
            self.write_patterns
                .iter()
                .cloned()
                .map(|pattern| Capability::WriteFile { pattern }),
        );

        CapabilityDeclaration::new(name, description, permission_level, capabilities)
    }

    pub fn command_invocation_capability(&self, command: &str) -> Option<Capability> {
        parse_command(command)
            .ok()
            .map(|parsed| Capability::RunCommand {
                pattern: escape(&parsed.executable_name),
            })
    }
}
