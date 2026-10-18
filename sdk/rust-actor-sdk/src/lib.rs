//! `rust-actor-sdk` — SDK for compiling SūtraOS `.wasm` DevOps actors.
//!
//! This crate provides the types and macros used by actor authors.
//! Compile with `--target wasm32-wasip1` (WASI 0.1) or `wasm32-wasip2` (WASI 0.2).

pub mod types;
pub mod macros;

pub use types::{ActorEvent, ActorResult, NyayaProofBuilder};
