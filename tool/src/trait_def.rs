use std::future::Future;
use std::pin::Pin;

use serde_json::Value;

pub use crate::types::Tool;

pub type ToolFuture<'a> = Pin<Box<dyn Future<Output = crate::types::ToolResult> + Send + 'a>>;

pub fn validate_tool_definition(tool: &dyn Tool) -> Result<(), crate::error::ToolError> {
    if tool.name().trim().is_empty() {
        return Err(crate::error::ToolError::InvalidParams(
            "tool name cannot be empty".into(),
        ));
    }
    if tool.description().trim().is_empty() {
        return Err(crate::error::ToolError::InvalidParams(
            "tool description cannot be empty".into(),
        ));
    }

    let parameters = tool.required_parameters();
    if !matches!(parameters.get("type"), Some(Value::String(type_name)) if type_name == "object") {
        return Err(crate::error::ToolError::InvalidParams(
            "tool parameter schema must be a JSON object schema".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;

    use mc_sandbox::{Capability, CapabilityDeclaration, PermissionLevel};
    use serde_json::json;

    use crate::types::{PermissionScope, ToolCategory, ToolResult};

    use super::{validate_tool_definition, Tool};

    struct ExampleTool;

    impl Tool for ExampleTool {
        fn name(&self) -> &str {
            "example"
        }

        fn description(&self) -> &str {
            "example tool"
        }

        fn execute(
            &self,
            _params: serde_json::Value,
        ) -> Pin<Box<dyn std::future::Future<Output = ToolResult> + Send + '_>> {
            Box::pin(async { ToolResult::success("ok") })
        }

        fn required_parameters(&self) -> serde_json::Value {
            json!({
                "type": "object",
                "properties": {},
                "required": []
            })
        }

        fn is_read_only(&self) -> bool {
            true
        }

        fn category(&self) -> ToolCategory {
            ToolCategory::Core
        }

        fn permission_level(&self) -> PermissionLevel {
            PermissionLevel::Public
        }

        fn permission_scope(&self) -> PermissionScope {
            PermissionScope::Workspace
        }

        fn capability(&self) -> CapabilityDeclaration {
            CapabilityDeclaration::new(
                "example",
                "example tool",
                PermissionLevel::Public,
                vec![Capability::ReadFile {
                    pattern: "**".into(),
                }],
            )
        }
    }

    #[test]
    fn tool_definition_validation_accepts_object_schema() {
        assert!(validate_tool_definition(&ExampleTool).is_ok());
    }
}
