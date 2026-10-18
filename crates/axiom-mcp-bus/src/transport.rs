//! stdio transport for MCP servers (child process).

use crate::McpError;
use serde_json::Value;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tracing::warn;

pub struct StdioTransport {
    child: Mutex<Option<Child>>,
    stdin: Mutex<Option<ChildStdin>>,
    stdout: Mutex<Option<BufReader<ChildStdout>>>,
    next_id: Mutex<u64>,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
            stdin: Mutex::new(None),
            stdout: Mutex::new(None),
            next_id: Mutex::new(1),
        }
    }

    pub async fn spawn(
        &self,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<(), McpError> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());
        for (k, v) in env {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take().ok_or_else(|| McpError::Transport("no stdin".into()))?;
        let stdout = child.stdout.take().ok_or_else(|| McpError::Transport("no stdout".into()))?;
        *self.child.lock().await = Some(child);
        *self.stdin.lock().await = Some(stdin);
        *self.stdout.lock().await = Some(BufReader::new(stdout));
        Ok(())
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value, McpError> {
        let id = {
            let mut g = self.next_id.lock().await;
            let id = *g;
            *g += 1;
            id
        };
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let mut line = serde_json::to_string(&req)?;
        line.push('\n');

        {
            let mut stdin_guard = self.stdin.lock().await;
            let stdin = stdin_guard.as_mut().ok_or(McpError::Transport("not spawned".into()))?;
            stdin.write_all(line.as_bytes()).await?;
            stdin.flush().await?;
        }

        // Read one line response
        let mut resp_line = String::new();
        {
            let mut stdout_guard = self.stdout.lock().await;
            let stdout = stdout_guard.as_mut().ok_or(McpError::Transport("not spawned".into()))?;
            stdout.read_line(&mut resp_line).await?;
        }
        if resp_line.trim().is_empty() {
            return Err(McpError::Transport("empty response".into()));
        }
        let resp: Value = serde_json::from_str(&resp_line)?;
        if let Some(err) = resp.get("error") {
            return Err(McpError::CallFailed(err.to_string()));
        }
        Ok(resp.get("result").cloned().unwrap_or(Value::Null))
    }

    pub async fn notify(&self, method: &str, params: Value) -> Result<(), McpError> {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        let mut line = serde_json::to_string(&req)?;
        line.push('\n');
        let mut stdin_guard = self.stdin.lock().await;
        if let Some(stdin) = stdin_guard.as_mut() {
            stdin.write_all(line.as_bytes()).await?;
            stdin.flush().await?;
        }
        Ok(())
    }

    pub async fn shutdown(&self) {
        if let Some(mut child) = self.child.lock().await.take() {
            let _ = child.kill().await;
        }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}
