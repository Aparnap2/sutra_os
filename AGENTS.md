# AGENTS.md — Master Build Specification for SūtraOS v1

> Drop this file in the root of the repo and instruct your coding agent:
> **"Read AGENTS.md and implement SūtraOS v1 from Milestone 1 to Milestone 5."**

## 0. Strict Architectural Guardrails (DO NOT VIOLATE)

1. **Language & Edition:** Rust `2021` edition (`rustc 1.93+`)
2. **No Python, No Docker Daemon, No External Databases** — pure Rust with embedded Git (`git2`) and Wasm host (`wasmtime`)
3. **LLM is a Syscall, Never the Outer Loop** — `SutraKernel` is the deterministic loop; Wasm calls `sys_llm_infer` as a host import
4. **Pinned Crates:** `wasmtime = "27"`, `git2 = "0.19"` — for Rust 1.93 compatibility
5. **`cargo check --workspace`** must pass with zero errors before each commit

---

## 1. Workspace Layout

```
sutra-os/
├── Cargo.toml                     # Workspace root
├── wit/
│   └── sutra-syscall.wit          # WIT / WASI 0.2 interface definition
├── crates/
│   ├── sutra-nyaya/               # Catuṣkoṭi lattice, Nyāya verifier, Piṅgala, Kuṭṭaka
│   ├── sutra-panini/              # .sutra manifest parser, Pratyāhāra, Anuvṛtti
│   ├── sutra-gitvfs/              # Transactional Git VFS (git2)
│   ├── sutra-wasm-host/           # Wasmtime actor host, sys_llm_infer, Phantom proxy
│   ├── sutra-kerneld/             # Core event reactor, scheduler, REM compactor
│   └── sutra-cli/                 # `sutra init`, `sutra trigger`, `sutra audit`, `sutra status`
├── sdk/
│   └── rust-actor-sdk/            # no_std SDK for compiling .wasm actors
├── stdlib-agents/                 # Built-in 24x7 DevOps self-healing agents
│   ├── k8s-oom-healer/
│   ├── tls-dns-sentinel/
│   └── pg-lock-surgeon/
└── examples/
    └── oom-trigger.event.json     # Sample event for `sutra trigger`
```

## 2. Milestones

### Milestone 1: `sutra-nyaya`
- `CatuskotiState` 4-valued lattice (join/meet/flap detection)
- `kuttaka_solve(a, b, c)` Extended Euclidean pulverizer
- `PingalaWatchdog` rolling 64-bit trajectory register (O(1) cycle detection)
- `NyayaCommitProof` 5-step syllogism with `verify_structure()`

### Milestone 2: `sutra-panini`
- `SutraManifest` TOML parser (`[nimitta]`, `[adhikara]`, `[finops]`, `nigamana_cmd`)
- `PratyaharaBitmask` 16-bit capability bitpacker with span encoding
- `AnuvrttiContext` differential context inheritance (delta compression)

### Milestone 3: `sutra-gitvfs`
- `SutraRepo::init()` — bare Git repo with seeded ontology structure
- `InfraNode` typed resource with `CatuskotiState`
- `begin_frame()` / `commit_frame()` / `rollback_frame()` transactional branching

### Milestone 4: `sutra-wasm-host`
- `ActorHost` with fuel metering (`wasmtime::Config::consume_fuel`)
- `MockLlmBackend` + `HttpLlmBackend` implementing `LlmBackend` trait
- `PhantomProxy` credential substitution (Wasm guest never sees secrets)

### Milestone 5: `sutra-kerneld` + `sutra-cli`
- `SutraKernel::process_event()` — full pipeline from interrupt → Nyāya merge
- `KuttakaScheduler` probe registration with jitter offsets
- `RemCompactor` — REM sleep memory compaction into `/runbooks/`
- CLI: `sutra init`, `sutra trigger`, `sutra audit`, `sutra status`

## 3. Running

```bash
# Build everything
cargo build --workspace

# Run the kernel daemon
cargo run -p sutra-kerneld

# Initialise a state repo
cargo run -p sutra-cli -- init ./my-state.git

# Show node states
cargo run -p sutra-cli -- status --repo ./my-state.git

# Audit the Nyāya proof history
cargo run -p sutra-cli -- audit --repo ./my-state.git

# Fire a simulated OOM event
cargo run -p sutra-cli -- trigger examples/oom-trigger.event.json
```
