use chrono::{DateTime, Utc};
use mc_daemon::{
    ApprovalMode, CompressionMode, IsolationProfile, NewTaskRequest, TaskPilePriority,
    TaskPileSchedule, TaskPileService, TaskPileTask, TaskTarget,
};

use crate::init::AppContext;

pub fn execute(args: &[String], ctx: &AppContext) -> String {
    let service = TaskPileService::new(ctx.config.daemon.taskpile.clone(), ctx.cwd.clone());
    if args.is_empty() {
        return render_list(service.list_tasks());
    }

    match args[0].as_str() {
        "list" => render_list(service.list_tasks()),
        "show" => {
            let Some(task_id) = args.get(1) else {
                return "usage: mc-cli taskpile show <task_id>".to_string();
            };
            match service.get_task(task_id) {
                Ok(task) => render_task(task),
                Err(error) => error.to_string(),
            }
        }
        "add" => {
            let parsed = parse_new_task(args, ctx);
            match parsed.and_then(|request| service.add_task(request).map_err(|error| error.to_string())) {
                Ok(task) => format!(
                    "task queued\nid: {}\nstatus: {:?}\npriority: {:?}\nnext_run_at: {}\nstorage: {}",
                    task.id,
                    task.status,
                    task.priority,
                    format_optional_time(task.next_run_at),
                    service.state_path()
                ),
                Err(error) => error.to_string(),
            }
        }
        "claim" => match service.claim_next_due(Utc::now()) {
            Ok(Some(task)) => render_claim(task),
            Ok(None) => "no due tasks".to_string(),
            Err(error) => error.to_string(),
        },
        "complete" => {
            let Some(task_id) = args.get(1) else {
                return "usage: mc-cli taskpile complete <task_id> [summary...]".to_string();
            };
            let summary = if args.len() > 2 {
                args[2..].join(" ")
            } else {
                "completed".to_string()
            };
            match service.complete_task(task_id, &summary) {
                Ok(task) => render_task(task),
                Err(error) => error.to_string(),
            }
        }
        "fail" => {
            let Some(task_id) = args.get(1) else {
                return "usage: mc-cli taskpile fail <task_id> [reason...]".to_string();
            };
            let reason = if args.len() > 2 {
                args[2..].join(" ")
            } else {
                "failed".to_string()
            };
            match service.fail_task(task_id, &reason) {
                Ok(task) => render_task(task),
                Err(error) => error.to_string(),
            }
        }
        "pause" => mutate_single_task(args, "pause", |task_id| service.pause_task(task_id)),
        "resume" => mutate_single_task(args, "resume", |task_id| service.resume_task(task_id)),
        "cancel" => mutate_single_task(args, "cancel", |task_id| service.cancel_task(task_id)),
        "stats" => match service.stats() {
            Ok(stats) => format!(
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
            ),
            Err(error) => error.to_string(),
        },
        "cloud-status" => {
            let status = service.cloud_status();
            format!(
                "taskpile cloud\nenabled: {}\nready: {}\nendpoint: {}\nproject_id: {}\nnote: {}",
                status.enabled,
                status.ready,
                status.endpoint.unwrap_or_else(|| "<unset>".to_string()),
                status.project_id.unwrap_or_else(|| "<unset>".to_string()),
                status.note
            )
        }
        "cloud-preview" => {
            let Some(task_id) = args.get(1) else {
                return "usage: mc-cli taskpile cloud-preview <task_id>".to_string();
            };
            match service.preview_cloud_payload(task_id) {
                Ok(payload) => format!(
                    "cloud preview\ntask_id: {}\naccepted_at: {}\nendpoint: {}\nproject_id: {}\ntarget: {:?}\nnote: {}",
                    payload.task_id,
                    payload.accepted_at.to_rfc3339(),
                    payload.endpoint.unwrap_or_else(|| "<unset>".to_string()),
                    payload.project_id.unwrap_or_else(|| "<unset>".to_string()),
                    payload.target,
                    payload.note
                ),
                Err(error) => error.to_string(),
            }
        }
        _ => "usage: mc-cli taskpile [list|add|show|claim|complete|fail|pause|resume|cancel|stats|cloud-status|cloud-preview]".to_string(),
    }
}

fn mutate_single_task<F>(args: &[String], action: &str, mutator: F) -> String
where
    F: FnOnce(&str) -> Result<TaskPileTask, mc_daemon::TaskPileError>,
{
    let Some(task_id) = args.get(1) else {
        return format!("usage: mc-cli taskpile {action} <task_id>");
    };
    match mutator(task_id) {
        Ok(task) => render_task(task),
        Err(error) => error.to_string(),
    }
}

fn parse_new_task(args: &[String], ctx: &AppContext) -> Result<NewTaskRequest, String> {
    let mut request = NewTaskRequest::default();
    request.token_budget = ctx.config.daemon.taskpile.default_token_budget;
    request.isolation =
        IsolationProfile::parse(&ctx.config.daemon.taskpile.default_isolation_profile);
    request.cloud_endpoint = ctx.config.daemon.taskpile.cloud.endpoint.clone();
    request.cloud_project_id = ctx.config.daemon.taskpile.cloud.project_id.clone();

    let mut instruction_parts = Vec::new();
    for token in args.iter().skip(1) {
        if let Some((key, value)) = token.split_once('=') {
            apply_option(&mut request, key, value)?;
        } else {
            instruction_parts.push(token.clone());
        }
    }
    if instruction_parts.is_empty() {
        return Err("usage: mc-cli taskpile add <instruction...> [priority=high] [schedule=manual|at:2026-04-20T10:00:00Z|interval:300] [target=local|cloud] [isolation=workspace-write] [budget=12000] [compression=balanced] [parallelism=1] [approval=auto] [tags=a,b] [title=custom] [model=name]".to_string());
    }
    request.instruction = instruction_parts.join(" ");
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

fn render_list(result: Result<Vec<TaskPileTask>, mc_daemon::TaskPileError>) -> String {
    match result {
        Ok(tasks) if tasks.is_empty() => "taskpile is empty".to_string(),
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
            lines.join("\n")
        }
        Err(error) => error.to_string(),
    }
}

fn render_task(task: TaskPileTask) -> String {
    format!(
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
    )
}

fn render_claim(task: TaskPileTask) -> String {
    format!(
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
    )
}

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

fn format_optional_time(value: Option<DateTime<Utc>>) -> String {
    value
        .map(|time| time.to_rfc3339())
        .unwrap_or_else(|| "<none>".to_string())
}
