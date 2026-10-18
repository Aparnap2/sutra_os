//! `sutra-kerneld` — SūtraOS Core Kernel
//!
//! The deterministic Rust event reactor that:
//! 1. Receives telemetry interrupts (Prometheus alerts, log patterns, cron)
//! 2. Updates Catuṣkoṭi node states
//! 3. Spawns Wasm actors on speculative Git frame branches
//! 4. Receives CausalProof from actors
//! 5. Verifies Catuṣkoṭi, Hetu, & Upādhi constraints
//! 6. Runs Nigamana canary command
//! 7. Fast-forward merges verified branches to `main`
//! 8. Runs REM memory compaction

pub mod error;
pub mod kernel;
pub mod reactor;
pub mod rem;
pub mod scheduler;

pub use error::KernelError;
pub use kernel::{KernelConfig, AxiomKernel};
pub use reactor::{TelemetryEvent, EventKind};
