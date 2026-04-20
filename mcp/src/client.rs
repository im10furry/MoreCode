use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Duration;

use http::{HeaderName, HeaderValue};
use mc_sandbox::{Capability, CapabilityDeclaration, PermissionLevel};
use mc_tool::{PermissionScope, Tool, ToolCategory, ToolRegistry, ToolResult, ToolResultStatus};
use rmcp::model::{CallToolRequestParams, ClientInfo, Content, ResourceContents, Tool as McpTool};
use rmcp::service::{Peer, RoleClient, RunningService};
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::{ConfigureCommandExt, StreamableHttpClientTransport, TokioChildProcess};
use rmcp::ServiceExt;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{McpClientConfig, McpClientTransportConfig, McpError};

type RunningClient = RunningService<RoleClient, ClientInfo>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedMcpTool {
    pub registry_name: String,
    pub remote_name: String,
    pub description: String,
}

#[derive(Clone)]
struct RemoteToolSpec {
    imported: ImportedMcpTool,
    input_schema: Value,
    read_only: bool,
}

impl RemoteToolSpec {
    fn from_remote(
        server_name: &str,
        tool_prefix: &str,
        tool: McpTool,
        respect_remote_tool_hints: bool,
    ) -> Self {
        let imported = ImportedMcpTool {
            registry_name: format!(
                "{}_{}",
                sanitize_identifier(tool_prefix, "mcp"),
                sanitize_identifier(tool.name.as_ref(), "tool")
            ),
            remote_name: tool.name.into_owned(),
            description: tool.description.as_deref().unwrap_or_default().to_string(),
        };
        let read_only = if respect_remote_tool_hints {
            tool.annotations
                .as_ref()
                .and_then(|annotations| annotations.read_only_hint)
                .unwrap_or(false)
                && !tool
                    .annotations
                    .as_ref()
                    .and_then(|annotations| annotations.destructive_hint)
                    .unwrap_or(false)
        } else {
            false
        };
        let description = if imported.description.is_empty() {
            format!(
                "Remote MCP tool `{}` from server `{server_name}`",
                imported.remote_name
            )
        } else {
            imported.description.clone()
        };

        Self {
            imported: ImportedMcpTool {
                description,
                ..imported
            },
            input_schema: Value::Object(tool.input_schema.as_ref().clone()),
            read_only,
        }
    }
}

struct McpClientConnection {
    alias: String,
    peer: Peer<RoleClient>,
    running: Mutex<Option<RunningClient>>,
    request_timeout: Option<Duration>,
}

pub struct McpClientSession {
    alias: String,
    tool_prefix: String,
    connection: Arc<McpClientConnection>,
    tools: Vec<RemoteToolSpec>,
}

impl McpClientSession {
    pub async fn connect(config: McpClientConfig) -> Result<Self, McpError> {
        let alias = config.server_name.clone();
        let tool_prefix = config.tool_prefix.clone();
        let client_info = config.client_info();
        let request_timeout = config.request_timeout();
        let respect_remote_tool_hints = config.respect_remote_tool_hints;

        match config.transport {
            McpClientTransportConfig::Stdio {
                command,
                args,
                env,
                cwd,
            } => {
                let transport = TokioChildProcess::new(
                    tokio::process::Command::new(command).configure(move |process| {
                        process.args(args);
                        process.envs(env);
                        if let Some(cwd) = cwd {
                            process.current_dir(cwd);
                        }
                    }),
                )?;

                Self::connect_with_transport(
                    alias,
                    tool_prefix,
                    client_info,
                    transport,
                    request_timeout,
                    respect_remote_tool_hints,
                )
                .await
            }
            McpClientTransportConfig::Http { url, headers } => {
                let transport = StreamableHttpClientTransport::from_config(
                    build_http_client_config(url, headers)?,
                );

                Self::connect_with_transport(
                    alias,
                    tool_prefix,
                    client_info,
                    transport,
                    request_timeout,
                    respect_remote_tool_hints,
                )
                .await
            }
            McpClientTransportConfig::UnixSocket {
                socket_path,
                uri,
                headers,
            } => {
                #[cfg(unix)]
                {
                    let socket_path = socket_path.to_string_lossy().to_string();
                    let transport = StreamableHttpClientTransport::from_unix_socket_with_config(
                        &socket_path,
                        build_http_client_config(uri, headers)?,
                    );

                    Self::connect_with_transport(
                        alias,
                        tool_prefix,
                        client_info,
                        transport,
                        request_timeout,
                        respect_remote_tool_hints,
                    )
                    .await
                }
                #[cfg(not(unix))]
                {
                    let _ = (socket_path, uri, headers);
                    Err(McpError::UnixSocketUnsupported)
                }
            }
        }
    }

    pub async fn connect_with_transport<T, E, A>(
        alias: impl Into<String>,
        tool_prefix: Option<String>,
        client_info: ClientInfo,
        transport: T,
        request_timeout: Option<Duration>,
        respect_remote_tool_hints: bool,
    ) -> Result<Self, McpError>
    where
        T: rmcp::transport::IntoTransport<RoleClient, E, A>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let alias = alias.into();
        let tool_prefix = tool_prefix.unwrap_or_else(|| format!("mcp_{alias}"));
        let running =
            client_info
                .serve(transport)
                .await
                .map_err(|error| McpError::ClientInitialization {
                    name: alias.clone(),
                    message: error.to_string(),
                })?;
        let remote_tools = running
            .list_all_tools()
            .await
            .map_err(|error| McpError::RequestFailed(error.to_string()))?;
        let tools = remote_tools
            .into_iter()
            .map(|tool| {
                RemoteToolSpec::from_remote(&alias, &tool_prefix, tool, respect_remote_tool_hints)
            })
            .collect::<Vec<_>>();
        let connection = Arc::new(McpClientConnection {
            alias: alias.clone(),
            peer: running.peer().clone(),
            running: Mutex::new(Some(running)),
            request_timeout,
        });

        Ok(Self {
            alias,
            tool_prefix,
            connection,
            tools,
        })
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub fn tool_prefix(&self) -> &str {
        &self.tool_prefix
    }

    pub fn imported_tools(&self) -> Vec<ImportedMcpTool> {
        self.tools
            .iter()
            .map(|tool| tool.imported.clone())
            .collect()
    }

    pub async fn register_tools(&self, registry: &ToolRegistry) -> Result<Vec<String>, McpError> {
        let mut registered = Vec::with_capacity(self.tools.len());

        for tool in &self.tools {
            if registry.get(&tool.imported.registry_name).await.is_some() {
                return Err(McpError::ToolNameConflict {
                    name: tool.imported.registry_name.clone(),
                });
            }

            registry
                .register_arc(Arc::new(RemoteMcpTool::new(
                    tool.clone(),
                    Arc::clone(&self.connection),
                )))
                .await;
            registered.push(tool.imported.registry_name.clone());
        }

        Ok(registered)
    }

    pub async fn shutdown(&self) -> Result<(), McpError> {
        let mut running = self.connection.running.lock().await;
        if let Some(mut running) = running.take() {
            running
                .close()
                .await
                .map_err(|error| McpError::ConnectionClose(error.to_string()))?;
        }
        Ok(())
    }
}

pub struct RemoteMcpTool {
    spec: RemoteToolSpec,
    connection: Arc<McpClientConnection>,
    capability: CapabilityDeclaration,
}

impl RemoteMcpTool {
    fn new(spec: RemoteToolSpec, connection: Arc<McpClientConnection>) -> Self {
        let capability = CapabilityDeclaration::new(
            spec.imported.registry_name.clone(),
            spec.imported.description.clone(),
            PermissionLevel::Elevated,
            vec![Capability::NetworkAccess {
                pattern: format!(
                    "mcp://{}/{}",
                    sanitize_identifier(&connection.alias, "server"),
                    sanitize_identifier(&spec.imported.remote_name, "tool")
                ),
            }],
        );

        Self {
            spec,
            connection,
            capability,
        }
    }

    async fn execute_remote_call(&self, params: Value) -> ToolResult {
        let start = std::time::Instant::now();
        let arguments = match params {
            Value::Null => None,
            Value::Object(arguments) => Some(arguments),
            _ => {
                return ToolResult::error("remote MCP tools require JSON object arguments")
                    .with_duration(start.elapsed())
            }
        };

        let request = match arguments {
            Some(arguments) => CallToolRequestParams::new(self.spec.imported.remote_name.clone())
                .with_arguments(arguments),
            None => CallToolRequestParams::new(self.spec.imported.remote_name.clone()),
        };

        let response = if let Some(timeout) = self.connection.request_timeout {
            match tokio::time::timeout(timeout, self.connection.peer.call_tool(request)).await {
                Ok(response) => response,
                Err(_) => {
                    return ToolResult::error(format!(
                        "remote MCP tool timed out after {}ms",
                        timeout.as_millis()
                    ))
                    .with_duration(start.elapsed())
                }
            }
        } else {
            self.connection.peer.call_tool(request).await
        };

        match response {
            Ok(result) => tool_result_from_remote(result).with_duration(start.elapsed()),
            Err(error) => ToolResult::error(error.to_string()).with_duration(start.elapsed()),
        }
    }
}

impl Tool for RemoteMcpTool {
    fn name(&self) -> &str {
        &self.spec.imported.registry_name
    }

    fn description(&self) -> &str {
        &self.spec.imported.description
    }

    fn execute(
        &self,
        params: Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move { self.execute_remote_call(params).await })
    }

    fn required_parameters(&self) -> Value {
        self.spec.input_schema.clone()
    }

    fn is_read_only(&self) -> bool {
        self.spec.read_only
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Extended
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Elevated
    }

    fn permission_scope(&self) -> PermissionScope {
        PermissionScope::External
    }

    fn capability(&self) -> CapabilityDeclaration {
        self.capability.clone()
    }
}

fn build_http_client_config(
    url: String,
    headers: BTreeMap<String, String>,
) -> Result<StreamableHttpClientTransportConfig, McpError> {
    Ok(StreamableHttpClientTransportConfig::with_uri(url).custom_headers(build_headers(headers)?))
}

fn build_headers(
    headers: BTreeMap<String, String>,
) -> Result<HashMap<HeaderName, HeaderValue>, McpError> {
    headers
        .into_iter()
        .map(|(name, value)| {
            let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
                McpError::InvalidHeader {
                    name: name.clone(),
                    message: error.to_string(),
                }
            })?;
            let header_value =
                HeaderValue::from_str(&value).map_err(|error| McpError::InvalidHeader {
                    name: name.clone(),
                    message: error.to_string(),
                })?;
            Ok((header_name, header_value))
        })
        .collect()
}

fn tool_result_from_remote(result: rmcp::model::CallToolResult) -> ToolResult {
    let mut content = collapse_content(&result.content);
    let data = result.structured_content;
    if content.is_empty() {
        if let Some(data) = &data {
            content = data.to_string();
        }
    }

    ToolResult {
        status: if result.is_error.unwrap_or(false) {
            ToolResultStatus::Error
        } else {
            ToolResultStatus::Success
        },
        content,
        data,
        metadata: HashMap::new(),
        duration_ms: 0,
    }
}

fn collapse_content(content: &[Content]) -> String {
    content
        .iter()
        .map(|item| {
            if let Some(text) = item.as_text() {
                return text.text.clone();
            }
            if let Some(resource) = item.as_resource() {
                return match &resource.resource {
                    ResourceContents::TextResourceContents { text, .. } => text.clone(),
                    _ => serde_json::to_string(item).unwrap_or_default(),
                };
            }
            serde_json::to_string(item).unwrap_or_default()
        })
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn sanitize_identifier(value: &str, fallback: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut last_was_separator = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            result.push('_');
            last_was_separator = true;
        }
    }

    let trimmed = result.trim_matches('_').to_string();
    if trimmed.is_empty() {
        fallback.to_string()
    } else if trimmed
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_digit())
    {
        format!("{fallback}_{trimmed}")
    } else {
        trimmed
    }
}
