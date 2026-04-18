# ludoSpring baseCamp — Game Design as Rigorous Science

**Date:** April 17, 2026
**Paper:** #17 in ecoPrimals baseCamp (gen3)
**Status:** V44 — 100 experiments, 30 JSON-RPC capabilities (27 game/health + 3 infrastructure), 790+ workspace tests. Level 5 primal proof operational: `validate_primal_proof` calls 10 barraCuda IPC methods (Fitts, Hick, sigmoid, log2, mean, std_dev, Perlin, rng, tensor, health) against Python golden values. Four-layer validation: Python→Rust→IPC composition→primal proof. `lifecycle.composition` handler wired. Capability-first discovery (`by_capability` → name fallback). Fragments: `tower_atomic`, `node_atomic`, `nest_atomic`, `meta_tier`. Provenance unified to single commit `19e402c0`. ecoBin harvested to plasmidBin (v0.10.0, sha256-verified). 10 primal gaps tracked (GAP-01–GAP-10, GAP-02 PARTIAL).

---

## Paper 17: Game Design as Rigorous Science — Validated HCI Models for Interactive Systems

### Abstract

Games are the most demanding real-time interactive systems humans build. This paper
validates 13 foundational models from HCI research — Fitts's law (1954), Hick's law
(1952), Steering law (1997), GOMS (1983), Flow theory (1990), Dynamic Difficulty
Adjustment (2005), Four Keys to Fun (2004), Engagement metrics (2018), Perlin noise
(1985), Wave Function Collapse (2016), L-systems (1968), BSP trees (1980), and
Tufte data-ink analysis (1983) — through the ecoPrimals Python→Rust→GPU evolution
pipeline.

### Key Finding

Game genres are interaction architectures, not aesthetic categories. FPS maps to
molecular explorer, RTS maps to systems biology dashboard, roguelike maps to
parameter space exploration. This structural correspondence means ludoSpring's
validated HCI models benefit every primal in the ecosystem.

### Validation Summary

| Track | Models | Experiments | Checks |
|-------|--------|-------------|--------|
| Core Game Systems | Raycaster, voxel, Tufte | 001–004 | 22 |
| Interaction Models | Fitts, Hick, Steering, GOMS, Flow | 005–007, 011–012, 019 | 47 |
| Procedural Generation | Noise, WFC, L-systems, BSP | 008–009, 013–014, 017 | 46 |
| Accessibility/Cognitive | Motor-limited Fitts, Tufte sweep | 015–016 | 16 |
| Fun & Engagement | Engagement, Four Keys, DDA, retention | 010, 018, 020–022 | 52 |
| Compute Dispatch | CPU-GPU parity, routing, mixed hw, NUCLEUS | 030–033 | 71 |
| Benchmark Validation | Python parity, noise BM-002, raycaster BM-003, tick budget | 034–037 | 45 |
| External Controls | External roguelike, 3-way noise, quality discrimination | 038–040 | 36 |
| Cross-Spring | NCBI QS pipeline, Tower Atomic, QS gene dataset, Anderson QS explorer | 041–044 | 44 |
| RPGPT Sovereign RPG | Ruleset control, text adventure DAG, MTG card provenance | 045–047 | 105 |
| Games@Home | Stack resolution folding, novel data combinatorics, game tree metrics, distributed computation | 048–051 | 127 |
| Provenance Trio | rhizoCrypt DAG + loamSpine certs + sweetGrass attribution wired into game sessions | 052 | 37 |
| Extraction Shooter | 12 fraud types, zone topology, spatial cheats, consumable lifecycle, per-round provenance | 053 | 65 |
| Composable Viz | biomeOS graph, songbird discovery, petalTongue DataBinding — zero chimeric deps | 054 | 40 |
| Lysogeny | Usurper, Integrase, Symbiont, Conjugant, Quorum, Pathogen — open recreation from prior-art math | 055–060 | 237 |
| Fermenting | Full NFT lifecycle: mint, trade, loan, consume, achievements, atomic swap, trio integration, IPC | 061 | 89 |
| BearDog-Signed Chain | Ed25519 signing on every trio operation, chain verification, tamper detection | 064 | 39 |
| Field Sample Provenance | wetSpring scaffold — sample lifecycle, 6 fraud types, DAG isomorphism | 062 | 39 |
| Medical Access | healthSpring scaffold — consent-gated access, 5 fraud types, zero-knowledge proofs | 063 | 35 |
| Cross-Domain Fraud | Same GenericFraudDetector across gaming/science/medical (>80% similarity) | 065 | 74 |
| Radiating Attribution | sunCloud value distribution — decay, role weighting, conservation of attribution | 066 | 41 |
| RPGPT Dialogue Plane | NPC knowledge bounds, lie detection, memory DAG, ruleset hot-swap, multi-voice, trust, factions, plane transitions | 067–075 | 321 |
| IPC Composition Parity | Rust library == IPC within analytical tolerance (1e-10) | 099 | 13 |
| NUCLEUS Composition Parity | Python → Rust → IPC → Primal golden chain, niche integrity, capability discovery | 100 | 27 |

### Composition Evolution V42

V42 completes the evolution from **validation spring** to **composition spring**:
Python was the validation target for Rust. Now both Python and Rust are validation
targets for the ecoPrimal NUCLEUS composition patterns. The golden chain traces a
single computation (e.g. Fitts cost) from the published paper's formula through
Python baseline → Rust library → IPC serialization → NUCLEUS primal graph.

```
Layer 1: Python ↔ Rust    (python_parity.rs — parity vs Python baselines)
Layer 2: Rust ↔ IPC       (exp099: 13/13, analytical tolerance 1e-10)
Layer 3: NUCLEUS          (exp100: 27 checks — niche integrity, health, capability, science, golden chain)
Layer 4: Composition      (lifecycle.composition — runtime probe of all 11 niche dependencies via JSON-RPC)
```

**V42 composition evolution:**
- `lifecycle.composition` handler — `CompositionReport` externally callable via JSON-RPC
- Capability-first discovery — `by_capability` → name fallback in composition probing
- `nest_atomic` in declared fragments — aspirational until GAP-06/07 resolve (trio `required: false`)
- Provenance unified to single commit `19e402c0` across 88 files
- ecoBin banned-crate enforcement — `deny.toml` denies 8 C deps per ecoBin v3.0

**V41 patterns absorbed from primalSpring ecosystem:**
- `IpcErrorPhase` + `PhasedIpcError` — retry/recovery classification for IPC failures
- Method normalization — strips biomeOS/peer prefixes before dispatch
- Three-tier dispatch — lifecycle → infrastructure → science routing
- Tiered discovery — 6-tier socket resolution chain with structured `DiscoveryResult`
- `NicheDependency` table — 11 typed proto-nucleate entries in `niche.rs`
- `CompositionReport` — runtime probe of all dependencies, reports live/absent/degraded
- Typed `inference.*` wire types — ready for neuralSpring WGSL ML evolution
- `--port` CLI flag — plasmidBin/orchestrator binding

These patterns make ludoSpring a **reference implementation** for how springs absorb
and validate composition standards. Other springs can use ludoSpring as a template
for their own composition evolution.

Key artifacts:
- **`config/capability_registry.toml`** — Machine-readable SSOT for ludoSpring capabilities, semantic mappings, external dependencies, and proto-nucleate graph reference
- **exp100** (`ludospring-exp100`) — 27-check NUCLEUS composition parity validator: niche integrity (7), health probes (2), capability discovery (4), science parity (8), golden chain (6)
- **ecoBin v0.10.0** — Harvested to `infra/plasmidBin/ludospring/` (sha256-verified)
- **Shared HUD fixtures** — `hud_fixtures.rs` extracted from dashboard binaries, eliminating duplication
- **Centralized dialogue constants** — `D6_SUCCESS_THRESHOLD`, `DIALOGUE_EMA_ALPHA` in `tolerances::game`
- **CI coverage** — `cargo-llvm-cov` at 90% floor enforced in `.github/workflows/ci.yml`

### Four-Layer Validation — Python → Rust → IPC → Primal Proof (V44)

The validation lifecycle now extends beyond Python↔Rust parity to prove
that peer-reviewed science works identically when composed from NUCLEUS
primals via IPC:

| Layer | Source | Target | Guard |
|-------|--------|--------|-------|
| 1 | Python baselines | Rust library | `python_parity.rs` + `check_drift.py` |
| 2 | Rust library | Golden JSON (`composition_targets.json`) | `composition_parity.rs` (6 tests) |
| 2.5 | Golden JSON | Library recomputation | `check_composition_drift` (CI) |
| 3 | Golden JSON | IPC composition | `validate_composition` binary |
| 5 | Python golden values | barraCuda IPC (primal proof) | `validate_primal_proof` binary |

**Level 5 primal proof (V44):** `validate_primal_proof` calls barraCuda's
JSON-RPC UDS socket directly — 10 IPC methods (`activation.fitts`,
`activation.hick`, `math.sigmoid`, `math.log2`, `stats.mean`, `stats.std_dev`,
`noise.perlin2d`, `rng.uniform`, `tensor.create`, `health.liveness`) compared
against the same Python golden values used for Level 2. Exit 0/1/2. This
proves: peer-reviewed science → Python → Rust → primal IPC = PASS.

**Golden chain example (Fitts's law):**
- Python `interaction_laws.py` computes `log2(100/10 + 1) * 150 + 50 = 543.43` ms
- Rust `fitts_movement_time(100.0, 10.0, 50.0, 150.0)` matches within `1e-10`
- `composition_targets.json["game.fitts_cost"]["mouse_d100_w10"]` stores `543.43`
- `validate_composition` calls `game.fitts_cost` over IPC → same `543.43`
- `validate_primal_proof` calls barraCuda `activation.fitts` → same `708.85` (Level 5)

Each layer independently validates the one below it. Drift at any layer is
caught by the guard (test or CI check) before it propagates.

### RPGPT Dialogue Plane (V18–V19)

Nine experiments implementing the first playable plane of the RPGPT system:

- **NPC personality certs** (loamSpine): motivations, knowledge bounds, voice, secrets, relationships, arc
- **Internal voices** (Squirrel AI): 10 Disco Elysium-style skills as constrained AI perspectives
- **Trust dynamics**: multi-factor disposition gates on knowledge sharing
- **Plane transitions**: 7 game modes (Exploration, Dialogue, Tactical, Investigation, Political, Crafting, Card/Stack) — state preserved across transitions
- **2D engine primitives**: TileWorld, EntityRegistry, ActionOutcome, NarrationCue pipeline
- **GPU compute**: fog of war, tile lighting, pathfinding wavefront, Perlin terrain — via toadStool/barraCuda WGSL shaders
- **Audio narration**: blind-accessible gameplay — every state change produces semantic narration cues

### Ecosystem Absorption V22

- **toadStool direct dispatch**: 3 new `compute.dispatch.*` methods for low-latency real-time GPU compute
- **Dual-format capability discovery**: handles both flat array and nested object `lifecycle.status` responses (neuralSpring S156 fix)
- **Python tolerance mirror**: `baselines/python/tolerances.py` with 46 named constants (wetSpring V121 pattern)
- **Write→Absorb→Lean documentation**: `procedural::noise` absorption status documented (2D absorbed, 3D pending)
- **Deploy graph evolution**: `compute.dispatch.submit/result/capabilities` in toadStool node

### Deep Debt Evolution V21 (preserved)

- Session decomposition: `GameSession::resolve()` extracted into per-command methods, eliminating `#[allow(clippy::too_many_lines)]`
- **Typed transition verification**: boolean fields → `TransitionIssue` enum + `Vec<TransitionIssue>`, eliminating `#[allow(clippy::struct_excessive_bools)]`
- **Pluggable validation output**: `ValidationSink` trait with `StderrSink` (default) and `BufferSink` (testing); `ValidationHarness<S>` generic over sink
- **Typed toadStool IPC client**: `ipc/toadstool.rs` — `submit_workload`, `workload_status`, `query_capabilities` with graceful degradation
- **IPC integration tests**: 6 tests covering lifecycle, capability list, game methods, error handling
- **`#[expect]` evolution**: `#[allow(dead_code)]` → `#[expect(dead_code, reason = "...")]` for justified IPC wire types
- **Platform-agnostic paths**: `temp_dir()` replaces hardcoded `/tmp` in test fixtures
- **Centralized game tolerance**: `GAME_STATE_TOL` replaces inline `0.01` across experiments
- **ValidationHarness adoption**: `exp001` rewritten from legacy `ValidationResult` to `ValidationHarness` + `BaselineProvenance`

### Deep Primal Integration V20 (preserved)

- IPC method alignment: 19 external methods aligned to canonical JSON-RPC specs
- Capability domains registry: 24 capabilities (10 local, 14 external)
- Tolerance decomposition: 6 domain-specific submodules
- Typed provenance pipeline: `DehydrationSummary` + `TrioStage`
- Game engine core: `RulesetCert` validation, concrete `apply()`, `GridMap` bridge
- Runtime discovery: `discover_by_capability()` for primal peer lookup

### Deep Debt Evolution (V19, preserved)

- Magic numbers eliminated: 9 tolerance constants with provenance citations
- Clone abuse removed: `&serde_json::Value` constructors; 13 `.clone()` calls eliminated
- Production panic removed: `BlockPalette::register()` → `Result<BlockId, String>`
- Provenance decomposed: 773-line monolith → 3 focused submodules
- Audio refactored: `compile_outcome` → 5 focused functions

### Cross-Spring Provenance

- **Python baselines** (7 scripts, stdlib only) → `barracuda/tests/python_parity.rs` (47 tests) + `check_drift.py` (automated drift detection)
- **barraCuda primitives** consumed: `sigmoid`, `dot`, `lcg_step`, `state_to_f64`
- **Tolerances** centralized with citations in `tolerances/mod.rs` (20 named constants, `RAYCASTER_HIT_RATE_TOL` tightened 20→5)
- **Proptest invariants** (12 tests): BSP area conservation, WFC entropy monotonicity, noise boundedness, engagement normalization, Fitts/Hick monotonicity, flow exhaustive partition
- **Structured tracing**: all library IPC/biomeOS uses `tracing` (no `eprintln!` in production)
- **Zero `#[allow()]`** in production — all clippy lints centralized in `Cargo.toml`
- **WGSL shaders extracted**: 11 standalone `.wgsl` files in `exp030/shaders/` for toadStool absorption
- **petalTongue** integration: 3 dashboard binaries, all 7 `GameChannelType` channels wired
- **GPU promotion**: 8 modules Tier A (pure math, embarrassingly parallel). Tier A WGSL shaders validated in exp030 (Perlin 2D, fBm, engagement batch, DDA raycaster — 32/32 GPU parity checks). metalForge evolved to capability-based routing (SubstrateKind, Capability, route(), fallback_chain). NPU→GPU direct PCIe transfer model validated. toadStool JSON-RPC 2.0 dispatch client wire format validated. biomeOS DeploymentGraph (5-node, 60Hz budget) validated.
- **NCBI integration**: Direct E-utilities access (esearch, esummary) for QS gene data — nestgate provider documented but needs module wiring
- **NUCLEUS atomics**: Tower Atomic (BearDog + Songbird) validated via JSON-RPC 2.0 over Unix sockets
- **wetSpring cross-spring**: Anderson QS disorder model (W = 3.5H' + 8.0·O₂) with Perlin noise landscapes and game metrics
- **Provenance trio**: rhizoCrypt DAG + loamSpine certificates + sweetGrass attribution directly imported as Cargo dependencies (data primals are direct deps, infrastructure primals are IPC-only)
- **Extraction shooter**: 12 fraud types across 3 tiers — basic, consumable, spatial — zone topology model catches cheats structurally
- **Composable architecture**: biomeOS `DeploymentGraph`, songbird discovery, petalTongue `DataBinding` — all via JSON-RPC 2.0 protocol types defined locally (zero chimeric dependencies)

### Connection to Constrained Evolution Thesis

ludoSpring demonstrates that constrained tools (Rust, GPU via barraCuda, validated
against published papers) produce validated science in a domain (game design) far
removed from the thesis's biological focus. The structural correspondence between
game genres and scientific visualization paradigms confirms the thesis's prediction
that constrained evolution produces transferable specializations.

### Faculty Anchors

- Fitts (1954), Hick (1952), Accot & Zhai (1997) — empirical HCI laws
- Card, Moran, Newell (1983) — GOMS/KLM cognitive model
- Csikszentmihalyi (1990) — Flow theory
- Hunicke (2005) — Dynamic Difficulty Adjustment
- Lazzaro (2004) — Four Keys to Fun
- Yannakakis & Togelius (2018) — Computational game science
- Perlin (1985, 2002), Gumin (2016), Lindenmayer (1968), Fuchs (1980) — PCG
- Tufte (1983, 1990) — Information design

## baseCamp Expeditions

| Exp | Title | What it proves | Doc |
|-----|-------|---------------|-----|
| 023 | Open-Systems Benchmark | ludoSpring vs fastnoise-lite, WFC crate, Bevy ECS | `exp023_benchmarks.md` |
| 024 | Doom-in-a-Terminal | Validated raycaster + BSP = playable first-person game | `exp024_doom_terminal.md` |
| 025 | Roguelike Explorer | Engagement-driven PCG with DDA, Flow, fun classification | `exp025_roguelike_explorer.md` |
| 026 | Game Telemetry Protocol | Portable NDJSON event protocol + analysis pipeline | `exp026_game_telemetry.md` |
| 027 | Veloren Adapter | SPECS ECS log parser -> ludoSpring telemetry | `exp027_veloren_adapter.md` |
| 028 | Fish Folk Adapter | Bevy plugin pattern for multiplayer PvP analysis | `exp028_fishfolk_adapter.md` |
| 029 | A/B Street Adapter | Simulation-as-game: city planning analyzed as gameplay | `exp029_abstreet_adapter.md` |
| 030 | CPU-vs-GPU Parity | Pure Rust math matches GPU WGSL shaders (Tier A: Perlin, fBm, engagement, raycaster) | `exp030_cpu_gpu_parity.md` |
| 031 | Dispatch Routing | Real wgpu adapter discovery + workload routing validation | `exp031_dispatch_routing.md` |
| 032 | Mixed Hardware | PCIe transfer cost, NPU→GPU direct bypass, 4-stage mixed pipeline, TransferPath model | `exp032_mixed_hardware.md` |
| 033 | NUCLEUS Pipeline | Tower/Node/Nest atomics + capability routing + toadStool dispatch + biomeOS graph | `exp033_nucleus_pipeline.md` |
| 034 | Python-Rust Parity | barraCuda CPU = Python math, Rust faster than interpreted | `exp034_python_parity_bench.md` |
| 035 | Noise Throughput (BM-002) | 13.1M samples/s Perlin, 0.93x fastnoise-lite | `exp035_noise_throughput.md` |
| 036 | Raycaster Throughput (BM-003) | 6,623 FPS DDA raycaster, 110x 60Hz target | `exp036_raycaster_throughput.md` |
| 037 | Tick Budget Validation | 10K entities in 910us, 70% headroom under 3ms budget | `exp037_tick_budget.md` |
| 038 | External Roguelike Control | Metrics work on foreign content (bracket-pathfinding) | `exp038_external_roguelike_control.md` |
| 039 | Noise Cross-Validation | 3-way comparison: ours fastest (0.93x fastnoise-lite) | `exp039_noise_cross_validation.md` |
| 040 | Quality Discrimination | Flow discriminates quality: 4/5 good in flow, 5/5 bad not | `exp040_quality_discrimination.md` |
| 041 | NCBI QS Integration | Live NCBI E-utilities: luxI/luxS/agrB gene search, SRA metagenomes | — |
| 042 | Tower Atomic Local | BearDog crypto.hash + Songbird IPC via JSON-RPC 2.0 Unix sockets | — |
| 043 | QS Gene Dataset | 6 QS gene families × 20 gut genera — AI-2 dominant in gut | — |
| 044 | Anderson QS Explorer | Cross-spring: Perlin disorder landscapes, QS propagation, game metrics | — |
| 045 | Ruleset Control Systems | PF2e, FATE Core, Cairn ingested as loamSpine certs; action economy validated | — |
| 046 | Text Adventure DAG | Session DAG with branching narrative, rhizoCrypt vertex tracking | — |
| 047 | MTG Card Provenance | Card mint/trade/transform lifecycle with loamSpine certs + sweetGrass attribution | — |
| 048 | Stack Resolution Folding | Card stack ≡ protein folding: same components, different order → different outcomes | — |
| 049 | Novel Data Combinatorics | Game tree ~10^358 (MTG), birthday bound ~10^179 — every game is novel data | — |
| 050 | Game Tree Design Metric | Tree complexity as measurable design metric; Commander hypothesis validated | — |
| 051 | Games@Home | Folding@Home isomorphism: 12 concepts mapped 1:1, 7 cross-domain transfers (avg 76%) | — |
| 052 | Provenance Trio Integration | rhizoCrypt DAG + loamSpine certs + sweetGrass braids wired into game sessions | — |
| 053 | Extraction Shooter Provenance | 12 fraud types, zone topology, spatial detection, consumable lifecycle tracking | — |
| 054 | Composable Raid Visualization | biomeOS graph + songbird discovery + petalTongue viz — zero chimeric deps | — |
| 055 | Usurper (Nemesis System) | Replicator dynamics + spatial PD + Lotka-Volterra with memory = persistent adaptive NPCs | — |
| 056 | Integrase (Capture) | Wright-Fisher fixation + QS threshold + Markov chains = capture probability | — |
| 057 | Symbiont (Faction/Reputation) | Multi-species Lotka-Volterra + frequency-dependent fitness = faction dynamics | — |
| 058 | Conjugant (Roguelite) | HGT + Wright-Fisher + Price equation + Red Queen = meta-progression | — |
| 059 | Quorum (Emergent Narrative) | Agent-based + Markov + DAG causality + QS threshold = procedural story | — |
| 060 | Pathogen (Gacha Anti-Pattern) | Operant conditioning + prospect theory + parasitism = exploitation quantification | — |
| 061 | Fermenting System | Full NFT lifecycle: mint/trade/loan/return/consume/achievements + trio + IPC wire format | — |
| 062 | Field Sample Provenance | wetSpring scaffold: Collect→Publish, 6 fraud types, DAG isomorphism with exp053 | — |
| 063 | Consent-Gated Medical Access | healthSpring scaffold: patient-owned records, consent lending, 5 fraud types, ZK proofs | — |
| 064 | BearDog-Signed Provenance Chain | Ed25519 signing on all trio operations, chain verification, tamper detection at exact point | — |
| 065 | Cross-Domain Fraud Unification | Same GenericFraudDetector across gaming/science/medical, >80% structural similarity | — |
| 066 | Radiating Attribution Calculator | sunCloud value distribution: decay models, role weighting, conservation (shares=1.0) | — |
| 067 | NPC Knowledge Bounds | Four-quadrant knowledge model, metadata-only responses for unknown topics | — |
| 068 | Lie Detection / Passive Checks | NPC deception with detection DCs, behavioral tells, passive perception | — |
| 069 | Internal Voice Personality | 10 Disco Elysium-style skill voices as constrained AI perspectives | — |
| 070 | Voice Priority / Concurrency | Priority ordering, max 3 voices per action, deterministic selection | — |
| 071 | NPC Memory DAG | Graph-aware memory retrieval, emotional tagging, recency/relevance scoring | — |
| 072 | Trust Dynamics Arc | Multi-factor disposition (faction+personal+relationship+debt), trust gates | — |
| 073 | Dialogue Skill Checks | D6 pool resolution, 5-degree outcomes, binomial distribution validation | — |
| 074 | Dialogue Flow Monitoring | Flow/DDA/Hick integration with dialogue pacing, stall detection | — |
| 075 | Plane Transition Continuity | 7 game modes, state preservation, condition mapping, round-trip verification | — |

### Barrier Removal Philosophy

Digital music expanded the field by removing barriers — more musicians, not fewer.
ludoSpring follows the same principle:

- **Validate from science** but make tools extensible beyond games
- **AGPL-3.0** ensures anyone can extend: musicians, educators, architects, indie devs
- **Terminal rendering** (ratatui) = zero GPU dependency, runs on any SSH session
- **Deterministic seeding** (LCG) = reproducible results across all platforms

The same WFC that generates dungeons can compose music (harmonic adjacency).
The same DDA that tunes monster density can tune exam difficulty.
The same Fitts's law that scores HUD reachability can evaluate any clickable UI.

### How to Reproduce

```bash
cd ludoSpring
python3 baselines/python/run_all_baselines.py       # Python reference data
cargo test --features ipc -p ludospring-barracuda --lib --tests  # part of 790+ workspace tests (V44)
cargo run --bin exp023_open_systems_benchmark        # benchmark: 16/16 checks
cargo run --bin exp024_doom_terminal                 # playable Doom walker
cargo run --bin exp025_roguelike_explorer            # playable roguelike
cargo run --bin exp026_game_telemetry -- validate   # telemetry protocol: 13/13 checks
cargo run --bin exp027_veloren_adapter -- validate  # Veloren adapter: 9/9 checks
cargo run --bin exp028_fishfolk_adapter -- validate # Fish Folk adapter: 7/7 checks
cargo run --bin exp029_abstreet_adapter -- validate # A/B Street adapter: 8/8 checks
cargo run --bin exp030_cpu_gpu_parity               # CPU-vs-GPU parity: 32/32 checks
cargo run --bin exp031_dispatch_routing              # dispatch routing: 10/10 checks
cargo run --bin exp032_mixed_hardware                # mixed hardware: 18/18 checks
cargo run --bin exp033_nucleus_pipeline              # NUCLEUS pipeline: 19/19 checks
cargo run --bin exp034_python_parity_bench           # Python parity: 15/15 checks
cargo run --bin exp035_noise_throughput              # BM-002 noise: 10/10 checks
cargo run --bin exp036_raycaster_throughput          # BM-003 raycaster: 10/10 checks
cargo run --bin exp037_tick_budget                   # tick budget: 10/10 checks
cargo run --bin exp038_external_roguelike_control    # external control: 12/12 checks
cargo run --bin exp039_noise_cross_validation        # noise cross-val: 12/12 checks
cargo run --bin exp040_quality_discrimination        # quality discrim: 12/12 checks
cargo run --release -p ludospring-exp041 -- validate # NCBI QS integration: 12/12 checks
cargo run --release -p ludospring-exp042 -- validate # Tower Atomic local: 10/10 checks
cargo run --release -p ludospring-exp043 -- validate # QS gene dataset: 10/10 checks
cargo run --release -p ludospring-exp044 -- validate # Anderson QS explorer: 12/12 checks
cargo run --release -p ludospring-exp045 -- validate # Ruleset control systems: 49/49 checks
cargo run --release -p ludospring-exp046 -- validate # Text adventure DAG: 33/33 checks
cargo run --release -p ludospring-exp047 -- validate # MTG card provenance: 23/23 checks
cargo run --release -p ludospring-exp048 -- validate # Stack resolution folding: 36/36 checks
cargo run --release -p ludospring-exp049 -- validate # Novel data combinatorics: 33/33 checks
cargo run --release -p ludospring-exp050 -- validate # Game tree design metric: 30/30 checks
cargo run --release -p ludospring-exp051 -- validate # Games@Home: 28/28 checks
cargo run --release -p ludospring-exp052 -- validate # Provenance trio integration: 37/37 checks
cargo run --release -p ludospring-exp053 -- validate # Extraction shooter provenance: 65/65 checks
cargo run --release -p ludospring-exp054 -- validate # Composable raid visualization: 40/40 checks
cargo run --release -p ludospring-exp055 -- validate # Usurper: Nemesis system: 48/48 checks
cargo run --release -p ludospring-exp056 -- validate # Integrase: capture mechanics: 47/47 checks
cargo run --release -p ludospring-exp057 -- validate # Symbiont: faction reputation: 35/35 checks
cargo run --release -p ludospring-exp058 -- validate # Conjugant: roguelite meta-progression: 40/40 checks
cargo run --release -p ludospring-exp059 -- validate # Quorum: emergent narrative: 39/39 checks
cargo run --release -p ludospring-exp060 -- validate # Pathogen: gacha anti-pattern: 28/28 checks
cargo run --release -p ludospring-exp061 -- validate # Fermenting: full lifecycle: 89/89 checks
cargo run --release -p ludospring-exp062              # Field sample provenance: 39/39 checks
cargo run --release -p ludospring-exp063              # Consent-gated medical access: 35/35 checks
cargo run --release -p ludospring-exp064              # BearDog-signed chain: 39/39 checks
cargo run --release -p ludospring-exp065              # Cross-domain fraud unification: 74/74 checks
cargo run --release -p ludospring-exp066              # Radiating attribution: 41/41 checks
cargo run -p ludospring-exp067                        # NPC knowledge bounds: 38/38 checks
cargo run -p ludospring-exp068                        # Lie detection: 21/21 checks
cargo run -p ludospring-exp069                        # Internal voices: 75/75 checks
cargo run -p ludospring-exp070                        # Voice priority: 25/25 checks
cargo run -p ludospring-exp071                        # NPC memory DAG: 26/26 checks
cargo run -p ludospring-exp072                        # Trust dynamics: 45/45 checks
cargo run -p ludospring-exp073                        # Dialogue skill checks: 34/34 checks
cargo run -p ludospring-exp074                        # Dialogue flow: 26/26 checks
cargo run -p ludospring-exp075                        # Plane transitions: 31/31 checks
cargo run --features ipc --bin ludospring -- dashboard  # petalTongue visualization
```

---

## Paper 18: RPGPT — Sovereign RPG Engine with Ingestible Rulesets

### Concept

The provenance trio (rhizoCrypt, sweetGrass, loamSpine) serves as the state engine for a
tabletop RPG system where any open ruleset can be ingested as a loamSpine certificate and
combined with any world to produce a playable RPG. The player acts as their own DM —
designing the world, quest hooks, NPC templates — then AI (Squirrel) assists with narration
constrained by the provably anchored ruleset.

### The Isomorphism

Anti-cheat in games is chain-of-custody in science. Same DAG, same BLAKE3 Merkle integrity,
same BearDog signing — different vocabulary:

| DAG Operation | Extraction Shooter | Field Genomics | Tabletop RPG |
|---------------|-------------------|----------------|-------------|
| Object creation | Item spawns | Sample collected | Sword found in chest |
| Object transform | Weapon modded | DNA amplified | Sword enchanted |
| Object transfer | Item traded | Sample to lab tech | Sword gifted |
| Audit | No item without loot vertex | No reads without sample vertex | No loot without roll vertex |

### Ingestible Rulesets

Any ORC/CC-BY licensed ruleset becomes a loamSpine certificate:

| System | License | Structural Gift |
|--------|---------|----------------|
| Pathfinder 2e | ORC | 3-action economy, 4 degrees of success, conditions, proficiency |
| FATE Core | CC-BY | Aspects (narrative tags → sweetGrass semantic entities), Fate Points |
| Powered by the Apocalypse | CC-BY | Moves, partial success, GM principles |
| Cypher System | Open License | Single target number, GM intrusions, effort/edge |

Any world + any ruleset = playable RPG. Lord of the Rings + PF2e. Dune + FATE. Original world + Cypher.

### Primal Roles

| Primal | Role in RPGPT | Cross-Domain Benefit |
|--------|--------------|---------------------|
| rhizoCrypt | Session DAG (turns, rolls, branches) | Multi-day field campaigns |
| loamSpine | Ruleset/character/NPC/world certs | Experimental protocol certs |
| sweetGrass | Player/AI creative attribution | Multi-lab collaboration tracking |
| Squirrel | AI narration constrained by ruleset cert | — |
| BearDog | Anti-cheat action signing | Sample chain-of-custody |
| ludoSpring | Flow/DDA/engagement session quality | Patient engagement metrics |

### Build Phases

1. Ruleset-as-certificate format (PF2e mechanics → loamSpine cert)
2. Session DAG (turn structure, conditions, phases in rhizoCrypt)
3. AI narration loop (Squirrel + ruleset cert + ludoSpring quality metrics)
4. Attribution + economics (sweetGrass + sunCloud)

**Full specification**: `ludoSpring/specs/RPGPT_DEEP_SYSTEM_DESIGN.md`
**baseCamp paper**: `gen3/baseCamp/18_rpgpt_sovereign_rpg_engine.md`

---

## Paper 19: Games@Home — Distributed Human Computation via Interactive Systems

### Concept

Human gameplay is distributed computation. Folding@Home uses volunteer CPUs to explore protein conformational space. Games@Home uses volunteer humans to explore infinite game decision trees — and humans bring creativity, intuition, and cross-domain pattern recognition that CPUs cannot.

### Stack Resolution as Folding (exp048 — 36/36)

Card text is the genotype. Resolution order is the phenotype. The same two cards (Lightning Bolt + Giant Growth) produce opposite outcomes depending on stack position. This is structurally identical to protein folding: same amino acid sequence, different fold → different function. The stack creates a DAG — each cast is a vertex, each "in response to" is a parent edge.

### Every Game is Novel Data (exp049 — 33/33)

MTG's computed game tree (~10^358 conservatively, 2^ℵ₀ provably — Turing complete) means the birthday bound for any game repeat is ~10^179. Total games ever played: ~10^10.5. Every game session generates data that has literally never existed. The provenance trio tracks all of it.

### Game Tree as Design Metric (exp050 — 30/30)

Game tree complexity is measurable and correlates with game longevity. Go's game tree (~10^505) comes from huge board (361), massive branching (~250), and long games (~211 plies). MTG is categorically beyond all finite games — Turing complete, undecidable.

**The Commander Hypothesis**: Format rules (singleton, 100-card, 4-player, 40 life) expand the tree ×216. Designed-for-commander cards (pre-built synergies, auto-includes, linear designs) contract it ×0.036 — destroying >96% of the format's branching.

**The Enzymatic Shortcut Model**: Wild-type cards (high branching, high activation energy) vs enzymatic cards (low branching, low activation — play on sight) vs catalytic cards (high branching, LOW activation — the ideal design that opens paths while being accessible).

### Games@Home (exp051 — 28/28)

The Folding@Home isomorphism maps 1:1 across 12 concepts (compute unit, search space, trajectory, parameters, output, aggregation, work unit, novelty, quality signal, discovery, attribution, cross-domain value).

Advantages: 200× more compute units (40M players vs 200K F@H CPUs), zero cost (entertainment is self-motivating), infinite search space, full creative attribution via sweetGrass.

Seven cross-domain transfer paths validated (avg 76% similarity):
- Game tree pruning → MCTS heuristics (90%)
- MTG stack resolution → Protein folding (85%)
- MTG meta evolution → Antibiotic resistance (80%)
- Commander deckbuilding → Materials science composition (75%)
- RPG narrative branching → Drug discovery pathways (70%)
- Combo/synergy discovery → Catalyst design (70%)
- Multiplayer politics → Multi-agent logistics (65%)

### AR Card Gaming — Physical-Anchored Digital Enhancement

AR assists physical card games without replacing them:
- **Physical stays physical**: Cards, shuffling, drawing, social interaction, trading
- **Digital overlay**: Life totals, counters, tokens, stack visualization, trigger management, phase tracking
- **loamSpine 1:1 mirror**: Every physical card has a digital certificate (set, number, condition, ownership chain)
- **Remote pod play**: A remote Commander player's physical cards on their table, AR captures board state, opponents see the digital mirror
- **Stack visualization**: LIFO stack (exp048) rendered as visible overlay — reduces rules confusion

**baseCamp paper**: `gen3/baseCamp/19_games_at_home_distributed_human_computation.md`

---

### Cross-Engine Portability

The telemetry protocol is pure JSON — any engine can emit events:

| Engine | Transport | Integration |
|--------|-----------|-------------|
| Rust (direct) | `use ludospring_barracuda::telemetry` | Zero-overhead library call |
| Rust (Bevy) | Bevy plugin `EventReader<T>` -> NDJSON | exp028 pattern |
| Unity (C#) | `File.AppendAllText()` or HTTP POST | JSON serialization |
| Godot (GDScript) | `file.store_line(JSON.stringify())` | JSON serialization |
| Web (JS) | `fetch('/telemetry', ...)` | Standard fetch API |
| Any language | Write NDJSON file | One JSON object per line |
