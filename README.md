# AxiomOS v1 — Pure Platform

> **AI-Native Headless OS Kernel where MCP is the Universal I/O Bus**
>
> _No hardcoded connectors. A machine-checked causal proof gates every mutation._

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.93%2B-orange.svg)](https://www.rust-lang.org)

---

## What Is AxiomOS?

AxiomOS is a **deterministic Rust kernel** where the LLM is a restricted system call (`sys_llm_infer`) and **all external I/O flows through MCP**. There are no hardcoded connectors — every capability is an MCP server mounted by the kernel. Agents sleep at zero token cost, wake on MCP notifications, stage mutations on Git branches, and must pass a machine-checked **causal proof** before merge.

```
                  ┌──────────────────────────────────────────────┐
                  │          AxiomOS Kernel (Rust)               │
                  │                                              │
    MCP servers ──►│  MCP Bus  Git VFS  Causal Gate  HealthState │──► Infra
    (stdio/SSE)   │     │       │         │           │          │    (GitOps)
                  │     ▼       ▼         ▼           ▼          │
                  │              [Wasmtime Actor Host]           │
                  │                    │                         │
                  │                    └──► sys_llm_infer()      │
                  └──────────────────────────────────────────────┘
```

---

## Mathematical Foundation (origins of the algorithms)

| Classical System | Concept | AxiomOS Application |
|---|---|---|
| Pāṇini's *Aṣṭādhyāyī* | *Anuvṛtti* + *Pratyāhāra* | Context compression (`ContextDelta`); capability spans (`CapabilitySpan`) |
| Navya-Nyāya (Gangeśa) | 5-step *Pañcāvayava* + *Upādhi* | Root-cause merge gate (`CausalProof`) |
| Jain *Catuṣkoṭi* | 4-valued logic (`Asti`/`Nāsti`/`Ubhaya`/`Avaktavya`) | Distributed state (`HealthState`: Healthy/Faulted/Flapping/Unobservable); blindspot lock |
| Piṅgala | Binary combinatorial enumeration | O(1) loop/MCP-tool-cycle detection (`CycleDetector`) |
| Āryabhaṭa *Kuṭṭaka* | Extended Euclidean solver | Zero-thundering-herd scheduling (`JitterScheduler`) |

---

## Architecture

### MCP as Universal I/O Bus

- **Semantic Layer (Nouns):** MCP `resources/*` mirrored into Git VFS at `/state/mcp/<server>/resources/`, typed as health-state nodes.
- **Kinetic Layer (Verbs):** MCP `tools/*` staged in a transactional outbox on the frame branch; dispatched only after the causal proof passes.
- **Secrets:** injected into MCP server env by the kernel; Wasm/LLM never see raw keys.
- **Unobservable:** MCP server disconnect → all its resource nodes become `Unobservable`; destructive mutations rejected.
- **Wakeup:** `notifications/resources/updated` wakes hibernated actors.

### Hybrid Executable (`.axiom` + `.wasm`)

Declarative manifest + deterministic Wasm actor. LLM only via `sys_llm_infer`. New syscalls: `mcp-read-resource`, `mcp-invoke-tool`.

### Git-Native Merkle VFS

Embedded bare Git repo (`gix`). Speculative frame branches. MCP resource snapshots + outbox entries are Git blobs.

### Causal Merge Gate

```rust
pub struct CausalProof {
    pub proposed_branch: String,
    pub evidence_blob_shas: Vec<String>,
    pub policy_rule_id: String,
    pub verified_dependencies: Vec<String>,
    pub postcondition_check: String,
}
```

---

## Workspace

```
axiom-os/
├── Cargo.toml
├── wit/axiom-syscall.wit
├── crates/
│   ├── axiom-verify/
│   ├── axiom-compile/
│   ├── axiom-gitvfs/
│   ├── axiom-mcp-bus/        # NEW: MCP host multiplexer
│   ├── axiom-wasm-host/
│   ├── axiom-kerneld/
│   └── axiom-cli/
├── sdk/rust-actor-sdk/
└── examples/oom-trigger.event.json
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust **2024**, tokio |
| Wasm | `wasmtime 49.0.3+` (WASI 0.2) |
| Git VFS | `gix 0.89` (`max-performance-safe`) |
| MCP Bus | `axiom-mcp-bus` (JSON-RPC 2.0, stdio + HTTP/SSE) |
| Workspace | resolver = **3** |

---

## Quick Start

### Prerequisites
- Rust 1.93+ (`rustup update stable`)

### Build
```bash
git clone https://github.com/Aparnap2/sutra_os.git
cd sutra_os
cargo build --workspace
```

### Initialise State Repo
```bash
cargo run -p axiom-cli -- init ./state.git
```

### Mount an MCP Server
```bash
cargo run -p axiom-cli -- mcp mount kubernetes --transport stdio --command mcp-server-kubernetes
```

### Run Daemon
```bash
cargo run -p axiom-cli -- daemon
```

### Status / Audit / Trigger
```bash
cargo run -p axiom-cli -- status --repo ./state.git
cargo run -p axiom-cli -- audit --repo ./state.git
cargo run -p axiom-cli -- trigger examples/oom-trigger.event.json
```

---

## `axiom.toml` Example

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
```

---

## The Four Health States

| State | Meaning | Mutation Policy |
|---|---|---|
| `Healthy` | Healthy | ✅ All allowed |
| `Faulted` | Faulted | ✅ Remediation allowed |
| `Flapping` | Flapping/Byzantine | ⚠️ Non-destructive only |
| `Unobservable` | Unobservable (incl. MCP disconnect) | 🔒 All forbidden |

---

## Milestone Test Scenarios (E2E, mock MCP server over stdio)

1. **Happy path:** MCP resource reports `Faulted`; actor stages remediation tool call; proof clears upstream URIs as `Healthy`; dispatch succeeds; merge to `main`.
2. **Unobservable hold:** MCP server disconnects mid-frame → resource nodes become `Unobservable` → commit rejected with `EpistemicBlindspotHold`; `main` untouched.
3. **Cycle termination:** actor repeats identical MCP tool call 3× → `CycleDetected` → branch rolled back.

---

## Contributing

```bash
cargo check --workspace
cargo test --workspace
```

Read [AGENTS.md](AGENTS.md) for the build specification.

---

_"The purpose of axioms is concision."_ — inspired by Pāṇini's Aṣṭādhyāyī
