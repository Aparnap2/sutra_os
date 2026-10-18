//! Git repository management for the SūtraOS state VFS.
//!
//! Manages a bare Git repository at `$SUTRA_HOME/state.git` seeded with
//! the standard SūtraOS ontology directory structure.

use crate::{GitVfsError, InfraNode};
use git2::{Repository, Signature};
use std::path::{Path, PathBuf};
use tracing::{info, instrument};

/// The canonical SūtraOS Git state repository wrapper.
pub struct SutraRepo {
    pub(crate) repo: Repository,
    pub(crate) path: PathBuf,
}

impl SutraRepo {
    /// Open an existing SūtraOS state repository.
    #[instrument(skip_all, fields(path = %path.as_ref().display()))]
    pub fn open(path: impl AsRef<Path>) -> Result<Self, GitVfsError> {
        let path = path.as_ref().to_path_buf();
        let repo = Repository::open(&path)?;
        info!("Opened SūtraOS state repository");
        Ok(Self { repo, path })
    }

    /// Initialise a new bare SūtraOS state repository with seed structure.
    #[instrument(skip_all, fields(path = %path.as_ref().display()))]
    pub fn init(path: impl AsRef<Path>) -> Result<Self, GitVfsError> {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&path).ok();

        let repo = Repository::init_bare(&path)?;
        info!("Initialised bare SūtraOS state repository");

        let mut this = Self { repo, path };
        this.seed_initial_commit()?;
        Ok(this)
    }

    /// Seed the repository with the canonical SūtraOS directory structure.
    fn seed_initial_commit(&mut self) -> Result<(), GitVfsError> {
        let sig = default_sig()?;
        let mut tb = self.repo.treebuilder(None)?;

        // Create placeholder blobs for directory structure
        let dirs = [
            ("ontology/nodes/.gitkeep", b"# SutraOS node ontology\n" as &[u8]),
            ("runbooks/sutra/.gitkeep", b"# SutraOS verified runbooks\n"),
            ("incidents/.gitkeep", b"# SutraOS active incidents\n"),
            ("ledger/wal/.gitkeep", b"# SutraOS write-ahead ledger\n"),
        ];

        for (path_str, content) in &dirs {
            let parts: Vec<&str> = path_str.split('/').collect();
            self.insert_path_into_tree(&mut tb, &parts, content)?;
        }

        let tree_oid = tb.write()?;
        let tree = self.repo.find_tree(tree_oid)?;

        self.repo.commit(
            Some("refs/heads/main"),
            &sig,
            &sig,
            "chore: initialise SūtraOS state repository\n\nSeeds ontology/, runbooks/, incidents/, ledger/ directories.",
            &tree,
            &[], // No parents (initial commit)
        )?;

        info!("Seeded initial SūtraOS commit on refs/heads/main");
        Ok(())
    }

    /// Insert nested path entries into a tree builder.
    fn insert_path_into_tree(
        &self,
        root_tb: &mut git2::TreeBuilder<'_>,
        parts: &[&str],
        content: &[u8],
    ) -> Result<(), GitVfsError> {
        if parts.is_empty() {
            return Ok(());
        }
        if parts.len() == 1 {
            let blob_oid = self.repo.blob(content)?;
            root_tb.insert(parts[0], blob_oid, 0o100644)?;
            return Ok(());
        }
        // Nested: create sub-tree
        let mut sub_tb = self.repo.treebuilder(None)?;
        self.insert_path_into_tree(&mut sub_tb, &parts[1..], content)?;
        let sub_oid = sub_tb.write()?;
        root_tb.insert(parts[0], sub_oid, 0o040000)?;
        Ok(())
    }

    /// Read raw bytes at a VFS path from the current `main` HEAD.
    pub fn read_path(&self, vfs_path: &str) -> Result<Vec<u8>, GitVfsError> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        let tree = commit.tree()?;
        let entry = tree
            .get_path(Path::new(vfs_path.trim_start_matches('/')))
            .map_err(|_| GitVfsError::PathNotFound { path: vfs_path.to_string() })?;
        let blob = self.repo.find_blob(entry.id())?;
        Ok(blob.content().to_vec())
    }

    /// Check whether a SHA-256 hex string exists as a Git blob in the object DB.
    pub fn blob_exists_by_sha(&self, sha_hex: &str) -> bool {
        // We store telemetry blobs tagged by their content SHA; the OID is the blob OID.
        // For V1, we look them up by blob content hash stored in the ledger.
        let ledger_path = format!("ledger/wal/{sha_hex}.json");
        self.read_path(&ledger_path).is_ok()
    }

    /// Write an `InfraNode` to the ontology at `ontology/nodes/<id>.json` on `main`.
    pub fn upsert_node(&self, node: &InfraNode) -> Result<(), GitVfsError> {
        let path = format!("ontology/nodes/{}.json", node.id);
        let content = node
            .to_json_bytes()
            .map_err(GitVfsError::Serialization)?;
        self.write_to_main(&path, &content, &format!("ontology: upsert node {}", node.id))
    }

    /// Read an `InfraNode` from the ontology.
    pub fn read_node(&self, node_id: &str) -> Result<InfraNode, GitVfsError> {
        let path = format!("ontology/nodes/{node_id}.json");
        let bytes = self.read_path(&path)?;
        InfraNode::from_json_bytes(&bytes)
            .map_err(GitVfsError::Serialization)
    }

    /// Commit a single file write directly to `main` (used for bootstrapping).
    pub fn write_to_main(
        &self,
        vfs_path: &str,
        content: &[u8],
        message: &str,
    ) -> Result<(), GitVfsError> {
        let sig = default_sig()?;
        let head = self.repo.head()?;
        let parent_commit = head.peel_to_commit()?;
        let parent_tree = parent_commit.tree()?;

        let blob_oid = self.repo.blob(content)?;

        // Build a new tree with the updated file
        let mut tb = self.repo.treebuilder(Some(&parent_tree))?;
        let parts: Vec<&str> = vfs_path.trim_start_matches('/').split('/').collect();
        self.insert_path_into_tree(&mut tb, &parts, content)?;
        // Simple flat override for top-level; for nested we rebuild
        let blob_oid2 = self.repo.blob(content)?;
        if parts.len() == 1 {
            tb.insert(parts[0], blob_oid2, 0o100644)?;
        }
        let _ = blob_oid; // used above

        let tree_oid = tb.write()?;
        let tree = self.repo.find_tree(tree_oid)?;

        self.repo.commit(
            Some("refs/heads/main"),
            &sig,
            &sig,
            message,
            &tree,
            &[&parent_commit],
        )?;

        Ok(())
    }

    /// Return the current HEAD commit SHA on `main`.
    pub fn head_sha(&self) -> Result<String, GitVfsError> {
        let head = self.repo.head()?;
        let oid = head.peel_to_commit()?.id();
        Ok(oid.to_string())
    }
}

fn default_sig() -> Result<Signature<'static>, GitVfsError> {
    Signature::now("sutra-kerneld", "kernel@sutraos.local").map_err(GitVfsError::Git)
}
