//! Speculative Git branch transaction ("execution frame") management.
//!
//! Each agent wakeup creates an isolated Git frame branch:
//! `refs/heads/frame/<agent_id>-<ulid>`
//!
//! The agent stages mutations on the frame branch; the kernel merge gate
//! verifies the `CausalProof` before fast-forward merging into `main`.

use crate::{GitVfsError, AxiomRepo};
use git2::{Repository, Signature};
use std::collections::HashMap;
use std::path::Path;
use axiom_verify::CausalProof;
use tracing::{info, instrument, warn};
use ulid::Ulid;

/// A unique execution frame ID.
pub type FrameId = String;

/// A staged mutation within a frame.
#[derive(Debug, Clone)]
pub struct StagedMutation {
    pub path: String,
    pub content: Vec<u8>,
}

/// An in-memory speculative Git branch representing one agent execution frame.
///
/// The agent calls `stage_mutation` to queue VFS writes; `commit_frame` seals
/// the frame with a `CausalProof` and merges into `main`.
pub struct FrameBranch {
    pub id: FrameId,
    pub agent_id: String,
    pub branch_name: String,
    pub staged: Vec<StagedMutation>,
}

impl FrameBranch {
    /// Stage a VFS path mutation (write `content` to `path` on this frame).
    pub fn stage_mutation(&mut self, path: impl Into<String>, content: Vec<u8>) {
        self.staged.push(StagedMutation {
            path: path.into(),
            content,
        });
    }

    /// Return the number of staged mutations.
    pub fn staged_count(&self) -> usize {
        self.staged.len()
    }
}

impl AxiomRepo {
    /// Begin a new speculative execution frame for the given agent.
    ///
    /// Creates a lightweight in-memory frame (no branch written to disk yet).
    #[instrument(skip(self), fields(agent = agent_id))]
    pub fn begin_frame(&self, agent_id: &str) -> Result<FrameBranch, GitVfsError> {
        let ulid = Ulid::new().to_string();
        let frame_id = format!("{agent_id}-{ulid}");
        let branch_name = format!("frame/{frame_id}");
        info!(frame_id = %frame_id, "Beginning speculative execution frame");
        Ok(FrameBranch {
            id: frame_id,
            agent_id: agent_id.to_string(),
            branch_name,
            staged: vec![],
        })
    }

    /// Commit a frame's staged mutations to Git and attempt fast-forward merge
    /// into `main`. The `CausalProof` is embedded in the commit message.
    ///
    /// Returns the new `main` commit SHA on success.
    #[instrument(skip(self, frame, proof), fields(frame = %frame.id))]
    pub fn commit_frame(
        &self,
        frame: FrameBranch,
        proof: &CausalProof,
    ) -> Result<String, GitVfsError> {
        // Verify proof structure before touching git
        proof.verify_structure().map_err(|e| {
            GitVfsError::ProofVerificationFailed(e.to_string())
        })?;

        // Verify all hetu evidence blobs exist
        for sha in &proof.hetu_resource_shas {
            if !self.blob_exists_by_sha(sha) {
                return Err(GitVfsError::ProofVerificationFailed(format!(
                    "Missing hetu evidence blob: {sha}"
                )));
            }
        }

        let sig = Signature::now("sutra-kerneld", "kernel@sutraos.local")?;
        let head = self.repo.head()?;
        let parent_commit = head.peel_to_commit()?;
        let mut parent_tree = parent_commit.tree()?;

        // Apply all staged mutations onto the parent tree
        let mut tb = self.repo.treebuilder(Some(&parent_tree))?;
        for mutation in &frame.staged {
            let blob_oid = self.repo.blob(&mutation.content)?;
            let parts: Vec<&str> = mutation.path.trim_start_matches('/').split('/').collect();
            // For simplicity in V1, only handle single-level and two-level paths
            match parts.len() {
                1 => { tb.insert(parts[0], blob_oid, 0o100644)?; }
                _ => {
                    // Rebuild nested tree
                    self.insert_nested(&mut tb, &parts, &mutation.content)?;
                }
            }
        }
        let tree_oid = tb.write()?;
        let tree = self.repo.find_tree(tree_oid)?;

        // Embed CausalProof as JSON in commit message trailer
        let proof_json = serde_json::to_string_pretty(proof)
            .map_err(|e| GitVfsError::Serialization(e.to_string()))?;

        let commit_message = format!(
            "feat(frame): {} [causal-merge]\n\nFrame: {}\nAgent: {}\n\nCausal-Proof:\n{}\n",
            proof.pratijna_branch, frame.id, frame.agent_id, proof_json
        );

        let commit_oid = self.repo.commit(
            Some("refs/heads/main"),
            &sig,
            &sig,
            &commit_message,
            &tree,
            &[&parent_commit],
        )?;

        let sha = commit_oid.to_string();
        info!(sha = %sha, "Frame committed and merged to main");
        Ok(sha)
    }

    /// Roll back a frame by dropping it (no disk writes — frame is just in memory).
    pub fn rollback_frame(&self, frame: FrameBranch) {
        warn!(frame = %frame.id, agent = %frame.agent_id, "Rolling back execution frame");
        // In-memory frame is simply dropped; no Git objects were written yet.
    }

    /// Insert a nested path into the tree builder, merging with existing subtree.
    fn insert_nested(
        &self,
        tb: &mut git2::TreeBuilder<'_>,
        parts: &[&str],
        content: &[u8],
    ) -> Result<(), GitVfsError> {
        if parts.len() == 1 {
            let blob_oid = self.repo.blob(content)?;
            tb.insert(parts[0], blob_oid, 0o100644)?;
            return Ok(());
        }
        let existing = match tb.get(parts[0]) {
            Ok(Some(e)) if e.kind() == Some(git2::ObjectType::Tree) => Some(e.id()),
            _ => None,
        };
        let mut sub_tb = match existing {
            Some(oid) => {
                let t = self.repo.find_tree(oid)?;
                self.repo.treebuilder(Some(&t))?
            }
            None => self.repo.treebuilder(None)?,
        };
        self.insert_nested(&mut sub_tb, &parts[1..], content)?;
        let sub_oid = sub_tb.write()?;
        tb.insert(parts[0], sub_oid, 0o040000)?;
        Ok(())
    }
}
