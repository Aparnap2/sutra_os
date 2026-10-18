//! `sutra-nyaya` — Epistemic Logic & Classical Mathematics Engine
//!
//! Implements four mathematical algorithms derived from Ancient Indian formal systems:
//! - Jain *Catuṣkoṭi* 4-valued epistemic state lattice
//! - Āryabhaṭa's *Kuṭṭaka* Extended Euclidean jitter scheduler  
//! - Piṅgala's binary trajectory watchdog (cycle detection)
//! - *Navya-Nyāya* 5-step causal syllogism verifier

pub mod health_state;
pub mod error;
pub mod jitter;
pub mod cycle_detector;
pub mod causal_proof;

pub use health_state::HealthState;
pub use error::VerifyError;
pub use jitter::kuttaka_solve;
pub use cycle_detector::CycleDetector;
pub use causal_proof::CausalProof;
