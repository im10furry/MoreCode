use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use mc_sandbox::{Guardian, GuardianDecision, PermissionLevel, ToolCallArgs};
use serde_json::Value;
use tokio::sync::RwLock;

use crate::types::{Tool, ToolDefinition, ToolResult, VisibilityLayer};

type DeferredFactory = Arc<dyn Fn() -> Arc<dyn Tool> + Send + Sync>;

pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn Tool>>>,
    visibility: RwLock<HashMap<String, VisibilityLayer>>,
    deferred_factories: RwLock<HashMap<String, DeferredFactory>>,
    guardian: Option<Arc<Guardian>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            visibility: RwLock::new(HashMap::new()),
            deferred_factories: RwLock::new(HashMap::new()),
            guardian: None,
        }
    }

    pub fn with_guardian(guardian: Arc<Guardian>) -> Self {
        Self {
            guardian: Some(guardian),
            ..Self::new()
        }
    }

    pub fn guardian(&self) -> Option<Arc<Guardian>> {
        self.guardian.clone()
    }

    pub async fn register<T>(&self, tool: T)
    where
        T: Tool + 'static,
    {
        self.register_arc(Arc::new(tool)).await;
    }

    pub async fn register_arc(&self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        let visibility = visibility_from_permission(tool.permission_level());

        self.tools.write().await.insert(name.clone(), tool);
        self.visibility.write().await.insert(name, visibility);
    }

    pub async fn register_deferred<F>(&self, name: &str, factory: F)
    where
        F: Fn() -> Arc<dyn Tool> + Send + Sync + 'static,
    {
        self.deferred_factories
            .write()
            .await
            .insert(name.to_string(), Arc::new(factory));
    }

    pub async fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        if let Some(tool) = self.tools.read().await.get(name).cloned() {
            return Some(tool);
        }

        let factory = self.deferred_factories.read().await.get(name).cloned()?;
        let tool = factory();
        let visibility = visibility_from_permission(tool.permission_level());
        let resolved_name = tool.name().to_string();

        self.tools
            .write()
            .await
            .insert(resolved_name.clone(), Arc::clone(&tool));
        self.visibility
            .write()
            .await
            .insert(resolved_name.clone(), visibility);
        self.deferred_factories.write().await.remove(name);

        Some(tool)
    }

    pub async fn unregister(&self, name: &str) -> bool {
        let removed_tool = self.tools.write().await.remove(name).is_some();
        let removed_visibility = self.visibility.write().await.remove(name).is_some();
        let removed_deferred = self.deferred_factories.write().await.remove(name).is_some();
        removed_tool || removed_visibility || removed_deferred
    }

    pub async fn list_tools(&self, layer: VisibilityLayer) -> Vec<Arc<dyn Tool>> {
        let tools = self.tools.read().await;
        let visibility = self.visibility.read().await;
        let mut result: Vec<Arc<dyn Tool>> = tools
            .iter()
            .filter_map(|(name, tool)| {
                let level = visibility.get(name)?;
                if *level != VisibilityLayer::Hidden && *level <= layer {
                    Some(Arc::clone(tool))
                } else {
                    None
                }
            })
            .collect();
        result.sort_by(|left, right| left.name().cmp(right.name()));
        result
    }

    pub async fn list_tools_with_deferred(&self, layer: VisibilityLayer) -> Vec<Arc<dyn Tool>> {
        let deferred_names = self
            .deferred_factories
            .read()
            .await
            .keys()
            .cloned()
            .collect::<Vec<_>>();

        for name in deferred_names {
            let _ = self.get(&name).await;
        }

        self.list_tools(layer).await
    }

    pub async fn tool_definitions(&self, layer: VisibilityLayer) -> Vec<ToolDefinition> {
        self.list_tools(layer)
            .await
            .into_iter()
            .map(|tool| tool.definition())
            .collect()
    }

    pub async fn execute_tool(&self, caller: &str, tool_name: &str, params: Value) -> ToolResult {
        let start = Instant::now();
        let tool = match self.get(tool_name).await {
            Some(tool) => tool,
            None => {
                return ToolResult::error(format!("工具 `{tool_name}` 未注册"))
                    .with_duration(start.elapsed())
            }
        };

        if let Some(guardian) = &self.guardian {
            let args = build_tool_call_args(&*tool, &params);
            let decision = guardian.check_tool_call(caller, tool_name, &args).await;
            match decision {
                GuardianDecision::Allow => {}
                GuardianDecision::Simulate { mock_result } => {
                    return ToolResult::content(mock_result).with_duration(start.elapsed())
                }
                GuardianDecision::Deny { reason }
                | GuardianDecision::ConfirmRequired { reason } => {
                    return ToolResult::error(reason).with_duration(start.elapsed())
                }
            }
        }

        let mut result = tool.execute(params).await;
        if result.duration_ms == 0 {
            result.duration_ms = start.elapsed().as_millis() as u64;
        }
        result
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn visibility_from_permission(level: PermissionLevel) -> VisibilityLayer {
    match level {
        PermissionLevel::Public | PermissionLevel::Standard => VisibilityLayer::Public,
        PermissionLevel::Elevated => VisibilityLayer::Project,
        PermissionLevel::Admin => VisibilityLayer::Admin,
    }
}

fn build_tool_call_args(tool: &dyn Tool, params: &Value) -> ToolCallArgs {
    let command = params
        .get("command")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let target_path = params
        .get("path")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .or_else(|| {
            params
                .get("module_path")
                .and_then(Value::as_str)
                .map(PathBuf::from)
        })
        .or_else(|| params.get("cwd").and_then(Value::as_str).map(PathBuf::from));

    ToolCallArgs {
        command,
        target_path,
        is_write: !tool.is_read_only(),
        extra: HashMap::new(),
        capability: Some(tool.capability()),
        task_id: None,
    }
}
