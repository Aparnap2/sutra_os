//! `sutra-mcp-bus` — Universal Kernel MCP Host Multiplexer (JSON-RPC 2.0).
//!
//! Speaks JSON-RPC 2.0 over stdio (child process) and Streamable HTTP/SSE.
//! Mounts MCP resources into the Git VFS ontology and gates MCP tool calls
//! behind the Navya-Nyaya commit proof.

pub mod bus;
pub mod error;
pub mod transport;

pub use bus::McpBus;
pub use error::McpError;
pub use transport::StdioTransport;
