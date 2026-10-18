//! MCP bus multiplexer.

use crate::error::McpError;
use crate::transport::StdioTransport;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct McpServerConfig {
    pub name: String,
    pub transport: String, // "stdio" | "http-sse"
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub url: Option<String>,
}

#[derive(Clone)]
pub struct MountedServer {
    pub config: McpServerConfig,
    pub transport: Arc<StdioTransport>,
    pub resources: Vec<Value>,
    pub tools: Vec<Value>,
}

pub struct McpBus {
    servers: RwLock<HashMap<String, MountedServer>>,
}

impl McpBus {
    pub fn new() -> Self {
        Self { servers: RwLock::new(HashMap::new()) }
    }

    pub async fn mount(&self, config: McpServerConfig) -> Result<(), McpError> {
        let transport = Arc::new(StdioTransport::new());
        if config.transport == "stdio" {
            let cmd = config.command.clone().ok_or_else(|| McpError::Transport("no command".into()))?;
            transport.spawn(&cmd, &config.args, &config.env).await?;
        }
        // initialize handshake
        let _ = transport.request("initialize", serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "sutra-kerneld", "version": "0.1.0"}
        })).await;
        let _ = transport.notify("notifications/initialized", serde_json::json!({})).await;

        // introspect
        let resources = transport
            .request("resources/list", serde_json::json!({}))
            .await
            .ok()
            .and_then(|v| v.get("resources").cloned())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        let tools = transport
            .request("tools/list", serde_json::json!({}))
            .await
            .ok()
            .and_then(|v| v.get("tools").cloned())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();

        info!(server = %config.name, resources = resources.len(), tools = tools.len(), "MCP server mounted");

        let mounted = MountedServer { config, transport, resources, tools };
        self.servers.write().await.insert(mounted.config.name.clone(), mounted);
        Ok(())
    }

    pub async fn unmount(&self, name: &str) {
        if let Some(srv) = self.servers.write().await.remove(name) {
            srv.transport.shutdown().await;
        }
    }

    pub async fn list_servers(&self) -> Vec<String> {
        self.servers.read().await.keys().cloned().collect()
    }

    pub async fn read_resource(&self, server: &str, uri: &str) -> Result<Value, McpError> {
        let guard = self.servers.read().await;
        let srv = guard.get(server).ok_or_else(|| McpError::NotMounted(server.to_string()))?;
        let transport = Arc::clone(&srv.transport);
        drop(guard);
        transport.request("resources/read", serde_json::json!({"uri": uri})).await
    }

    pub async fn call_tool(&self, server: &str, tool: &str, args: Value) -> Result<Value, McpError> {
        let guard = self.servers.read().await;
        let srv = guard.get(server).ok_or_else(|| McpError::NotMounted(server.to_string()))?;
        let transport = Arc::clone(&srv.transport);
        drop(guard);
        transport.request("tools/call", serde_json::json!({"name": tool, "arguments": args})).await
    }

    pub async fn on_disconnect(&self, server: &str) {
        warn!(server = %server, "MCP server disconnected — marking resources Avaktavya");
        // State transition handled by kernel via gitvfs; bus just logs.
    }
}

impl Default for McpBus {
    fn default() -> Self {
        Self::new()
    }
}
