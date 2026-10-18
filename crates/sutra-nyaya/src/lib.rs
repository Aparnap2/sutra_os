//! `sutra-nyaya` — Epistemic Logic & Classical Mathematics Engine
//!
//! Implements four mathematical algorithms derived from Ancient Indian formal systems:
//! - Jain *Catuṣkoṭi* 4-valued epistemic state lattice
//! - Āryabhaṭa's *Kuṭṭaka* Extended Euclidean jitter scheduler  
//! - Piṅgala's binary trajectory watchdog (cycle detection)
//! - *Navya-Nyāya* 5-step causal syllogism verifier

pub mod catuskoti;
pub mod error;
pub mod kuttaka;
pub mod pingala;
pub mod syllogism;

pub use catuskoti::CatuskotiState;
pub use error::NyayaError;
pub use kuttaka::kuttaka_solve;
pub use pingala::PingalaWatchdog;
pub use syllogism::NyayaCommitProof;
