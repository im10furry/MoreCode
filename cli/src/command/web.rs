use std::collections::HashMap;
use std::path::{Path, PathBuf};

use mc_core::{
    ApprovalStatus, PatchStatus, RunEvent, RunEventEnvelope, RunRecorder, RunStatus, RunStore,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration};

use super::workflow::{execute_run_with_recorder, WorkflowOptions};
use crate::{AppContext, ApprovalMode, ReviewCommand, RunCommand, WebCommand};

#[derive(Clone)]
struct WebState {
    project_root: PathBuf,
    config: mc_config::AppConfig,
}

#[derive(Debug, Clone)]
struct HttpRequest {
    method: String,
    path: String,
    query: HashMap<String, String>,
    body: Vec<u8>,
}

struct HttpResponse {
    status_line: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
}

impl HttpResponse {
    fn html(body: String) -> Self {
        Self {
            status_line: "200 OK",
            content_type: "text/html; charset=utf-8",
            body: body.into_bytes(),
            headers: Vec::new(),
        }
    }

    fn json(body: Vec<u8>) -> Self {
        Self {
            status_line: "200 OK",
            content_type: "application/json; charset=utf-8",
            body,
            headers: Vec::new(),
        }
    }

    fn redirect(location: String) -> Self {
        Self {
            status_line: "302 Found",
            content_type: "text/plain; charset=utf-8",
            body: b"redirect".to_vec(),
            headers: vec![("Location".to_string(), location)],
        }
    }

    fn not_found(message: &str) -> Self {
        Self {
            status_line: "404 Not Found",
            content_type: "text/plain; charset=utf-8",
            body: message.as_bytes().to_vec(),
            headers: Vec::new(),
        }
    }

    fn with_status(mut self, status_line: &'static str) -> Self {
        self.status_line = status_line;
        self
    }

    fn method_not_allowed() -> Self {
        Self {
            status_line: "405 Method Not Allowed",
            content_type: "text/plain; charset=utf-8",
            body: b"method not allowed".to_vec(),
            headers: vec![("Allow".to_string(), "GET".to_string())],
        }
    }
}

pub async fn execute(context: &AppContext, command: &WebCommand) -> Result<String, String> {
    serve(
        context,
        command.port,
        command.run_id.clone(),
        command.request.clone(),
        None,
    )
    .await
}

pub async fn execute_from_run(
    context: &AppContext,
    command: &RunCommand,
) -> Result<String, String> {
    let options = WorkflowOptions {
        plan_only: command.plan_only,
        approval: normalize_web_approval(command.approval),
    };
    serve(
        context,
        3000,
        None,
        Some(command.request.clone()),
        Some(options),
    )
    .await
}

pub async fn execute_from_review(
    context: &AppContext,
    command: &ReviewCommand,
) -> Result<String, String> {
    serve(context, 3000, Some(command.run_id.clone()), None, None).await
}

async fn serve(
    context: &AppContext,
    port: u16,
    run_id: Option<String>,
    initial_request: Option<String>,
    initial_options: Option<WorkflowOptions>,
) -> Result<String, String> {
    let state = WebState {
        project_root: context.project_root.clone(),
        config: context.config.clone(),
    };

    let initial_run_id = if let Some(request) = initial_request {
        Some(launch_run(
            state.clone(),
            request,
            initial_options.unwrap_or(WorkflowOptions {
                plan_only: false,
                approval: ApprovalMode::Auto,
            }),
        )?)
    } else {
        run_id
    };

    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|error| error.to_string())?;
    let launch_url = match initial_run_id.as_deref() {
        Some(run_id) => format!("http://127.0.0.1:{port}/runs/{run_id}"),
        None => format!("http://127.0.0.1:{port}/runs"),
    };
    println!("MoreCode Web UI listening on {launch_url}");

    loop {
        let (stream, _) = listener.accept().await.map_err(|error| error.to_string())?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream, state).await {
                eprintln!("web connection error: {error}");
            }
        });
    }
}

fn launch_run(
    state: WebState,
    request: String,
    options: WorkflowOptions,
) -> Result<String, String> {
    let recorder =
        RunRecorder::create(&state.project_root, &request).map_err(|error| error.to_string())?;
    let run_id = recorder.snapshot().summary.run_id.clone();
    let project_root = state.project_root.clone();
    let fail_root = project_root.clone();
    let fail_run_id = run_id.clone();
    tokio::spawn(async move {
        let memory = match mc_memory::MemorySystem::new(&project_root).await {
            Ok(memory) => std::sync::Arc::new(memory),
            Err(error) => {
                let msg = error.to_string();
                eprintln!("failed to initialize memory for web run: {msg}");
                finalize_failed_run(&fail_root, &fail_run_id, &msg);
                return;
            }
        };
        let context = AppContext {
            cwd: project_root.clone(),
            project_root,
            config: state.config.clone(),
            memory,
        };
        if let Err(error) =
            execute_run_with_recorder(&context, &request, options, None, recorder).await
        {
            eprintln!("web run failed: {error}");
            finalize_failed_run(&context.project_root, &fail_run_id, &error);
        }
    });
    Ok(run_id)
}

fn finalize_failed_run(project_root: &std::path::Path, run_id: &str, error: &str) {
    match RunRecorder::open(project_root, run_id) {
        Ok(mut recorder) => {
            let started_at = recorder.snapshot().summary.started_at;
            let total_tokens = recorder.snapshot().summary.total_tokens;
            let duration_ms = chrono::Utc::now()
                .signed_duration_since(started_at)
                .num_milliseconds()
                .max(0) as u64;
            let _ = recorder.emit(RunEvent::RunFinished {
                status: mc_core::RunStatus::Failed,
                summary: Some(format!("run failed before completion: {error}")),
                total_tokens,
                total_duration_ms: duration_ms,
                review_verdict: None,
                changed_files: vec![],
            });
        }
        Err(e) => {
            eprintln!("failed to finalize failed run {run_id}: {e}");
        }
    }
}

async fn handle_connection(mut stream: TcpStream, state: WebState) -> Result<(), String> {
    let Some(request) = read_request(&mut stream).await? else {
        return Ok(());
    };

    let segments = request
        .path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if let ["api", "runs", run_id, "stream"] = segments.as_slice() {
        return stream_run_events(&mut stream, &state, run_id).await;
    }

    let response = route_request(&state, request).await;
    write_response(&mut stream, response)
        .await
        .map_err(|error| error.to_string())
}

async fn route_request(state: &WebState, request: HttpRequest) -> HttpResponse {
    let segments = request
        .path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    // POST /api/runs — launch a new run with JSON body
    if request.method == "POST" && segments.as_slice() == &["api", "runs"] {
        return api_launch_run(state, &request.body);
    }

    if request.method != "GET" {
        return HttpResponse::method_not_allowed();
    }

    match segments.as_slice() {
        [] => HttpResponse::redirect("/runs".to_string()),
        ["runs"] => render_runs_response(state, request.query),
        ["launch"] => launch_response(state, request.query),
        ["api", "runs"] => api_runs_response(state),
        ["api", "runs", run_id] => api_run_response(state, run_id),
        ["api", "runs", _, "stream"] => HttpResponse::not_found("stream handled upstream"),
        ["runs", run_id] => render_run_response(state, run_id),
        ["runs", run_id, "patches", patch_id, action] => {
            patch_action_response(state, run_id, patch_id, action)
        }
        ["runs", run_id, "approvals", approval_id, action] => {
            approval_action_response(state, run_id, approval_id, action)
        }
        ["artifacts", run_id, tail @ ..] => artifact_response(state, run_id, tail),
        ["favicon.ico"] => HttpResponse {
            status_line: "204 No Content",
            content_type: "text/plain; charset=utf-8",
            body: Vec::new(),
            headers: Vec::new(),
        },
        _ => HttpResponse::not_found("not found"),
    }
}

fn render_runs_response(state: &WebState, query: HashMap<String, String>) -> HttpResponse {
    let store = run_store_from_state(state);
    let summaries = match store.list_summaries() {
        Ok(summaries) => summaries,
        Err(error) => {
            return HttpResponse::html(layout_html(
                "MoreCode Runs",
                format!(
                    "<section class=\"panel\"><p>{}</p></section>",
                    escape_html(&error.to_string())
                ),
                None,
            ));
        }
    };

    let mut cards = String::new();
    for summary in &summaries {
        let status_class = status_class(summary.status);
        let changed = if summary.changed_files.is_empty() {
            "none".to_string()
        } else {
            summary.changed_files.join(", ")
        };
        cards.push_str(&format!(
            "<a class=\"run-card\" href=\"/runs/{run_id}\">\
                <div class=\"run-card__meta\">\
                    <span class=\"chip {status_class}\">{status}</span>\
                    <span>{started}</span>\
                </div>\
                <h3>{request}</h3>\
                <p class=\"muted\">tokens {tokens} · changed {changed}</p>\
            </a>",
            run_id = escape_html(&summary.run_id),
            status_class = status_class,
            status = escape_html(&format!("{:?}", summary.status).to_ascii_lowercase()),
            started = escape_html(
                &summary
                    .started_at
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string()
            ),
            request = escape_html(&summary.request),
            tokens = summary.total_tokens,
            changed = escape_html(&changed),
        ));
    }

    let launch_message = query
        .get("launched")
        .map(|run_id| {
            format!(
                "<div class=\"banner\">Run started: <a href=\"/runs/{id}\">{id}</a></div>",
                id = escape_html(run_id)
            )
        })
        .unwrap_or_default();

    let body = format!(
        "{launch_message}\
        <section class=\"hero\">\
            <div class=\"hero__copy\">\
                <p class=\"eyebrow\">MoreCode Control Room</p>\
                <h1>Editorial trace, terminal-grade execution.</h1>\
                <p>Launch a run, inspect every step, review patch previews, and replay execution from the browser.</p>\
            </div>\
            <form class=\"launch-form panel\" action=\"/launch\" method=\"get\">\
                <label>Request</label>\
                <textarea name=\"request\" placeholder=\"Describe what MoreCode should do\"></textarea>\
                <div class=\"launch-form__row\">\
                    <label><input type=\"checkbox\" name=\"plan_only\" value=\"1\"> Plan only</label>\
                    <label>Approval\
                        <select name=\"approval\">\
                            <option value=\"auto\">Auto</option>\
                            <option value=\"deny\">Deny risky ops</option>\
                        </select>\
                    </label>\
                </div>\
                <button type=\"submit\">Launch Run</button>\
            </form>\
        </section>\
        <section class=\"panel\">\
            <div class=\"section-head\">\
                <h2>Runs</h2>\
                <span class=\"muted\">{count} recorded</span>\
            </div>\
            <div class=\"run-grid\">{cards}</div>\
        </section>",
        launch_message = launch_message,
        count = summaries.len(),
        cards = if cards.is_empty() {
            "<p class=\"muted\">No runs yet.</p>".to_string()
        } else {
            cards
        }
    );

    HttpResponse::html(layout_html("MoreCode Runs", body, Some(3)))
}

fn render_run_response(state: &WebState, run_id: &str) -> HttpResponse {
    let snapshot = match run_store_from_state(state).load_snapshot(run_id) {
        Ok(snapshot) => snapshot,
        Err(error) => return HttpResponse::not_found(&error.to_string()),
    };

    let steps = snapshot
        .summary
        .steps
        .iter()
        .map(|step| {
            format!(
                "<li class=\"step {status}\">\
                    <span class=\"step__title\">{title}</span>\
                    <span class=\"step__meta\">{tokens} tokens</span>\
                    <p>{summary}</p>\
                </li>",
                status = status_class_from_step(step.status),
                title = escape_html(&step.title),
                tokens = step.token_used,
                summary = escape_html(step.summary.as_deref().unwrap_or("")),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let timeline = snapshot
        .events
        .iter()
        .rev()
        .take(25)
        .map(render_event_item)
        .collect::<Vec<_>>()
        .join("");

    let patches = snapshot
        .summary
        .patches
        .iter()
        .map(|patch| {
            let actions = format!(
                "<div class=\"action-row\">\
                    <a href=\"/runs/{run_id}/patches/{patch_id}/accept\">Accept</a>\
                    <a href=\"/runs/{run_id}/patches/{patch_id}/reject\">Reject</a>\
                </div>",
                run_id = escape_html(run_id),
                patch_id = escape_html(&patch.patch_id)
            );
            format!(
                "<article class=\"patch-card panel\">\
                    <div class=\"section-head\">\
                        <h3>{file}</h3>\
                        <span class=\"chip {status_class}\">{status}</span>\
                    </div>\
                    <p>{rationale}</p>\
                    {actions}\
                    <pre>{preview}</pre>\
                </article>",
                file = escape_html(&patch.file_path),
                status_class = patch_status_class(patch.status),
                status = escape_html(&format!("{:?}", patch.status).to_ascii_lowercase()),
                rationale = escape_html(&patch.rationale),
                actions = actions,
                preview = escape_html(&patch.preview),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let approvals = snapshot
        .summary
        .approvals
        .iter()
        .map(|approval| {
            let actions = if approval.status == ApprovalStatus::Pending {
                format!(
                    "<div class=\"action-row\">\
                        <a href=\"/runs/{run_id}/approvals/{approval_id}/approve\">Approve</a>\
                        <a href=\"/runs/{run_id}/approvals/{approval_id}/reject\">Reject</a>\
                    </div>",
                    run_id = escape_html(run_id),
                    approval_id = escape_html(&approval.approval_id)
                )
            } else {
                String::new()
            };
            format!(
                "<article class=\"mini-card\">\
                    <div class=\"section-head\">\
                        <strong>{title}</strong>\
                        <span class=\"chip {status_class}\">{status}</span>\
                    </div>\
                    <p>{reason}</p>\
                    {actions}\
                </article>",
                title = escape_html(&approval.title),
                status_class = approval_status_class(approval.status),
                status = escape_html(&format!("{:?}", approval.status).to_ascii_lowercase()),
                reason = escape_html(&approval.reason),
                actions = actions
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let commands = snapshot
        .summary
        .commands
        .iter()
        .map(|command| {
            format!(
                "<article class=\"mini-card\">\
                    <div class=\"section-head\">\
                        <strong>{title}</strong>\
                        <span class=\"chip {status_class}\">{status}</span>\
                    </div>\
                    <p class=\"muted\">{command}</p>\
                    <pre>{stdout}</pre>\
                </article>",
                title = escape_html(&command.title),
                status_class = command_status_class(command.status),
                status = escape_html(&format!("{:?}", command.status).to_ascii_lowercase()),
                command = escape_html(&command.command),
                stdout = escape_html(&if command.stdout_tail.is_empty() {
                    command.stderr_tail.clone()
                } else {
                    command.stdout_tail.clone()
                }),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let artifacts = snapshot
        .summary
        .artifacts
        .iter()
        .map(|artifact| {
            format!(
                "<li><a href=\"/artifacts/{run_id}/{path}\">{title}</a></li>",
                run_id = escape_html(run_id),
                path = escape_path_segment(&artifact.relative_path),
                title = escape_html(&artifact.title),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let body = format!(
        "<section id=\"run-hero\" class=\"hero hero--compact\">\
            <div class=\"hero__copy\">\
                <p class=\"eyebrow\">Run Detail</p>\
                <h1>{request}</h1>\
                <p>Status <span class=\"chip {status_class}\">{status}</span> · {tokens} tokens</p>\
            </div>\
            <div class=\"hero__actions\">\
                <a class=\"button\" href=\"/runs\">Back to runs</a>\
                <a class=\"button button--ghost\" href=\"/api/runs/{run_id}\">JSON</a>\
            </div>\
        </section>\
        <section id=\"run-detail-grid\" class=\"detail-grid\">\
            <div class=\"column\">\
                <section class=\"panel\"><div class=\"section-head\"><h2>Step Tree</h2></div><ol class=\"step-list\">{steps}</ol></section>\
                <section class=\"panel\"><div class=\"section-head\"><h2>Timeline</h2></div><div class=\"timeline\">{timeline}</div></section>\
            </div>\
            <div class=\"column\">\
                <section class=\"panel\"><div class=\"section-head\"><h2>Approvals</h2></div>{approvals}</section>\
                <section class=\"panel\"><div class=\"section-head\"><h2>Patches</h2></div><div class=\"patch-stack\">{patches}</div></section>\
                <section class=\"panel\"><div class=\"section-head\"><h2>Commands</h2></div>{commands}</section>\
                <section class=\"panel\"><div class=\"section-head\"><h2>Artifacts</h2></div><ul class=\"artifact-list\">{artifacts}</ul></section>\
            </div>\
        </section>",
        request = escape_html(&snapshot.summary.request),
        status_class = status_class(snapshot.summary.status),
        status = escape_html(&format!("{:?}", snapshot.summary.status).to_ascii_lowercase()),
        tokens = snapshot.summary.total_tokens,
        run_id = escape_html(run_id),
        steps = if steps.is_empty() {
            "<p class=\"muted\">No steps.</p>".to_string()
        } else {
            steps
        },
        timeline = if timeline.is_empty() {
            "<p class=\"muted\">No events.</p>".to_string()
        } else {
            timeline
        },
        approvals = if approvals.is_empty() {
            "<p class=\"muted\">No approvals.</p>".to_string()
        } else {
            approvals
        },
        patches = if patches.is_empty() {
            "<p class=\"muted\">No patches.</p>".to_string()
        } else {
            patches
        },
        commands = if commands.is_empty() {
            "<p class=\"muted\">No commands.</p>".to_string()
        } else {
            commands
        },
        artifacts = if artifacts.is_empty() {
            "<li class=\"muted\">No artifacts.</li>".to_string()
        } else {
            artifacts
        },
    );

    let body = if snapshot.summary.status.is_terminal() {
        body
    } else {
        format!(
            "{body}<script>const source=new EventSource('/api/runs/{run_id}/stream');async function syncRunPage(){{try{{const html=await fetch(window.location.href,{{headers:{{'X-Requested-With':'morecode-live'}}}}).then(r=>r.text());const doc=new DOMParser().parseFromString(html,'text/html');const nextHero=doc.getElementById('run-hero');const nextGrid=doc.getElementById('run-detail-grid');const hero=document.getElementById('run-hero');const grid=document.getElementById('run-detail-grid');if(nextHero&&hero) hero.replaceWith(nextHero);if(nextGrid&&grid) grid.replaceWith(nextGrid);}}catch(_error){{window.location.reload();}}}}source.onmessage=()=>syncRunPage();source.onerror=()=>source.close();</script>",
            body = body,
            run_id = escape_html(run_id)
        )
    };
    HttpResponse::html(layout_html(
        &format!("Run {}", snapshot.summary.run_id),
        body,
        None,
    ))
}

fn launch_response(state: &WebState, query: HashMap<String, String>) -> HttpResponse {
    let request = query
        .get("request")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    if request.is_empty() {
        return HttpResponse::redirect("/runs".to_string());
    }
    let options = WorkflowOptions {
        plan_only: query.contains_key("plan_only"),
        approval: match query.get("approval").map(|value| value.as_str()) {
            Some("deny") => ApprovalMode::Deny,
            _ => ApprovalMode::Auto,
        },
    };
    match launch_run(state.clone(), request, options) {
        Ok(run_id) => HttpResponse::redirect(format!("/runs/{run_id}")),
        Err(error) => HttpResponse::html(layout_html(
            "Launch Failed",
            format!(
                "<section class=\"panel\"><p>{}</p><a href=\"/runs\">Back</a></section>",
                escape_html(&error)
            ),
            None,
        )),
    }
}

fn api_launch_run(state: &WebState, body: &[u8]) -> HttpResponse {
    let request_text: String = match serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|value| value.get("request")?.as_str().map(ToOwned::to_owned))
    {
        Some(text) => text,
        None => {
            return HttpResponse::json(
                serde_json::json!({ "error": "missing 'request' field in JSON body" })
                    .to_string()
                    .into_bytes(),
            )
            .with_status("400 Bad Request");
        }
    };
    if request_text.trim().is_empty() {
        return HttpResponse::json(
            serde_json::json!({ "error": "request field is empty" })
                .to_string()
                .into_bytes(),
        )
        .with_status("400 Bad Request");
    }
    let options = WorkflowOptions {
        plan_only: false,
        approval: ApprovalMode::Auto,
    };
    match launch_run(state.clone(), request_text, options) {
        Ok(run_id) => HttpResponse::json(
            serde_json::json!({ "run_id": run_id, "status": "started" })
                .to_string()
                .into_bytes(),
        ),
        Err(error) => HttpResponse::json(
            serde_json::json!({ "error": error })
                .to_string()
                .into_bytes(),
        )
        .with_status("500 Internal Server Error"),
    }
}

fn api_runs_response(state: &WebState) -> HttpResponse {
    match run_store_from_state(state).list_summaries() {
        Ok(summaries) => HttpResponse::json(
            serde_json::to_vec_pretty(&summaries).unwrap_or_else(|_| b"[]".to_vec()),
        ),
        Err(error) => HttpResponse::json(
            serde_json::json!({ "error": error.to_string() })
                .to_string()
                .into_bytes(),
        ),
    }
}

fn api_run_response(state: &WebState, run_id: &str) -> HttpResponse {
    match run_store_from_state(state).load_snapshot(run_id) {
        Ok(snapshot) => HttpResponse::json(
            serde_json::to_vec_pretty(&snapshot).unwrap_or_else(|_| b"{}".to_vec()),
        ),
        Err(error) => HttpResponse::json(
            serde_json::json!({ "error": error.to_string() })
                .to_string()
                .into_bytes(),
        ),
    }
}

async fn stream_run_events(
    stream: &mut TcpStream,
    state: &WebState,
    run_id: &str,
) -> Result<(), String> {
    let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n";
    stream
        .write_all(head.as_bytes())
        .await
        .map_err(|error| error.to_string())?;
    stream.flush().await.map_err(|error| error.to_string())?;

    let store = run_store_from_state(state);
    let mut last_sequence = 0u64;
    loop {
        let snapshot = store
            .load_snapshot(run_id)
            .map_err(|error| error.to_string())?;
        let pending = snapshot
            .events
            .iter()
            .filter(|event| event.sequence > last_sequence)
            .cloned()
            .collect::<Vec<_>>();

        if pending.is_empty() {
            if stream.write_all(b": ping\n\n").await.is_err() {
                break;
            }
        } else {
            for event in pending {
                let payload = serde_json::to_string(&event).map_err(|error| error.to_string())?;
                let line = format!("event: run\ndata: {payload}\n\n");
                if stream.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                last_sequence = event.sequence;
            }
        }
        if stream.flush().await.is_err() {
            break;
        }
        if snapshot.summary.status.is_terminal() {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}

fn patch_action_response(
    state: &WebState,
    run_id: &str,
    patch_id: &str,
    action: &str,
) -> HttpResponse {
    let status = match action {
        "accept" => PatchStatus::Accepted,
        "reject" => PatchStatus::Rejected,
        _ => return HttpResponse::not_found("unknown patch action"),
    };
    let store = run_store_from_state(state);
    match store.open_recorder(run_id) {
        Ok(mut recorder) => {
            let _ = recorder.emit(RunEvent::PatchResolved {
                patch_id: patch_id.to_string(),
                hunk_id: None,
                status,
            });
            HttpResponse::redirect(format!("/runs/{run_id}"))
        }
        Err(error) => HttpResponse::not_found(&error.to_string()),
    }
}

fn approval_action_response(
    state: &WebState,
    run_id: &str,
    approval_id: &str,
    action: &str,
) -> HttpResponse {
    let (status, choice) = match action {
        "approve" => (ApprovalStatus::Approved, "approve"),
        "reject" => (ApprovalStatus::Rejected, "reject"),
        _ => return HttpResponse::not_found("unknown approval action"),
    };
    let store = run_store_from_state(state);
    match store.open_recorder(run_id) {
        Ok(mut recorder) => {
            let _ = recorder.emit(RunEvent::ApprovalResolved {
                approval_id: approval_id.to_string(),
                status,
                choice: Some(choice.to_string()),
                comment: Some("resolved from web ui".to_string()),
            });
            HttpResponse::redirect(format!("/runs/{run_id}"))
        }
        Err(error) => HttpResponse::not_found(&error.to_string()),
    }
}

fn artifact_response(state: &WebState, run_id: &str, tail: &[&str]) -> HttpResponse {
    let relative = tail.join("/");
    let path = run_store_from_state(state)
        .run_dir(run_id)
        .join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
    match std::fs::read(&path) {
        Ok(contents) => HttpResponse {
            status_line: "200 OK",
            content_type: guess_content_type(&path),
            body: contents,
            headers: Vec::new(),
        },
        Err(error) => HttpResponse::not_found(&error.to_string()),
    }
}

fn run_store_from_state(state: &WebState) -> RunStore {
    RunStore::new(&state.project_root)
}

const MAX_REQUEST_HEADER_SIZE: usize = 65536;

async fn read_request(stream: &mut TcpStream) -> Result<Option<HttpRequest>, String> {
    let mut buffer = vec![0u8; 8192];
    let mut total = 0usize;
    loop {
        let read = stream
            .read(&mut buffer[total..])
            .await
            .map_err(|error| error.to_string())?;
        if read == 0 {
            if total == 0 {
                return Ok(None);
            }
            break;
        }
        total += read;
        if total >= 4
            && buffer[..total]
                .windows(4)
                .any(|window| window == b"\r\n\r\n")
        {
            break;
        }
        if total >= MAX_REQUEST_HEADER_SIZE {
            return Err("request header too large".to_string());
        }
        if total == buffer.len() {
            let new_size = (buffer.len() * 2).min(MAX_REQUEST_HEADER_SIZE);
            buffer.resize(new_size, 0);
        }
    }

    let header_end = buffer[..total]
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap_or(total);
    let request_text = String::from_utf8_lossy(&buffer[..header_end]);
    let first_line = request_text
        .lines()
        .next()
        .ok_or_else(|| "empty request".to_string())?;
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let target = parts.next().unwrap_or("/").to_string();
    let (path, query) = split_target(&target);

    let mut body = Vec::new();
    let mut content_length = 0usize;
    for line in request_text.lines() {
        if let Some(value) = line.strip_prefix("content-length:") {
            content_length = value.trim().parse().unwrap_or(0);
        } else if let Some(value) = line.strip_prefix("Content-Length:") {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }

    let body_start = header_end + 4;
    if body_start < total {
        body.extend_from_slice(&buffer[body_start..total]);
    }
    while body.len() < content_length {
        let mut chunk = vec![0u8; 8192.min(content_length - body.len())];
        let read = stream
            .read(&mut chunk)
            .await
            .map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..read]);
    }

    Ok(Some(HttpRequest {
        method,
        path,
        query,
        body,
    }))
}

async fn write_response(
    stream: &mut TcpStream,
    response: HttpResponse,
) -> Result<(), std::io::Error> {
    let mut head = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        response.status_line,
        response.content_type,
        response.body.len()
    );
    for (key, value) in response.headers {
        head.push_str(&format!("{key}: {value}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes()).await?;
    if !response.body.is_empty() {
        stream.write_all(&response.body).await?;
    }
    stream.flush().await
}

fn split_target(target: &str) -> (String, HashMap<String, String>) {
    let mut parts = target.splitn(2, '?');
    let path = parts.next().unwrap_or("/").to_string();
    let query = parts.next().unwrap_or("");
    (path, parse_query(query))
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let mut parts = pair.splitn(2, '=');
        let key = percent_decode(parts.next().unwrap_or(""));
        let value = percent_decode(parts.next().unwrap_or(""));
        values.insert(key, value);
    }
    values
}

fn percent_decode(value: &str) -> String {
    let mut bytes = Vec::with_capacity(value.len());
    let raw = value.as_bytes();
    let mut index = 0usize;
    while index < raw.len() {
        match raw[index] {
            b'+' => {
                bytes.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < raw.len() => {
                let hex = &value[index + 1..index + 3];
                if let Ok(parsed) = u8::from_str_radix(hex, 16) {
                    bytes.push(parsed);
                    index += 3;
                } else {
                    bytes.push(raw[index]);
                    index += 1;
                }
            }
            byte => {
                bytes.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn normalize_web_approval(mode: ApprovalMode) -> ApprovalMode {
    match mode {
        ApprovalMode::Prompt => ApprovalMode::Auto,
        other => other,
    }
}

fn layout_html(title: &str, body: String, auto_refresh_secs: Option<u64>) -> String {
    let refresh = auto_refresh_secs
        .map(|seconds| format!("<meta http-equiv=\"refresh\" content=\"{seconds}\">"))
        .unwrap_or_default();
    format!(
        "<!doctype html>\
        <html lang=\"en\">\
        <head>\
            <meta charset=\"utf-8\">\
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
            {refresh}\
            <title>MoreCode — {title}</title>\
            <style>\
                :root {{\
                    --bg-root: #1e1e1e;\
                    --bg-sidebar: #252526;\
                    --bg-content: #1e1e1e;\
                    --bg-card: #2d2d30;\
                    --bg-dropdown: #3c3c3c;\
                    --bg-hover: #2a2d2e;\
                    --bg-active: #37373d;\
                    --bg-input: #3c3c3c;\
                    --bg-terminal: #0c0c0c;\
                    --text-primary: rgba(255,255,255,0.90);\
                    --text-secondary: rgba(255,255,255,0.70);\
                    --text-tertiary: rgba(255,255,255,0.50);\
                    --text-disabled: rgba(255,255,255,0.30);\
                    --accent-blue: #007acc;\
                    --accent-green: #4ec9b0;\
                    --accent-yellow: #dcdcaa;\
                    --accent-red: #f44747;\
                    --accent-purple: #c586c0;\
                    --accent-orange: #ce9178;\
                    --border-subtle: rgba(255,255,255,0.06);\
                    --border-default: rgba(255,255,255,0.08);\
                    --border-visible: rgba(255,255,255,0.10);\
                    --font-ui: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'PingFang SC', 'Microsoft YaHei', sans-serif;\
                    --font-mono: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', 'SF Mono', monospace;\
                    --radius-sm: 3px;\
                    --radius-md: 4px;\
                    --radius-lg: 6px;\
                }}\
                * {{ margin:0; padding:0; box-sizing:border-box; }}\
                body {{\
                    background: var(--bg-content);\
                    color: var(--text-primary);\
                    font-family: var(--font-ui);\
                    font-size: 13px;\
                    line-height: 1.5;\
                    -webkit-font-smoothing: antialiased;\
                }}\
                ::-webkit-scrollbar {{ width:10px; }}\
                ::-webkit-scrollbar-track {{ background:transparent; }}\
                ::-webkit-scrollbar-thumb {{ background:rgba(255,255,255,0.12); border-radius:5px; border:3px solid transparent; background-clip:padding-box; }}\
                ::-webkit-scrollbar-thumb:hover {{ background:rgba(255,255,255,0.25); border:2px solid transparent; }}\
                ::selection {{ background:#264f78; color:#fff; }}\
                :focus-visible {{ outline:1px solid var(--accent-blue); outline-offset:-1px; }}\
                a {{ color:var(--accent-blue); text-decoration:none; }}\
                a:hover {{ color:#58a6ff; }}\
                code,pre {{ font-family:var(--font-mono); font-size:12px; }}\
                code {{ background:var(--bg-card); color:#569cd6; padding:1px 4px; border-radius:var(--radius-sm); }}\
                .shell {{ max-width:1200px; margin:0 auto; padding:24px; }}\
                .header {{ background:var(--bg-card); border-bottom:1px solid var(--border-subtle); padding:8px 24px; display:flex; align-items:center; gap:12px; }}\
                .header-logo {{ font-weight:700; font-size:14px; color:var(--accent-blue); }}\
                .header-nav {{ display:flex; gap:4px; margin-left:auto; }}\
                .header-nav a {{ padding:6px 12px; border-radius:var(--radius-sm); font-size:12px; color:var(--text-secondary); transition:background 0.1s; }}\
                .header-nav a:hover,.header-nav a.active {{ background:var(--bg-active); color:var(--text-primary); text-decoration:none; }}\
                .panel {{ background:var(--bg-card); border:1px solid var(--border-subtle); border-radius:var(--radius-md); margin-bottom:16px; overflow:hidden; }}\
                .panel-header {{ display:flex; align-items:center; height:32px; padding:0 12px; font-size:11px; font-weight:600; color:var(--text-secondary); text-transform:uppercase; letter-spacing:0.5px; background:var(--bg-sidebar); border-bottom:1px solid var(--border-subtle); }}\
                .panel-body {{ padding:12px; }}\
                .muted {{ color:var(--text-tertiary); }}\
                .run-grid {{ display:grid; grid-template-columns:repeat(auto-fill, minmax(300px,1fr)); gap:12px; }}\
                .run-card {{ display:grid; gap:8px; padding:14px; border-radius:var(--radius-md); border:1px solid var(--border-subtle); background:var(--bg-card); cursor:pointer; transition:border-color 0.15s; color:inherit; }}\
                .run-card:hover {{ border-color:var(--accent-blue); text-decoration:none; }}\
                .run-card h3 {{ margin:0; font-size:14px; font-weight:500; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }}\
                .run-card__meta {{ display:flex; justify-content:space-between; align-items:center; gap:8px; font-size:11px; color:var(--text-tertiary); }}\
                .chip {{ display:inline-flex; align-items:center; gap:4px; padding:1px 6px; border-radius:var(--radius-sm); font-size:11px; font-weight:500; }}\
                .chip.running {{ background:rgba(86,156,214,0.15); color:#569cd6; }}\
                .chip.succeeded,.chip.accepted,.chip.approved,.chip.completed,.step.done {{ background:rgba(78,201,176,0.15); color:#4ec9b0; }}\
                .chip.failed,.chip.rejected,.chip.error,.step.failed {{ background:rgba(244,71,71,0.15); color:#f44747; }}\
                .chip.pending,.chip.queued {{ background:rgba(255,255,255,0.06); color:var(--text-tertiary); }}\
                .chip.warn {{ background:rgba(220,220,170,0.15); color:#dcdcaa; }}\
                .chip.skipped {{ background:rgba(255,255,255,0.04); color:var(--text-disabled); }}\
                .chip.accent {{ background:rgba(0,122,204,0.15); color:var(--accent-blue); }}\
                .step {{ display:flex; align-items:center; padding:6px 12px; gap:10px; font-size:12px; border-bottom:1px solid var(--border-subtle); }}\
                .step:last-child {{ border-bottom:none; }}\
                .step-icon {{ width:18px; font-family:var(--font-mono); text-align:center; flex-shrink:0; }}\
                .step-body {{ flex:1; min-width:0; }}\
                .step-title {{ color:var(--text-primary); }}\
                .step-summary {{ color:var(--text-tertiary); font-size:11px; margin-top:1px; }}\
                .step-tokens {{ font-family:var(--font-mono); font-size:11px; color:var(--text-tertiary); flex-shrink:0; }}\
                .artifact {{ background:var(--bg-card); border-radius:var(--radius-sm); margin-top:6px; padding:8px 10px; }}\
                .artifact pre {{ white-space:pre-wrap; color:var(--text-secondary); font-size:11px; line-height:1.5; }}\
                .patch-block {{ background:var(--bg-content); border:1px solid var(--border-subtle); border-radius:var(--radius-sm); padding:10px; margin:6px 0; }}\
                .patch-block pre {{ white-space:pre-wrap; font-family:var(--font-mono); font-size:11px; line-height:1.5; color:var(--text-primary); }}\
                .patch-block .add {{ color:var(--accent-green); }}\
                .patch-block .rem {{ color:var(--accent-red); }}\
                .patch-block .ctx {{ color:var(--text-tertiary); }}\
                .launch-form {{ display:flex; flex-direction:column; gap:10px; }}\
                .launch-form textarea {{ width:100%; min-height:100px; resize:vertical; padding:10px 12px; border-radius:var(--radius-sm); border:1px solid var(--border-default); font:inherit; color:var(--text-primary); background:var(--bg-input); }}\
                .launch-form textarea:focus {{ outline:none; border-color:var(--accent-blue); }}\
                .launch-form button {{ align-self:flex-start; padding:8px 16px; border-radius:var(--radius-sm); border:none; background:var(--accent-blue); color:#fff; font-size:13px; cursor:pointer; }}\
                .launch-form button:hover {{ background:#1a8ad4; }}\
                .back-link {{ display:inline-flex; align-items:center; gap:4px; font-size:12px; color:var(--text-secondary); padding:4px 0; margin-bottom:12px; }}\
                .back-link:hover {{ color:var(--text-primary); text-decoration:none; }}\
                .token-bar {{ display:flex; gap:12px; flex-wrap:wrap; }}\
                .token-card {{ text-align:center; background:var(--bg-card); border:1px solid var(--border-subtle); border-radius:var(--radius-md); padding:14px 18px; }}\
                .token-card .token-val {{ font-family:var(--font-mono); font-size:20px; font-weight:600; color:var(--accent-blue); }}\
                .token-card .token-label {{ font-size:11px; color:var(--text-tertiary); text-transform:uppercase; margin-top:2px; }}\
                .empty-state {{ text-align:center; color:var(--text-tertiary); padding:40px; }}\
                h1,h2,h3 {{ font-weight:600; }}\
                h1 {{ font-size:16px; }}\
                h2 {{ font-size:14px; }}\
                h3 {{ font-size:13px; }}\
                .banner {{ padding:10px 14px; border-radius:var(--radius-sm); background:rgba(86,156,214,0.12); border:1px solid rgba(86,156,214,0.22); color:var(--text-primary); font-size:12px; }}\
                .detail-grid {{ display:grid; grid-template-columns:minmax(0,0.95fr) minmax(400px,1.05fr); gap:16px; }}\
                .column {{ display:grid; gap:16px; align-content:start; }}\
                .step-list {{ display:flex; flex-direction:column; }}\
                .timeline {{ display:flex; flex-direction:column; gap:8px; }}\
                .timeline-item {{ padding:8px 12px; border-left:2px solid var(--accent-blue); background:var(--bg-card); }}\
                .timeline-item__meta {{ display:flex; justify-content:space-between; gap:8px; font-family:var(--font-mono); font-size:11px; color:var(--text-tertiary); }}\
                .patch-stack,.mini-card-stack {{ display:flex; flex-direction:column; gap:12px; }}\
                .patch-card pre,.mini-card pre {{ overflow:auto; padding:10px; border-radius:var(--radius-sm); background:var(--bg-content); color:var(--text-primary); font-family:var(--font-mono); font-size:11px; line-height:1.5; border:1px solid var(--border-subtle); }}\
                .mini-card {{ padding:10px 0; border-bottom:1px solid var(--border-subtle); }}\
                .action-row {{ display:flex; gap:8px; margin:8px 0; flex-wrap:wrap; }}\
                .action-row a {{ padding:6px 10px; border-radius:var(--radius-sm); border:1px solid var(--border-visible); background:var(--bg-card); font-size:11px; color:var(--text-secondary); }}\
                .action-row a:hover {{ border-color:var(--accent-blue); background:var(--bg-hover); color:var(--text-primary); text-decoration:none; }}\
                textarea:focus,select:focus,.action-row a:hover {{ outline:none; border-color:var(--accent-blue); }}\
                @media (max-width:980px) {{ .detail-grid {{ grid-template-columns:1fr; }} .shell {{ padding:16px; }} }}\
            </style>\
        </head>\
        <body>\
            <main class=\"shell\">{body}</main>\
        </body>\
        </html>",
        refresh = refresh,
        title = escape_html(title),
        body = body
    )
}

fn render_event_item(event: &RunEventEnvelope) -> String {
    format!(
        "<article class=\"timeline-item\">\
            <div class=\"timeline-item__meta\">\
                <span>#{sequence}</span>\
                <span>{time}</span>\
            </div>\
            <div>{summary}</div>\
        </article>",
        sequence = event.sequence,
        time = escape_html(&event.at.format("%H:%M:%S").to_string()),
        summary = escape_html(&event_summary(&event.event)),
    )
}

fn event_summary(event: &RunEvent) -> String {
    match event {
        RunEvent::RunStarted { request, .. } => format!("Run started: {request}"),
        RunEvent::StepStarted { step } => format!("Step started: {}", step.title),
        RunEvent::StepFinished {
            step_id, summary, ..
        } => format!(
            "Step finished: {step_id} {}",
            summary.clone().unwrap_or_default()
        ),
        RunEvent::Message { message, .. } => message.clone(),
        RunEvent::ApprovalRequested { approval } => {
            format!("Approval requested: {}", approval.title)
        }
        RunEvent::ApprovalResolved {
            approval_id,
            status,
            ..
        } => format!("Approval {approval_id}: {status:?}"),
        RunEvent::PatchProposed { patch } => format!("Patch proposed: {}", patch.file_path),
        RunEvent::PatchResolved {
            patch_id, status, ..
        } => format!("Patch {patch_id}: {status:?}"),
        RunEvent::ArtifactWritten { artifact } => format!("Artifact written: {}", artifact.title),
        RunEvent::CommandStarted { command } => format!("Command started: {}", command.command),
        RunEvent::CommandOutput { command_id, .. } => format!("Command output: {command_id}"),
        RunEvent::CommandFinished {
            command_id, status, ..
        } => format!("Command {command_id}: {status:?}"),
        RunEvent::RunFinished { summary, .. } => summary
            .clone()
            .unwrap_or_else(|| "Run finished".to_string()),
        RunEvent::Error { message, .. } => format!("Error: {message}"),
    }
}

fn status_class(status: RunStatus) -> &'static str {
    match status {
        RunStatus::Queued => "pending",
        RunStatus::Running => "running",
        RunStatus::WaitingApproval => "waitingapproval",
        RunStatus::Failed => "failed",
        RunStatus::Succeeded => "succeeded",
        RunStatus::Canceled => "failed",
    }
}

fn status_class_from_step(status: mc_core::StepStatus) -> &'static str {
    match status {
        mc_core::StepStatus::Pending => "pending",
        mc_core::StepStatus::Running => "running",
        mc_core::StepStatus::Done => "done",
        mc_core::StepStatus::Skipped => "accepted",
        mc_core::StepStatus::Failed => "failed",
    }
}

fn patch_status_class(status: PatchStatus) -> &'static str {
    match status {
        PatchStatus::Pending => "pending",
        PatchStatus::Accepted => "accepted",
        PatchStatus::Rejected => "rejected",
    }
}

fn approval_status_class(status: ApprovalStatus) -> &'static str {
    match status {
        ApprovalStatus::Pending => "pending",
        ApprovalStatus::Approved => "approved",
        ApprovalStatus::Rejected => "rejected",
    }
}

fn command_status_class(status: mc_core::CommandStatus) -> &'static str {
    match status {
        mc_core::CommandStatus::Running => "running",
        mc_core::CommandStatus::Completed => "completed",
        mc_core::CommandStatus::Failed => "failed",
        mc_core::CommandStatus::Skipped => "accepted",
    }
}

fn guess_content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "jsonl" => "application/x-ndjson; charset=utf-8",
        "md" | "patch" | "log" | "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_path_segment(value: &str) -> String {
    value.replace('\\', "/")
}
