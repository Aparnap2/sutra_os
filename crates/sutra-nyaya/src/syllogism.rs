//! *Navya-Nyāya* 5-Step Causal Syllogism Verifier.
//!
//! Every agent Git-branch merge into `main` must submit a `NyayaCommitProof`
//! satisfying all five members of the classical Indian causal syllogism
//! (*Pañcāvayava Nyāya*) as formalised by Gangeśa in the *Tattvacintāmaṇi*
//! (13th century CE).
//!
//! The five members:
//! 1. *Pratijñā* — The Thesis / Proposed Mutation
//! 2. *Hetu*     — The Ground / Empirical Evidence (cryptographic hashes)
//! 3. *Udāharaṇa* — Universal Invariant + Historical Precedent (*Vyāpti*)
//! 4. *Upādhi-Nirāsa* — Refutation of Confounding Defeaters
//! 5. *Nigamana*  — The Verified Conclusion & Canary Invariant

use crate::{CatuskotiState, NyayaError};
use serde::{Deserialize, Serialize};

/// A machine-verifiable 5-member Nyāya syllogism attached to every Git
/// branch merge request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NyayaCommitProof {
    /// 1. *Pratijñā* — Proposed git diff hash / action description.
    pub pratijna_branch: String,

    /// 2. *Hetu* — Cryptographic (SHA-256) hashes of raw telemetry blobs
    ///    stored in the Git VFS that *causally prove* the fault exists.
    pub hetu_evidence_shas: Vec<String>,

    /// 3. *Udāharaṇa* — ID of the causal invariant rule in `/runbooks/sutra/`
    ///    that generalises from past incidents (*Vyāpti* — invariable concomitance).
    pub udaharana_runbook_id: String,

    /// 4. *Upādhi-Nirāsa* — List of `(node_id, observed_state)` tuples.
    ///    Every node's state MUST be `CatuskotiState::Asti` (confirmed healthy)
    ///    to rule out confounding upstream outages.
    pub upadhi_confounders_cleared: Vec<(String, CatuskotiState)>,

    /// 5. *Nigamana* — Shell command (or structured invariant) that the kernel
    ///    runs post-merge. Non-zero exit triggers automatic `git revert`.
    pub nigamana_verify_cmd: String,
}

impl NyayaCommitProof {
    /// Verify the structural integrity of this proof (local checks only —
    /// VFS blob existence is verified separately by the kernel merge gate).
    ///
    /// Returns `Ok(())` when all five members are structurally valid.
    pub fn verify_structure(&self) -> Result<(), NyayaError> {
        // 1. Pratijñā must not be empty
        if self.pratijna_branch.trim().is_empty() {
            return Err(NyayaError::EmptyPratijna);
        }

        // 2. Hetu evidence must be non-empty
        if self.hetu_evidence_shas.is_empty() {
            return Err(NyayaError::EmptyHetu);
        }

        // 3. Udāharaṇa runbook reference must be present
        if self.udaharana_runbook_id.trim().is_empty() {
            return Err(NyayaError::RunbookNotFound {
                id: "<empty>".to_string(),
            });
        }

        // 4. Every Upādhi confounder must be in Asti state
        for (node, state) in &self.upadhi_confounders_cleared {
            if *state != CatuskotiState::Asti {
                return Err(NyayaError::UpadhiNotCleared {
                    node: node.clone(),
                    state: state.to_string(),
                });
            }
        }

        // 5. Nigamana canary command must be non-empty
        if self.nigamana_verify_cmd.trim().is_empty() {
            return Err(NyayaError::EmptyNigamana);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_proof() -> NyayaCommitProof {
        NyayaCommitProof {
            pratijna_branch: "exec/inc-8841-oom-healer".to_string(),
            hetu_evidence_shas: vec!["abc123def456".to_string()],
            udaharana_runbook_id: "runbook-oom-restart-v1".to_string(),
            upadhi_confounders_cleared: vec![
                ("aws-network-us-east-1".to_string(), CatuskotiState::Asti),
                ("pg-primary".to_string(), CatuskotiState::Asti),
            ],
            nigamana_verify_cmd: "kubectl get pod auth-service -o json | jq .status.phase".to_string(),
        }
    }

    #[test]
    fn valid_proof_passes() {
        assert!(valid_proof().verify_structure().is_ok());
    }

    #[test]
    fn empty_pratijna_fails() {
        let mut p = valid_proof();
        p.pratijna_branch = "  ".to_string();
        assert_eq!(p.verify_structure(), Err(NyayaError::EmptyPratijna));
    }

    #[test]
    fn empty_hetu_fails() {
        let mut p = valid_proof();
        p.hetu_evidence_shas = vec![];
        assert_eq!(p.verify_structure(), Err(NyayaError::EmptyHetu));
    }

    #[test]
    fn avaktavya_confounder_fails() {
        let mut p = valid_proof();
        p.upadhi_confounders_cleared
            .push(("upstream-api".to_string(), CatuskotiState::Avaktavya));
        assert!(matches!(
            p.verify_structure(),
            Err(NyayaError::UpadhiNotCleared { .. })
        ));
    }

    #[test]
    fn empty_nigamana_fails() {
        let mut p = valid_proof();
        p.nigamana_verify_cmd = String::new();
        assert_eq!(p.verify_structure(), Err(NyayaError::EmptyNigamana));
    }
}
