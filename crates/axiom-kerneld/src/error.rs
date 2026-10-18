//! Error types for `sutra-kerneld`.

use thiserror::Error;

/// All kernel-level errors.
#[derive(Debug, Error)]
pub enum KernelError {
    #[error("Epistemic blindspot hold: node '{node}' is Avaktavya — mutation forbidden")]
    EpistemicBlindspotHold { node: String },

    #[error("Byzantine hold: node '{node}' is Ubhaya — destructive mutation unsafe")]
    ByzantineHold { node: String },

    #[error("Nyāya proof verification failed: {0}")]
    ProofVerificationFailed(String),

    #[error("Nigamana canary command failed with exit code {code}: {cmd}")]
    CanaryFailed { cmd: String, code: i32 },

    #[error("Git VFS error: {0}")]
    GitVfs(#[from] axiom_gitvfs::GitVfsError),

    #[error("Wasm actor error: {0}")]
    WasmActor(#[from] axiom_wasm_host::WasmHostError),

    #[error("Scheduler error: {0}")]
    Scheduler(String),

    #[error("Tokio join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Agent manifest error: {0}")]
    Manifest(String),
}
