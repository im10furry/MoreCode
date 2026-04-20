use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use mc_mcp::{
    McpClientConfig, McpClientSession, McpClientTransportConfig, McpServerConfig,
    McpServerEndpoint, McpServerTransportConfig, ToolRegistryMcpServer,
};
use mc_sandbox::{Capability, CapabilityDeclaration, PermissionLevel};
use mc_tool::{PermissionScope, Tool, ToolCategory, ToolRegistry, ToolResult, ToolResultStatus};
use rmcp::model::ClientInfo;
use serde_json::json;
#[cfg(unix)]
use tempfile::tempdir;

struct EchoTool;

impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes the incoming message."
    }

    fn execute(
        &self,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let message = params
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            ToolResult::success_with_data(
                format!("echo: {message}"),
                json!({
                    "echo": message,
                }),
            )
        })
    }

    fn required_parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to echo"
                }
            },
            "required": ["message"]
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Core
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Public
    }

    fn permission_scope(&self) -> PermissionScope {
        PermissionScope::Workspace
    }

    fn capability(&self) -> CapabilityDeclaration {
        CapabilityDeclaration::new(
            "echo",
            "Echo tool",
            PermissionLevel::Public,
            vec![Capability::ReadFile {
                pattern: "**".to_string(),
            }],
        )
    }
}

async fn build_server_registry() -> Arc<ToolRegistry> {
    let registry = Arc::new(ToolRegistry::new());
    registry.register(EchoTool).await;
    registry
}

#[tokio::test]
async fn in_memory_transport_roundtrip_registers_remote_tools() {
    let server = ToolRegistryMcpServer::new(
        build_server_registry().await,
        McpServerConfig::new(McpServerTransportConfig::Stdio),
    );
    let (client_io, server_io) = tokio::io::duplex(8 * 1024);
    let server_task = tokio::spawn(async move { server.serve_over_transport(server_io).await });
    let client = McpClientSession::connect_with_transport(
        "loopback",
        Some("mcp_loopback".to_string()),
        ClientInfo::default(),
        client_io,
        Some(Duration::from_secs(5)),
        true,
    )
    .await
    .expect("client should connect");
    let mut server_handle = server_task
        .await
        .expect("server task should complete")
        .expect("server should start");

    let local_registry = ToolRegistry::new();
    client
        .register_tools(&local_registry)
        .await
        .expect("remote tools should register");

    let imported = client.imported_tools();
    assert_eq!(imported.len(), 1);

    let result = local_registry
        .execute_tool(
            "tester",
            &imported[0].registry_name,
            json!({ "message": "hello" }),
        )
        .await;

    assert_eq!(result.status, ToolResultStatus::Success);
    assert_eq!(result.data.expect("structured data")["echo"], "hello");

    client.shutdown().await.expect("client shutdown");
    server_handle.shutdown().await.expect("server shutdown");
}

#[tokio::test]
async fn http_transport_roundtrip_registers_remote_tools() {
    let server = ToolRegistryMcpServer::new(
        build_server_registry().await,
        McpServerConfig::new(McpServerTransportConfig::Http {
            bind_address: "127.0.0.1:0".parse().expect("socket address"),
            mount_path: "/mcp".to_string(),
            stateful_mode: false,
            json_response: true,
            sse_keep_alive_secs: Some(1),
            allowed_hosts: vec!["localhost".to_string(), "127.0.0.1".to_string()],
        }),
    );
    let mut server_handle = server.serve().await.expect("server should start");
    let endpoint = match server_handle.endpoint() {
        McpServerEndpoint::Http { uri } => uri.clone(),
        other => panic!("expected HTTP endpoint, got {other:?}"),
    };

    let client = McpClientSession::connect(McpClientConfig::new(
        "http",
        McpClientTransportConfig::Http {
            url: endpoint,
            headers: BTreeMap::new(),
        },
    ))
    .await
    .expect("client should connect");

    let local_registry = ToolRegistry::new();
    client
        .register_tools(&local_registry)
        .await
        .expect("remote tools should register");

    let imported = client.imported_tools();
    let result = local_registry
        .execute_tool(
            "tester",
            &imported[0].registry_name,
            json!({ "message": "from-http" }),
        )
        .await;

    assert_eq!(result.status, ToolResultStatus::Success);
    assert_eq!(result.data.expect("structured data")["echo"], "from-http");

    client.shutdown().await.expect("client shutdown");
    server_handle.shutdown().await.expect("server shutdown");
}

#[cfg(unix)]
#[tokio::test]
async fn unix_socket_transport_roundtrip_registers_remote_tools() {
    let dir = tempdir().expect("tempdir");
    let socket_path = dir.path().join("morecode-mcp.sock");
    let server = ToolRegistryMcpServer::new(
        build_server_registry().await,
        McpServerConfig::new(McpServerTransportConfig::UnixSocket {
            socket_path: socket_path.clone(),
            mount_path: "/mcp".to_string(),
            stateful_mode: false,
            json_response: true,
            sse_keep_alive_secs: Some(1),
            allowed_hosts: vec!["localhost".to_string()],
        }),
    );
    let mut server_handle = server.serve().await.expect("server should start");
    let (socket_path, uri) = match server_handle.endpoint() {
        McpServerEndpoint::UnixSocket { socket_path, uri } => (socket_path.clone(), uri.clone()),
        other => panic!("expected unix socket endpoint, got {other:?}"),
    };

    let client = McpClientSession::connect(McpClientConfig::new(
        "unix",
        McpClientTransportConfig::UnixSocket {
            socket_path,
            uri,
            headers: BTreeMap::new(),
        },
    ))
    .await
    .expect("client should connect");

    let local_registry = ToolRegistry::new();
    client
        .register_tools(&local_registry)
        .await
        .expect("remote tools should register");

    let imported = client.imported_tools();
    let result = local_registry
        .execute_tool(
            "tester",
            &imported[0].registry_name,
            json!({ "message": "from-unix" }),
        )
        .await;

    assert_eq!(result.status, ToolResultStatus::Success);
    assert_eq!(result.data.expect("structured data")["echo"], "from-unix");

    client.shutdown().await.expect("client shutdown");
    server_handle.shutdown().await.expect("server shutdown");
}
