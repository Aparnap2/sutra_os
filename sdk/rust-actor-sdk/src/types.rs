//! Core types for SūtraOS actor development.

use serde::{Deserialize, Serialize};

/// The event payload passed to the actor's `on_interrupt` export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorEvent {
    pub id: String,
    pub target_node_id: String,
    pub kind: serde_json::Value,
    pub timestamp: String,
    pub evidence_sha: Option<String>,
}

/// Result type returned by an actor's `on_interrupt`.
pub type ActorResult<T> = Result<T, alloc::string::String>;

extern crate alloc;

/// Builder for constructing a `NyayaCommitProof` from within a Wasm actor.
#[derive(Default)]
pub struct NyayaProofBuilder {
    pratijna_branch: alloc::string::String,
    hetu_evidence_shas: alloc::vec::Vec<alloc::string::String>,
    udaharana_runbook_id: alloc::string::String,
    upadhi_confounders_cleared: alloc::vec::Vec<(alloc::string::String, alloc::string::String)>,
    nigamana_verify_cmd: alloc::string::String,
}

impl NyayaProofBuilder {
    pub fn new() -> Self { Self::default() }

    pub fn pratijna(mut self, branch: impl Into<alloc::string::String>) -> Self {
        self.pratijna_branch = branch.into(); self
    }
    pub fn hetu(mut self, sha: impl Into<alloc::string::String>) -> Self {
        self.hetu_evidence_shas.push(sha.into()); self
    }
    pub fn udaharana(mut self, runbook_id: impl Into<alloc::string::String>) -> Self {
        self.udaharana_runbook_id = runbook_id.into(); self
    }
    pub fn upadhi_cleared(mut self, node: impl Into<alloc::string::String>, state: impl Into<alloc::string::String>) -> Self {
        self.upadhi_confounders_cleared.push((node.into(), state.into())); self
    }
    pub fn nigamana(mut self, cmd: impl Into<alloc::string::String>) -> Self {
        self.nigamana_verify_cmd = cmd.into(); self
    }

    pub fn build_json(&self) -> alloc::string::String {
        // Minimal JSON serialization without std
        let hetu: alloc::string::String = self.hetu_evidence_shas
            .iter()
            .map(|s| alloc::format!("\"{}\"", s))
            .collect::<alloc::vec::Vec<_>>()
            .join(",");
        let upadhi: alloc::string::String = self.upadhi_confounders_cleared
            .iter()
            .map(|(n, s)| alloc::format!("[\"{}\",\"{}\"]", n, s))
            .collect::<alloc::vec::Vec<_>>()
            .join(",");
        alloc::format!(
            r#"{{"pratijna_branch":"{}","hetu_evidence_shas":[{}],"udaharana_runbook_id":"{}","upadhi_confounders_cleared":[{}],"nigamana_verify_cmd":"{}"}}"#,
            self.pratijna_branch, hetu, self.udaharana_runbook_id, upadhi, self.nigamana_verify_cmd
        )
    }
}
