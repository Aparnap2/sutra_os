//! `sutra-gitvfs` — Transactional Git Merkle Virtual File System
//!
//! Every state mutation in SūtraOS is staged on an isolated Git branch
//! (a "frame"), cryptographically sealed with a NyayaCommitProof, and
//! merged into `main` only after kernel verification.

pub mod error;
pub mod node;
pub mod repository;
pub mod transaction;

pub use error::GitVfsError;
pub use node::{InfraNode, NodeId};
pub use repository::SutraRepo;
pub use transaction::{FrameBranch, FrameId};
