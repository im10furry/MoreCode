use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http::{Request, Response, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use mc_tool::{PermissionScope, Tool, ToolRegistry, ToolResult, VisibilityLayer};
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ErrorData, ListToolsResult, ServerInfo,
    Tool as McpTool, ToolAnnotations,
};
use rmcp::service::{RoleServer, RunningService};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{stdio, StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{ServerHandler, ServiceExt};
use serde_json::Value;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{McpError, McpServerConfig, McpServerEndpoint, McpServerTransportConfig};

type RunningServer = RunningService<RoleServer, ToolRegistryServerHandler>;
type HttpMcpService = StreamableHttpService<ToolRegistryServerHandler, LocalSessionManager>;

#[derive(Clone)]
struct ToolRegistryServerHandler {
    registry: Arc<ToolRegistry>,
    visibility: VisibilityLayer,
    info: ServerInfo,
}

impl ToolRegistryServerHandler {
    fn new(registry: Arc<ToolRegistry>, visibility: VisibilityLayer, info: ServerInfo) -> Self {
        Self {
            registry,
            visibility,
            info,
        }
    }
}

impl ServerHandler for ToolRegistryServerHandler {
    fn get_info(&self) -> ServerInfo {
        self.info.clone()
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let tools = self
            .registry
            .list_tools_with_deferred(self.visibility)
            .await;
        let tools = tools
            .into_iter()
            .map(|tool| tool_to_mcp(tool.as_ref()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;

        Ok(ListToolsResult {
            tools,
            ..Default::default()
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let caller = context
            .peer
            .peer_info()
            .map(|peer| peer.client_info.name.clone())
            .unwrap_or_else(|| "mcp-client".to_string());
        let params = request.arguments.map(Value::Object).unwrap_or(Value::Null);
        let result = self
            .registry
            .execute_tool(&caller, request.name.as_ref(), params)
            .await;
        Ok(tool_result_to_mcp(result))
    }
}

pub struct ToolRegistryMcpServer {
    registry: Arc<ToolRegistry>,
    config: McpServerConfig,
}

impl ToolRegistryMcpServer {
    pub fn new(registry: Arc<ToolRegistry>, config: McpServerConfig) -> Self {
        Self { registry, config }
    }

    pub async fn serve(&self) -> Result<McpServerHandle, McpError> {
        match &self.config.transport {
            McpServerTransportConfig::Stdio => self.serve_over_transport(stdio()).await,
            McpServerTransportConfig::Http {
                bind_address,
                mount_path,
                stateful_mode,
                json_response,
                sse_keep_alive_secs,
                allowed_hosts,
            } => {
                let listener =
                    tokio::net::TcpListener::bind(bind_address)
                        .await
                        .map_err(|error| McpError::HttpBind {
                            address: bind_address.to_string(),
                            message: error.to_string(),
                        })?;
                let local_addr = listener.local_addr().map_err(McpError::Io)?;
                let mount_path = normalize_mount_path(mount_path);
                let endpoint = McpServerEndpoint::Http {
                    uri: format!("http://{local_addr}{mount_path}"),
                };
                let cancellation_token = CancellationToken::new();
                let service = self.build_http_service(
                    cancellation_token.child_token(),
                    *stateful_mode,
                    *json_response,
                    *sse_keep_alive_secs,
                    allowed_hosts,
                );
                let join_handle = tokio::spawn(run_tcp_server(
                    listener,
                    service,
                    mount_path,
                    cancellation_token.clone(),
                ));

                Ok(McpServerHandle::background(
                    endpoint,
                    join_handle,
                    cancellation_token,
                    None,
                ))
            }
            McpServerTransportConfig::UnixSocket {
                socket_path,
                mount_path,
                stateful_mode,
                json_response,
                sse_keep_alive_secs,
                allowed_hosts,
            } => {
                #[cfg(unix)]
                {
                    if socket_path.exists() {
                        std::fs::remove_file(socket_path)?;
                    }

                    let listener =
                        tokio::net::UnixListener::bind(socket_path).map_err(|error| {
                            McpError::UnixBind {
                                path: socket_path.clone(),
                                message: error.to_string(),
                            }
                        })?;
                    let mount_path = normalize_mount_path(mount_path);
                    let cancellation_token = CancellationToken::new();
                    let service = self.build_http_service(
                        cancellation_token.child_token(),
                        *stateful_mode,
                        *json_response,
                        *sse_keep_alive_secs,
                        allowed_hosts,
                    );
                    let join_handle = tokio::spawn(run_unix_server(
                        listener,
                        service,
                        mount_path.clone(),
                        cancellation_token.clone(),
                    ));

                    Ok(McpServerHandle::background(
                        McpServerEndpoint::UnixSocket {
                            socket_path: socket_path.clone(),
                            uri: format!("http://localhost{mount_path}"),
                        },
                        join_handle,
                        cancellation_token,
                        Some(socket_path.clone()),
                    ))
                }
                #[cfg(not(unix))]
                {
                    let _ = (
                        socket_path,
                        mount_path,
                        stateful_mode,
                        json_response,
                        sse_keep_alive_secs,
                        allowed_hosts,
                    );
                    Err(McpError::UnixSocketUnsupported)
                }
            }
        }
    }

    pub async fn serve_over_transport<T, E, A>(
        &self,
        transport: T,
    ) -> Result<McpServerHandle, McpError>
    where
        T: rmcp::transport::IntoTransport<RoleServer, E, A>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let running = self
            .handler()
            .serve(transport)
            .await
            .map_err(|error| McpError::ServerInitialization(error.to_string()))?;

        Ok(McpServerHandle::stdio(running))
    }

    fn handler(&self) -> ToolRegistryServerHandler {
        ToolRegistryServerHandler::new(
            Arc::clone(&self.registry),
            self.config.visibility,
            self.config.server_info(),
        )
    }

    fn build_http_service(
        &self,
        cancellation_token: CancellationToken,
        stateful_mode: bool,
        json_response: bool,
        sse_keep_alive_secs: Option<u64>,
        allowed_hosts: &[String],
    ) -> HttpMcpService {
        let mut config = StreamableHttpServerConfig::default()
            .with_stateful_mode(stateful_mode)
            .with_json_response(json_response)
            .with_allowed_hosts(allowed_hosts.iter().cloned())
            .with_cancellation_token(cancellation_token);

        config = config.with_sse_keep_alive(sse_keep_alive_secs.map(Duration::from_secs));

        StreamableHttpService::new(
            {
                let handler = self.handler();
                move || Ok(handler.clone())
            },
            Default::default(),
            config,
        )
    }
}

pub struct McpServerHandle {
    endpoint: McpServerEndpoint,
    runtime: ServerRuntime,
    cancellation_token: CancellationToken,
    cleanup_socket: Option<PathBuf>,
}

enum ServerRuntime {
    Stdio(Option<RunningServer>),
    Background(Option<JoinHandle<Result<(), McpError>>>),
}

impl McpServerHandle {
    fn stdio(running: RunningServer) -> Self {
        Self {
            endpoint: McpServerEndpoint::Stdio,
            runtime: ServerRuntime::Stdio(Some(running)),
            cancellation_token: CancellationToken::new(),
            cleanup_socket: None,
        }
    }

    fn background(
        endpoint: McpServerEndpoint,
        join_handle: JoinHandle<Result<(), McpError>>,
        cancellation_token: CancellationToken,
        cleanup_socket: Option<PathBuf>,
    ) -> Self {
        Self {
            endpoint,
            runtime: ServerRuntime::Background(Some(join_handle)),
            cancellation_token,
            cleanup_socket,
        }
    }

    pub fn endpoint(&self) -> &McpServerEndpoint {
        &self.endpoint
    }

    pub async fn shutdown(&mut self) -> Result<(), McpError> {
        match &mut self.runtime {
            ServerRuntime::Stdio(running) => {
                if let Some(mut running) = running.take() {
                    running
                        .close()
                        .await
                        .map_err(|error| McpError::ConnectionClose(error.to_string()))?;
                }
            }
            ServerRuntime::Background(handle) => {
                self.cancellation_token.cancel();
                if let Some(handle) = handle.take() {
                    let result = handle
                        .await
                        .map_err(|error| McpError::BackgroundTask(error.to_string()))?;
                    result?;
                }
            }
        }

        if let Some(socket_path) = &self.cleanup_socket {
            let _ = tokio::fs::remove_file(socket_path).await;
        }

        Ok(())
    }
}

async fn run_tcp_server(
    listener: tokio::net::TcpListener,
    service: HttpMcpService,
    mount_path: String,
    cancellation_token: CancellationToken,
) -> Result<(), McpError> {
    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => return Ok(()),
            accept = listener.accept() => {
                let (stream, _) = accept.map_err(McpError::Io)?;
                let service = service.clone();
                let mount_path = mount_path.clone();
                tokio::spawn(async move {
                    let _ = serve_http_connection(stream, service, mount_path).await;
                });
            }
        }
    }
}

#[cfg(unix)]
async fn run_unix_server(
    listener: tokio::net::UnixListener,
    service: HttpMcpService,
    mount_path: String,
    cancellation_token: CancellationToken,
) -> Result<(), McpError> {
    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => return Ok(()),
            accept = listener.accept() => {
                let (stream, _) = accept.map_err(McpError::Io)?;
                let service = service.clone();
                let mount_path = mount_path.clone();
                tokio::spawn(async move {
                    let _ = serve_http_connection(stream, service, mount_path).await;
                });
            }
        }
    }
}

async fn serve_http_connection<IO>(
    io: IO,
    service: HttpMcpService,
    mount_path: String,
) -> Result<(), McpError>
where
    IO: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let mount_path = Arc::<str>::from(mount_path);
    let hyper_service = service_fn(move |request: Request<Incoming>| {
        let service = service.clone();
        let mount_path = Arc::clone(&mount_path);
        async move {
            if request.uri().path() != mount_path.as_ref() {
                return Ok::<_, Infallible>(not_found_response());
            }

            Ok::<_, Infallible>(service.handle(request).await)
        }
    });

    hyper::server::conn::http1::Builder::new()
        .serve_connection(TokioIo::new(io), hyper_service)
        .await
        .map_err(|error| McpError::HttpServe(error.to_string()))
}

fn not_found_response() -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from("Not Found")).boxed())
        .expect("404 response should be valid")
}

fn normalize_mount_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn tool_to_mcp(tool: &dyn Tool) -> Result<McpTool, McpError> {
    let definition = tool.definition();
    let Value::Object(schema) = definition.parameters else {
        return Err(McpError::ToolSchemaNotObject {
            tool_name: definition.name,
        });
    };

    let annotations = ToolAnnotations::new()
        .read_only(tool.is_read_only())
        .destructive(!tool.is_read_only())
        .idempotent(tool.is_read_only())
        .open_world(matches!(
            tool.permission_scope(),
            PermissionScope::Search | PermissionScope::Process | PermissionScope::External
        ));

    Ok(
        McpTool::new(definition.name, definition.description, Arc::new(schema))
            .with_annotations(annotations),
    )
}

fn tool_result_to_mcp(result: ToolResult) -> CallToolResult {
    let content = if result.content.is_empty() {
        Vec::new()
    } else {
        vec![Content::text(result.content)]
    };
    let mut call_result = if matches!(result.status, mc_tool::ToolResultStatus::Error) {
        CallToolResult::error(content)
    } else {
        CallToolResult::success(content)
    };
    call_result.structured_content = result.data;
    call_result
}
