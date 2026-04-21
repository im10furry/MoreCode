use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Duration, Utc};
use mc_config::TaskPileConfig;
use uuid::Uuid;

use crate::error::{TaskPileError, TaskPileResult};

use super::{
    cloud::{CloudAdapterStatus, CloudPayload, CloudTaskAdapter, CloudTaskResponse, HttpCloudAdapter},
    crypto::init_encryption,
    logger::{init_logger, log_task_claim, log_task_completion, log_task_creation, log_task_failure, log_task_pause, log_task_resume, log_task_cancel},
    state_machine::TaskPileStateMachine,
    store::{SqliteTaskPileStore, TaskPileState, TaskPileStorage},
    types::{
        ApprovalMode, CompressionMode, ExecutionOptions, IsolationProfile, NewTaskRequest,
        TaskPilePriority, TaskPileSchedule, TaskPileStats, TaskPileStatus, TaskPileTask,
        TaskTarget, TokenControls,
    },
    utils::{count_status, task_digest, truncate_title},
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
        let cloud_adapter = Arc::new(HttpCloudAdapter::new(
            config.cloud.enabled,
            config.cloud.endpoint.clone(),
            config.cloud.project_id.clone(),
        ));
        
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

    pub fn list_tasks(&self, page: Option<u32>, page_size: Option<u32>) -> TaskPileResult<Vec<TaskPileTask>> {
        let _lock = self.mutex.lock().unwrap();
        let mut tasks = self.store.load()?.tasks;
        tasks.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.created_at.cmp(&right.created_at))
        });
        
        // Apply pagination
        if let (Some(page), Some(page_size)) = (page, page_size) {
            let page = page.saturating_sub(1); // Convert to 0-based index
            let start = (page * page_size) as usize;
            let end = start + page_size as usize;
            tasks = tasks.into_iter().skip(start).take(page_size as usize).collect();
        }
        
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

    pub fn list_tasks_by_status(&self, status: TaskPileStatus) -> TaskPileResult<Vec<TaskPileTask>> {
        let _lock = self.mutex.lock().unwrap();
        let tasks = self.store.load()?.tasks;
        let filtered_tasks: Vec<TaskPileTask> = tasks
            .into_iter()
            .filter(|task| task.status == status)
            .collect();
        Ok(filtered_tasks)
    }

    pub fn list_tasks_by_priority(&self, priority: TaskPilePriority) -> TaskPileResult<Vec<TaskPileTask>> {
        let _lock = self.mutex.lock().unwrap();
        let tasks = self.store.load()?.tasks;
        let filtered_tasks: Vec<TaskPileTask> = tasks
            .into_iter()
            .filter(|task| task.priority == priority)
            .collect();
        Ok(filtered_tasks)
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
        
        // Check for lease conflict
        if task.status == TaskPileStatus::Running {
            if let Some(lease_expires) = task.lease_expires_at {
                if now <= lease_expires {
                    // Task is already running with a valid lease
                    return Ok(None);
                }
            }
        }
        
        // Use state machine to transition to running
        TaskPileStateMachine::transition_to_running(task, now)?;
        let claimed = task.clone();
        self.store.save(state)?;
        
        // Log task claim
        log_task_claim(&claimed.id, &claimed.title);
        
        Ok(Some(claimed))
    }

    pub fn complete_task(&self, task_id: &str, summary: &str) -> TaskPileResult<TaskPileTask> {
        let _lock = self.mutex.lock().unwrap();
        let now = Utc::now();
        let result = self.mutate_task(task_id, |task| {
            TaskPileStateMachine::transition_to_completed(task, summary, now)
        });
        
        if let Ok(task) = &result {
            // Log task completion
            log_task_completion(&task.id, &task.title, summary);
        }
        
        result
    }

    pub fn fail_task(&self, task_id: &str, reason: &str) -> TaskPileResult<TaskPileTask> {
        let _lock = self.mutex.lock().unwrap();
        let now = Utc::now();
        let result = self.mutate_task(task_id, |task| {
            TaskPileStateMachine::transition_to_failed(task, reason, now)
        });
        
        if let Ok(task) = &result {
            // Log task failure
            log_task_failure(&task.id, &task.title, reason);
        }
        
        result
    }

    pub fn pause_task(&self, task_id: &str) -> TaskPileResult<TaskPileTask> {
        let _lock = self.mutex.lock().unwrap();
        let now = Utc::now();
        let result = self.mutate_task(task_id, |task| {
            TaskPileStateMachine::transition_to_paused(task, now)
        });
        
        if let Ok(task) = &result {
            // Log task pause
            log_task_pause(&task.id, &task.title);
        }
        
        result
    }

    pub fn resume_task(&self, task_id: &str) -> TaskPileResult<TaskPileTask> {
        let _lock = self.mutex.lock().unwrap();
        let now = Utc::now();
        let result = self.mutate_task(task_id, |task| {
            TaskPileStateMachine::transition_to_resumed(task, now)
        });
        
        if let Ok(task) = &result {
            // Log task resume
            log_task_resume(&task.id, &task.title);
        }
        
        result
    }

    pub fn cancel_task(&self, task_id: &str) -> TaskPileResult<TaskPileTask> {
        let _lock = self.mutex.lock().unwrap();
        let now = Utc::now();
        let result = self.mutate_task(task_id, |task| {
            TaskPileStateMachine::transition_to_cancelled(task, now)
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
        let now = Utc::now();
        
        // Basic status counts
        let total = state.tasks.len();
        let queued = count_status(&state, TaskPileStatus::Queued);
        let running = count_status(&state, TaskPileStatus::Running);
        let paused = count_status(&state, TaskPileStatus::Paused);
        let completed = count_status(&state, TaskPileStatus::Completed);
        let failed = count_status(&state, TaskPileStatus::Failed);
        let cancelled = count_status(&state, TaskPileStatus::Cancelled);
        
        let next_due_at = state
            .tasks
            .iter()
            .filter(|task| matches!(task.status, TaskPileStatus::Queued))
            .filter_map(|task| task.next_run_at.as_ref().cloned())
            .min();
        
        // Calculate execution time metrics
        let mut total_execution_time = 0.0;
        let mut execution_count = 0;
        let mut tasks_completed_24h = 0;
        let mut tasks_failed_24h = 0;
        let day_ago = now - Duration::hours(24);
        
        for task in &state.tasks {
            if let Some(duration) = task.execution_duration {
                total_execution_time += duration;
                execution_count += 1;
            }
            
            if task.status == TaskPileStatus::Completed {
                if let Some(completed_at) = task.completed_at {
                    if completed_at >= day_ago {
                        tasks_completed_24h += 1;
                    }
                }
            } else if task.status == TaskPileStatus::Failed {
                if let Some(updated_at) = task.updated_at {
                    if updated_at >= day_ago {
                        tasks_failed_24h += 1;
                    }
                }
            }
        }
        
        let average_execution_time = if execution_count > 0 {
            Some(total_execution_time / execution_count as f64)
        } else {
            None
        };
        
        // Calculate success/failure rates
        let terminal_tasks = completed + failed + cancelled;
        let failure_rate = if terminal_tasks > 0 {
            (failed as f64 / terminal_tasks as f64) * 100.0
        } else {
            0.0
        };
        let success_rate = if terminal_tasks > 0 {
            (completed as f64 / terminal_tasks as f64) * 100.0
        } else {
            0.0
        };
        
        // Calculate throughput (tasks per hour)
        let throughput = if total > 0 {
            let oldest_task = state.tasks.iter().map(|t| t.created_at).min().unwrap_or(now);
            let hours_since_oldest = now.signed_duration_since(oldest_task).num_hours() as f64;
            if hours_since_oldest > 0 {
                total as f64 / hours_since_oldest
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        // Calculate lease metrics
        let mut active_leases = 0;
        let mut expired_leases = 0;
        
        for task in &state.tasks {
            if task.status == TaskPileStatus::Running {
                if let Some(lease_expires) = task.lease_expires_at {
                    if lease_expires > now {
                        active_leases += 1;
                    } else {
                        expired_leases += 1;
                    }
                }
            }
        }
        
        // Resource usage (placeholder values, would need actual system metrics)
        let memory_usage = None; // Would use sysinfo crate to get actual memory usage
        let cpu_usage = None; // Would use sysinfo crate to get actual CPU usage
        
        // Peak concurrent tasks (placeholder, would need historical data)
        let peak_concurrent_tasks = running;
        
        Ok(TaskPileStats {
            total,
            queued,
            running,
            paused,
            completed,
            failed,
            cancelled,
            next_due_at,
            storage_path: self.store.state_path().display().to_string(),
            cloud_ready: self.cloud_adapter.status().ready,
            average_execution_time,
            total_execution_time,
            failure_rate,
            success_rate,
            throughput,
            active_leases,
            expired_leases,
            memory_usage,
            cpu_usage,
            tasks_completed_24h,
            tasks_failed_24h,
            peak_concurrent_tasks,
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

    pub fn renew_lease(&self, task_id: &str) -> TaskPileResult<TaskPileTask> {
        let _lock = self.mutex.lock().unwrap();
        let now = Utc::now();
        let result = self.mutate_task(task_id, |task| {
            TaskPileStateMachine::renew_lease(task, now)
        });
        
        result
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
            TaskPileStateMachine::cleanup_expired_lease(task, now);
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
        let created = service.add_task(request).expect("create task");

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
        let created = service.add_task(request).expect("create task");

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

    #[test]
    fn concurrent_task_operations() {
        use std::sync::Arc;
        use std::thread;

        let temp = tempdir().expect("tempdir");
        let service = Arc::new(TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path())));

        // Create multiple tasks concurrently
        let mut handles = vec![];
        for i in 0..5 {
            let service_clone = service.clone();
            let handle = thread::spawn(move || {
                let request = NewTaskRequest {
                    instruction: format!("concurrent task {}", i),
                    priority: if i % 2 == 0 { TaskPilePriority::High } else { TaskPilePriority::Normal },
                    ..NewTaskRequest::default()
                };
                service_clone.add_task(request).expect("create task")
            });
            handles.push(handle);
        }

        // Wait for all tasks to be created
        let task_ids: Vec<String> = handles.into_iter().map(|h| h.join().unwrap().id).collect();
        assert_eq!(task_ids.len(), 5);

        // Check that all tasks were created
        let tasks = service.list_tasks(None, None).expect("list tasks");
        assert_eq!(tasks.len(), 5);
    }

    #[test]
    fn lease_renewal() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        let request = NewTaskRequest {
            instruction: "test lease renewal".to_string(),
            ..NewTaskRequest::default()
        };
        let created = service.add_task(request).expect("create task");

        // Claim the task
        let now = Utc::now();
        let claimed = service.claim_next_due(now).expect("claim").expect("task");
        assert_eq!(claimed.id, created.id);
        assert_eq!(claimed.status, TaskPileStatus::Running);

        // Renew the lease
        let renewed = service.renew_lease(&claimed.id).expect("renew lease");
        assert_eq!(renewed.id, claimed.id);
        assert!(renewed.lease_expires_at.is_some());
        assert!(renewed.lease_expires_at.unwrap() > now);
    }

    #[test]
    fn lease_conflict_detection() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        let request = NewTaskRequest {
            instruction: "test lease conflict".to_string(),
            ..NewTaskRequest::default()
        };
        let created = service.add_task(request).expect("create task");

        // Claim the task
        let now = Utc::now();
        let claimed = service.claim_next_due(now).expect("claim").expect("task");
        assert_eq!(claimed.id, created.id);

        // Try to claim the same task again immediately (should fail due to active lease)
        let claimed_again = service.claim_next_due(now).expect("claim again");
        assert!(claimed_again.is_none());
    }

    #[test]
    fn storage_recovery() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        
        // Create some tasks
        for i in 0..3 {
            let request = NewTaskRequest {
                instruction: format!("recovery test task {}", i),
                ..NewTaskRequest::default()
            };
            service.add_task(request).expect("create task");
        }

        // Create a new service instance with the same storage directory
        let service2 = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        let tasks = service2.list_tasks(None, None).expect("list tasks");
        assert_eq!(tasks.len(), 3);
    }

    #[test]
    fn task_execution_time_tracking() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        let request = NewTaskRequest {
            instruction: "test execution time tracking".to_string(),
            ..NewTaskRequest::default()
        };
        let created = service.add_task(request).expect("create task");

        // Claim the task
        let now = Utc::now();
        let claimed = service.claim_next_due(now).expect("claim").expect("task");
        assert!(claimed.started_at.is_some());

        // Simulate some work
        std::thread::sleep(std::time::Duration::milliseconds(100));

        // Complete the task
        let completed = service.complete_task(&claimed.id, "completed").expect("complete task");
        assert_eq!(completed.status, TaskPileStatus::Completed);
        assert!(completed.completed_at.is_some());
        assert!(completed.execution_duration.is_some());
        assert!(completed.execution_duration.unwrap() > 0.0);
    }

    #[test]
    fn pagination() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        
        // Create 10 tasks
        for i in 0..10 {
            let request = NewTaskRequest {
                instruction: format!("pagination test task {}", i),
                ..NewTaskRequest::default()
            };
            service.add_task(request).expect("create task");
        }

        // Test pagination
        let page1 = service.list_tasks(Some(1), Some(5)).expect("page 1");
        assert_eq!(page1.len(), 5);
        
        let page2 = service.list_tasks(Some(2), Some(5)).expect("page 2");
        assert_eq!(page2.len(), 5);
        
        let page3 = service.list_tasks(Some(3), Some(5)).expect("page 3");
        assert_eq!(page3.len(), 0);
    }

    #[test]
    fn task_filtering() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        
        // Create tasks with different statuses
        let request1 = NewTaskRequest {
            instruction: "queued task".to_string(),
            ..NewTaskRequest::default()
        };
        service.add_task(request1).expect("create queued task");

        let request2 = NewTaskRequest {
            instruction: "running task".to_string(),
            ..NewTaskRequest::default()
        };
        let running_task = service.add_task(request2).expect("create running task");
        service.claim_next_due(Utc::now()).expect("claim task");

        // Test filtering by status
        let queued_tasks = service.list_tasks_by_status(TaskPileStatus::Queued).expect("queued tasks");
        assert_eq!(queued_tasks.len(), 1);
        
        let running_tasks = service.list_tasks_by_status(TaskPileStatus::Running).expect("running tasks");
        assert_eq!(running_tasks.len(), 1);
    }

    #[test]
    fn stats_calculation() {
        let temp = tempdir().expect("tempdir");
        let service = TaskPileService::new(TaskPileConfig::default(), PathBuf::from(temp.path()));
        
        // Create and complete a task
        let request = NewTaskRequest {
            instruction: "test stats".to_string(),
            ..NewTaskRequest::default()
        };
        let created = service.add_task(request).expect("create task");
        service.claim_next_due(Utc::now()).expect("claim task");
        service.complete_task(&created.id, "completed").expect("complete task");

        // Check stats
        let stats = service.stats().expect("stats");
        assert_eq!(stats.total, 1);
        assert_eq!(stats.completed, 1);
        assert!(stats.average_execution_time.is_some());
        assert!(stats.total_execution_time > 0.0);
        assert_eq!(stats.success_rate, 100.0);
        assert_eq!(stats.failure_rate, 0.0);
    }
}  
