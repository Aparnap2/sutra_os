//! `SutraKernel` — The main kernel orchestrator.
//!
//! Implements the core event-processing pipeline:
//!
//! ```text
//! Telemetry Interrupt
//!   → Update CatuskotiState
//!   → Enforce Blindspot Lock
//!   → Spawn Wasm Actor on Frame Branch
//!   → Receive NyayaCommitProof
//!   → Verify Catuṣkoṭi + Hetu + Upādhi
//!   → Run Nigamana Canary Command
//!   → Fast-Forward Merge to main
//!   → REM Compaction
//! ```

use crate::{KernelError, TelemetryEvent, reactor::EventKind};
use crate::rem::RemCompactor;
use crate::scheduler::KuttakaScheduler;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use sutra_gitvfs::{InfraNode, SutraRepo};
use sutra_nyaya::{CatuskotiState, NyayaCommitProof};
use sutra_wasm_host::{ActorHost, LlmBackend, PhantomProxy};
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
            state_repo_path: PathBuf::from("sutra_state.git"),
            default_wasm_fuel: 1_000_000_000,
            run_canary: false, // Safe default for CI/testing
        }
    }
}

/// The SūtraOS kernel orchestrator.
pub struct SutraKernel {
    pub config: KernelConfig,
    pub repo: Arc<SutraRepo>,
    pub actor_host: ActorHost,
    pub scheduler: KuttakaScheduler,
    /// In-memory Catuṣkoṭi node state cache (mirrors Git VFS).
    node_states: HashMap<String, CatuskotiState>,
}

impl SutraKernel {
    /// Create a new kernel, initialising the Git VFS if it doesn't exist.
    pub fn new(
        config: KernelConfig,
        llm_backend: Arc<dyn LlmBackend>,
        phantom_proxy: Arc<PhantomProxy>,
    ) -> Result<Self, KernelError> {
        let repo = if config.state_repo_path.exists() {
            SutraRepo::open(&config.state_repo_path).map_err(KernelError::GitVfs)?
        } else {
            SutraRepo::init(&config.state_repo_path).map_err(KernelError::GitVfs)?
        };
        let repo = Arc::new(repo);
        let actor_host = ActorHost::new(llm_backend, phantom_proxy, Arc::clone(&repo))
            .map_err(KernelError::WasmActor)?;

        Ok(Self {
            config,
            repo,
            actor_host,
            scheduler: KuttakaScheduler::new(),
            node_states: HashMap::new(),
        })
    }

    /// Update the Catuṣkoṭi state for a node, persisting to the Git VFS.
    #[instrument(skip(self), fields(node = %node_id, new_state = ?new_state))]
    pub fn update_node_state(
        &mut self,
        node_id: &str,
        new_state: CatuskotiState,
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
    pub fn node_state(&self, node_id: &str) -> CatuskotiState {
        self.node_states
            .get(node_id)
            .copied()
            .unwrap_or(CatuskotiState::Avaktavya) // Unknown = Unobservable
    }

    /// Process a telemetry event through the full kernel pipeline.
    ///
    /// This is the main entry point for the event reactor.
    #[instrument(skip(self, event), fields(event_id = %event.id, node = %event.target_node_id))]
    pub async fn process_event(
        &mut self,
        event: TelemetryEvent,
        wasm_bytes: Option<&[u8]>,
        manifest: Option<sutra_panini::SutraManifest>,
    ) -> Result<String, KernelError> {
        info!("Processing telemetry event");

        // Step 1: Derive new CatuskotiState from event
        let new_state = self.derive_state_from_event(&event);

        // Step 2: Update node state
        self.update_node_state(&event.target_node_id, new_state)?;

        // Step 3: Enforce blindspot lock — abort if Avaktavya
        if new_state == CatuskotiState::Avaktavya {
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

        // Step 7: Verify all Upādhi confounders are Asti
        for (node_id, expected_state) in &proof.upadhi_confounders_cleared {
            let actual = self.node_state(node_id);
            if actual != CatuskotiState::Asti {
                return Err(KernelError::EpistemicBlindspotHold {
                    node: node_id.clone(),
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
            self.run_nigamana(&proof.nigamana_verify_cmd, &sha)?;
        }

        // Step 10: REM compaction
        let _runbook = RemCompactor::compact(&proof, &sha);

        info!(sha = %sha, "Event fully processed — merged to main");
        Ok(sha)
    }

    /// Derive the new Catuṣkoṭi state from a telemetry event.
    fn derive_state_from_event(&self, event: &TelemetryEvent) -> CatuskotiState {
        match &event.kind {
            EventKind::PrometheusAlert { alert_name, value, .. } => {
                if alert_name.contains("OOM") || alert_name.contains("CrashLoop") {
                    CatuskotiState::Nasti
                } else if alert_name.contains("Flapping") {
                    CatuskotiState::Ubhaya
                } else {
                    CatuskotiState::Nasti
                }
            }
            EventKind::LogPattern { matched_line, .. } => {
                if matched_line.contains("OOM killer") || matched_line.contains("killed process") {
                    CatuskotiState::Nasti
                } else {
                    CatuskotiState::Ubhaya
                }
            }
            EventKind::CronTick { .. } => CatuskotiState::Asti, // Health check tick
            EventKind::GitWebhook { .. } => CatuskotiState::Asti,
            EventKind::ManualTrigger { payload } => {
                // Allow manual override of state in payload
                payload
                    .get("state")
                    .and_then(|s| s.as_str())
                    .and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok())
                    .unwrap_or(CatuskotiState::Nasti)
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

    /// Process a `NyayaCommitProof` directly (used by `sutra trigger` CLI).
    pub fn verify_and_commit_proof(
        &self,
        proof: NyayaCommitProof,
        agent_id: &str,
    ) -> Result<String, KernelError> {
        // Structural verification
        proof.verify_structure().map_err(|e| {
            KernelError::ProofVerificationFailed(e.to_string())
        })?;

        // Check all confounders
        for (node_id, expected_state) in &proof.upadhi_confounders_cleared {
            let actual = self.node_state(node_id);
            if actual != CatuskotiState::Asti {
                return Err(KernelError::EpistemicBlindspotHold {
                    node: node_id.clone(),
                });
            }
        }

        // Begin frame and immediately commit
        let mut frame = self.repo.begin_frame(agent_id).map_err(KernelError::GitVfs)?;
        let sha = self.repo.commit_frame(frame, &proof).map_err(KernelError::GitVfs)?;
        Ok(sha)
    }
}
