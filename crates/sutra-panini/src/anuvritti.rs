//! *Anuvṛtti* Context Differential Inheritance Engine.
//!
//! Implements Pāṇini's *Anuvṛtti* principle: in the *Aṣṭādhyāyī*, context from
//! a rule carries forward implicitly into subsequent rules — only *changes* need
//! to be stated explicitly. This dramatically compresses the LLM syscall payload
//! by sending only deltas + a parent context hash rather than full state.

use crate::PaniniError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A versioned context snapshot with parent reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnuvrttiContext {
    /// SHA-256 hex hash of this context snapshot (computed from `fields`).
    pub hash: String,
    /// SHA-256 hex hash of the parent context (`None` = root context).
    pub parent_hash: Option<String>,
    /// The key-value fields for this context turn.
    pub fields: HashMap<String, Value>,
    /// The minimal delta from the parent (fields that changed or were added).
    pub delta: HashMap<String, Value>,
}

impl AnuvrttiContext {
    /// Create a root context (no parent).
    pub fn root(fields: HashMap<String, Value>) -> Result<Self, PaniniError> {
        let hash = compute_hash(&fields)?;
        Ok(Self {
            hash,
            parent_hash: None,
            delta: fields.clone(),
            fields,
        })
    }

    /// Create a child context by inheriting the parent and applying `new_fields`.
    ///
    /// Only fields that differ from the parent are recorded in `delta`.
    /// The LLM syscall receives `delta` + `parent_hash` — not the full `fields`.
    pub fn inherit(
        parent: &AnuvrttiContext,
        new_fields: HashMap<String, Value>,
    ) -> Result<Self, PaniniError> {
        // Merge: parent fields overridden by new_fields
        let mut merged = parent.fields.clone();
        merged.extend(new_fields.clone());

        // Compute delta: only keys that differ from parent
        let mut delta = HashMap::new();
        for (k, v) in &merged {
            match parent.fields.get(k) {
                Some(pv) if pv == v => {} // Unchanged — inherited implicitly
                _ => {
                    delta.insert(k.clone(), v.clone());
                }
            }
        }

        let hash = compute_hash(&merged)?;
        Ok(Self {
            hash,
            parent_hash: Some(parent.hash.clone()),
            fields: merged,
            delta,
        })
    }

    /// Serialize only the delta + parent hash for the LLM syscall payload.
    ///
    /// This is the *Anuvṛtti* compressed representation — the LLM can
    /// reconstruct full context by chaining parent hashes through the VFS.
    pub fn to_anuvritti_payload(&self) -> Result<String, PaniniError> {
        let payload = serde_json::json!({
            "ctx_hash": self.hash,
            "parent_hash": self.parent_hash,
            "delta": self.delta,
        });
        serde_json::to_string(&payload)
            .map_err(|e| PaniniError::SerializationError(e.to_string()))
    }

    /// Compute the compressed delta size vs. full context size.
    pub fn compression_ratio(&self) -> f64 {
        let full_size = self.fields.len() as f64;
        let delta_size = self.delta.len() as f64;
        if full_size == 0.0 {
            1.0
        } else {
            delta_size / full_size
        }
    }
}

fn compute_hash(fields: &HashMap<String, Value>) -> Result<String, PaniniError> {
    use sha2::{Digest, Sha256};
    let serialised = serde_json::to_string(fields)
        .map_err(|e| PaniniError::SerializationError(e.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(serialised.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn root_context_has_no_parent() {
        let fields = HashMap::from([
            ("node".to_string(), json!("worker-1")),
            ("state".to_string(), json!("Nasti")),
        ]);
        let ctx = AnuvrttiContext::root(fields).unwrap();
        assert!(ctx.parent_hash.is_none());
        assert_eq!(ctx.delta.len(), 2); // Root delta = full context
    }

    #[test]
    fn child_context_delta_is_minimal() {
        let root_fields = HashMap::from([
            ("node".to_string(), json!("worker-1")),
            ("state".to_string(), json!("Nasti")),
            ("region".to_string(), json!("us-east-1")),
        ]);
        let root = AnuvrttiContext::root(root_fields).unwrap();

        // Only change one field
        let child_fields = HashMap::from([("state".to_string(), json!("Asti"))]);
        let child = AnuvrttiContext::inherit(&root, child_fields).unwrap();

        // Delta should only contain the changed "state" field
        assert_eq!(child.delta.len(), 1);
        assert_eq!(child.delta["state"], json!("Asti"));
        assert_eq!(child.fields["region"], json!("us-east-1")); // Inherited
    }

    #[test]
    fn compression_ratio_is_small_for_stable_contexts() {
        let root_fields: HashMap<String, Value> = (0..20)
            .map(|i| (format!("key_{i}"), json!(i)))
            .collect();
        let root = AnuvrttiContext::root(root_fields).unwrap();

        // Only change 1 of 20 fields
        let child_fields = HashMap::from([("key_0".to_string(), json!(999))]);
        let child = AnuvrttiContext::inherit(&root, child_fields).unwrap();

        assert!(child.compression_ratio() <= 0.1, "Delta should be ≤10% of full context");
    }
}
