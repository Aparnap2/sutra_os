//! Error types for `sutra-gitvfs`.

use thiserror::Error;

/// All errors from the Git VFS layer.
#[derive(Debug, Error)]
pub enum GitVfsError {
    #[error("Git operation failed: {0}")]
    Git(#[from] git2::Error),

    #[error("Repository not initialised at path: {path}")]
    NotInitialised { path: String },

    #[error("Branch '{name}' already exists")]
    BranchExists { name: String },

    #[error("Branch '{name}' not found")]
    BranchNotFound { name: String },

    #[error("Path '{path}' not found in VFS tree")]
    PathNotFound { path: String },

    #[error("Merge conflict: branch '{branch}' cannot fast-forward into main")]
    MergeConflict { branch: String },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Nyāya proof verification failed: {0}")]
    ProofVerificationFailed(String),

    #[error("Permission denied: agent cannot access path '{path}'")]
    PermissionDenied { path: String },

    #[error("Blob SHA not found in object store: {sha}")]
    BlobNotFound { sha: String },
}
