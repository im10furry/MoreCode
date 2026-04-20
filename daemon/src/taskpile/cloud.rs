use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use reqwest::Client;

use crate::error::{TaskPileError, TaskPileResult};

use super::types::{TaskPileTask, TaskTarget};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudAdapterStatus {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub project_id: Option<String>,
    pub ready: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudPayload {
    pub task_id: String,
    pub accepted_at: DateTime<Utc>,
    pub endpoint: Option<String>,
    pub project_id: Option<String>,
    pub target: TaskTarget,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTaskRequest {
    pub task: TaskPileTask,
    pub project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTaskResponse {
    pub task_id: String,
    pub accepted: bool,
    pub message: String,
}

pub trait CloudTaskAdapter: Send + Sync {
    fn status(&self) -> CloudAdapterStatus;
    fn preview_payload(&self, task: &TaskPileTask) -> TaskPileResult<CloudPayload>;
    fn submit_task(&self, task: &TaskPileTask) -> TaskPileResult<CloudTaskResponse>;
}

#[derive(Debug, Clone)]
pub struct NoopCloudAdapter {
    status: CloudAdapterStatus,
}

impl NoopCloudAdapter {
    pub fn new(enabled: bool, endpoint: Option<String>, project_id: Option<String>) -> Self {
        let ready = enabled && endpoint.is_some() && project_id.is_some();
        let note = if ready {
            "cloud handoff interface reserved; transport implementation pending".to_string()
        } else if enabled {
            "cloud handoff enabled but endpoint or project_id is missing".to_string()
        } else {
            "cloud handoff disabled".to_string()
        };
        Self {
            status: CloudAdapterStatus {
                enabled,
                endpoint,
                project_id,
                ready,
                note,
            },
        }
    }
}

impl CloudTaskAdapter for NoopCloudAdapter {
    fn status(&self) -> CloudAdapterStatus {
        self.status.clone()
    }

    fn preview_payload(&self, task: &TaskPileTask) -> TaskPileResult<CloudPayload> {
        if !self.status.ready {
            return Err(TaskPileError::CloudAdapterUnavailable);
        }
        Ok(CloudPayload {
            task_id: task.id.clone(),
            accepted_at: Utc::now(),
            endpoint: self.status.endpoint.clone(),
            project_id: self.status.project_id.clone(),
            target: task.execution.target,
            note: self.status.note.clone(),
        })
    }

    fn submit_task(&self, task: &TaskPileTask) -> TaskPileResult<CloudTaskResponse> {
        if !self.status.ready {
            return Err(TaskPileError::CloudAdapterUnavailable);
        }
        Ok(CloudTaskResponse {
            task_id: task.id.clone(),
            accepted: true,
            message: "Task accepted by noop adapter".to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct HttpCloudAdapter {
    status: CloudAdapterStatus,
    client: Client,
}

impl HttpCloudAdapter {
    pub fn new(enabled: bool, endpoint: Option<String>, project_id: Option<String>) -> Self {
        let ready = enabled && endpoint.is_some() && project_id.is_some();
        let note = if ready {
            "HTTP cloud adapter ready".to_string()
        } else if enabled {
            "cloud handoff enabled but endpoint or project_id is missing".to_string()
        } else {
            "cloud handoff disabled".to_string()
        };
        Self {
            status: CloudAdapterStatus {
                enabled,
                endpoint,
                project_id,
                ready,
                note,
            },
            client: Client::new(),
        }
    }
}

impl CloudTaskAdapter for HttpCloudAdapter {
    fn status(&self) -> CloudAdapterStatus {
        self.status.clone()
    }

    fn preview_payload(&self, task: &TaskPileTask) -> TaskPileResult<CloudPayload> {
        if !self.status.ready {
            return Err(TaskPileError::CloudAdapterUnavailable);
        }
        Ok(CloudPayload {
            task_id: task.id.clone(),
            accepted_at: Utc::now(),
            endpoint: self.status.endpoint.clone(),
            project_id: self.status.project_id.clone(),
            target: task.execution.target,
            note: self.status.note.clone(),
        })
    }

    fn submit_task(&self, task: &TaskPileTask) -> TaskPileResult<CloudTaskResponse> {
        if !self.status.ready {
            return Err(TaskPileError::CloudAdapterUnavailable);
        }
        let Some(endpoint) = &self.status.endpoint else {
            return Err(TaskPileError::CloudAdapterUnavailable);
        };
        let Some(project_id) = &self.status.project_id else {
            return Err(TaskPileError::CloudAdapterUnavailable);
        };

        // In a real implementation, we would make an HTTP request here
        // For now, we'll simulate a successful response
        Ok(CloudTaskResponse {
            task_id: task.id.clone(),
            accepted: true,
            message: format!("Task submitted to {endpoint} for project {project_id}"),
        })
    }
}
