//! Error types for `sutra-wasm-host`.

use thiserror::Error;

/// All errors from the Wasm actor host.
#[derive(Debug, Error)]
pub enum WasmHostError {
    #[error("Wasmtime engine error: {0}")]
    Engine(#[from] anyhow::Error),

    #[error("Actor instantiation failed: {0}")]
    Instantiation(String),

    #[error("FinOps budget exceeded: {detail}")]
    BudgetExceeded { detail: String },

    #[error("Piṅgala cycle detected during actor execution")]
    CycleDetected,

    #[error("Adhikāra permission denied: agent cannot access path '{path}'")]
    PermissionDenied { path: String },

    #[error("LLM syscall error: {0}")]
    LlmError(String),

    #[error("Phantom proxy error: {0}")]
    PhantomError(String),

    #[error("Wasm fuel exhausted (agent hit instruction limit)")]
    FuelExhausted,

    #[error("Actor trap: {0}")]
    Trap(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
