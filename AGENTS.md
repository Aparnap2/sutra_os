# AGENTS.md — Master Build Specification for AxiomOS v1 (Pure Platform)

> Drop this file in the root of the repo and instruct your coding agent:
> **"Read AGENTS.md and implement AxiomOS v1 from Milestone 1 to Milestone 5."**

## 0. Strict Architectural Guardrails (DO NOT VIOLATE)

1. **Language & Edition:** Rust `2024` edition (`rustc 1.93+`).
2. **No Python, No Docker Daemon, No External Databases:** Pure Rust with embedded Git (`gix`) and Wasm host (`wasmtime`).
3. **MCP is the Universal I/O Bus:** No hardcoded connectors. All external I/O flows through `axiom-mcp-bus` (JSON-RPC 2.0, stdio + HTTP/SSE). Never add a bespoke client for a specific service.
4. **LLM is a Syscall, Never the Outer Loop:** `AxiomKernel` is the deterministic loop; Wasm calls `sys_llm_infer` as a host import.
5. **Pinned Crates:** `wasmtime = "49.0.3"` (or `>=49.0.3`), `gix = { version = "0.89", default-features = false, features = ["max-performance-safe"] }` (do NOT enable `gix-submodule` — CVE-2026-40034). `resolver = "3"`.
6. **Secrets:** injected into MCP server env by the kernel. Wasm/LLM never see raw keys.
7. **`cargo check --workspace`** must pass with zero errors before each commit; `cargo test --workspace` before marking a milestone done.

---

## 1. Workspace Layout

```
axiom-os/
├── Cargo.toml
├── wit/
│   └── axiom-syscall.wit
├── crates/
│   ├── axiom-verify/       # HealthState, CausalProof, CycleDetector, JitterScheduler
│   ├── axiom-compile/      # .axiom manifest, CapabilitySpan, ContextDelta
│   ├── axiom-gitvfs/       # Transactional Git VFS (gix)
│   ├── axiom-mcp-bus/      # NEW: MCP host multiplexer (JSON-RPC 2.0)
│   ├── axiom-wasm-host/    # Wasmtime host, sys_llm_infer, Phantom proxy
│   ├── axiom-kerneld/      # Event reactor, scheduler, REM compactor
│   └── axiom-cli/          # axiom init, mcp, daemon, audit, status, trigger
├── sdk/
│   └── rust-actor-sdk/     # no_std SDK (kept)
├── examples/
│   └── oom-trigger.event.json
└── axiom.toml              # Platform config
```

Removed vs. old spec: `stdlib-agents/`, `sutra-sandbox/`. Agents are external MCP-driven bundles.

---

## 2. Root `Cargo.toml`

```toml
[workspace]
resolver = "3"
members = [
    "crates/axiom-verify",
    "crates/axiom-compile",
    "crates/axiom-gitvfs",
    "crates/axiom-mcp-bus",
    "crates/axiom-wasm-host",
    "crates/axiom-kerneld",
    "crates/axiom-cli",
    "sdk/rust-actor-sdk",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "Apache-2.0"

[workspace.dependencies]
tokio = { version = "1.43", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
thiserror = "2.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
sha2 = "0.10"
hex = "0.4"
ulid = "1.1"
gix = { version = "0.89", default-features = false, features = ["max-performance-safe"] }
wasmtime = { version = "49.0.3", features = ["component-model", "async", "cranelift"] }
wasmtime-wasi = "49.0.3"
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "stream"] }
clap = { version = "4.5", features = ["derive"] }
globset = "0.4"
regex = "1"
```

---

## 3. Milestones

### M1 — `crates/axiom-verify`

1. `HealthState` enum: `Healthy`, `Faulted`, `Flapping`, `Unobservable`. Join/meet truth tables; `can_mutate_destructively()` false when `Unobservable`. Flap detection `Healthy`↔`Faulted` → `Flapping`.
2. `kuttaka_solve(a, b, c) -> Option<(i64,i64)>` solving `a x + b y = c` (jitter scheduling primitive).
3. `CycleDetector` rolling 64-bit register; hash `(ActionType, TargetNodeId, ExitStatus)` into 4-bit symbol; detect repeating k-mers (len 1/2/3 repeated ≥3) in O(1); return `Err(CycleDetected)`. Must also cover repeating MCP tool calls.
4. `CausalProof`:
   ```rust
   pub struct CausalProof {
       pub proposed_branch: String,
       pub evidence_blob_shas: Vec<String>,
       pub policy_rule_id: String,
       pub verified_dependencies: Vec<String>,
       pub postcondition_check: String,
   }
   ```
   `verify_structure()`: evidence non-empty & resolvable; every dependency URI resolves to `Healthy`; runbook id exists.

**Tests:** lattice truth tables; Kuṭṭaka known solutions; cycle 3-repeat (incl. MCP tool repeat); proof rejection on missing evidence SHAs / non-Healthy dependencies.

### M2 — `crates/axiom-compile`

1. `AxiomManifest` TOML parser: `[triggers]`, `[permissions]`, `[budget]`, `verification_cmd`.
2. `CapabilitySpan` 16-bit capability span encoding.
3. `ContextDelta` delta compression (parent hash + delta).

### M3 — `crates/axiom-gitvfs`

1. `AxiomRepo::init()` bare repo via `gix`; seed `/state/mcp/`, `/runbooks/`, `/incidents/`.
2. `InfraNode` typed resource with `HealthState`.
3. `begin_frame(agent_id)`, `stage_node_mutation`, `commit_frame(branch, proof)`, `rollback_frame(branch)`. CAS ref lock on merge.

### M4 — `crates/axiom-mcp-bus` + `crates/axiom-wasm-host`

`axiom-mcp-bus`:
1. JSON-RPC 2.0 host multiplexer over stdio and HTTP/SSE.
2. `mount(server_name, transport, config)` spawns/connects server; injects secrets into server env.
3. `list_resources`, `read_resource`, `list_tools`, `call_tool` proxies.
4. Mirror resources into Git VFS at `/state/mcp/<server>/resources/`.
5. Tool calls staged into frame outbox; dispatch after causal proof.
6. On server disconnect: transition all its resource nodes to `Unobservable`.

`axiom-wasm-host`:
1. `ActorHost` with `wasmtime::Config::consume_fuel(true)` + async.
2. `MockLlmBackend` + `HttpLlmBackend` implementing `LlmBackend`.
3. `PhantomProxy` secret substitution (Wasm guest never sees keys).
4. Implement WIT syscalls including `mcp-read-resource`, `mcp-invoke-tool`.

**Tests:** mock MCP server over stdio (E2E); resource read round-trip; tool staging + dispatch after proof; secret never appears in actor output; disconnect → Unobservable.

### M5 — `crates/axiom-kerneld` + `crates/axiom-cli`

`axiom-kerneld`:
1. `AxiomKernel::process_event()` pipeline: interrupt → MCP resource sync → spawn actor on frame branch → receive `CausalProof` → verify health states & evidence & dependencies → run postcondition check → dispatch outbox → FF merge → REM compaction.
2. `JitterScheduler` probe registration with jitter offsets.
3. `RemCompactor` compaction into `/runbooks/`.
4. MCP notification `resources/updated` → wake matching agents.

`axiom-cli`:
- `axiom init <dir>`
- `axiom mcp mount <name> [--transport stdio|http-sse] ...`
- `axiom daemon`
- `axiom audit`
- `axiom status [--repo <path>]`
- `axiom trigger <event.json>`

**E2E scenarios (mock MCP server over stdio):**
1. **Happy path:** MCP resource reports `Faulted`; actor stages remediation tool call; proof clears upstream URIs as `Healthy`; dispatch succeeds; merge to `main`.
2. **Unobservable hold:** MCP server disconnects mid-frame → resource nodes become `Unobservable` → commit rejected with `EpistemicBlindspotHold`; `main` untouched.
3. **Cycle termination:** actor repeats identical MCP tool call 3× → `CycleDetected` → branch rolled back.

---

## 4. WIT ABI (`wit/axiom-syscall.wit`)

```wit
package axiom:kernel@0.1.0;

interface syscalls {
    record llm-request { system-tier: u8, context-delta-json: string, response-schema-json: string }
    record causal-proof {
        proposed-branch: string,
        evidence-blob-shas: list<string>,
        policy-rule-id: string,
        verified-dependencies: list<string>,
        postcondition-check: string,
    }
    record mcp-tool-call {
        server: string,
        tool: string,
        arguments-json: string,
        staged-blob-sha: string,
    }
    vfs-read: func(path: string) -> result<list<u8>, string>;
    vfs-stage-write: func(path: string, data: list<u8>) -> result<string, string>;
    sys-llm-infer: func(req: llm-request) -> result<string, string>;
    mcp-read-resource: func(server: string, uri: string) -> result<list<u8>, string>;
    mcp-invoke-tool: func(server: string, tool: string, args-json: string) -> result<string, string>;
    sys-http-phantom-get: func(url: string, secret-placeholder: string) -> result<string, string>;
}

world axiom-actor {
    import syscalls;
    export on-wakeup: func(event-payload-json: string) -> result<causal-proof, string>;
}
```

---

## 5. Running

```bash
cargo build --workspace
cargo run -p axiom-cli -- init ./state.git
cargo run -p axiom-cli -- mcp mount kubernetes --transport stdio --command mcp-server-kubernetes
cargo run -p axiom-cli -- daemon
cargo run -p axiom-cli -- status --repo ./state.git
cargo run -p axiom-cli -- audit --repo ./state.git
cargo run -p axiom-cli -- trigger examples/oom-trigger.event.json
```

---

## 6. `axiom.toml` Example

```toml
[platform]
edition = "2024"
state_repo = "./state.git"

[[mcp.mount]]
name = "kubernetes"
transport = "stdio"
command = "mcp-server-kubernetes"

[[mcp.mount]]
name = "github"
transport = "http-sse"
url = "https://mcp.github.com/sse"
secret_placeholders = ["GITHUB_TOKEN"]

[budget]
max_tokens_per_frame = 4096
max_millicents_per_frame = 500
```
