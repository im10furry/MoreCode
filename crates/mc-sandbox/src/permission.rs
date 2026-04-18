use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};

use crate::capability::Capability;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionCheckResult {
    Allowed,
    Denied { reason: String },
}

impl PermissionCheckResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, PermissionCheckResult::Allowed)
    }
}

#[derive(Debug, Clone)]
pub struct TaskPermission {
    pub task_id: String,
    pub capabilities: Vec<Capability>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub remaining_uses: Option<u32>,
    pub revoked: bool,
}

impl TaskPermission {
    pub fn new(
        task_id: impl Into<String>,
        capabilities: Vec<Capability>,
        ttl: Duration,
        max_uses: Option<u32>,
    ) -> Self {
        let created_at = Utc::now();
        let expires_at = DateTime::<Utc>::from(SystemTime::now() + ttl);

        Self {
            task_id: task_id.into(),
            capabilities,
            created_at,
            expires_at,
            remaining_uses: max_uses,
            revoked: false,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn has_capability(&self, required: &Capability) -> bool {
        self.capabilities
            .iter()
            .any(|capability| capability.matches(required))
    }
}

#[derive(Debug, Default)]
pub struct TaskPermissionManager {
    permissions: HashMap<String, TaskPermission>,
}

impl TaskPermissionManager {
    pub fn grant(
        &mut self,
        task_id: impl Into<String>,
        capabilities: Vec<Capability>,
        ttl: Duration,
        max_uses: Option<u32>,
    ) {
        let permission = TaskPermission::new(task_id.into(), capabilities, ttl, max_uses);
        self.permissions
            .insert(permission.task_id.clone(), permission);
    }

    pub fn check_permission(
        &mut self,
        task_id: &str,
        required: &Capability,
    ) -> PermissionCheckResult {
        self.cleanup_expired();

        let Some(permission) = self.permissions.get_mut(task_id) else {
            return PermissionCheckResult::Denied {
                reason: format!("任务 `{task_id}` 没有权限凭证"),
            };
        };

        if permission.revoked {
            return PermissionCheckResult::Denied {
                reason: format!("任务 `{task_id}` 的权限已撤销"),
            };
        }

        if permission.is_expired() {
            self.permissions.remove(task_id);
            return PermissionCheckResult::Denied {
                reason: format!("任务 `{task_id}` 的权限已过期"),
            };
        }

        if !permission.has_capability(required) {
            return PermissionCheckResult::Denied {
                reason: format!("任务 `{task_id}` 缺少能力 `{}`", required.description()),
            };
        }

        if let Some(remaining_uses) = permission.remaining_uses.as_mut() {
            if *remaining_uses == 0 {
                return PermissionCheckResult::Denied {
                    reason: format!("任务 `{task_id}` 的权限使用次数已耗尽"),
                };
            }
            *remaining_uses -= 1;
        }

        PermissionCheckResult::Allowed
    }

    pub fn check_permissions(
        &mut self,
        task_id: &str,
        required_capabilities: &[Capability],
    ) -> PermissionCheckResult {
        for capability in required_capabilities {
            let result = self.check_permission(task_id, capability);
            if !result.is_allowed() {
                return result;
            }
        }
        PermissionCheckResult::Allowed
    }

    pub fn revoke(&mut self, task_id: &str) {
        if let Some(permission) = self.permissions.get_mut(task_id) {
            permission.revoked = true;
        }
    }

    pub fn cleanup_expired(&mut self) {
        self.permissions
            .retain(|_, permission| !permission.is_expired() && !permission.revoked);
    }

    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }
}

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(test)]
static DROP_CLEANUP_COUNT: AtomicUsize = AtomicUsize::new(0);

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn drop_cleanup_count() -> usize {
    DROP_CLEANUP_COUNT.load(Ordering::SeqCst)
}

impl Drop for TaskPermissionManager {
    fn drop(&mut self) {
        self.cleanup_expired();
        #[cfg(test)]
        DROP_CLEANUP_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}
