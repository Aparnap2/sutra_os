//! `sutra-kerneld` binary entry point.
//!
//! The SūtraOS kernel daemon. Initialises the event reactor and runs the
//! main telemetry processing loop.

use std::sync::Arc;
use sutra_kerneld::{KernelConfig, SutraKernel};
use sutra_wasm_host::{MockLlmBackend, PhantomProxy};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise structured logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("sutra=info".parse()?))
        .json()
        .init();

    info!("SūtraOS kernel daemon starting");

    let config = KernelConfig {
        state_repo_path: "sutra_state.git".into(),
        default_wasm_fuel: 1_000_000_000,
        run_canary: false,
    };

    // Default to mock LLM backend (override via SUTRA_LLM_BACKEND env var)
    let llm_backend = Arc::new(MockLlmBackend::with_response(
        r#"{"status": "kernel_running"}"#,
    ));
    let phantom_proxy = Arc::new(PhantomProxy::from_env());

    let mut kernel = SutraKernel::new(config, llm_backend, phantom_proxy)?;

    info!("SūtraOS kernel initialised — entering event loop");
    info!("Send SIGINT to stop gracefully");

    // In V1: wait for SIGINT (full event loop in V1.1)
    tokio::signal::ctrl_c().await?;
    info!("SūtraOS kernel shutting down gracefully");

    Ok(())
}
