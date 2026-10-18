//! Infrastructure node representation stored in the Git VFS ontology.

use axiom_verify::HealthState;
use serde::{Deserialize, Serialize};

/// A unique node identifier.
pub type NodeId = String;

/// A strongly-typed infrastructure resource node stored at
/// `/ontology/nodes/<id>.json` in the Git VFS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraNode {
    /// Unique identifier (e.g., `k8s-worker-1`, `aws-us-east-1`).
    pub id: NodeId,
    /// Human-readable display name.
    pub display_name: String,
    /// Node type (e.g., `kubernetes_pod`, `aws_ec2`, `postgres_primary`).
    pub node_type: String,
    /// Current 4-valued epistemic state.
    pub state: HealthState,
    /// Metadata key-value pairs (e.g., namespace, region, labels).
    pub metadata: std::collections::HashMap<String, String>,
    /// ISO-8601 timestamp of last state observation.
    pub last_observed_at: String,
    /// List of upstream dependency node IDs (used for Upādhi checking).
    pub depends_on: Vec<NodeId>,
}

impl InfraNode {
    /// Create a new node in the default `Asti` (healthy) state.
    pub fn new(id: impl Into<String>, display_name: impl Into<String>, node_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            node_type: node_type.into(),
            state: HealthState::Asti,
            metadata: Default::default(),
            last_observed_at: chrono_now(),
            depends_on: vec![],
        }
    }

    /// Serialize to JSON bytes for storage in the Git object database.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(self).map_err(|e| e.to_string())
    }

    /// Deserialize from JSON bytes read from the Git object database.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| e.to_string())
    }
}

/// Returns current UTC time as ISO-8601 string (simple implementation).
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Minimal ISO-8601 (seconds since epoch) — enough for V1 ordering.
    format!("{}Z", secs)
}
