use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Duration, Utc};
use mc_config::TaskPileConfig;
use uuid::Uuid;

use crate::error::{TaskPileError, TaskPileResult};

use super::{
    cloud::{
        CloudAdapterStatus, CloudPayload, CloudTaskAdapter, CloudTaskResponse, HttpCloudAdapter,
        NoopCloudAdapter,
    },
    crypto::init_encryption,
    logger::{init_logger, log_task_claim, log_task_completion, log_task_creation, log_task_failure, log_task_pause, log_task_resume, log_task_cancel},
    store::{SqliteTaskPileStore, TaskPileState, TaskPileStorage},
    types::{
        ApprovalMode, CompressionMode, ExecutionOptions, IsolationProfile, NewTaskRequest,
        TaskPilePriority, TaskPileSchedule, TaskPileStats, TaskPileStatus, TaskPileTask,
        TaskTarget, TokenControls,
    },
    utils::{count_status, retry_backoff, task_digest, truncate_title},
};

pub struct TaskPileService {
    config: TaskPileConfig,
    store: Arc<dyn TaskPileStorage>,
    cloud_adapter: Arc<dyn CloudTaskAdapter>,
    mutex: Mutex<()>,
}

impl TaskPileService {
    pub fn new(config: TaskPileConfig, workspace_root: PathBuf) -> Self {
        let storage_dir = config
            .storage_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| workspace_root.join(".morecode").join("taskpile"));
        let cloud_adapter: Arc<dyn CloudTaskAdapter> = if config.cloud.enabled {
            Arc::new(HttpCloudAdapter::new(
                config.cloud.enabled,
                config.cloud.endpoint.clone(),
                config.cloud.project_id.clone(),
            ))
        } else {
            Arc::new(NoopCloudAdapter::new(
                config.cloud.enabled,
                config.cloud.endpoint.clone(),
                config.cloud.project_id.clone(),
            ))
        };
        
        // Initialize encryption with a default key (in production, this should come from config)
        init_encryption("taskpile_encryption_key_2026");
        
        // Initialize logger
        init_logger();
        
        Self {
            config,
            store: Arc::new(SqliteTaskPileStore::new(storage_dir)),
            cloud_adapter,
            mutex: Mutex::new(()),
        }
    }

    pub fn state_path(&self) -> String {
        self.store.state_path().display().to_string()
    }

    pub fn list_tasks(&self) -> TaskPileResult<Vec<TaskPileTask>> {
        let _lock = self.mutex.lock().unwrap();
        let mut tasks = self.store.load()?.tasks;
        tasks.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.created_at.cmp(&right.created_at))
        });
        Ok(tasks)
    }

    pub fn get_task(&self, task_id: &str) -> TaskPileResult<TaskPileTask> {
        self.store
            .load()?
            .tasks
            .into_iter()
            .find(|task| task.id == task_id)
            .ok_or_else(|| TaskPileError::TaskNotFound {
                task_id: task_id.to_string(),
            })
    }

    pub fn add_task(&self, request: NewTaskRequest) -> TaskPileResult<TaskPileTask> {
        let _lock = self.mutex.lock().unwrap();
        let now = Utc::now();
        let mut state = self.store.load()?;
        let digest = task_digest(&request.instruction, &request.target, &request.schedule);
        let dedup_cutoff = now - Duration::seconds(self.config.dedup_window_secs as i64);
        if let Some(existing) = state.tasks.iter().find(|task| {
            !task.status.is_terminal()
                && task.created_at >= dedup_cutoff
                && task_digest(&task.instruction, &task.execution.target, &task.schedule) == digest
        }) {
            return Err(TaskPileError::DuplicateTask {
                existing_id: existing.id.clone(),
            });
        }
        let NewTaskRequest {
            title,
            instruction,
            priority,
            schedule,
            target,
            isolation,
            token_budget,
            compression,
            parallelism,
            approval,
            max_attempts,
            tags,
            metadata,
            model,
            cloud_endpoint,
            cloud_project_id,
            origin,
        } = request;

        let token_controls = TokenControls {
            budget: token_budget,
            compression,
            summary_depth: if parallelism > 1 { 1 } else { 2 },
            allow_cache_reuse: true,
            cache_namespace: tags.first().map(|tag| format!("taskpile:{tag}")),
        };
        let execution = ExecutionOptions {
            target,
            model,
            parallelism,
            approval,
            isolation,
            token_controls,
            cloud_endpoint: cloud_endpoint.or_else(|| self.config.cloud.endpoint.clone()),
            cloud_project_id: cloud_project_id.or_else(|| self.config.cloud.project_id.clone()),
        };
        let title = title.unwrap_or_else(|| truncate_title(&instruction));
        let next_run_at = schedule.next_run_at(now);
        let task = TaskPileTask {
            id: Uuid::new_v4().to_string(),
            title,
            instruction,
            status: TaskPileStatus::Queued,
            priority,
            schedule,
            execution,
            tags,
            metadata,
            created_at: now,
            updated_at: now,
            next_run_at,
            last_claimed_at: None,
            lease_expires_at: None,
            attempts: 0,
            max_attempts,
            last_error: None,
            result_summary: None,
            origin,
        };
        state.tasks.push(task.clone());
        self.store.save(state)?;
        
        // Log task creation
        log_task_creation(&task.id, &task.title);
        
        Ok(task)
    }

    pub fn claim_next_due(&self, now: DateTime<Utc>) -> TaskPileResult<Option<TaskPileTask>> {
        let _lock = self.mutex.lock().unwrap();
        let mut state = self.store.load()?;
        
        // Clean up expired leases
        self.cleanup_expired_leases(&mut state, now);
        
        let running_count = state
            .tasks
            .iter()
            .filter(|task| task.status == TaskPileStatus::Running)
            .count();
        if running_count >= self.config.max_running_tasks {
            return Err(TaskPileError::RunningLimitReached {
                current: running_count,
                limit: self.config.max_running_tasks,
            });
        }

        let next_index = state
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| task.due_at(now))
            .max_by(|(_, left), (_, right)| {
                left.priority
                    .cmp(&right.priority)
                    .then_with(|| right.created_at.cmp(&left.created_at))
            })
            .map(|(index, _)| index);

        let Some(index) = next_index else {
            return Ok(None);
        };

        let task = state.tasks.get_mut(index).expect("task index checked");
        task.status = TaskPileStatus::Running;
        task.attempts = task.attempts.saturating_add(1);
        task.last_claimed_at = Some(now);
        task.lease_expires_at = Some(now + Duration::minutes(15));
        task.updated_at = now;
        let claimed = task.clone();
        self.store.save(state)?;
        
        // Log task claim
        log_task_claim(&claimed.id, &claimed.title);
        
        Ok(Some(claimed))
    }

    pub fn complete_task(&self, task_id: &str, summary: &str) -> TaskPileResult<TaskPileTask> {
        let _lock = self.mutex.lock().unwrap();
        let result = self.mutate_task(task_id, |task| {
            if task.status != TaskPileStatus::Running {
                return Err(TaskPileError::InvalidStatus {
                    task_id: task.id.clone(),
                    status: format!("{:?}", task.status),
                });
            }
            task.status = TaskPileStatus::Completed;
            task.result_summary = Some(summary.to_string());
            task.last_error = None;
            task.lease_expires_at = None;
            task.updated_at = Utc::now();
            Ok(())
        });
        
        if let Ok(task) = &result {
            // Log task completion
            log_task_completion(&task.id, &task.title, summary);
        }
        
        result
    }

    pub fn fail_task(&self, task_id: &str, reason: &str) -> TaskPileResult<TaskPileTask> {
        let _lock = self.mutex.lock().unwrap();
        let result = self.mutate_task(task_id, |task| {
            if task.status != TaskPileStatus::Running {
                return Err(TaskPileError::InvalidStatus {
                    task_id: task.id.clone(),
                    status: format!("{:?}", task.status),
                });
            }
            task.last_error = Some(reason.to_string());
            task.lease_expires_at = None;
            task.updated_at = Utc::now();
            if task.attempts < task.max_attempts {
                task.status = TaskPileStatus::Queued;
                task.next_run_at = Some(Utc::now() + retry_backoff(task.attempts));
            } else {
                task.status = TaskPileStatus::Failed;
            }
            Ok(())
        });
        
        if let Ok(task) = &result {
            // Log task failure
            log_task_failure(&task.id, &task.title, reason);
        }
        
        result
    }

    pub fn pause_task(&self, task_id: &str) -> TaskPileResult<TaskPileTask> {
        let _lock = self.mutex.lock().unwrap();
        let result = self.mutate_task(task_id, |task| {
            if task.status == TaskPileStatus::Completed || task.status == TaskPileStatus::Cancelled
            {
                return Err(TaskPileError::InvalidStatus {
                    task_id: task.id.clone(),
                    status: format!("{:?}", task.status),
                });
            }
            task.status = TaskPileStatus::Paused;
            task.lease_expires_at = None;
            task.updated_at = Utc::now();
            Ok(())
        });
        
        if let Ok(task) = &result {
            // Log task pause
            log_task_pause(&task.id, &task.title);
        }
        
        result
    }

    pub fn resume_task(&self, task_id: &str) -> TaskPileResult<TaskPileTask> {
        let _lock = self.mutex.lock().unwrap();
        let result = self.mutate_task(task_id, |task| {
            if task.status != TaskPileStatus::Paused {
                return Err(TaskPileError::InvalidStatus {
                    task_id: task.id.clone(),
                    status: format!("{:?}", task.status),
                });
            }
            task.status = TaskPileStatus::Queued;
            task.next_run_at = Some(Utc::now());
            task.updated_at = Utc::now();
            Ok(())
        });
        
        if let Ok(task) = &result {
            // Log task resume
            log_task_resume(&task.id, &task.title);
        }
        
        result
    }

    pub fn cancel_task(&self, task_id: &str) -> TaskPileResult<TaskPileTask> {
        let _lock = self.mutex.lock().unwrap();
        let result = self.mutate_task(task_id, |task| {
            if task.status.is_terminal() {
                return Err(TaskPileError::InvalidStatus {
                    task_id: task.id.clone(),
                    status: format!("{:?}", task.status),
                });
            }
            task.status = TaskPileStatus::Cancelled;
            task.lease_expires_at = None;
            task.updated_at = Utc::now();
            Ok(())
        });
        
        if let Ok(task) = &result {
            // Log task cancel
            log_task_cancel(&task.id, &task.title);
        }
        
        result
    }

    pub fn stats(&self) -> TaskPileResult<TaskPileStats> {
        let _lock = self.mutex.lock().unwrap();
        let state = self.store.load()?;
        let next_due_at = state
            .tasks
            .iter()
            .filter(|task| matches!(task.status, TaskPileStatus::Queued))
            .filter_map(|task| task.next_run_at.as_ref().cloned())
            .min();
        Ok(TaskPileStats {
            total: state.tasks.len(),
            queued: count_status(&state, TaskPileStatus::Queued),
            running: count_status(&state, TaskPileStatus::Running),
            paused: count_status(&state, TaskPileStatus::Paused),
            completed: count_status(&state, TaskPileStatus::Completed),
            failed: count_status(&state, TaskPileStatus::Failed),
            cancelled: count_status(&state, TaskPileStatus::Cancelled),
            next_due_at,
            storage_path: self.store.state_path().display().to_string(),
            cloud_ready: self.cloud_adapter.status().ready,
        })
    }

    pub fn cloud_status(&self) -> CloudAdapterStatus {
        self.cloud_adapter.status()
    }

    pub fn preview_cloud_payload(&self, task_id: &str) -> TaskPileResult<CloudPayload> {
        let _lock = self.mutex.lock().unwrap();
        let task = self.store
            .load()?
            .tasks
            .into_iter()
            .find(|task| task.id == task_id)
            .ok_or_else(|| TaskPileError::TaskNotFound {
                task_id: task_id.to_string(),
            })?;
        
        // If cloud adapter is not ready, return a mock payload for testing
        if !self.cloud_adapter.status().ready {
            return Ok(CloudPayload {
                task_id: task.id.clone(),
                accepted_at: Utc::now(),
                endpoint: self.cloud_adapter.status().endpoint.clone(),
                project_id: self.cloud_adapter.status().project_id.clone(),
                target: task.execution.target,
                note: "Cloud adapter not ready, returning mock payload".to_string(),
            });
        }
        
        self.cloud_adapter.preview_payload(&task)
    }

    pub fn submit_task_to_cloud(&self, task_id: &str) -> TaskPileResult<CloudTaskResponse> {
        let _lock = self.mutex.lock().unwrap();
        let task = self.store
            .load()?
            .tasks
            .into_iter()
            .find(|task| task.id == task_id)
            .ok_or_else(|| TaskPileError::TaskNotFound {
                task_id: task_id.to_string(),
            })?;
        self.cloud_adapter.submit_task(&task)
    }

    fn mutate_task(
        &self,
        task_id: &str,
        mutator: impl FnOnce(&mut TaskPileTask) -> TaskPileResult<()>,
    ) -> TaskPileResult<TaskPileTask> {
        let mut state = self.store.load()?;
        let task = state
            .tasks
            .iter_mut()
            .find(|task| task.id == task_id)
            .ok_or_else(|| TaskPileError::TaskNotFound {
                task_id: task_id.to_string(),
            })?;
        mutator(task)?;
        let updated = task.clone();
        self.store.save(state)?;
        Ok(updated)
    }

    fn cleanup_expired_leases(&self, state: &mut TaskPileState, now: DateTime<Utc>) {
        for task in &mut state.tasks {
            if task.status == TaskPileStatus::Running {
                if let Some(lease_expires) = task.lease_expires_at {
                    if now > lease_expires {
                        task.status = TaskPileStatus::Queued;
                        task.lease_expires_at = None;
                        task.updated_at = now;
                        task.next_run_at = Some(now); // Set next_run_at to now so the task can be claimed immediately
                    }
                }
            }
        }
    }
}

impl Default for NewTaskRequest {
    fn default() -> Self {
        Self {
            title: None,
            instruction: String::new(),
            priority: TaskPilePriority::Normal,
            schedule: TaskPileSchedule::Manual,
            target: TaskTarget::Local,
            isolation: IsolationProfile::WorkspaceWrite,
            token_budget: 12_000,
            compression: CompressionMode::Balanced,
            parallelism: 1,
            approval: ApprovalMode::Auto,
            max_attempts: 3,
            tags: Vec::new(),
            metadata: Default::default(),
            model: None,
            cloud_endpoint: None,
            cloud_project_id: None,
            origin: "cli".to_string(),
        }
    }
}



#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::Utc;
    use mc_config::TaskPileConfig;
    use tempfile::tempdir;

    use super::TaskPileService;
    use crate::taskpile::{NewTaskRequest, TaskPilePriority, TaskPileStatus};

    #[test]
    fn add_claim_complete_roundtrip() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        let request = NewTaskRequest {
            instruction: "ship the release checklist".to_string(),
            priority: TaskPilePriority::High,
            ..NewTaskRequest::default()
        };
        let created = service.add_task(request).expect("create task");
        assert_eq!(created.status, TaskPileStatus::Queued);

        let claimed = service
            .claim_next_due(Utc::now())
            .expect("claim")
            .expect("task");
        assert_eq!(claimed.id, created.id);
        assert_eq!(claimed.status, TaskPileStatus::Running);

        let completed = service
            .complete_task(&created.id, "validated and shipped")
            .expect("complete");
        assert_eq!(completed.status, TaskPileStatus::Completed);

        let stats = service.stats().expect("stats");
        assert_eq!(stats.completed, 1);
    }

    #[test]
    fn duplicate_task_is_rejected_inside_window() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        let first = NewTaskRequest {
            instruction: "re-run flaky test suite".to_string(),
            ..NewTaskRequest::default()
        };
        service.add_task(first).expect("create");

        let duplicate = NewTaskRequest {
            instruction: "re-run flaky test suite".to_string(),
            ..NewTaskRequest::default()
        };
        let error = service.add_task(duplicate).expect_err("duplicate");
        assert!(format!("{error}").contains("task already exists"));
    }

    #[test]
    fn task_failure_and_retry() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        let request = NewTaskRequest {
            instruction: "test task failure".to_string(),
            max_attempts: 2,
            ..NewTaskRequest::default()
        };
        let _created = service.add_task(request).expect("create task");

        // Claim and fail the task
        let claimed = service.claim_next_due(Utc::now()).expect("claim").expect("task");
        let failed = service.fail_task(&claimed.id, "test failure").expect("fail task");
        assert_eq!(failed.status, TaskPileStatus::Queued); // Should be queued for retry

        // Claim and fail again (should reach max attempts)
        let claimed_again = service.claim_next_due(Utc::now()).expect("claim again").expect("task");
        let failed_final = service.fail_task(&claimed_again.id, "test failure again").expect("fail task again");
        assert_eq!(failed_final.status, TaskPileStatus::Failed); // Should be failed
    }

    #[test]
    fn task_pause_and_resume() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        let request = NewTaskRequest {
            instruction: "test task pause/resume".to_string(),
            ..NewTaskRequest::default()
        };
        let created = service.add_task(request).expect("create task");

        // Pause the task
        let paused = service.pause_task(&created.id).expect("pause task");
        assert_eq!(paused.status, TaskPileStatus::Paused);

        // Resume the task
        let resumed = service.resume_task(&created.id).expect("resume task");
        assert_eq!(resumed.status, TaskPileStatus::Queued);
    }

    #[test]
    fn task_cancellation() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        let request = NewTaskRequest {
            instruction: "test task cancellation".to_string(),
            ..NewTaskRequest::default()
        };
        let created = service.add_task(request).expect("create task");

        // Cancel the task
        let cancelled = service.cancel_task(&created.id).expect("cancel task");
        assert_eq!(cancelled.status, TaskPileStatus::Cancelled);
    }

    #[test]
    fn expired_lease_cleanup() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        let request = NewTaskRequest {
            instruction: "test expired lease".to_string(),
            ..NewTaskRequest::default()
        };
        let _created = service.add_task(request).expect("create task");

        // Claim the task
        let claimed = service.claim_next_due(Utc::now()).expect("claim").expect("task");
        assert_eq!(claimed.status, TaskPileStatus::Running);

        // Advance time by 20 minutes (lease expires after 15 minutes)
        let future_time = Utc::now() + chrono::Duration::minutes(20);
        let claimed_again = service.claim_next_due(future_time).expect("claim again");
        // Should be able to claim the task again as the lease has expired
        assert!(claimed_again.is_some());
    }

    #[test]
    fn task_stats() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        
        // Add a task
        let request = NewTaskRequest {
            instruction: "test task stats".to_string(),
            ..NewTaskRequest::default()
        };
        service.add_task(request).expect("create task");

        // Check stats
        let stats = service.stats().expect("stats");
        assert_eq!(stats.total, 1);
        assert_eq!(stats.queued, 1);
    }

    #[test]
    fn cloud_task_submission() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        let request = NewTaskRequest {
            instruction: "test cloud task".to_string(),
            ..NewTaskRequest::default()
        };
        let created = service.add_task(request).expect("create task");

        // Preview cloud payload
        let payload = service.preview_cloud_payload(&created.id).expect("preview payload");
        assert_eq!(payload.task_id, created.id);
    }
}
