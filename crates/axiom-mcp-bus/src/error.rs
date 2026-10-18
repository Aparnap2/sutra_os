//! Error types for `sutra-mcp-bus`.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON-RPC error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Server '{0}' not mounted")]
    NotMounted(String),

    #[error("MCP call failed: {0}")]
    CallFailed(String),

    #[error("Timeout waiting for response")]
    Timeout,
}
