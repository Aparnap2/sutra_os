//! `AxiomKernel` — The main kernel orchestrator.
//!
//! Implements the core event-processing pipeline:
//!
//! ```text
//! Telemetry Interrupt
//!   → Update HealthState
//!   → Enforce Blindspot Lock
//!   → Spawn Wasm Actor on Frame Branch
//!   → Receive CausalProof
//!   → Verify Catuṣkoṭi + Hetu + Upādhi
//!   → Run Nigamana Canary Command
//!   → Fast-Forward Merge to main
//!   → REM Compaction
//! ```

use crate::{KernelError, TelemetryEvent, reactor::EventKind};
use crate::rem::RemCompactor;
use crate::scheduler::JitterScheduler;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use axiom_gitvfs::{InfraNode, AxiomRepo};
use axiom_verify::{HealthState, CausalProof};
use axiom_wasm_host::{ActorHost, LlmBackend, PhantomProxy};
use tracing::{error, info, instrument, warn};

/// Kernel configuration.
#[derive(Debug, Clone)]
pub struct KernelConfig {
    /// Path to the SūtraOS Git state repository.
    pub state_repo_path: PathBuf,
    /// Default Wasm fuel limit per actor invocation.
    pub default_wasm_fuel: u64,
    /// Whether to actually run the nigamana canary command (disable in tests).
    pub run_canary: bool,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            state_repo_path: PathBuf::from("axiom_state.git"),
            default_wasm_fuel: 1_000_000_000,
            run_canary: false, // Safe default for CI/testing
        }
    }
}

/// The SūtraOS kernel orchestrator.
pub struct AxiomKernel {
    pub config: KernelConfig,
    pub repo: Arc<AxiomRepo>,
    pub actor_host: ActorHost,
    pub scheduler: JitterScheduler,
    /// In-memory Catuṣkoṭi node state cache (mirrors Git VFS).
    node_states: HashMap<String, HealthState>,
}

impl AxiomKernel {
    /// Create a new kernel, initialising the Git VFS if it doesn't exist.
    pub fn new(
        config: KernelConfig,
        llm_backend: Arc<dyn LlmBackend>,
        phantom_proxy: Arc<PhantomProxy>,
    ) -> Result<Self, KernelError> {
        let repo = if config.state_repo_path.exists() {
            AxiomRepo::open(&config.state_repo_path).map_err(KernelError::GitVfs)?
        } else {
            AxiomRepo::init(&config.state_repo_path).map_err(KernelError::GitVfs)?
        };
        let repo = Arc::new(repo);
        let actor_host = ActorHost::new(llm_backend, phantom_proxy, Arc::clone(&repo))
            .map_err(KernelError::WasmActor)?;

        Ok(Self {
            config,
            repo,
            actor_host,
            scheduler: JitterScheduler::new(),
            node_states: HashMap::new(),
        })
    }

    /// Update the Catuṣkoṭi state for a node, persisting to the Git VFS.
    #[instrument(skip(self), fields(node = %node_id, new_state = ?new_state))]
    pub fn update_node_state(
        &mut self,
        node_id: &str,
        new_state: HealthState,
    ) -> Result<(), KernelError> {
        let prev = self.node_states.insert(node_id.to_string(), new_state);
        info!(
            node = %node_id,
            prev_state = ?prev,
            new_state = ?new_state,
            "Catuṣkoṭi state updated"
        );

        // Persist to Git VFS
        let mut node = self
            .repo
            .read_node(node_id)
            .unwrap_or_else(|_| InfraNode::new(node_id, node_id, "unknown"));
        node.state = new_state;
        self.repo.upsert_node(&node).map_err(KernelError::GitVfs)?;
        Ok(())
    }

    /// Get the current Catuṣkoṭi state for a node.
    pub fn node_state(&self, node_id: &str) -> HealthState {
        self.node_states
            .get(node_id)
            .copied()
            .unwrap_or(HealthState::Avaktavya) // Unknown = Unobservable
    }

    /// Process a telemetry event through the full kernel pipeline.
    ///
    /// This is the main entry point for the event reactor.
    #[instrument(skip(self, event), fields(event_id = %event.id, node = %event.target_node_id))]
    pub async fn process_event(
        &mut self,
        event: TelemetryEvent,
        wasm_bytes: Option<&[u8]>,
        manifest: Option<axiom_compile::AxiomManifest>,
    ) -> Result<String, KernelError> {
        info!("Processing telemetry event");

        // Step 1: Derive new HealthState from event
        let new_state = self.derive_state_from_event(&event);

        // Step 2: Update node state
        self.update_node_state(&event.target_node_id, new_state)?;

        // Step 3: Enforce blindspot lock — abort if Avaktavya
        if new_state == HealthState::Avaktavya {
            warn!(node = %event.target_node_id, "Blindspot lock: refusing to spawn actor for Avaktavya node");
            return Err(KernelError::EpistemicBlindspotHold {
                node: event.target_node_id.clone(),
            });
        }

        // Step 4: Begin speculative frame
        let frame = self
            .repo
            .begin_frame(&event.target_node_id)
            .map_err(KernelError::GitVfs)?;

        let frame_id = frame.id.clone();
        info!(frame = %frame_id, "Speculative frame opened");

        // Step 5: Invoke Wasm actor (if bytes provided) or use placeholder proof
        let proof = if let (Some(bytes), Some(mf)) = (wasm_bytes, manifest) {
            self.actor_host
                .run_actor(bytes, mf, &event.to_json().unwrap_or_default(), self.config.default_wasm_fuel)
                .map_err(KernelError::WasmActor)?
        } else {
            // No actor binary — kernel requires a proof to be injected externally
            // (used in CLI `sutra trigger` with manual proof)
            self.repo.rollback_frame(frame);
            return Err(KernelError::ProofVerificationFailed(
                "No actor binary or proof provided".to_string(),
            ));
        };

        // Step 6: Verify proof structure
        proof.verify_structure().map_err(|e| {
            KernelError::ProofVerificationFailed(e.to_string())
        })?;

        // Step 7: Verify all Upādhi cleared URIs are in Asti state
        for uri in &proof.upadhi_cleared_uris {
            let actual = self.node_state(uri);
            if actual != HealthState::Asti {
                return Err(KernelError::EpistemicBlindspotHold {
                    node: uri.clone(),
                });
            }
        }

        // Step 8: Commit frame
        let sha = self
            .repo
            .commit_frame(frame, &proof)
            .map_err(KernelError::GitVfs)?;

        // Step 9: Run Nigamana canary (if configured)
        if self.config.run_canary {
            self.run_nigamana(&proof.nigamana_postcondition, &sha)?;
        }

        // Step 10: REM compaction
        let _runbook = RemCompactor::compact(&proof, &sha, &self.repo);

        info!(sha = %sha, "Event fully processed — merged to main");
        Ok(sha)
    }

    /// Derive the new Catuṣkoṭi state from a telemetry event.
    fn derive_state_from_event(&self, event: &TelemetryEvent) -> HealthState {
        match &event.kind {
            EventKind::PrometheusAlert { alert_name, value, .. } => {
                if alert_name.contains("OOM") || alert_name.contains("CrashLoop") {
                    HealthState::Nasti
                } else if alert_name.contains("Flapping") {
                    HealthState::Ubhaya
                } else {
                    HealthState::Nasti
                }
            }
            EventKind::LogPattern { matched_line, .. } => {
                if matched_line.contains("OOM killer") || matched_line.contains("killed process") {
                    HealthState::Nasti
                } else {
                    HealthState::Ubhaya
                }
            }
            EventKind::CronTick { .. } => HealthState::Asti, // Health check tick
            EventKind::GitWebhook { .. } => HealthState::Asti,
            EventKind::ManualTrigger { payload } => {
                // Allow manual override of state in payload
                payload
                    .get("state")
                    .and_then(|s| s.as_str())
                    .and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok())
                    .unwrap_or(HealthState::Nasti)
            }
        }
    }

    /// Run the Nigamana canary verification command.
    fn run_nigamana(&self, cmd: &str, commit_sha: &str) -> Result<(), KernelError> {
        info!(cmd = %cmd, commit = %commit_sha, "Running Nigamana canary verification");
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }
        let output = Command::new(parts[0])
            .args(&parts[1..])
            .output()
            .map_err(KernelError::Io)?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            error!(cmd = %cmd, code = code, "Nigamana canary FAILED — rollback required");
            return Err(KernelError::CanaryFailed {
                cmd: cmd.to_string(),
                code,
            });
        }
        info!("Nigamana canary passed");
        Ok(())
    }

    /// Process a `CausalProof` directly (used by `sutra trigger` CLI).
    pub fn verify_and_commit_proof(
        &self,
        proof: CausalProof,
        agent_id: &str,
    ) -> Result<String, KernelError> {
        // Structural verification
        proof.verify_structure().map_err(|e| {
            KernelError::ProofVerificationFailed(e.to_string())
        })?;

        // Check all upadhi cleared URIs
        for uri in &proof.upadhi_cleared_uris {
            let actual = self.node_state(uri);
            if actual != HealthState::Asti {
                return Err(KernelError::EpistemicBlindspotHold {
                    node: uri.clone(),
                });
            }
        }

        // Begin frame and immediately commit
        let mut frame = self.repo.begin_frame(agent_id).map_err(KernelError::GitVfs)?;
        let sha = self.repo.commit_frame(frame, &proof).map_err(KernelError::GitVfs)?;
        // REM compaction
        let _ = RemCompactor::compact(&proof, &sha, &self.repo);
        Ok(sha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use axiom_wasm_host::{MockLlmBackend, PhantomProxy};
    use axiom_verify::{HealthState, CausalProof};
    use std::fs;

    fn make_kernel(dir: &std::path::Path) -> AxiomKernel {
        let config = KernelConfig {
            state_repo_path: dir.to_path_buf(),
            default_wasm_fuel: 1_000_000,
            run_canary: false,
        };
        let llm = Arc::new(MockLlmBackend::with_response("{}"));
        let phantom = Arc::new(PhantomProxy::with_secrets(std::collections::HashMap::new()));
        AxiomKernel::new(config, llm, phantom).unwrap()
    }

    #[test]
    fn verify_and_commit_proof_roundtrip() {
        let dir = std::env::temp_dir().join(format!("sutra-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let mut k = make_kernel(&dir);
        // seed a node state
        k.update_node_state("aws-us-east-1", HealthState::Asti).unwrap();

        let proof = CausalProof {
            pratijna_branch: "frame/test-trigger".to_string(),
            hetu_resource_shas: vec!["abc123def456789012345678901234567890abcd".to_string()],
            udaharana_sutra_id: "runbook-oom-restart-v1".to_string(),
            upadhi_cleared_uris: vec!["aws-us-east-1".to_string()],
            nigamana_postcondition: "true".to_string(),
        };
        let sha = k.verify_and_commit_proof(proof, "test-agent").unwrap();
        assert!(!sha.is_empty());
    }
}
