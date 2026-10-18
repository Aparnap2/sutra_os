//! Error types for the `sutra-nyaya` crate.

use thiserror::Error;

/// All errors that can be produced by the Nyāya epistemic engine.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum NyayaError {
    /// Attempted to mutate a node in the `Avaktavya` (unobservable) state.
    #[error("Epistemic blindspot hold: node is Avaktavya — destructive mutation forbidden")]
    EpistemicBlindspotHold,

    /// Attempted to mutate a node in the `Ubhaya` (flapping) state.
    #[error("Byzantine hold: node is Ubhaya (flapping) — mutation unsafe")]
    ByzantineHold,

    /// The Piṅgala watchdog detected a repeating action cycle.
    #[error("Piṅgala cycle detected: agent is looping (cycle len={cycle_len}, repeats={repeats})")]
    PingalaCycleDetected { cycle_len: usize, repeats: usize },

    /// A required telemetry evidence hash is missing from the VFS.
    #[error("Missing hetu evidence blob: SHA={sha}")]
    MissingHetuEvidence { sha: String },

    /// A confounding node (*Upādhi*) was not in `Asti` state.
    #[error("Upādhi defeater not cleared: node '{node}' is in state {state}")]
    UpadhiNotCleared { node: String, state: String },

    /// The *Udāharaṇa* runbook rule ID was not found.
    #[error("Udāharaṇa runbook not found: id='{id}'")]
    RunbookNotFound { id: String },

    /// The *Pratijñā* action string was empty.
    #[error("Pratijñā (proposed action) must be non-empty")]
    EmptyPratijna,

    /// The *Nigamana* canary command string was empty.
    #[error("Nigamana (canary verification command) must be non-empty")]
    EmptyNigamana,

    /// No hetu evidence was provided.
    #[error("Hetu evidence list must be non-empty — causal proof requires telemetry")]
    EmptyHetu,

    /// The Kuṭṭaka solver found no integer solution.
    #[error("Kuṭṭaka: no solution exists for {a}x + {b}y = {c} (gcd({a},{b}) does not divide {c})")]
    KuttakaNoSolution { a: i64, b: i64, c: i64 },
}
