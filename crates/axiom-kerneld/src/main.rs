//! `sutra-kerneld` binary entry point.
//!
//! The SūtraOS kernel daemon. Initialises the event reactor and runs the
//! main telemetry processing loop.

use std::sync::Arc;
use axiom_kerneld::{KernelConfig, AxiomKernel};
use axiom_wasm_host::{MockLlmBackend, PhantomProxy};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise structured logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("sutra=info".parse()?))
        .json()
        .init();

    info!("SūtraOS kernel daemon starting");

    let config = KernelConfig {
        state_repo_path: "axiom_state.git".into(),
        default_wasm_fuel: 1_000_000_000,
        run_canary: false,
    };

    // Default to mock LLM backend (override via SUTRA_LLM_BACKEND env var)
    let llm_backend = Arc::new(MockLlmBackend::with_response(
        r#"{"status": "kernel_running"}"#,
    ));
    let phantom_proxy = Arc::new(PhantomProxy::from_env());

    let mut kernel = AxiomKernel::new(config, llm_backend, phantom_proxy)?;

    // Register default probes
    kernel.scheduler.register("health-check", 5000, "kerneld");
    kernel.scheduler.register("rem-gc", 30000, "kerneld");

    info!("SūtraOS kernel initialised — entering event loop");
    info!("Send SIGINT to stop gracefully");

    let mut tick = tokio::time::interval(std::time::Duration::from_secs(5));
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("SIGINT received — shutting down gracefully");
                break;
            }
            _ = tick.tick() => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let due = kernel.scheduler.due_probes(now);
                if !due.is_empty() {
                    info!(count = due.len(), "Scheduler tick: probes due");
                }
            }
        }
    }

    info!("SūtraOS kernel shutting down gracefully");

    Ok(())
}
