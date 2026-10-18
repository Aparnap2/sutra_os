//! REM (Residual Episodic Memory) Garbage Collector.
//!
//! After a frame is merged or aborted, raw episodic scratchpads are discarded
//! while verified causal invariants are compacted into `/runbooks/` commits.
//! Named after REM sleep — the brain's memory consolidation phase.

use sutra_nyaya::NyayaCommitProof;
use tracing::{info, warn};

/// A compacted runbook entry extracted from a verified NyayaCommitProof.
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
    /// Compact a verified NyayaCommitProof into a runbook entry.
    ///
    /// In production this would write the entry to the Git VFS at
    /// `/runbooks/sutra/<id>.json`. In V1, it logs and returns the entry.
    pub fn compact(proof: &NyayaCommitProof, commit_sha: &str) -> RunbookEntry {
        let entry = RunbookEntry {
            id: format!("runbook-{}", &commit_sha[..8]),
            pratijna: proof.pratijna_branch.clone(),
            udaharana_rule_id: proof.udaharana_runbook_id.clone(),
            nigamana_cmd: proof.nigamana_verify_cmd.clone(),
            verified_at_commit: commit_sha.to_string(),
            times_applied: 1,
        };
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
