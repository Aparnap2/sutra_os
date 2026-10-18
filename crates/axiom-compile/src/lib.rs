//! `sutra-panini` — Pāṇinian Manifest Compiler & Context Engine
//!
//! Implements three linguistic algorithms inspired by Pāṇini's *Aṣṭādhyāyī*:
//! - `.sutra` TOML manifest parser
//! - *Pratyāhāra* capability bitpacking
//! - *Anuvṛtti* context differential inheritance

pub mod context_delta;
pub mod error;
pub mod manifest;
pub mod capability_span;

pub use context_delta::ContextDelta;
pub use error::CompileError;
pub use manifest::{AxiomManifest, NimittaTrigger, AdhikaraPermission, FinOpsConfig};
pub use capability_span::CapabilitySpan;
