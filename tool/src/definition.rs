pub use crate::types::{ToolDefinition, ToolResult, ToolResultStatus};

use serde_json::{json, Value};

pub fn string_param(description: &str) -> Value {
    json!({
        "type": "string",
        "description": description,
    })
}

pub fn boolean_param(description: &str, default_value: bool) -> Value {
    json!({
        "type": "boolean",
        "description": description,
        "default": default_value,
    })
}

pub fn object_schema(properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{boolean_param, object_schema, string_param};

    #[test]
    fn helper_builders_create_expected_json_schema_fragments() {
        assert_eq!(string_param("path")["type"], json!("string"));
        assert_eq!(boolean_param("recursive", true)["default"], json!(true));
        assert_eq!(
            object_schema(json!({"path": string_param("path")}), &["path"])["required"],
            json!(["path"])
        );
    }
}
