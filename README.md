# 🕉 SūtraOS v1 — `sutra-kerneld`

> **AI-Native Headless Operating System Kernel for 24×7 Autonomous DevOps Self-Healing**
>
> _Where Ancient Indian Mathematical Logic Meets Systems Programming_

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.93%2B-orange.svg)](https://www.rust-lang.org)
[![Architecture](https://img.shields.io/badge/arch-Rust%20%2B%20WASM%20%2B%20Git-purple.svg)]()

---

## What Is SūtraOS?

SūtraOS inverts the conventional "LLM as outer loop" paradigm. Instead of a Python prompt loop calling tools forever, SūtraOS is a **deterministic Rust kernel** where the LLM is a restricted **system call** (`sys_llm_infer`) — just like `write()` or `mmap()` in a POSIX kernel.

```
                ┌─────────────────────────────────────────────┐
                │          SūtraOS Kernel (Rust)              │
                │                                             │
  Prometheus ──►│  Catuṣkoṭi    Git VFS      Nyāya Gate      │──► Infra
  journald  ──►│  State Update  Frame Branch  Proof Verify   │    (GitOps)
  cron      ──►│       │            │              │          │
                │       ▼            ▼              ▼          │
                │    [Wasmtime Actor Host]                     │
                │       │                                     │
                │       └──► sys_llm_infer() ──► LLM API     │
                └─────────────────────────────────────────────┘
```

**95% of execution uses 0 LLM tokens.** Agents sleep at zero cost, wake on hardware interrupts, and only call the LLM when they encounter an unknown anomaly.

---

## Mathematical Foundation

SūtraOS is powered by five formal systems from Ancient Indian mathematics — not as a gimmick, but because these 2,500-year-old frameworks **literally solve** the key problems of modern AI agent design:

| Classical Indian System | Mathematical Concept | SūtraOS Application |
|------------------------|---------------------|-------------------|
| **Pāṇini's *Aṣṭādhyāyī*** (c. 500 BCE) | *Anuvṛtti* (Context Inheritance) + *Pratyāhāra* (Algebraic Compression) | Context compression for LLM syscalls. Sends only **deltas** + parent hash instead of full 20K-token context dumps |
| **Gangeśa's *Navya-Nyāya*** (13th C.) | 5-Step *Pañcāvayava* Syllogism + *Upādhi* Defeater Detection | **Root-Cause Analysis Gate** — every agent merge requires a machine-checked 5-part causal proof. Prevents confusing correlation with causation during outages |
| **Jain *Syādvāda / Catuṣkoṭi*** | 4-Valued Epistemic Logic (`Asti`, `Nāsti`, `Ubhaya`, `Avaktavya`) | **Distributed State Machine** — forbids destructive mutation when a node is unobservable (instead of crashing with `false == false` booleans) |
| **Piṅgala's *Chandaḥśāstra*** (c. 200 BCE) | Binary Combinatorial Enumeration (*Meru Prastāra*) | **O(1) Loop Detection** — detects repeating agent action cycles using 64-bit rolling register, no vector embeddings needed |
| **Āryabhaṭa's *Kuṭṭaka*** (499 CE) | Extended Euclidean Modular Solver (`ax + by = c`) | **Zero-Thundering-Herd Scheduler** — assigns coprime jitter offsets to thousands of health probes so they never collide on I/O |

---

## Architecture

### Pillar 1 — Hybrid Executable (`.sutra` manifest + `.wasm` actor)

Every agent is a two-part bundle:

```toml
# agent.sutra — The Pāṇinian Declarative Envelope
name = "k8s-oom-healer"
actor_wasm_path = "actor.wasm"
nigamana_cmd = "kubectl get pod auth-service | grep Running"

[[nimitta]]          # Wakeup trigger
kind = "prometheus_alert"
alert_name = "KubePodOOMKilling"

[[adhikara]]         # VFS permissions (Adhikāra = jurisdiction)
path_glob = "/ontology/nodes/k8s/**"
read = true
write = false
```

```rust
// actor.wasm — The Deterministic Procedural Body
// 95% execution: pure Wasm, 0 LLM tokens
// 5% execution: trapped into kernel via typed syscall
let diagnosis: NyayaRootCause = host_kernel::sys_llm_infer(
    ModelTier::System2Frontier,
    &anuvritti_compressed_context,   // Delta-only context
    Schema::NyayaSyllogism,          // Schema-constrained output
)?;
```

### Pillar 2 — Git-Native Merkle VFS

Every state mutation is a Git commit on an isolated branch:

```
state.git/
├── refs/heads/main          ← Verified production truth
├── refs/heads/frame/...     ← Speculative agent branches
└── objects/
    ├── ontology/nodes/      ← Catuṣkoṭi-typed infrastructure nodes
    ├── runbooks/sutra/      ← Verified causal invariants (REM memory)
    ├── incidents/           ← Active incident timelines
    └── ledger/wal/          ← Cryptographically-signed action ledger
```

### Pillar 3 — Navya-Nyāya Merge Gate

**No agent can merge into `main` without proving causation** (not just correlation):

```rust
pub struct NyayaCommitProof {
    pub pratijna_branch: String,          // 1. Proposed action
    pub hetu_evidence_shas: Vec<String>,  // 2. Cryptographic evidence hashes
    pub udaharana_runbook_id: String,     // 3. Vyāpti causal rule reference
    pub upadhi_confounders_cleared: Vec<(String, CatuskotiState)>, // 4. Defeaters ruled out
    pub nigamana_verify_cmd: String,      // 5. Post-merge canary command
}
```

---

## Workspace Structure

```
sutra-os/
├── Cargo.toml                     # Rust Workspace (resolver = "2")
├── wit/
│   └── sutra-syscall.wit          # WIT / WASI 0.2 Actor ABI
├── crates/
│   ├── sutra-nyaya/               # Epistemic logic & classical math engine
│   │   ├── src/catuskoti.rs       # 4-valued lattice (join/meet/flap)
│   │   ├── src/kuttaka.rs         # Extended Euclidean jitter scheduler
│   │   ├── src/pingala.rs         # O(1) trajectory cycle watchdog
│   │   └── src/syllogism.rs       # 5-step Nyāya proof verifier
│   ├── sutra-panini/              # Pāṇinian manifest & context engine
│   │   ├── src/manifest.rs        # .sutra TOML parser
│   │   ├── src/pratyahara.rs      # 16-bit capability bitpacker
│   │   └── src/anuvritti.rs       # Context delta inheritance
│   ├── sutra-gitvfs/              # Transactional Git Merkle VFS
│   │   ├── src/repository.rs      # Bare Git repo management
│   │   ├── src/node.rs            # InfraNode typed resource
│   │   └── src/transaction.rs     # Frame begin/commit/rollback
│   ├── sutra-wasm-host/           # Wasmtime actor host
│   │   ├── src/host.rs            # ActorHost + fuel metering
│   │   ├── src/llm.rs             # LlmBackend trait (Mock + HTTP)
│   │   └── src/phantom.rs         # Phantom credential proxy
│   ├── sutra-kerneld/             # Core kernel (binary: sutra-kerneld)
│   │   ├── src/kernel.rs          # SutraKernel event pipeline
│   │   ├── src/reactor.rs         # TelemetryEvent types
│   │   ├── src/scheduler.rs       # KuttakaScheduler probe registration
│   │   └── src/rem.rs             # REM memory compactor
│   └── sutra-cli/                 # CLI binary: sutra
│       └── src/commands/          # init / trigger / audit / status
├── sdk/
│   └── rust-actor-sdk/            # no_std crate for .wasm actor authors
├── stdlib-agents/                 # Built-in 24×7 DevOps agents
│   ├── k8s-oom-healer/            # Kubernetes OOM crash-loop healer
│   ├── tls-dns-sentinel/          # TLS certificate & DNS self-healing
│   └── pg-lock-surgeon/           # PostgreSQL deadlock remediation
└── examples/
    └── oom-trigger.event.json     # Sample trigger event
```

---

## Tech Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| **Core Language** | Rust 2021 + `tokio` | Memory-safe, fearless concurrency, compiles to single static binary |
| **Wasm Runtime** | `wasmtime 27` (WASI 0.1) | Sub-millisecond cold starts, fuel metering, capability-based security |
| **Git VFS** | `git2` (libgit2 bindings) | Transactional state management with full history and cryptographic integrity |
| **Logic Engine** | Pure Rust (Nyāya, Catuṣkoṭi) | Datalog-equivalent horn-clause inference in native code |
| **LLM Bridge** | `reqwest` + JSON schema | Grammar-constrained structured outputs, multi-backend (OpenAI/Anthropic/Ollama) |
| **Secrets** | Phantom Proxy (in-process) | Wasm guest never sees credentials — substituted in-flight by the kernel |

---

## Quick Start

### Prerequisites

- Rust 1.93+ (`rustup update stable`)
- `libgit2` system library (`apt install libgit2-dev` or `brew install libgit2`)

### Build

```bash
git clone https://github.com/Aparnap2/sutra_os.git
cd sutra_os
cargo build --workspace
```

### Initialise a State Repository

```bash
cargo run -p sutra-cli -- init ./my-state.git
```

Output:
```
🕉  SūtraOS — Initialising state repository at: ./my-state.git
✅ Initialised SūtraOS state repository
   Nodes seeded: k8s-worker-1, aws-us-east-1, pg-primary, auth-service
   Runbooks: runbook-oom-restart-v1
```

### Show Node States

```bash
cargo run -p sutra-cli -- status --repo ./my-state.git
```

Output:
```
🕉  SūtraOS Status — Node Catuṣkoṭi States
   Node ID                        Catuṣkoṭi State
   ──────────────────────────────────────────────────────────────
   k8s-worker-1                   ✅ Asti (Healthy)
   aws-us-east-1                  ✅ Asti (Healthy)
   pg-primary                     ✅ Asti (Healthy)
   auth-service                   ✅ Asti (Healthy)
```

### Audit Nyāya Proof History

```bash
cargo run -p sutra-cli -- audit --repo ./my-state.git
```

### Fire a Simulated OOM Event

```bash
cargo run -p sutra-cli -- trigger examples/oom-trigger.event.json
```

### Run the Kernel Daemon

```bash
cargo run -p sutra-kerneld
```

---

## The Four Catuṣkoṭi States

| State | Sanskrit | Meaning | Mutation Policy |
|-------|----------|---------|----------------|
| `Asti` | अस्ति | Deterministically Healthy | ✅ All mutations allowed |
| `Nasti` | नास्ति | Deterministically Faulted | ✅ Remediation allowed |
| `Ubhaya` | उभय | Flapping / Byzantine | ⚠️ Non-destructive only |
| `Avaktavya` | अवक्तव्य | Unobservable / Blindspot | 🔒 **All mutations FORBIDDEN** |

---

## Nyāya Commit Proof Example

Every merge into `main` requires:

```json
{
  "pratijna_branch": "exec/inc-8841-oom-healer",
  "hetu_evidence_shas": ["abc123def456..."],
  "udaharana_runbook_id": "runbook-oom-restart-v1",
  "upadhi_confounders_cleared": [
    ["aws-us-east-1", "Asti"],
    ["pg-primary", "Asti"]
  ],
  "nigamana_verify_cmd": "kubectl get pod auth-service | grep Running"
}
```

The kernel verifies:
1. `pratijna` is non-empty
2. All `hetu` evidence SHA blobs exist in the Git VFS
3. The `udaharana` runbook rule ID is valid
4. Every `upadhi` confounder is confirmed `Asti` (ruling out upstream outages)
5. `nigamana` canary command returns exit code `0`

---

## Roadmap

### v1.0 (Current)
- [x] Catuṣkoṭi 4-valued state lattice
- [x] Kuṭṭaka jitter scheduler
- [x] Piṅgala O(1) cycle watchdog
- [x] Navya-Nyāya 5-step proof verifier
- [x] Pāṇinian manifest parser (`.sutra` TOML)
- [x] Pratyāhāra 16-bit capability bitpacker
- [x] Anuvṛtti context differential engine
- [x] Git-native transactional VFS (`sutra-gitvfs`)
- [x] Wasmtime actor host with fuel metering
- [x] Phantom credential proxy
- [x] CLI: `init`, `trigger`, `audit`, `status`
- [x] stdlib agents: k8s-oom-healer, tls-dns-sentinel, pg-lock-surgeon

### v1.1 (Next)
- [ ] WASI 0.2 Component Model ABI (WIT-generated bindings)
- [ ] eBPF telemetry ingestion via `aya`
- [ ] Prometheus + Alertmanager webhook receiver
- [ ] Real Wasm actor compilation pipeline
- [ ] Parallel multi-agent racing on incident branches
- [ ] `tantivy` + `usearch` embedded REM memory index

### v2.0 (Future)
- [ ] Multi-tenant kernel with namespace isolation
- [ ] Bhāskara's *Cakravāla* for fleet-wide scheduling
- [ ] Distributed Git VFS (CRDTs over gossip)
- [ ] OpenTelemetry trace export

---

## Contributing

SūtraOS is open-source under Apache-2.0. Contributions welcome!

```bash
git clone https://github.com/Aparnap2/sutra_os.git
cd sutra_os
cargo check --workspace   # Must pass with zero errors
cargo test --workspace    # Run the test suite
```

Please read [AGENTS.md](AGENTS.md) for the architectural specification and coding standards.

---

## License

Apache License 2.0 — see [LICENSE](LICENSE) for details.

---

*"सूत्राणां च प्रयोजनम् — The purpose of sūtras is concision."*  
*— Pāṇini, Aṣṭādhyāyī*
