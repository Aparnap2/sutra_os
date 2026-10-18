# AxiomOS v1 — Pure-Platform Master Specification

> **AI-Native Headless Operating System Kernel where MCP is the Universal I/O Bus**
>
> _No hardcoded connectors. Every external system is an MCP server. A machine-checked causal proof gates every mutation._

| Metadata | Details |
|---|---|
| Product | **AxiomOS v1** (daemon: `axiomd`, CLI: `axiom`) |
| Category | AI-Native Headless OS Kernel & Runtime |
| Architecture | Pure Rust Kernel + Git Merkle VFS (`gix`) + WASI 0.2 Actor Host (`wasmtime`) + **MCP Universal I/O Bus** |
| Mathematical Foundation | Pāṇinian Grammar, Navya-Nyāya 5-Step Proof, Catuṣkoṭi 4-Valued Logic, Piṅgala Cycle Detection, Āryabhaṭa Kuṭṭaka Scheduling (origins of the algorithms; identifiers use clean English) |
| License / Deploy | Apache-2.0, single static binary, zero external DB |

---

## 1. Problem & Vision

Existing agent frameworks hardcode connectors (Kubernetes clients, DB drivers, cloud SDKs) and run the LLM as the outer loop. AxiomOS inverts both: **the LLM is a restricted syscall (`sys_llm_infer`)**, and **all external I/O flows through the Model Context Protocol (MCP)**. There are no hardcoded connectors — every capability is discovered, mounted, and invoked via MCP servers. Agents sleep at zero token cost, wake on MCP notifications, stage mutations on Git branches, and must satisfy a machine-checked **causal proof** before merge.

---

## 2. Architecture: MCP as the Universal I/O Bus

### 2.1 Semantic Layer — Nouns (MCP Resources)

MCP `resources/*` are the **Semantic Layer**. The kernel mirrors every mounted MCP server's resources into the Git VFS at:

```
/state/mcp/<server>/resources/<resource-uri-hash>
```

Each resource becomes a typed `InfraNode`-style object carrying a `HealthState`. Resource reads are pure, idempotent, and never mutate `main`.

### 2.2 Kinetic Layer — Verbs (MCP Tools)

MCP `tools/*` are the **Kinetic Layer**. Tool invocations are **never executed inline**:

1. Actor calls `mcp-invoke-tool` → kernel stages the call into a **transactional outbox** on the current frame branch (`refs/heads/frame/<agent>-<ulid>`).
2. The outbox entry is a Git blob under `/incidents/<frame>/outbox/`.
3. After the actor returns a `CausalProof` and the causal gate passes, the kernel **dispatches** the staged tool calls in order.
4. Dispatch results are committed back as evidence blobs; failures trigger rollback.

Mutating tools (those whose MCP schema marks `destructiveHint: true` or which the kernel classifies as mutating) are gated by the full causal proof. Read-only tools may be invoked eagerly but still flow through the bus.

### 2.3 Secrets Injection

Secrets are **injected into MCP server process environments by the kernel** at mount time. Wasm actors and the LLM never observe raw keys. Actors reference secrets only via placeholders (`AXIOM_SECRET_*`), which the kernel resolves in-flight when dispatching MCP tool calls. A disconnected or crashed MCP server's env is scrubbed.

### 2.4 Unobservable Protection on Disconnect

If an MCP server disconnects (stdio EOF, HTTP/SSE drop, healthcheck failure), every resource node under `/state/mcp/<server>/` transitions to `Unobservable`. The kernel rejects (`-EPERM` / `EpistemicBlindspotHold`) any destructive mutation targeting those nodes until the server reconnects and resources are re-synced. This is the **Blindspot Lock** applied to the entire MCP surface.

### 2.5 Cycle Detection for MCP Tool Calls

Repeating MCP tool-call sequences are detected by the `CycleDetector` rolling 64-bit trajectory register. If an actor issues the same `(tool_name, arguments_hash)` tuple 3 times within a frame, the kernel halts the actor with `CycleDetected` and rolls back the branch. This prevents runaway tool loops even when the LLM is driving.

### 2.6 Wakeup via MCP Notifications

MCP `notifications/resources/updated` (and server-initiated notifications) wake hibernated actors. The kernel maps a notification to the agent's `triggers`; matching agents are spawned on a fresh frame branch with the updated resource SHAs in their `ContextDelta`.

---

## 3. Hybrid Executable (`.axiom` manifest + `.wasm` actor)

Unchanged in spirit: `.axiom` is the declarative envelope (`[triggers]`, `[permissions]`, `[budget]`, `[verification]`); `.wasm` is the deterministic procedural body. The LLM is invoked only via `sys_llm_infer`. New: actors may call `mcp-read-resource` and `mcp-invoke-tool` (see §7 WIT).

---

## 4. Git-Native Merkle VFS

All state — resource nodes, MCP resource mirrors, incidents, runbooks, ledger — lives in an embedded bare Git repo managed by `gix`. Speculative frame branching is the transaction model. MCP resource snapshots are committed as blobs; outbox entries are committed as tree entries; dispatch results append evidence blobs.

---

## 5. Causal Merge Gate

No frame merges into `main` without a `CausalProof`:

```rust
pub struct CausalProof {
    pub proposed_branch: String,        // 1. Proposed action / branch
    pub evidence_blob_shas: Vec<String>, // 2. MCP resource + telemetry blob SHAs
    pub policy_rule_id: String,         // 3. Causal rule id in /runbooks/
    pub verified_dependencies: Vec<String>, // 4. MCP resource URIs verified Healthy
    pub postcondition_check: String,    // 5. Post-merge invariant command
}
```

The kernel verifies: `evidence_blob_shas` non-empty and resolvable; every URI in `verified_dependencies` resolves to a `Healthy` node; the runbook rule exists; the postcondition command exits 0.

---

## 6. Tech Stack

| Layer | Technology | Why |
|---|---|---|
| Core Language | Rust **2024** edition + tokio | Memory-safe, single static binary |
| Wasm Runtime | `wasmtime 49.0.3+` (WASI 0.2 Component Model) | Fuel metering, capability security |
| Git VFS | `gix 0.89` (pure Rust, `max-performance-safe`) | No libgit2, no CVE-2026-40034 submodule |
| MCP Bus | **`axiom-mcp-bus`** (new crate) — JSON-RPC 2.0 host multiplexer, stdio + HTTP/SSE | Universal I/O; no hardcoded connectors |
| Logic Engine | Pure Rust (health-state lattice, causal proof, cycle detection, jitter scheduling) | Native inference |
| LLM Bridge | reqwest + JSON schema | Grammar-constrained structured output |
| Secrets | Kernel env injection into MCP servers | Wasm/LLM never see raw keys |
| Workspace | resolver = **3** | Rust 2024 default |

---

## 7. WIT ABI (updated)

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

Note: export named `on-wakeup`.

---

## 8. Workspace Layout

```
axiom-os/
├── Cargo.toml
├── wit/axiom-syscall.wit
├── crates/
│   ├── axiom-verify/       # HealthState, CausalProof, CycleDetector, JitterScheduler
│   ├── axiom-compile/      # .axiom manifest, CapabilitySpan, ContextDelta
│   ├── axiom-gitvfs/       # Transactional Git VFS (gix)
│   ├── axiom-mcp-bus/      # NEW: MCP host multiplexer (JSON-RPC 2.0)
│   ├── axiom-wasm-host/    # Wasmtime host, sys_llm_infer, Phantom proxy
│   ├── axiom-kerneld/      # Event reactor, scheduler, REM compactor
│   └── axiom-cli/          # axiom init/mcp/daemon/audit/status/trigger
├── sdk/rust-actor-sdk/     # no_std SDK (kept)
├── examples/oom-trigger.event.json
└── axiom.toml              # Platform config (see §10)
```

Removed: `stdlib-agents/`, `sutra-sandbox/`. Agents are now external MCP-driven bundles, not hardcoded.

---

## 9. CLI

```
axiom init <dir>            # init Git VFS + seed state
axiom mcp mount <name> ...  # mount an MCP server (stdio or http-sse)
axiom daemon                # run the kernel daemon
axiom audit                 # validate causal proof history
axiom status                # show node health states
axiom trigger <event.json>  # fire a simulated event
```

---

## 10. `axiom.toml` Example

```toml
# axiom.toml — AxiomOS platform configuration
[platform]
edition = "2024"
state_repo = "./state.git"

[[mcp.mount]]
name = "kubernetes"
transport = "stdio"
command = "mcp-server-kubernetes"
env = { KUBECONFIG = "/etc/kube/config" }

[[mcp.mount]]
name = "github"
transport = "http-sse"
url = "https://mcp.github.com/sse"
secret_placeholders = ["GITHUB_TOKEN"]

[budget]
max_tokens_per_frame = 4096
max_millicents_per_frame = 500

[agents.k8s-oom-healer]
manifest = "agents/k8s-oom-healer/agent.axiom"
actor_wasm = "agents/k8s-oom-healer/actor.wasm"
```

---

## 11. Milestones & Test Scenarios

### M1 — `axiom-verify`
- `HealthState` lattice (join/meet/flap) + Unobservable blindspot lock.
- `kuttaka_solve(a,b,c)` (jitter scheduling primitive).
- `CycleDetector` O(1) cycle detection (incl. repeating MCP tool calls).
- `CausalProof` with new fields + `verify_structure()`.

**Tests:** lattice truth tables; Kuṭṭaka known solutions; cycle 3-repeat detection; proof rejection on missing evidence SHAs / non-Healthy dependencies.

### M2 — `axiom-compile`
- `AxiomManifest` TOML parser (`[triggers]`, `[permissions]`, `[budget]`, `verification_cmd`).
- `CapabilitySpan` 16-bit span encoding.
- `ContextDelta` delta compression.

### M3 — `axiom-gitvfs`
- `AxiomRepo::init()` bare repo + seeded `/state/mcp/`, `/runbooks/`, `/incidents/`.
- `InfraNode` with `HealthState`.
- `begin_frame` / `commit_frame` / `rollback_frame`.
- CAS ref lock on merge.

### M4 — `axiom-mcp-bus` + `axiom-wasm-host`
- JSON-RPC 2.0 multiplexer over stdio + HTTP/SSE.
- Resource mirror into Git VFS; tool outbox staging.
- `ActorHost` fuel metering; `MockLlmBackend` + `HttpLlmBackend`.
- `PhantomProxy` secret substitution into MCP server env.
- Unobservable transition on server disconnect.

**Tests:** mock MCP server over stdio (E2E); resource read round-trip; tool staging + dispatch after proof; secret never appears in actor output.

### M5 — `axiom-kerneld` + `axiom-cli`
- `AxiomKernel::process_event()` full pipeline: interrupt → MCP resource sync → causal merge.
- `JitterScheduler` probe registration with jitter.
- `RemCompactor` compaction into `/runbooks/`.
- CLI commands above.

**E2E scenarios (mock MCP server over stdio):**
1. **Happy path:** MCP resource reports node `Faulted`; actor stages remediation tool call; proof clears upstream URIs as `Healthy`; dispatch succeeds; merge to `main`.
2. **Unobservable hold:** MCP server disconnects mid-frame → resource nodes become `Unobservable` → kernel rejects commit with `EpistemicBlindspotHold`; `main` untouched.
3. **Cycle termination:** actor repeats identical MCP tool call 3× → `CycleDetected` → branch rolled back.

---

## 12. Running

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

_"The purpose of axioms is concision."_ — inspired by Pāṇini's Aṣṭādhyāyī
