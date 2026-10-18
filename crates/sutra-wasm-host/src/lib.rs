//! `sutra-wasm-host` — Wasmtime WASI Actor Host
//!
//! Hosts SūtraOS `.wasm` actors with:
//! - Fuel metering (kills infinite loops)
//! - `sys_llm_infer` host syscall (LLM as a typed syscall)
//! - Phantom credential proxy (secrets never reach guest Wasm)
//! - `vfs_read` / `vfs_stage_write` with Adhikāra path-glob enforcement

pub mod error;
pub mod host;
pub mod llm;
pub mod phantom;

pub use error::WasmHostError;
pub use host::{ActorHost, ActorKernelContext, LlmRequest};
pub use llm::LlmBackend;
pub use phantom::PhantomProxy;
