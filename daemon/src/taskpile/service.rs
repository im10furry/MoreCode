use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::Arc,
};

use chrono::{DateTime, Duration, Utc};
use mc_config::TaskPileConfig;
use uuid::Uuid;

use crate::error::{TaskPileError, TaskPileResult};

use super::{
    cloud::{CloudAdapterStatus, CloudPayload, CloudTaskAdapter, NoopCloudAdapter},
    store::{TaskPileState, TaskPileStore},
    types::{
        ApprovalMode, CompressionMode, ExecutionOptions, IsolationProfile, NewTaskRequest,
        TaskPilePriority, TaskPileSchedule, TaskPileStats, TaskPileStatus, TaskPileTask,
        TaskTarget, TokenControls,
    },
};

pub struct TaskPileService {
    config: TaskPileConfig,
    store: TaskPileStore,
    cloud_adapter: Arc<dyn CloudTaskAdapter>,
}

impl TaskPileService {
    pub fn new(config: TaskPileConfig, workspace_root: PathBuf) -> Self {
        let storage_dir = config
            .storage_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| workspace_root.join(".morecode").join("taskpile"));
        let cloud_adapter = Arc::new(NoopCloudAdapter::new(
            config.cloud.enabled,
            config.cloud.endpoint.clone(),
            config.cloud.project_id.clone(),
        ));
        Self {
            config,
            store: TaskPileStore::new(storage_dir),
            cloud_adapter,
        }
    }

    pub fn state_path(&self) -> String {
        self.store.state_path().display().to_string()
    }

    pub fn list_tasks(&self) -> TaskPileResult<Vec<TaskPileTask>> {
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
        Ok(task)
    }

    pub fn claim_next_due(&self, now: DateTime<Utc>) -> TaskPileResult<Option<TaskPileTask>> {
        let mut state = self.store.load()?;
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
        Ok(Some(claimed))
    }

    pub fn complete_task(&self, task_id: &str, summary: &str) -> TaskPileResult<TaskPileTask> {
        self.mutate_task(task_id, |task| {
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
        })
    }

    pub fn fail_task(&self, task_id: &str, reason: &str) -> TaskPileResult<TaskPileTask> {
        self.mutate_task(task_id, |task| {
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
        })
    }

    pub fn pause_task(&self, task_id: &str) -> TaskPileResult<TaskPileTask> {
        self.mutate_task(task_id, |task| {
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
        })
    }

    pub fn resume_task(&self, task_id: &str) -> TaskPileResult<TaskPileTask> {
        self.mutate_task(task_id, |task| {
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
        })
    }

    pub fn cancel_task(&self, task_id: &str) -> TaskPileResult<TaskPileTask> {
        self.mutate_task(task_id, |task| {
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
        })
    }

    pub fn stats(&self) -> TaskPileResult<TaskPileStats> {
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
        let task = self.get_task(task_id)?;
        self.cloud_adapter.preview_payload(&task)
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

fn task_digest(instruction: &str, target: &TaskTarget, schedule: &TaskPileSchedule) -> u64 {
    let mut hasher = DefaultHasher::new();
    instruction.hash(&mut hasher);
    target.hash(&mut hasher);
    std::mem::discriminant(schedule).hash(&mut hasher);
    hasher.finish()
}

fn truncate_title(instruction: &str) -> String {
    const MAX: usize = 48;
    let trimmed = instruction.trim();
    if trimmed.chars().count() <= MAX {
        trimmed.to_string()
    } else {
        let short = trimmed.chars().take(MAX).collect::<String>();
        format!("{short}...")
    }
}

fn retry_backoff(attempts: u32) -> Duration {
    let seconds = 30_i64.saturating_mul(2_i64.saturating_pow(attempts.saturating_sub(1)));
    Duration::seconds(seconds.clamp(30, 1800))
}

fn count_status(state: &TaskPileState, status: TaskPileStatus) -> usize {
    state
        .tasks
        .iter()
        .filter(|task| task.status == status)
        .count()
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
}
