use std::sync::Arc;

use crate::builtin::TerminalTool;
use crate::registry::ToolRegistry;

pub async fn register_extended_tools(registry: &ToolRegistry) {
    let tool = Arc::new(TerminalTool::new(registry.guardian()));
    registry.register_arc(tool).await;
}
