use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use mc_tool::VisibilityLayer;
use rmcp::model::{ClientCapabilities, ClientInfo, Implementation, ServerCapabilities, ServerInfo};
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_mount_path() -> String {
    "/mcp".to_string()
}

fn default_client_name() -> String {
    "morecode".to_string()
}

fn default_client_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_server_name() -> String {
    "morecode".to_string()
}

fn default_server_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_visibility() -> VisibilityLayer {
    VisibilityLayer::Project
}

fn default_http_keep_alive_secs() -> Option<u64> {
    Some(15)
}

fn default_allowed_hosts() -> Vec<String> {
    vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum McpClientTransportConfig {
    Stdio {
        command: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        env: BTreeMap<String, String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cwd: Option<PathBuf>,
    },
    Http {
        url: String,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        headers: BTreeMap<String, String>,
    },
    UnixSocket {
        socket_path: PathBuf,
        uri: String,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        headers: BTreeMap<String, String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpClientConfig {
    pub server_name: String,
    pub transport: McpClientTransportConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_prefix: Option<String>,
    #[serde(default = "default_client_name")]
    pub client_name: String,
    #[serde(default = "default_client_version")]
    pub client_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_timeout_ms: Option<u64>,
    #[serde(default = "default_true")]
    pub respect_remote_tool_hints: bool,
}

impl McpClientConfig {
    pub fn new(server_name: impl Into<String>, transport: McpClientTransportConfig) -> Self {
        Self {
            server_name: server_name.into(),
            transport,
            tool_prefix: None,
            client_name: default_client_name(),
            client_version: default_client_version(),
            request_timeout_ms: Some(30_000),
            respect_remote_tool_hints: true,
        }
    }

    pub fn request_timeout(&self) -> Option<Duration> {
        self.request_timeout_ms.map(Duration::from_millis)
    }

    pub fn client_info(&self) -> ClientInfo {
        let mut client_info = ClientInfo::default();
        client_info.capabilities = ClientCapabilities::default();
        client_info.client_info = Implementation::new(&self.client_name, &self.client_version);
        client_info
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum McpServerTransportConfig {
    Stdio,
    Http {
        bind_address: SocketAddr,
        #[serde(default = "default_mount_path")]
        mount_path: String,
        #[serde(default = "default_true")]
        stateful_mode: bool,
        #[serde(default)]
        json_response: bool,
        #[serde(default = "default_http_keep_alive_secs")]
        sse_keep_alive_secs: Option<u64>,
        #[serde(default = "default_allowed_hosts")]
        allowed_hosts: Vec<String>,
    },
    UnixSocket {
        socket_path: PathBuf,
        #[serde(default = "default_mount_path")]
        mount_path: String,
        #[serde(default = "default_true")]
        stateful_mode: bool,
        #[serde(default)]
        json_response: bool,
        #[serde(default = "default_http_keep_alive_secs")]
        sse_keep_alive_secs: Option<u64>,
        #[serde(default = "default_allowed_hosts")]
        allowed_hosts: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerConfig {
    #[serde(default = "default_server_name")]
    pub server_name: String,
    #[serde(default = "default_server_version")]
    pub server_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default = "default_visibility")]
    pub visibility: VisibilityLayer,
    pub transport: McpServerTransportConfig,
}

impl McpServerConfig {
    pub fn new(transport: McpServerTransportConfig) -> Self {
        Self {
            server_name: default_server_name(),
            server_version: default_server_version(),
            instructions: None,
            visibility: default_visibility(),
            transport,
        }
    }

    pub fn server_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_tool_list_changed()
            .build();
        let implementation = Implementation::new(&self.server_name, &self.server_version)
            .with_description("MoreCode Tool Registry MCP bridge");

        let mut info = ServerInfo::new(capabilities).with_server_info(implementation);
        if let Some(instructions) = &self.instructions {
            info = info.with_instructions(instructions.clone());
        }
        info
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpServerEndpoint {
    Stdio,
    Http { uri: String },
    UnixSocket { socket_path: PathBuf, uri: String },
}
