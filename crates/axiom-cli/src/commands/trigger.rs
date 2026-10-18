//! `sutra trigger` — Fire a simulated telemetry event into the kernel.

use std::io::Read;
use std::sync::Arc;
use axiom_gitvfs::AxiomRepo;
use axiom_kerneld::{KernelConfig, AxiomKernel, TelemetryEvent};
use axiom_verify::{HealthState, CausalProof};
use axiom_wasm_host::{MockLlmBackend, PhantomProxy};
use tracing::info;

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
        state_repo_path: "axiom_state.git".into(),
        default_wasm_fuel: 1_000_000_000,
        run_canary: false,
    };

    let llm_backend = Arc::new(MockLlmBackend::with_response(
        r#"{"analysis": "OOM kill detected — increasing memory limit"}"#,
    ));
    let phantom_proxy = Arc::new(PhantomProxy::from_env());

    let mut kernel = AxiomKernel::new(config, llm_backend, phantom_proxy)?;

    // Derive state from event and update node
    let node_id = event.target_node_id.clone();
    kernel.update_node_state(&node_id, HealthState::Nasti)?;
    // Seed upstream confounder as healthy
    let _ = kernel.update_node_state("aws-us-east-1", HealthState::Asti);
    println!("⚠️  Node '{}' state → Nāsti (Faulted)", node_id);

    // Build a CausalProof from the event evidence
    let evidence = event
        .evidence_sha
        .clone()
        .unwrap_or_else(|| "abc123def456789012345678901234567890abcd".to_string());

    let proof = CausalProof {
        pratijna_branch: format!("frame/{}-trigger", node_id),
        hetu_resource_shas: vec![evidence],
        udaharana_sutra_id: "runbook-oom-restart-v1".to_string(),
        upadhi_cleared_uris: vec!["aws-us-east-1".to_string()],
        nigamana_postcondition: "kubectl get pod auth-service -o json | jq .status.phase".to_string(),
    };

    let proof_for_rem = proof.clone();
    match kernel.verify_and_commit_proof(proof, &node_id) {
        Ok(sha) => {
            println!("✅ Nyāya proof verified — merged to main: {}", &sha[..12]);
            let entry = axiom_kerneld::rem::RemCompactor::compact(&proof_for_rem, &sha, &kernel.repo);
            println!("   REM compaction: {}", entry.id);
            info!(sha = %sha, "Trigger processed");
        }
        Err(e) => {
            println!("❌ Kernel rejected proof: {}", e);
        }
    }

    // Also ensure repo exists (init if needed) handled by kernel
    let _ = AxiomRepo::open("axiom_state.git");

    println!();
    println!("💡 Use 'sutra audit' to inspect the Git VFS commit history.");
    println!("   Use 'sutra status' to see updated node states.");

    Ok(())
}