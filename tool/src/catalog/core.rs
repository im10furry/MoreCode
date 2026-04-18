use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::builtin::{FileReadTool, FileWriteTool, SearchTool};
use crate::registry::ToolRegistry;
use crate::types::Tool;

pub static CORE_TOOLS: Lazy<Vec<Arc<dyn Tool>>> = Lazy::new(|| {
    vec![
        Arc::new(FileReadTool::new()),
        Arc::new(FileWriteTool::new()),
        Arc::new(SearchTool::new()),
    ]
});

pub async fn register_core_tools(registry: &ToolRegistry) {
    for tool in CORE_TOOLS.iter() {
        registry.register_arc(Arc::clone(tool)).await;
    }
}
