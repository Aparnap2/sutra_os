//! `sutra trigger` — Fire a simulated telemetry event into the kernel.

use std::sync::Arc;
use sutra_kerneld::{KernelConfig, SutraKernel, TelemetryEvent};
use sutra_wasm_host::{MockLlmBackend, PhantomProxy};

pub async fn run(event_file: &str) -> anyhow::Result<()> {
    println!("🕉  SūtraOS — Triggering event from: {event_file}");

    let event_json = if event_file == "-" {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        buf
    } else {
        std::fs::read_to_string(event_file)
            .map_err(|e| anyhow::anyhow!("Failed to read event file '{}': {}", event_file, e))?
    };

    let event: TelemetryEvent = serde_json::from_str(&event_json)
        .map_err(|e| anyhow::anyhow!("Invalid event JSON: {}", e))?;

    println!("📡 Event: {} → node '{}'", event.id, event.target_node_id);
    println!("   Kind: {}", serde_json::to_string(&event.kind)?);

    let config = KernelConfig {
        state_repo_path: "sutra_state.git".into(),
        default_wasm_fuel: 1_000_000_000,
        run_canary: false,
    };

    let llm_backend = Arc::new(MockLlmBackend::with_response(
        r#"{"analysis": "OOM kill detected — increasing memory limit"}"#,
    ));
    let phantom_proxy = Arc::new(PhantomProxy::from_env());

    let mut kernel = SutraKernel::new(config, llm_backend, phantom_proxy)?;

    // For trigger, we don't have a real Wasm binary — use inline proof
    let node_id = event.target_node_id.clone();
    kernel.update_node_state(&node_id, sutra_nyaya::CatuskotiState::Nasti)?;

    println!("⚠️  Node '{}' state → Nāsti (Faulted)", node_id);
    println!("🔍 Verifying Nyāya proof would be required for merge...");
    println!("   (Full actor execution requires compiled .wasm binary)");
    println!();
    println!("💡 Use 'sutra audit' to inspect the Git VFS commit history.");

    Ok(())
}
