# ludoSpring — The Science of Play, Interaction, and Game Design

An ecoPrimals Spring. Treats game design with the same rigor that wetSpring treats bioinformatics and hotSpring treats nuclear physics: validated models, reproducible experiments, GPU-accelerated computation where it matters.

**Date:** May 12, 2026
**Version:** V67 (Tier 2 convergence: `toadstool.validate` pre-flight + `barracuda.precision.route` advisory wired. Pass 14 unblocked. 9 validation scenarios. 858 tests, zero clippy, zero unsafe.)
**Spring alignment table:** The ludoSpring row in sibling `../primalSpring/wateringHole/NUCLEUS_SPRING_ALIGNMENT.md` uses the same workspace test total as this README; if they diverge, treat this README and `cargo test --workspace` as canonical.
**License:** AGPL-3.0-or-later (scyBorg triple: AGPL + ORC + CC-BY-SA-4.0)
**MSRV:** 1.87 (edition 2024)
**barraCuda:** v0.4.0 (`optional = true`, feature-gated behind `local` — IPC-first default since V62, library opt-in via `--features local`)
**ecoBin:** Pure Rust application code. One `-sys` dep: `renderdoc-sys` (transitive via `wgpu-hal`, GPU feature only — infrastructure C per ecoBin v3.0 guidance). `deny.toml` enforces ecoBin v3.0 banned-crate list (openssl-sys, ring, aws-lc-sys, native-tls, zstd-sys, lz4-sys, libsqlite3-sys, cryptoki-sys). Harvested to genomeBin v5.1 (46 binaries, 6 target triples).
**Deployment model:** Pure composition — no spring binary in plasmidBin. Game science is served by composing primals via `ludospring_cell.toml` (12 NUCLEUS nodes: barraCuda for math/science, petalTongue for viz/interaction, Squirrel for AI, provenance trio, Tower Atomic). The ludospring binary is the Rust validation target (tier 2); it validates science locally but does not deploy as a primal. 30 capabilities across 11 composed primals. `lifecycle.composition` handler for runtime proto-nucleate validation.
**Audit Status:** Complete — zero hardcoded primal names (V55: capability-first `NicheDependency` with `hint_name` fallback), zero hardcoded paths, **zero hardcoded method strings** (V55: `ipc::methods` expanded to 10 domain modules with compile-time consistency test), zero `#[allow()]` in application code, zero `unsafe`, **zero `Result<_, String>` in entire codebase** (V55: library modules use `VoxelError`/`BaselineError`/`ComparisonError`; binaries use `CliError`/`RunnerError`/`IpcError`), zero external deps removable (base64 inlined), zero clippy warnings (workspace-wide), zero TODO/FIXME, all experiments use `ValidationHarness` + `BaselineProvenance` (provenance unified to `19e402c0`), all tolerances centralized (named constants with citations, v1.2.0 ordering invariant: 7 constants), `GpuContext` + `TensorSession` wired behind `gpu` feature, CI pipeline with baseline drift check + three-tier validation (LOCAL_CAPABILITIES→IPC-WIRED→FULL NUCLEUS) + `validate_composition` + `validate_primal_proof` + `ludospring_guidestone` + `cargo-llvm-cov` gated at 90% floor. Fragments: `tower_atomic`, `node_atomic`, `nest_atomic`, `meta_tier`. **858** workspace tests (V67: +4 Tier 2 convergence tests), **14** primal gaps documented (GAP-01–GAP-15 in `docs/PRIMAL_GAPS.md`; 13 resolved). **9 validation scenarios** (interaction, procedural, engagement, composition, raycaster, tier4 math, audit integration, composition gaps, tier2 convergence). guideStone readiness 4 (three-tier: bare + IPC + NUCLEUS cross-atomic). Shared `RpcClient` for all UDS JSON-RPC transport (V55: deduplicated from 4 files). `IpcError` source chaining (`From<serde_json::Error>`, `From<io::Error>`). `is_skip_error` graceful degradation. `game.tick` composite handler (V52). MCP surface complete (15/15 tools). Conforms to guideStone Composition Standard v1.2.0. Cell graph ready (`ludospring_cell.toml`). All upstream blockers resolved.

---

## Philosophy

Digital music resulted in more musicians, not fewer. Acoustic music and bands still exist. The field expanded on barrier removal.

ludoSpring follows the same principle: validate the science rigorously, then build tools that remove barriers for indie devs, musicians, creative tool makers. We sketch from real games, recreate the core mechanics with validated math, and document *why* each design decision works — not just that it does. AGPL-3.0 ensures anyone can extend this.

## What This Is

```
Python baseline → barraCuda CPU → GPU (WGSL) → sovereign pipeline (coralReef)
```

ludoSpring validates 13 foundational HCI/game science models against published research, with Python baselines proving faithful port to Rust, and GPU shader promotion maps for every pure-math module. Then it uses that validated math to build playable prototypes.

Games are the most demanding real-time interactive systems humans build. They solve problems every primal needs: input handling, spatial navigation, physics simulation, procedural content generation, accessibility, and the deep question of what makes interaction *engaging*.

## Domains

| Module | What it studies | Key models | Status |
|--------|----------------|------------|--------|
| `game` | Mechanics, state, genre taxonomy | Raycasting (DDA), voxel worlds, session state, RulesetCert validation | Validated |
| `interaction` | Input science, flow, accessibility | Fitts, Hick, Steering, GOMS, Flow, DDA | All 4 HCI laws validated |
| `procedural` | Content generation | Perlin noise, fBm, WFC, L-systems, BSP trees | All 4 PCG algorithms validated |
| `metrics` | Quantifying fun | Tufte-on-games, engagement curves, Four Keys to Fun | All 3 frameworks validated |

## Foundational Research Coverage

| Model | Source | Module | Experiments |
|-------|--------|--------|-------------|
| Fitts's law | Fitts (1954), MacKenzie (1992) | `interaction::input_laws` | 005, 015, 019 |
| Hick's law | Hick (1952), Hyman (1953) | `interaction::input_laws` | 006, 016, 019 |
| Steering law | Accot & Zhai (1997) | `interaction::input_laws` | 007, 019 |
| GOMS / KLM | Card, Moran, Newell (1983) | `interaction::goms` | 011, 019 |
| Flow theory | Csikszentmihalyi (1990) | `interaction::flow` | 010, 012, 020 |
| Dynamic difficulty | Hunicke (2005) | `interaction::difficulty` | 004, 020 |
| Four Keys to Fun | Lazzaro (2004) | `metrics::fun_keys` | 018, 021 |
| Engagement metrics | Yannakakis & Togelius (2018) | `metrics::engagement` | 010, 021 |
| Perlin noise | Perlin (1985, 2002) | `procedural::noise` | 002, 009, 014 |
| Wave function collapse | Gumin (2016) | `procedural::wfc` | 008, 014 |
| L-systems | Lindenmayer (1968) | `procedural::lsystem` | 013 |
| BSP trees | Fuchs, Kedem, Naylor (1980) | `procedural::bsp` | 017 |
| Tufte data-ink | Tufte (1983, 1990) | `metrics::tufte_gaming` | 003, 016, 022 |

## barraCuda Primitive Consumption

| Primitive | Consumer | Why |
|-----------|---------|-----|
| `activations::sigmoid` | `interaction::flow::DifficultyCurve` | Replaced hand-rolled sigmoid |
| `stats::dot` | `metrics::engagement::compute_engagement` | Weighted composite score |
| `rng::lcg_step` | `procedural::bsp::generate_bsp` | Deterministic spatial subdivision |
| `rng::state_to_f64` | `procedural::bsp::generate_bsp` | Float from LCG state |

## GPU Shader Promotion Readiness

| Module | Tier | GPU target | Blocking |
|--------|------|-----------|----------|
| `procedural::noise` | A | Perlin/fBm compute shader | Nothing — pure math |
| `game::raycaster` | A | Per-column DDA (embarrassingly parallel) | Nothing |
| `metrics::engagement` | A | Batch evaluation | Nothing — dot product |
| `metrics::fun_keys` | A | Batch classification | Nothing — weighted sum |
| `interaction::flow` | A | Batch flow evaluation | Nothing — comparisons |
| `interaction::input_laws` | A | Batch Fitts/Hick/Steering | Nothing — log2 only |
| `interaction::goms` | A | Batch KLM task time | Nothing — sum of ops |
| `procedural::wfc` | B | Parallel constraint propagation | Barrier sync needed |
| `procedural::bsp` | B | Recursive → iterative conversion | Stack elimination |
| `procedural::lsystem` | B | Parallel string rewriting | Variable-length output |

## Validated Experiments (100 — Fossilized to `fossilRecord/`)

All 100 experiments have been validated and absorbed into the eukaryotic UniBin. The
science results are preserved here; original sources live in `fossilRecord/experiments_prokaryotic_may2026/`.
Modern validation uses `ludospring validate` (8 scenarios) and `ludospring certify` (three-tier).

### Playable Prototypes (exp023-025)

Built on validated math — every game mechanic traces to a published paper:
- **Doom-in-a-terminal** (exp024): BSP levels + DDA raycaster + collision + ratatui
- **Roguelike explorer** (exp025): engagement-driven dungeon with DDA, Flow, fun classification
- **Open-systems benchmark** (exp023): ludoSpring vs fastnoise-lite (0.93x — we're faster), Bevy patterns

### GPU Parity + NUCLEUS Coordination (exp030-033)

- 32/32 CPU-vs-GPU parity (fog-of-war, tile lighting, pathfind wavefront WGSL shaders)
- 10/10 real hardware discovery via toadStool dispatch routing
- 23/23 PCIe + mixed pipelines + NPU→GPU direct + metalForge integration
- 27/27 NUCLEUS atomics + toadStool dispatch + biomeOS graph

### External Control Groups (exp038-040)

- **Metrics work on foreign content**: bracket-pathfinding roguelike produces valid engagement, flow, fun, DDA
- **Fastest noise impl**: 0.93x fastnoise-lite (C), 2.85x faster than noise-rs
- **Flow discriminates quality**: 4/5 good games in Flow, 5/5 bad games NOT in Flow
- **Scientific finding**: engagement alone doesn't measure quality — you need Flow state (Csikszentmihalyi 1990)

### Cross-Spring Experiments (exp041-044)

- **Live NCBI integration**: luxI/luxS/agrB gene search, protein databases via E-utilities
- **Tower Atomic boot**: Crypto primal discovered by `crypto.hash` capability
- **Anderson QS explorer**: QS propagation with localization transition, engagement/flow/fun/DDA

### RPGPT — Sovereign RPG Engine (Paper 18, exp045-047)

Architecture for a provenance-backed RPG system: any open ruleset (Pathfinder 2e, FATE Core, Cypher, PbtA) ingested as loamSpine certificate, AI (Squirrel) narrates within provably anchored rules. Core insight: **anti-cheat is chain-of-custody** — the same rhizoCrypt DAG tracks item lineage in extraction shooters, sample lineage in field genomics, and loot lineage in tabletop RPGs.

| Primal | RPGPT Role |
|--------|-----------|
| rhizoCrypt | Session DAG (turns, rolls, conditions, branches) |
| loamSpine | Ruleset/character/NPC/world certificates |
| sweetGrass | Player/AI creative attribution |
| ludoSpring | Flow/DDA/engagement session quality |
| BearDog | Anti-cheat action signing |
| Squirrel | AI narration constrained by ruleset cert |

See `specs/RPGPT_DEEP_SYSTEM_DESIGN.md` for planes architecture, NPC personality, internal voices.

### Games@Home — Distributed Human Computation (Paper 19, exp048-051)

Proves human gameplay IS distributed computation. Stack resolution is protein folding (same components, different order → different outcomes). Folding@Home isomorphism maps 1:1 across 12 concepts.

### Provenance Trio + Extraction Shooters + Fraud Detection (exp052-054, Track 14-16)

- **rhizoCrypt DAG**: game session as vertex graph, content-addressed (Blake3)
- **loamSpine certificates**: ruleset and card certificates mint correctly
- **sweetGrass braids**: PROV-O attribution links game actions to player DIDs
- **12 fraud types across 3 tiers**: spatial, consumable, basic — all detected structurally
- **Composable architecture**: zero chimeric deps, 5-node topology, 60 Hz budget met

### Lysogeny — Open Recreation of Proprietary Mechanics (Track 17, exp055-060)

Recreates proprietary game mechanics from published scientific math under AGPL-3.0:
- **Usurper** (Nemesis): replicator dynamics + Lotka-Volterra with memory
- **Integrase** (capture): Wright-Fisher fixation + QS bond threshold
- **Symbiont** (faction rep): multi-species Lotka-Volterra competition
- **Conjugant** (roguelite meta): horizontal gene transfer + Price equation
- **Quorum** (emergent narrative): agent-based modeling + DAG causality
- **Pathogen** (gacha anti-pattern): operant conditioning + prospect theory

See `specs/LYSOGENY_CATALOG.md` for full citation tables and cross-domain mapping.

### Cross-Spring Provenance (exp062-066)

- **BearDog signing end-to-end**: Ed25519 on every vertex, certificate, and braid
- **Cross-domain fraud unification**: Same `GenericFraudDetector` catches fraud across gaming, science, medical (>80% structural similarity)
- **Radiating attribution**: sunCloud value distribution with conservation (shares sum to 1.0)

### RPGPT Dialogue Plane (exp067-075)

- **NPC personality certificates**: loamSpine-anchored personality + knowledge bounds
- **Internal voices**: Disco Elysium-style skill-as-perspective (10 voices)
- **Trust dynamics**: multi-factor disposition, trust gates on knowledge sharing
- **Plane transitions**: state preserved across 7 planes (Exploration ↔ Dialogue ↔ Tactical ↔ Investigation ↔ Political ↔ Crafting ↔ Card/Stack)

### Performance Benchmarks (exp034-037)

- **Python parity proven**: sigmoid, Fitts, Hick, LCG, dot, L2, Perlin match within 1e-15
- **110x 60Hz headroom**: raycaster at 6,623 FPS on CPU alone
- **70% tick budget headroom**: 10K entities ticked in 910us (budget: 3,000us)

### Portable Game Telemetry Protocol (exp026-029)

13 event types, all `Serialize + Deserialize`. External game adapters validated:
Veloren (SPECS ECS), Fish Folk (Bevy), A/B Street (simulation). Any language
that writes JSON is compatible: Rust, Unity (C#), Godot (GDScript), web (JS).

## Beyond Games: Extensibility

The same validated models work outside games. AGPL-3.0 means anyone can extend:

| ludoSpring model | Game use | Non-game use |
|-----------------|----------|-------------|
| Fitts's law | HUD reachability | Any clickable UI |
| Hick's law | Menu depth | Decision interface design |
| Flow theory | Difficulty tuning | Learning software, adaptive assessments |
| DDA | Monster density | Exam difficulty, workout intensity |
| Engagement metrics | Session quality | Student attention, UX research |
| WFC | Dungeon layout | Music composition (harmonic adjacency) |
| BSP | Level generation | Office floor plans, warehouse routing |
| Perlin noise | Terrain, item placement | Data visualization, texture synthesis |
| Tufte data-ink | HUD clarity | Any dashboard or chart |

A musician editing digital sheet music. A teacher building adaptive quizzes. An architect testing floor plan navigation. The math is the same — only the domain changes.

## petalTongue Live Visualization

ludoSpring pushes game science data to petalTongue for live visualization:

```bash
# Dashboard: push 8 scenarios from validated math
cargo run --features ipc --bin ludospring -- dashboard

# Live session: 120-tick streaming game simulation
cargo run --features ipc --bin ludospring -- live-session

# Tufte dashboard: genre comparison, minimap analysis, cognitive load sweep
cargo run --features ipc --bin ludospring -- tufte-dashboard
```

All subcommands discover petalTongue automatically via Unix socket. If petalTongue is not running, scenarios are saved as JSON to `$LUDOSPRING_OUTPUT_DIR`.

## Niche Deployment (biomeOS)

ludoSpring is a first-class biomeOS niche citizen — discoverable, composable, and
orchestratable via Neural API graphs.

```bash
# UniBin server (germination mode)
cargo run --features ipc --bin ludospring -- server

# Health check
cargo run --features ipc --bin ludospring -- status

# Version and capabilities
cargo run --features ipc --bin ludospring -- version
```

**Niche artifacts:**

| Artifact | Path | Purpose |
|----------|------|---------|
| UniBin binary | `barracuda/src/bin/ludospring.rs` | `server`, `status`, `version`, `dashboard`, `live-session`, `tufte-dashboard` subcommands |
| Deploy graph | `deploy/ludospring.toml` | primalSpring deploy fragment: 30 capabilities (27 game + 3 infrastructure), optional trio + viz deps |
| Gaming niche graph | `graphs/ludospring_gaming_niche.toml` | Composes ludoSpring + petalTongue into gaming niche |
| Niche YAML | `niches/ludospring-game.yaml` | BYOB definition with organisms and customization |
| Self-knowledge | `barracuda/src/niche.rs` | Identity, capabilities, semantic mappings, cost estimates, socket resolution |
| Neural bridge | `barracuda/src/ipc/neural_bridge.rs` | Typed IPC client for biomeOS Neural API |
| Capability domains | `barracuda/src/capability_domains.rs` | Structured registry: 30 (27 game + 3 infrastructure), local/external classification |
| Domain registration | `barracuda/src/biomeos/mod.rs` | `game` domain registration via NeuralBridge |

**Compliance with Spring-as-Niche Deployment Standard:**

- UniBin binary with `server`, `status`, `version`
- JSON-RPC 2.0 over Unix socket (`$XDG_RUNTIME_DIR/biomeos/ludospring-${FAMILY_ID}.sock`)
- `health.check`, `health.liveness`, `health.readiness`, `lifecycle.status`, and `capability.list` with domain, dependencies, cost estimates
- Capability domain registration with semantic mappings via Neural API
- Clean SIGTERM shutdown with `capability.deregister`
- Provenance Trio wired at graph level (all nodes `fallback = "skip"`)
- No hardcoded primal names — capability-based discovery only
- `niche.rs` single source of truth — all identity, capabilities, and metadata centralized
- `NeuralBridge` typed client — `capability.call`, `discover_capability`, `register`, `deregister`
- Platform-agnostic paths — `temp_dir()` instead of `/tmp`, XDG-compliant socket chain
- `#![forbid(unsafe_code)]` and AGPL-3.0-or-later

## Architecture

```
ludoSpring/
├── barracuda/             # Core library + UniBin binary + validation binaries
│   ├── src/
│   │   ├── game/          # Mechanics, raycaster, voxel, genre, state
│   │   ├── interaction/   # Fitts, Hick, Steering, GOMS, Flow, DDA
│   │   ├── procedural/    # Noise, WFC, L-systems, BSP
│   │   ├── metrics/       # Tufte, engagement, Four Keys to Fun
│   │   ├── tolerances/    # 6 submodules (game, interaction, ipc, metrics, procedural, validation)
│   │   ├── validation/    # ValidationHarness + scenarios/ (ScenarioMeta registry)
│   │   ├── certification/ # Absorbed guidestone three-tier validation (organelle)
│   │   ├── telemetry/     # Portable event protocol + analysis pipeline
│   │   ├── visualization/ # Data channels + VisualizationPushClient (capability-based)
│   │   ├── ipc/           # JSON-RPC 2.0 server + handlers + typed clients + discovery + methods constants
│   │   ├── biomeos/       # Niche deployment: domain, registration, Neural API
│   │   └── bin/           # ludospring UniBin (certify/validate/serve/status/version) + validation binaries
│   └── tests/             # python_parity, validation, determinism, proptest_invariants, ipc_integration
├── fossilRecord/          # Archived prokaryotic experiment crates (100 experiments, May 2026)
├── baselines/python/      # 7 Python reference implementations
├── benchmarks/            # Criterion benchmarks (noise, raycaster, ECS)
├── metalForge/forge/      # Capability-based routing (26 tests, 4 domain modules, GPU>NPU>CPU)
├── graphs/                # Deploy graphs (ludospring_deploy.toml, gaming_niche.toml)
├── niches/                # Niche YAML (ludospring-game.yaml)
├── deploy/                # primalSpring deploy graph fragment
├── specs/                 # 14 domain specifications
├── whitePaper/            # Local paper staging (baseCamp)
└── wateringHole/          # Handoff documentation (37 versioned handoffs)
```

## Key Insight: Games ↔ Science Visualization

Game genres are interaction architectures, not aesthetic categories:

| Genre pattern | Scientific analogue |
|---------------|-------------------|
| FPS (first-person spatial) | Molecular explorer, particle cave |
| RTS (top-down command) | Systems biology dashboard |
| Sandbox (open-ended building) | Molecule builder, circuit simulator |
| Roguelike (procedural discovery) | Parameter space exploration |
| Puzzle (constraint satisfaction) | Protein folding, crystal packing |

## Build

```bash
# All tests (854 workspace: barracuda lib + barracuda --tests + forge + benchmarks)
cargo test --workspace

# UniBin subcommands (ipc + guidestone features required)
cargo run --bin ludospring -- certify          # Three-tier certification (L0-L8)
cargo run --bin ludospring -- validate         # Run scenario registry
cargo run --bin ludospring -- validate --list  # List available scenarios
cargo run --bin ludospring -- server           # biomeOS niche deployment
cargo run --bin ludospring -- status           # Composition health
cargo run --bin ludospring -- version          # Version + capabilities

# Python baselines + drift check
python3 baselines/python/run_all_baselines.py
python3 baselines/python/check_drift.py

# Quality checks (all must pass with zero warnings)
cargo fmt --check
cargo clippy --workspace --all-targets
cargo doc -p ludospring-barracuda --all-features --no-deps
cargo llvm-cov -p ludospring-barracuda --features ipc --lib --tests \
    --ignore-filename-regex bin/ --fail-under-lines 90

# Fossilized experiments (archaeology — not in workspace)
# See fossilRecord/README.md for instructions on building old experiment crates
```

## Quality

| Check | Result |
|-------|--------|
| `cargo fmt --check` | 0 diffs |
| `cargo clippy --all-features -D warnings` | 0 warnings (pedantic + nursery) |
| `cargo test --workspace` | 854 total (barracuda lib + 4 parity modules + 8 scenarios + forge), 0 failures |
| `cargo build --no-default-features --features ipc` | IPC-only (no barraCuda linkage) — 0 errors |
| `cargo doc --all-features --no-deps` | 0 warnings |
| 8 validation scenarios | UniBin `ludospring validate` (interaction, procedural, engagement, composition, raycaster, tier4 math, audit integration, composition gaps) |
| 7 Python baselines | All pass (with embedded provenance: commit, date, Python version) |
| Baseline drift check | 0 drift (automated via `check_drift.py`) |
| `proptest` invariants | 19 property tests (BSP, WFC, noise, engagement, flow, Fitts, Hick, JSON-RPC, capability parsing, DispatchOutcome) |
| `#![forbid(unsafe_code)]` | All crate roots + all binaries |
| `#[allow()]` in application code | 0 — all lint suppressions use `#[expect(reason)]` with curated reasons (60+ sites); `#[allow(unwrap_used)]` reserved for `#[cfg(test)]` modules only |
| `llvm-cov` (library) | 91.27% line coverage (90% floor enforced in CI, binaries excluded) |
| CI pipeline | `.github/workflows/ci.yml` — fmt, clippy, test (barracuda + forge), doc (workspace), cargo deny |
| SPDX headers | All `.rs` + all `Cargo.toml` |
| Error handling | `thiserror` — all error types derive `thiserror::Error` |
| Files > 1000 LOC | 0 — handlers split into 5 submodules, exp030 into 4 modules |
| TODO/FIXME/HACK in source | 0 |
| Structured logging | `tracing` for all library IPC/biomeOS; `ValidationSink` trait for validation output |
| Hardcoded primal names | 0 — `discover_primals()` by capability, `viz_register()` parameterized, zero name literals |
| Hardcoded paths | 0 — `LUDOSPRING_OUTPUT_DIR` env var + `temp_dir()` + XDG-compliant socket chain |
| IPC integration tests | 23 tests (lifecycle, capability list, game methods, error handling, neural bridge, discovery, push client, 7 composition parity, 5 degradation) |
| MCP support | `tools.list` + `tools.call` for AI integration (15 tool descriptors: 8 science + 7 delegation) |
| tarpc option | `tarpc-ipc` feature with `LudoSpringService` trait mirroring JSON-RPC surface |
| GPU tolerances | Named constants in `tolerances::gpu` + `tolerances::validation` (single source of truth; raycaster tolerances re-exported from validation where shared) |
| Validation infrastructure | `check_abs_or_rel`, `exit_skipped` (exit 2), `load_baseline_f64`, `OrExit<T>` |

## Evolution History (V17–V63)

Detailed version history is in `CHANGELOG.md`. Key milestones:

| Version | Date | Milestone |
|---------|------|-----------|
| V67 | May 12 | Tier 2 convergence: `toadstool.validate` + `precision.route` wired, Pass 14 unblocked |
| V66 | May 12 | `--format json` Tier 2 readiness, barraCuda v0.4.0, params GPU split |
| V65 | May 12 | Foundation Thread 9+10 expressions, notebook CI verification |
| V64 | May 11 | `default = []`, coralReef IPC wired, domain method parity |
| V63 | May 11 | SPDX on all 122 `.rs` files, `unreachable!` eliminated, doc alignment |
| V62 | May 11 | Tier 4 IPC-first defaults (`default = ["ipc"]`), Foundation Thread 10 seeded |
| V61 | May 11 | 29 constant-invariant tests, GAP-12/13/15 RESOLVED, deep debt zero |
| V60 | May 11 | skunkBat Rust IPC, 8 validation scenarios, Foundation Thread 9 seeded |
| V59 | May 10 | Tier 4 rewiring (barracuda optional), biomeOS v3.51 absorbed, `crate::math` dual-path |
| V58 | May 9 | UniBin eukaryotic evolution, certification organelle, 100 experiments fossilized |
| V55 | Apr 27 | Deep debt: zero `#[allow]`, zero `Result<_, String>`, `ipc::methods` 10 modules |
| V52 | Apr 25 | `game.tick` composite handler, composition loop |
| V32 | Mar 29 | Full audit: provenance integrity, tolerance centralization, 110 files remediated |
| V30 | Mar 23 | Handler refactor (1208→5×300 LOC), UniBin consolidation, MCP tools, 91% coverage |
| V24 | Mar 17 | Ecosystem absorption: 8 patterns from 7 springs, `OrExit<T>`, health probes |
| V17 | Mar 15 | Foundation: `niche.rs` SSOT, `NeuralBridge`, 11 WGSL shaders, `#[forbid(unsafe_code)]` |

## Benchmark Gaps (Documented)

### Python-vs-barraCuda CPU Execution Timing
Python baselines validate **correctness parity** only — they produce reference
values that the Rust implementation must match. exp034 measures Rust-only
throughput; the "inline-python" comparison is Rust code that mirrors Python logic.

The flow is: run Python → capture JSON → transcribe values into Rust tests → run Rust tests. There is no automated single-run Python-vs-barraCuda CPU comparison. `combined_baselines.json` is not loaded by default in experiments; `validation::load_baseline_f64` is available for opt-in runtime loads and is covered by unit tests in `barracuda/src/validation/mod.rs`.

### Industry GPU Benchmarks
GPU validation (exp030) confirms CPU-vs-GPU **correctness parity** via wgpu/WGSL.
There are no benchmarks against industry GPU frameworks (Kokkos, CUDA, OpenCL, cuBLAS, Galaxy). GPU performance parity against industry standards is a toadStool/coralReef concern — ludoSpring validates correctness, not throughput.

Industry benchmark targets for future work:
- **Math primitives**: cuBLAS (gemm, gemv), Kokkos (parallel reduce, scan) for barraCuda GPU ops
- **Noise generation**: libnoise, FastNoiseLite for Perlin/fBm throughput comparison
- **Raycasting**: Vulkan raytracing extensions for DDA parity
- **Constraint solving**: Gecode, MiniZinc for WFC propagation speed

## License

### Triple License

This repository follows the **scyBorg provenance trio** standard.

- **Software/code:** AGPL-3.0-or-later — see [`LICENSE`](LICENSE).
- **Game mechanics:** ORC (Open RPG Creative) — see [`LICENSE-ORC`](LICENSE-ORC).
- **Documentation/creative:** CC-BY-SA-4.0 — see [`LICENSE-CC-BY-SA`](LICENSE-CC-BY-SA).
