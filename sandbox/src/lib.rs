#![cfg_attr(not(target_os = "linux"), forbid(unsafe_code))]

pub mod audit;
pub mod capability;
pub mod command;
pub mod command_whitelist;
pub mod error;
pub mod guardian;
pub mod os_layer;
pub mod path_restriction;
pub mod permission;
pub mod policy;
pub mod tool;

#[cfg(test)]
mod tests;

pub use audit::{AuditEntry, AuditFilter, AuditLogger};
pub use capability::{Capability, CapabilityDeclaration, PermissionLevel};
pub use command::{
    check_destructive_patterns, contains_shell_control_operators, is_destructive_command,
    parse_command, render_command, ParsedCommand,
};
pub use command_whitelist::{CommandRule, CommandWhitelist};
pub use error::{CommandParseError, SandboxError, WasmSandboxError};
pub use guardian::{Guardian, GuardianConfig, GuardianDecision, GuardianMode};
pub use os_layer::{
    WasiAccessPlan, WasiDirectoryAccess, WasmExecutionRequest, WasmExecutionResult, WasmModule,
    WasmSandbox, WasmSandboxLimits,
};
pub use path_restriction::{PathRestriction, PathRestrictionType};
pub use permission::{PermissionCheckResult, TaskPermission, TaskPermissionManager};
pub use tool::{ShellExecTool, ToolCallArgs};
