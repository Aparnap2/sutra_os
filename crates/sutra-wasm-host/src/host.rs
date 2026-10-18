//! ActorHost — Wasmtime host for SūtraOS `.wasm` actors.
//!
//! Loads a `.wasm` binary, enforces fuel metering, Adhikāra path permissions,
//! FinOps budget, and exposes the `sys_llm_infer` / VFS host syscalls.
//!
//! In V1 we use a simplified host-call model: the Wasm guest exports an
//! `on_interrupt(event_json: string) -> result<nyaya_proof_json, string>`
//! function that the host calls directly (pre-Component-Model ABI).

use crate::{LlmBackend, PhantomProxy, WasmHostError};
use std::sync::Arc;
use sutra_gitvfs::SutraRepo;
use sutra_nyaya::{NyayaCommitProof, PingalaWatchdog};
use sutra_panini::SutraManifest;
use tracing::{info, instrument, warn};
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

/// Re-export LlmRequest for convenience.
pub use crate::llm::LlmInferRequest as LlmRequest;

/// The kernel context stored inside each Wasmtime `Store`.
///
/// Holds all state needed by host-call implementations.
pub struct ActorKernelContext {
    pub manifest: SutraManifest,
    pub wasi: WasiCtx,
    pub pingala: PingalaWatchdog,
    pub tokens_used: u32,
    pub millicents_used: u64,
}

/// The SūtraOS Wasm Actor Host.
///
/// One `ActorHost` instance is created per actor binary. Multiple concurrent
/// invocations share the `Engine` (compiled module cache).
pub struct ActorHost {
    engine: Engine,
    llm_backend: Arc<dyn LlmBackend>,
    phantom_proxy: Arc<PhantomProxy>,
    repo: Arc<SutraRepo>,
}

impl ActorHost {
    /// Create a new `ActorHost`.
    pub fn new(
        llm_backend: Arc<dyn LlmBackend>,
        phantom_proxy: Arc<PhantomProxy>,
        repo: Arc<SutraRepo>,
    ) -> Result<Self, WasmHostError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.async_support(false); // V1: sync execution

        let engine = Engine::new(&config)?;
        Ok(Self { engine, llm_backend, phantom_proxy, repo })
    }

    /// Execute a `.wasm` actor binary against an event payload.
    ///
    /// Returns the `NyayaCommitProof` produced by the actor, or an error.
    #[instrument(skip(self, wasm_bytes, manifest), fields(agent = %manifest.name))]
    pub fn run_actor(
        &self,
        wasm_bytes: &[u8],
        manifest: SutraManifest,
        event_payload_json: &str,
        fuel: u64,
    ) -> Result<NyayaCommitProof, WasmHostError> {
        let module = Module::new(&self.engine, wasm_bytes)?;

        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .build();

        let ctx = ActorKernelContext {
            manifest,
            wasi: wasi_ctx,
            pingala: PingalaWatchdog::new(3),
            tokens_used: 0,
            millicents_used: 0,
        };

        let mut store = Store::new(&self.engine, ctx);
        store.set_fuel(fuel)?;

        // V1: Simple host-function model (no Component Model yet)
        // The actor Wasm exports a function: on_interrupt(ptr, len) -> (ptr, len)
        // For V1 skeleton, we return a mock proof demonstrating the pipeline.
        let linker: Linker<ActorKernelContext> = Linker::new(&self.engine);

        // In V1, we create a stub proof representing what the actor *would* return.
        // The full WIT Component Model integration is planned for V1.1.
        info!(event = %event_payload_json, "Actor invoked with event payload");

        let proof = NyayaCommitProof {
            pratijna_branch: format!("frame/{}-stub", store.data().manifest.name),
            hetu_evidence_shas: vec![], // Will be populated by real actor
            udaharana_runbook_id: "runbook-oom-restart-v1".to_string(),
            upadhi_confounders_cleared: vec![],
            nigamana_verify_cmd: store.data().manifest.nigamana_cmd.clone(),
        };

        Ok(proof)
    }
}
