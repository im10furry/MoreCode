use chrono::{DateTime, Utc};
use mc_daemon::{
    ApprovalMode, CompressionMode, IsolationProfile, NewTaskRequest, TaskPilePriority,
    TaskPileSchedule, TaskPileService, TaskPileTask, TaskTarget,
};

use crate::cli::TaskpileCommand;
use crate::init::AppContext;

pub async fn execute(context: &AppContext, command: &TaskpileCommand) -> Result<String, String> {
    let service = TaskPileService::new(context.config.daemon.taskpile.clone(), context.cwd.clone());
    match command {
        TaskpileCommand::List => render_list(service.list_tasks()),
        TaskpileCommand::Show { task_id } => match service.get_task(task_id) {
            Ok(task) => render_task(task),
            Err(error) => Err(error.to_string()),
        },
        TaskpileCommand::Add {
            instruction,
            options,
        } => {
            let request = parse_new_task_request(instruction, options, context)?;
            match service
                .add_task(request)
                .map_err(|error| error.to_string())
            {
                Ok(task) => Ok(format!(
                    "task queued\nid: {}\nstatus: {:?}\npriority: {:?}\nnext_run_at: {}\nstorage: {}",
                    task.id,
                    task.status,
                    task.priority,
                    format_optional_time(task.next_run_at),
                    service.state_path()
                )),
                Err(error) => Err(error),
            }
        }
        TaskpileCommand::Claim => match service.claim_next_due(Utc::now()) {
            Ok(Some(task)) => render_claim(task),
            Ok(None) => Ok("no due tasks".to_string()),
            Err(error) => Err(error.to_string()),
        },
        TaskpileCommand::Complete {
            task_id, summary,
        } => {
            let summary = summary.clone().unwrap_or_else(|| "completed".to_string());
            match service.complete_task(task_id, &summary) {
                Ok(task) => render_task(task),
                Err(error) => Err(error.to_string()),
            }
        }
        TaskpileCommand::Fail { task_id, reason } => {
            let reason = reason.clone().unwrap_or_else(|| "failed".to_string());
            match service.fail_task(task_id, &reason) {
                Ok(task) => render_task(task),
                Err(error) => Err(error.to_string()),
            }
        }
        TaskpileCommand::Pause { task_id } => {
            mutate_single_result(service.pause_task(task_id))
        }
        TaskpileCommand::Resume { task_id } => {
            mutate_single_result(service.resume_task(task_id))
        }
        TaskpileCommand::Cancel { task_id } => {
            mutate_single_result(service.cancel_task(task_id))
        }
        TaskpileCommand::Stats => match service.stats() {
            Ok(stats) => Ok(format!(
                "taskpile stats\ntotal: {}\nqueued: {}\nrunning: {}\npaused: {}\ncompleted: {}\nfailed: {}\ncancelled: {}\nnext_due_at: {}\ncloud_ready: {}\nstorage: {}",
                stats.total,
                stats.queued,
                stats.running,
                stats.paused,
                stats.completed,
                stats.failed,
                stats.cancelled,
                format_optional_time(stats.next_due_at),
                stats.cloud_ready,
                stats.storage_path
            )),
            Err(error) => Err(error.to_string()),
        },
        TaskpileCommand::CloudStatus => {
            let status = service.cloud_status();
            Ok(format!(
                "taskpile cloud\nenabled: {}\nready: {}\nendpoint: {}\nproject_id: {}\nnote: {}",
                status.enabled,
                status.ready,
                status.endpoint.unwrap_or_else(|| "<unset>".to_string()),
                status.project_id.unwrap_or_else(|| "<unset>".to_string()),
                status.note
            ))
        }
        TaskpileCommand::CloudPreview { task_id } => {
            match service.preview_cloud_payload(task_id) {
                Ok(payload) => Ok(format!(
                    "cloud preview\ntask_id: {}\naccepted_at: {}\nendpoint: {}\nproject_id: {}\ntarget: {:?}\nnote: {}",
                    payload.task_id,
                    payload.accepted_at.to_rfc3339(),
                    payload.endpoint.unwrap_or_else(|| "<unset>".to_string()),
                    payload.project_id.unwrap_or_else(|| "<unset>".to_string()),
                    payload.target,
                    payload.note
                )),
                Err(error) => Err(error.to_string()),
            }
        }
    }
}

fn mutate_single_result(
    result: Result<TaskPileTask, mc_daemon::TaskPileError>,
) -> Result<String, String> {
    match result {
        Ok(task) => render_task(task),
        Err(error) => Err(error.to_string()),
    }
}

fn parse_new_task_request(
    instruction: &str,
    options: &[String],
    ctx: &AppContext,
) -> Result<NewTaskRequest, String> {
    let mut request = NewTaskRequest {
        instruction: instruction.to_string(),
        token_budget: ctx.config.daemon.taskpile.default_token_budget,
        isolation: IsolationProfile::parse(&ctx.config.daemon.taskpile.default_isolation_profile),
        cloud_endpoint: ctx.config.daemon.taskpile.cloud.endpoint.clone(),
        cloud_project_id: ctx.config.daemon.taskpile.cloud.project_id.clone(),
        ..NewTaskRequest::default()
    };

    for option in options {
        let Some((key, value)) = option.split_once('=') else {
            continue;
        };
        apply_option(&mut request, key, value)?;
    }
    Ok(request)
}

fn apply_option(request: &mut NewTaskRequest, key: &str, value: &str) -> Result<(), String> {
    match key.to_ascii_lowercase().as_str() {
        "priority" => {
            request.priority = TaskPilePriority::parse(value)
                .ok_or_else(|| format!("invalid priority: {value}"))?;
        }
        "schedule" => request.schedule = parse_schedule(value)?,
        "target" => {
            request.target =
                TaskTarget::parse(value).ok_or_else(|| format!("invalid target: {value}"))?;
        }
        "isolation" => request.isolation = IsolationProfile::parse(value),
        "budget" => {
            request.token_budget = value
                .parse::<u32>()
                .map_err(|_| format!("invalid budget: {value}"))?;
        }
        "compression" => {
            request.compression = CompressionMode::parse(value)
                .ok_or_else(|| format!("invalid compression: {value}"))?;
        }
        "parallelism" => {
            request.parallelism = value
                .parse::<u8>()
                .map_err(|_| format!("invalid parallelism: {value}"))?;
        }
        "approval" => {
            request.approval =
                ApprovalMode::parse(value).ok_or_else(|| format!("invalid approval: {value}"))?;
        }
        "retries" => {
            request.max_attempts = value
                .parse::<u32>()
                .map_err(|_| format!("invalid retries: {value}"))?;
        }
        "tags" => {
            request.tags = value
                .split(',')
                .filter(|item| !item.trim().is_empty())
                .map(|item| item.trim().to_string())
                .collect();
        }
        "title" => request.title = Some(value.to_string()),
        "model" => request.model = Some(value.to_string()),
        "cloud_endpoint" => request.cloud_endpoint = Some(value.to_string()),
        "cloud_project_id" => request.cloud_project_id = Some(value.to_string()),
        other => {
            request
                .metadata
                .insert(other.to_string(), value.to_string());
        }
    }
    Ok(())
}

fn parse_schedule(raw: &str) -> Result<TaskPileSchedule, String> {
    if raw.eq_ignore_ascii_case("manual") {
        return Ok(TaskPileSchedule::Manual);
    }
    if let Some(value) = raw.strip_prefix("at:") {
        let at = DateTime::parse_from_rfc3339(value)
            .map_err(|_| format!("invalid RFC3339 schedule: {value}"))?
            .with_timezone(&Utc);
        return Ok(TaskPileSchedule::At(at));
    }
    if let Some(value) = raw.strip_prefix("interval:") {
        let seconds = value
            .parse::<u64>()
            .map_err(|_| format!("invalid interval seconds: {value}"))?;
        return Ok(TaskPileSchedule::IntervalSeconds(seconds));
    }
    Err(format!("invalid schedule: {raw}"))
}

fn render_list(
    result: Result<Vec<TaskPileTask>, mc_daemon::TaskPileError>,
) -> Result<String, String> {
    match result {
        Ok(tasks) if tasks.is_empty() => Ok("taskpile is empty".to_string()),
        Ok(tasks) => {
            let mut lines = vec!["taskpile".to_string()];
            for task in tasks {
                lines.push(format!(
                    "{} | {:?} | {:?} | {} | {}",
                    short_id(&task.id),
                    task.status,
                    task.priority,
                    format_optional_time(task.next_run_at),
                    task.title
                ));
            }
            Ok(lines.join("\n"))
        }
        Err(error) => Err(error.to_string()),
    }
}

fn render_task(task: TaskPileTask) -> Result<String, String> {
    Ok(format!(
        "task\nid: {}\ntitle: {}\nstatus: {:?}\npriority: {:?}\nschedule: {:?}\ntarget: {:?}\nisolation: {}\nparallelism: {}\ntoken_budget: {}\nnext_run_at: {}\nattempts: {}/{}\nlast_error: {}\nsummary: {}\ntags: {}\ninstruction: {}",
        task.id,
        task.title,
        task.status,
        task.priority,
        task.schedule,
        task.execution.target,
        task.execution.isolation.as_str(),
        task.execution.parallelism,
        task.execution.token_controls.budget,
        format_optional_time(task.next_run_at),
        task.attempts,
        task.max_attempts,
        task.last_error.unwrap_or_else(|| "<none>".to_string()),
        task.result_summary.unwrap_or_else(|| "<none>".to_string()),
        if task.tags.is_empty() {
            "<none>".to_string()
        } else {
            task.tags.join(",")
        },
        task.instruction
    ))
}

fn render_claim(task: TaskPileTask) -> Result<String, String> {
    Ok(format!(
        "claimed\nid: {}\nstatus: {:?}\npriority: {:?}\ntarget: {:?}\nisolation: {}\nparallelism: {}\nbudget: {}\nlease_expires_at: {}\ninstruction: {}",
        task.id,
        task.status,
        task.priority,
        task.execution.target,
        task.execution.isolation.as_str(),
        task.execution.parallelism,
        task.execution.token_controls.budget,
        format_optional_time(task.lease_expires_at),
        task.instruction
    ))
}

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

fn format_optional_time(value: Option<DateTime<Utc>>) -> String {
    value
        .map(|time| time.to_rfc3339())
        .unwrap_or_else(|| "<none>".to_string())
}
