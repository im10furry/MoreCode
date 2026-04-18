use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;

use crate::audit::{AuditEntry, AuditLogger};
use crate::capability::{Capability, PermissionLevel};
use crate::command::{check_destructive_patterns, parse_command};
use crate::command_whitelist::CommandWhitelist;
use crate::path_restriction::PathRestriction;
use crate::permission::{PermissionCheckResult, TaskPermissionManager};
use crate::tool::ToolCallArgs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardianMode {
    #[default]
    Default,
    Auto,
    ReadOnly,
    Plan,
    Bypass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardianDecision {
    Allow,
    Deny { reason: String },
    ConfirmRequired { reason: String },
    Simulate { mock_result: String },
}

impl GuardianDecision {
    pub fn allow() -> Self {
        Self::Allow
    }

    pub fn deny(reason: impl Into<String>) -> Self {
        Self::Deny {
            reason: reason.into(),
        }
    }

    pub fn confirm(reason: impl Into<String>) -> Self {
        Self::ConfirmRequired {
            reason: reason.into(),
        }
    }

    pub fn simulate(message: impl Into<String>) -> Self {
        Self::Simulate {
            mock_result: message.into(),
        }
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, GuardianDecision::Deny { .. })
    }

    pub fn decision_result(&self) -> &'static str {
        match self {
            GuardianDecision::Allow => "allow",
            GuardianDecision::Deny { .. } => "deny",
            GuardianDecision::ConfirmRequired { .. } => "confirm_required",
            GuardianDecision::Simulate { .. } => "simulate",
        }
    }

    pub fn detail(&self) -> String {
        match self {
            GuardianDecision::Allow => "allowed".to_string(),
            GuardianDecision::Deny { reason } | GuardianDecision::ConfirmRequired { reason } => {
                reason.clone()
            }
            GuardianDecision::Simulate { mock_result } => mock_result.clone(),
        }
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            GuardianDecision::Deny { reason } | GuardianDecision::ConfirmRequired { reason } => {
                Some(reason.as_str())
            }
            GuardianDecision::Simulate { mock_result } => Some(mock_result.as_str()),
            GuardianDecision::Allow => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianConfig {
    #[serde(default)]
    pub mode: GuardianMode,
    #[serde(default)]
    pub safe_commands: CommandWhitelist,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path_restrictions: Vec<PathRestriction>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub blocked_tools: HashSet<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_permission: Option<PermissionLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_log_path: Option<PathBuf>,
}

impl Default for GuardianConfig {
    fn default() -> Self {
        Self {
            mode: GuardianMode::Default,
            safe_commands: CommandWhitelist::default(),
            path_restrictions: Vec::new(),
            blocked_tools: HashSet::new(),
            max_permission: None,
            audit_log_path: None,
        }
    }
}

#[derive(Debug)]
pub struct Guardian {
    mode: RwLock<GuardianMode>,
    safe_commands: RwLock<CommandWhitelist>,
    path_restrictions: RwLock<Vec<PathRestriction>>,
    blocked_tools: RwLock<HashSet<String>>,
    task_permissions: Mutex<TaskPermissionManager>,
    audit_log: Arc<AuditLogger>,
    max_permission: Option<PermissionLevel>,
    audit_enabled: bool,
}

impl Guardian {
    pub fn new(config: GuardianConfig) -> Self {
        let audit_log = config
            .audit_log_path
            .clone()
            .map(|path| AuditLogger::new().with_log_file(path))
            .unwrap_or_default();

        Self {
            mode: RwLock::new(config.mode),
            safe_commands: RwLock::new(config.safe_commands),
            path_restrictions: RwLock::new(config.path_restrictions),
            blocked_tools: RwLock::new(config.blocked_tools),
            task_permissions: Mutex::new(TaskPermissionManager::default()),
            audit_log: Arc::new(audit_log),
            max_permission: config.max_permission,
            audit_enabled: true,
        }
    }

    pub fn allow_all() -> Self {
        Self::new(GuardianConfig {
            mode: GuardianMode::Auto,
            safe_commands: CommandWhitelist::new(),
            ..GuardianConfig::default()
        })
    }

    pub fn audit_enabled(&self) -> bool {
        self.audit_enabled
    }

    pub fn audit_log(&self) -> Arc<AuditLogger> {
        Arc::clone(&self.audit_log)
    }

    pub async fn set_mode(&self, mode: GuardianMode) {
        *self.mode.write().await = mode;
    }

    pub async fn add_path_restriction(&self, restriction: PathRestriction) {
        self.path_restrictions.write().await.push(restriction);
    }

    pub fn grant_task_permission(
        &self,
        task_id: impl Into<String>,
        capabilities: Vec<Capability>,
        ttl: Duration,
        max_uses: Option<u32>,
    ) {
        if let Ok(mut permissions) = self.task_permissions.lock() {
            permissions.grant(task_id, capabilities, ttl, max_uses);
        }
    }

    pub fn revoke_task_permission(&self, task_id: &str) {
        if let Ok(mut permissions) = self.task_permissions.lock() {
            permissions.revoke(task_id);
        }
    }

    pub async fn check_tool_call(
        &self,
        caller: &str,
        tool_name: &str,
        args: &ToolCallArgs,
    ) -> GuardianDecision {
        let mode = *self.mode.read().await;
        let decision = self.evaluate(mode, tool_name, args).await;
        self.record(caller, tool_name, args, &decision);
        decision
    }

    async fn evaluate(
        &self,
        mode: GuardianMode,
        tool_name: &str,
        args: &ToolCallArgs,
    ) -> GuardianDecision {
        if self.blocked_tools.read().await.contains(tool_name) {
            return GuardianDecision::deny(format!("工具 `{tool_name}` 被策略阻止"));
        }

        let Some(capability) = args.capability.as_ref() else {
            return GuardianDecision::deny("工具调用缺少 CapabilityDeclaration");
        };

        if !capability.is_complete() {
            return GuardianDecision::deny(format!(
                "工具 `{}` 未声明完整能力需求",
                capability.name
            ));
        }

        if let Some(max_permission) = self.max_permission {
            if capability.permission_level > max_permission {
                return GuardianDecision::deny(format!(
                    "权限级别 `{}` 超过允许上限 `{}`",
                    permission_level_name(capability.permission_level),
                    permission_level_name(max_permission),
                ));
            }
        }

        if let Some(command) = &args.command {
            let parsed = match parse_command(command) {
                Ok(parsed) => parsed,
                Err(error) => return GuardianDecision::deny(error.to_string()),
            };

            if let Some(reason) = check_destructive_patterns(&parsed) {
                return GuardianDecision::deny(reason);
            }

            if mode != GuardianMode::Bypass && !self.safe_commands.read().await.is_safe(&parsed) {
                return GuardianDecision::deny(format!(
                    "命令 `{}` 未通过白名单检查",
                    parsed.executable_name
                ));
            }
        }

        if let Some(path) = &args.target_path {
            let restrictions = self.path_restrictions.read().await;
            if !PathRestriction::allows_path(&restrictions, path, args.is_write) {
                return GuardianDecision::deny(format!("路径访问被拒绝: {}", path.display()));
            }
        }

        if let Some(task_id) = args.task_id.as_deref() {
            let required_capabilities = capability.capabilities.as_slice();
            let result = self
                .task_permissions
                .lock()
                .ok()
                .map(|mut permissions| {
                    permissions.check_permissions(task_id, required_capabilities)
                })
                .unwrap_or_else(|| PermissionCheckResult::Denied {
                    reason: "无法获取任务权限锁".to_string(),
                });

            if let PermissionCheckResult::Denied { reason } = result {
                return GuardianDecision::deny(reason);
            }
        }

        if mode == GuardianMode::Plan {
            return GuardianDecision::simulate(format!("[plan] 模拟执行工具 `{tool_name}`"));
        }

        if mode == GuardianMode::ReadOnly && args.is_write {
            return GuardianDecision::deny("ReadOnly 模式禁止写操作");
        }

        if mode == GuardianMode::Default && args.is_write {
            return GuardianDecision::confirm(format!("写操作 `{tool_name}` 需要审批"));
        }

        GuardianDecision::allow()
    }

    fn record(
        &self,
        caller: &str,
        tool_name: &str,
        args: &ToolCallArgs,
        decision: &GuardianDecision,
    ) {
        let parameters = serde_json::to_value(args).unwrap_or_else(|error| {
            json!({
                "serialization_error": error.to_string(),
            })
        });
        self.audit_log.log(AuditEntry {
            timestamp: Utc::now(),
            caller: caller.to_string(),
            tool_name: tool_name.to_string(),
            parameters,
            decision_result: decision.decision_result().to_string(),
            decision_detail: decision.detail(),
        });
    }
}

impl Default for Guardian {
    fn default() -> Self {
        Self::new(GuardianConfig::default())
    }
}

fn permission_level_name(level: PermissionLevel) -> &'static str {
    match level {
        PermissionLevel::Public => "public",
        PermissionLevel::Standard => "standard",
        PermissionLevel::Elevated => "elevated",
        PermissionLevel::Admin => "admin",
    }
}
