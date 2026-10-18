//! `sutra-panini` — Pāṇinian Manifest Compiler & Context Engine
//!
//! Implements three linguistic algorithms inspired by Pāṇini's *Aṣṭādhyāyī*:
//! - `.sutra` TOML manifest parser
//! - *Pratyāhāra* capability bitpacking
//! - *Anuvṛtti* context differential inheritance

pub mod anuvritti;
pub mod error;
pub mod manifest;
pub mod pratyahara;

pub use anuvritti::AnuvrttiContext;
pub use error::PaniniError;
pub use manifest::{SutraManifest, NimittaTrigger, AdhikaraPermission, FinOpsConfig};
pub use pratyahara::PratyaharaBitmask;
