//! *Navya-Nyāya* 5-Step Causal Syllogism Verifier.

use crate::{HealthState, VerifyError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalProof {
    pub pratijna_branch: String,
    pub hetu_resource_shas: Vec<String>,
    pub udaharana_sutra_id: String,
    pub upadhi_cleared_uris: Vec<String>,
    pub nigamana_postcondition: String,
}

impl CausalProof {
    pub fn verify_structure(&self) -> Result<(), VerifyError> {
        if self.pratijna_branch.trim().is_empty() {
            return Err(VerifyError::EmptyPratijna);
        }
        if self.hetu_resource_shas.is_empty() {
            return Err(VerifyError::EmptyHetu);
        }
        if self.udaharana_sutra_id.trim().is_empty() {
            return Err(VerifyError::RunbookNotFound { id: "<empty>".to_string() });
        }
        if self.nigamana_postcondition.trim().is_empty() {
            return Err(VerifyError::EmptyNigamana);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_proof() -> CausalProof {
        CausalProof {
            pratijna_branch: "exec/inc-8841-oom-healer".to_string(),
            hetu_resource_shas: vec!["abc123def456".to_string()],
            udaharana_sutra_id: "runbook-oom-restart-v1".to_string(),
            upadhi_cleared_uris: vec!["mcp://aws/network".to_string()],
            nigamana_postcondition: "kubectl get pod auth-service -o json | jq .status.phase".to_string(),
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
        assert_eq!(p.verify_structure(), Err(VerifyError::EmptyPratijna));
    }

    #[test]
    fn empty_hetu_fails() {
        let mut p = valid_proof();
        p.hetu_resource_shas = vec![];
        assert_eq!(p.verify_structure(), Err(VerifyError::EmptyHetu));
    }

    #[test]
    fn empty_nigamana_fails() {
        let mut p = valid_proof();
        p.nigamana_postcondition = String::new();
        assert_eq!(p.verify_structure(), Err(VerifyError::EmptyNigamana));
    }
}
