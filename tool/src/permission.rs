use mc_sandbox::PermissionLevel;
use serde::{Deserialize, Serialize};

use crate::types::{PermissionScope, ToolPermission, VisibilityLayer};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolPermissionPolicy {
    pub level: PermissionLevel,
    pub scope: PermissionScope,
    pub visibility: VisibilityLayer,
    pub read_only: bool,
}

impl ToolPermissionPolicy {
    pub fn from_permission(permission: ToolPermission, read_only: bool) -> Self {
        let visibility = visibility_for_permission_level(permission.level);
        Self {
            level: permission.level,
            scope: permission.scope,
            visibility,
            read_only,
        }
    }
}

pub fn visibility_for_permission_level(level: PermissionLevel) -> VisibilityLayer {
    match level {
        PermissionLevel::Public | PermissionLevel::Standard => VisibilityLayer::Public,
        PermissionLevel::Elevated => VisibilityLayer::Project,
        PermissionLevel::Admin => VisibilityLayer::Admin,
    }
}

#[cfg(test)]
mod tests {
    use mc_sandbox::PermissionLevel;

    use crate::types::{PermissionScope, ToolPermission, VisibilityLayer};

    use super::{visibility_for_permission_level, ToolPermissionPolicy};

    #[test]
    fn visibility_matches_permission_levels() {
        assert_eq!(
            visibility_for_permission_level(PermissionLevel::Public),
            VisibilityLayer::Public
        );
        assert_eq!(
            visibility_for_permission_level(PermissionLevel::Elevated),
            VisibilityLayer::Project
        );
        assert_eq!(
            visibility_for_permission_level(PermissionLevel::Admin),
            VisibilityLayer::Admin
        );
    }

    #[test]
    fn policy_derives_visibility_from_permission() {
        let policy = ToolPermissionPolicy::from_permission(
            ToolPermission {
                tool_name: "terminal".into(),
                level: PermissionLevel::Elevated,
                scope: PermissionScope::Process,
            },
            false,
        );
        assert_eq!(policy.visibility, VisibilityLayer::Project);
        assert!(!policy.read_only);
    }
}
