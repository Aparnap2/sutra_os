//! REM (Residual Episodic Memory) Garbage Collector.
//!
//! After a frame is merged or aborted, raw episodic scratchpads are discarded
//! while verified causal invariants are compacted into `/runbooks/` commits.
//! Named after REM sleep — the brain's memory consolidation phase.

use std::sync::Arc;
use axiom_gitvfs::AxiomRepo;
use axiom_verify::CausalProof;
use tracing::{info, warn};

/// A compacted runbook entry extracted from a verified CausalProof.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RunbookEntry {
    pub id: String,
    pub pratijna: String,
    pub udaharana_rule_id: String,
    pub nigamana_cmd: String,
    pub verified_at_commit: String,
    pub times_applied: u32,
}

/// The REM garbage collector / memory compactor.
pub struct RemCompactor;

impl RemCompactor {
    /// Compact a verified CausalProof into a runbook entry and persist it
    /// to the Git VFS at `/runbooks/axiom/<id>.json`.
    pub fn compact(proof: &CausalProof, commit_sha: &str, repo: &AxiomRepo) -> RunbookEntry {
        let entry = RunbookEntry {
            id: format!("runbook-{}", &commit_sha[..8]),
            pratijna: proof.pratijna_branch.clone(),
            udaharana_rule_id: proof.udaharana_sutra_id.clone(),
            nigamana_cmd: proof.nigamana_postcondition.clone(),
            verified_at_commit: commit_sha.to_string(),
            times_applied: 1,
        };
        if let Ok(bytes) = serde_json::to_vec_pretty(&entry) {
            let path = format!("runbooks/axiom/{}.json", entry.id);
            if let Err(e) = repo.write_to_main(&path, &bytes, &format!("rem: compact runbook {}", entry.id)) {
                warn!(error = %e, "REM compaction: failed to persist runbook");
            }
        }
        info!(
            runbook_id = %entry.id,
            commit = %commit_sha,
            "REM compaction: new runbook entry created"
        );
        entry
    }

    /// Discard a failed frame's episodic scratchpad (no-op in V1).
    pub fn discard_frame(frame_id: &str) {
        warn!(frame_id = %frame_id, "REM: discarding failed frame episodic memory");
    }
}
