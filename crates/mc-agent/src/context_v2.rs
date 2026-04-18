use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use mc_core::{AgentType, ProjectContext, TaskDescription};
use mc_tool::{ToolRegistry, ToolResult};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{AgentConfig, AgentError, AgentHandoff};

#[derive(Clone)]
pub struct SharedResources {
    pub tool_registry: Arc<ToolRegistry>,
}

impl SharedResources {
    pub fn new(tool_registry: Arc<ToolRegistry>) -> Self {
        Self { tool_registry }
    }
}

#[derive(Clone)]
pub struct AgentContext {
    pub task: Arc<TaskDescription>,
    pub execution_id: Uuid,
    pub parent_execution_id: Option<Uuid>,
    pub project_ctx: Option<Arc<ProjectContext>>,
    pub handoff: Arc<AgentHandoff>,
    pub tool_registry: Arc<ToolRegistry>,
    pub cancel_token: CancellationToken,
    pub config: Arc<AgentConfig>,
    pub started_at: DateTime<Utc>,
    pub recursion_depth: u8,
    pub metadata: HashMap<String, Value>,
}

impl AgentContext {
    pub fn new(task: TaskDescription, shared: &SharedResources, config: AgentConfig) -> Self {
        Self {
            task: Arc::new(task),
            execution_id: Uuid::new_v4(),
            parent_execution_id: None,
            project_ctx: None,
            handoff: Arc::new(AgentHandoff::new()),
            tool_registry: Arc::clone(&shared.tool_registry),
            cancel_token: CancellationToken::new(),
            config: Arc::new(config.clone()),
            started_at: Utc::now(),
            recursion_depth: 0,
            metadata: config.metadata.clone(),
        }
    }

    pub fn with_project_context(mut self, project_ctx: ProjectContext) -> Self {
        self.project_ctx = Some(Arc::new(project_ctx));
        self
    }

    pub fn create_child_context(&self, task: TaskDescription) -> Self {
        Self {
            task: Arc::new(task),
            execution_id: Uuid::new_v4(),
            parent_execution_id: Some(self.execution_id),
            project_ctx: self.project_ctx.clone(),
            handoff: Arc::new(AgentHandoff::with_parent(self.handoff.clone())),
            tool_registry: Arc::clone(&self.tool_registry),
            cancel_token: self.cancel_token.child_token(),
            config: Arc::clone(&self.config),
            started_at: Utc::now(),
            recursion_depth: self.recursion_depth.saturating_add(1),
            metadata: self.metadata.clone(),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    pub fn elapsed(&self) -> std::time::Duration {
        Utc::now()
            .signed_duration_since(self.started_at)
            .to_std()
            .unwrap_or_default()
    }

    pub fn insert_metadata(&mut self, key: impl Into<String>, value: Value) {
        self.metadata.insert(key.into(), value);
    }

    pub fn get_metadata<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.metadata
            .get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    pub async fn has_tool(&self, tool_name: &str) -> bool {
        self.tool_registry.get(tool_name).await.is_some()
    }

    pub async fn call_tool(
        &self,
        caller: AgentType,
        tool_name: &str,
        params: Value,
    ) -> Result<ToolResult, AgentError> {
        if self.is_cancelled() {
            return Err(AgentError::Cancelled {
                reason: "agent context cancelled".to_string(),
            });
        }

        if !self.has_tool(tool_name).await {
            return Err(AgentError::MissingTool {
                tool: tool_name.to_string(),
            });
        }

        let result = self
            .tool_registry
            .execute_tool(caller.as_str(), tool_name, params)
            .await;

        if result.is_success() {
            Ok(result)
        } else {
            Err(AgentError::ToolExecutionFailed {
                tool: tool_name.to_string(),
                reason: result.content,
            })
        }
    }

    pub async fn call_tool_value(
        &self,
        caller: AgentType,
        tool_name: &str,
        params: Value,
    ) -> Result<Value, AgentError> {
        let result = self.call_tool(caller, tool_name, params).await?;
        tool_result_to_value(tool_name, result)
    }

    pub async fn call_tool_json<T: DeserializeOwned>(
        &self,
        caller: AgentType,
        tool_name: &str,
        params: Value,
    ) -> Result<T, AgentError> {
        let value = self.call_tool_value(caller, tool_name, params).await?;
        serde_json::from_value(value).map_err(|error| AgentError::ToolPayloadInvalid {
            tool: tool_name.to_string(),
            reason: error.to_string(),
        })
    }

    pub fn project_root(&self) -> Option<PathBuf> {
        self.project_ctx
            .as_ref()
            .map(|project| PathBuf::from(project.root_path.clone()))
    }

    pub fn output_root(&self) -> PathBuf {
        if let Some(directory) = &self.config.output_directory {
            let path = PathBuf::from(directory);
            if path.is_absolute() {
                return path;
            }
            if let Some(root) = self.project_root() {
                return root.join(path);
            }
            return path;
        }

        self.project_root().unwrap_or_else(|| PathBuf::from("."))
    }
}

fn tool_result_to_value(tool_name: &str, result: ToolResult) -> Result<Value, AgentError> {
    if let Some(data) = result.data {
        return Ok(data);
    }

    match serde_json::from_str::<Value>(&result.content) {
        Ok(value) => Ok(value),
        Err(_) => {
            if result.content.is_empty() {
                Err(AgentError::ToolPayloadInvalid {
                    tool: tool_name.to_string(),
                    reason: "tool did not return structured data".to_string(),
                })
            } else {
                Ok(json!({ "content": result.content }))
            }
        }
    }
}
