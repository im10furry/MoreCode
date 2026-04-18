use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::builtin::GitTool;
use crate::registry::ToolRegistry;
use crate::types::Tool;

type DeferredFactory = fn() -> Arc<dyn Tool>;

static GIT_TOOL: Lazy<Arc<dyn Tool>> = Lazy::new(|| Arc::new(GitTool::new()));

pub static DEFERRED_FACTORIES: Lazy<HashMap<String, DeferredFactory>> = Lazy::new(|| {
    let mut factories = HashMap::new();
    factories.insert("git".to_string(), load_git_tool as DeferredFactory);
    factories
});

pub async fn register_deferred_tools(registry: &ToolRegistry) {
    for (name, factory) in DEFERRED_FACTORIES.iter() {
        registry.register_deferred(name, *factory).await;
    }
}

fn load_git_tool() -> Arc<dyn Tool> {
    Arc::clone(&*GIT_TOOL)
}
