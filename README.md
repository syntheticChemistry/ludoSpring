# ludoSpring — The Science of Play, Interaction, and Game Design

An ecoPrimals Spring. Treats game design with the same rigor that wetSpring treats bioinformatics and hotSpring treats nuclear physics: validated models, reproducible experiments, GPU-accelerated computation where it matters.

**Date:** March 14, 2026
**Version:** V14 (66 experiments, 1349 validation checks, 212 tests)
**License:** AGPL-3.0-or-later
**MSRV:** 1.87 (edition 2024)
**barraCuda:** v0.3.5 (standalone, 150+ primitives)

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
| `game` | Mechanics, state, genre taxonomy | Raycasting (DDA), voxel worlds, session state | Validated |
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

## Playable Prototypes (baseCamp Expeditions)

These build on validated math — every game mechanic traces to a published paper:

```bash
# Doom-in-a-terminal: BSP levels + DDA raycaster + collision + ratatui
cargo run --bin exp024_doom_terminal

# Roguelike explorer: engagement-driven dungeon with DDA, Flow, fun classification
cargo run --bin exp025_roguelike_explorer

# Open-systems benchmark: compare ludoSpring vs fastnoise-lite, Bevy patterns
cargo run --bin exp023_open_systems_benchmark
```

## Compute Dispatch + metalForge (GPU Parity, Mixed Hardware, NUCLEUS)

Validates the CPU → GPU evolution pipeline and NUCLEUS atomic coordination:

```bash
cargo run --bin exp030_cpu_gpu_parity               # 16/16 CPU-vs-GPU parity checks
cargo run --bin exp031_dispatch_routing              # 10/10 real hardware discovery
cargo run --bin exp032_mixed_hardware                # 12/12 PCIe + mixed pipelines
cargo run --bin exp033_nucleus_pipeline              # 11/11 Tower/Node/Nest atomics
```

## External Control Groups

Validates the metrics framework against real external game libraries:

```bash
cargo run --bin exp038_external_roguelike_control    # 12/12 metrics on foreign content
cargo run --bin exp039_noise_cross_validation        # 12/12 three-way noise comparison
cargo run --bin exp040_quality_discrimination        # 12/12 archetype quality separation
```

Key results:
- **Metrics work on foreign content**: bracket-pathfinding roguelike produces valid engagement, flow, fun, DDA
- **We're the fastest noise impl**: 0.93x fastnoise-lite (C), 2.85x faster than noise-rs
- **Flow discriminates quality**: 4/5 good games in Flow, 5/5 bad games NOT in Flow
- **Scientific finding**: engagement alone doesn't measure quality — you need Flow state (Csikszentmihalyi 1990)

## Cross-Spring Experiments (NCBI, NUCLEUS, Anderson QS)

First experiments bridging ludoSpring game science with wetSpring bioinformatics and biomeOS infrastructure:

```bash
cargo run --release -p ludospring-exp041 -- validate  # 12/12 NCBI QS gene pipeline
cargo run --release -p ludospring-exp042 -- validate  # 10/10 Tower Atomic (BearDog+Songbird)
cargo run --release -p ludospring-exp043 -- validate  # 10/10 QS gene dataset (6 families × 20 genera)
cargo run --release -p ludospring-exp044 -- validate  # 12/12 Anderson QS interactive explorer
```

Key results:
- **Live NCBI integration**: luxI/luxS/agrB gene search, SRA metagenomes, protein databases via E-utilities
- **Biological validation**: gut microbes use AI-2 (luxS) not AHL (luxI) — NCBI data confirms published biology
- **Tower Atomic boot**: BearDog crypto.hash (Blake3, SHA3-256) validated via JSON-RPC over Unix sockets
- **Anderson QS explorer**: Perlin noise as disorder landscape, QS propagation with localization transition, engagement/flow/fun/DDA metrics on microbial exploration. Diversity dominates O2 in the W model (wetSpring Exp356).

## RPGPT — Sovereign RPG Engine (Paper 18)

Architecture defined for a provenance-backed RPG system where **any open ruleset** (Pathfinder 2e, FATE Core, Cypher, PbtA) can be ingested as a loamSpine certificate and combined with **any world** to produce a playable RPG. The player designs the world and quest hooks; AI (Squirrel) narrates within provably anchored rules.

The core insight: **anti-cheat is chain-of-custody**. The same rhizoCrypt DAG that tracks item lineage in extraction shooters tracks sample lineage in field genomics and loot lineage in tabletop RPGs. Same code path, different vocabulary.

| Primal | RPGPT Role |
|--------|-----------|
| rhizoCrypt | Session DAG (turns, rolls, conditions, branches) |
| loamSpine | Ruleset/character/NPC/world certificates |
| sweetGrass | Player/AI creative attribution |
| ludoSpring | Flow/DDA/engagement session quality |
| BearDog | Anti-cheat action signing |
| Squirrel | AI narration constrained by ruleset cert |

See `specs/RPGPT_ARCHITECTURE_SKETCH.md` and `gen3/baseCamp/18_rpgpt_sovereign_rpg_engine.md`.

```bash
cargo run --release -p ludospring-exp045 -- validate  # 49/49 Ruleset control systems (PF2e, FATE, Cairn)
cargo run --release -p ludospring-exp046 -- validate  # 33/33 Text adventure DAG
cargo run --release -p ludospring-exp047 -- validate  # 23/23 MTG card provenance
```

## Games@Home — Distributed Human Computation (Paper 19)

Proves that human gameplay is distributed computation. Stack resolution is protein folding (same components, different order → different outcomes). Game tree complexity is a measurable design metric. Folding@Home isomorphism maps 1:1 across 12 concepts.

```bash
cargo run --release -p ludospring-exp048 -- validate  # 36/36 Stack resolution as folding
cargo run --release -p ludospring-exp049 -- validate  # 33/33 Novel data combinatorics
cargo run --release -p ludospring-exp050 -- validate  # 30/30 Game tree design metric
cargo run --release -p ludospring-exp051 -- validate  # 28/28 Games@Home distributed human computation
```

## Provenance Trio Integration (Track 14)

First direct import of the three provenance primals into ludoSpring. The trio lives
among the biomeOS atomics — deployed via the rootpulse niche as a Continuous 60 Hz graph.

```bash
cargo run --release -p ludospring-exp052 -- validate  # 37/37 Trio integration
```

Key results:
- **rhizoCrypt DAG wired**: game session as vertex graph, content-addressed (Blake3), frontier tracking
- **loamSpine certificates wired**: ruleset (PF2e, FATE) and card (Grizzly Bears, Lightning Bolt) certificates mint correctly
- **sweetGrass braids wired**: PROV-O attribution links game actions to player DIDs with source primal tagging
- **biomeOS topology verified**: 4-node graph (ludoSpring → rhizoCrypt → loamSpine + sweetGrass) fits in 16.67ms tick at 60 Hz
- **Cross-primal round-trip**: vertex hex → braid data hash → DID identity preserved across all three primals

## Extraction Shooter Provenance + Fraud Detection (Track 15)

Models extraction shooters (Tarkov, DMZ, The Cycle) as a provenance problem.
Every raid action is a rhizoCrypt DAG vertex. Every item is a loamSpine certificate.
Fraud detection reduces to checking provenance chain integrity.

```bash
cargo run --release -p ludospring-exp053 -- validate  # 65/65 Fraud detection + spatial cheats
```

Key results:
- **12 fraud types across 3 tiers**: basic (orphan/dupe/speed/range/unattributed/aimbot), consumable (phantom rounds/overconsumption), spatial (identity spoof/ghost action/through-wall shot/teleport)
- **Zone topology model**: adjacency + line-of-sight graph catches spatial fraud structurally
- **Identity spoof**: DAG timeline mismatch between claimed shooter zone and actual zone at tick
- **Ghost action**: kill/loot in a zone with no prior Spawn or Move vertex
- **Through-wall shot**: shooter and target in zones with no `LoS` per map topology
- **Teleport detection**: non-adjacent zone transitions with no intermediate Move vertices
- **Per-round provenance**: every bullet is an individual cert — mint on spawn/loot, consume on fire
- **Consumable lifecycle**: medical, food, and ammo tracked through full lifecycle with cert proof

## Composable Raid Visualization (Track 16)

Demonstrates the composable primal architecture. Infrastructure primals (biomeOS, songbird, petalTongue) are NOT Cargo dependencies — they are independent binaries that communicate via JSON-RPC 2.0 over Unix sockets. Data primals (trio) remain direct deps.

```bash
cargo run --release -p ludospring-exp054 -- validate  # 40/40 Composable architecture
```

Key results:
- **Zero chimeric deps**: protocol types defined locally, matching wire format of 3 infrastructure primals
- **biomeOS `DeploymentGraph`**: Continuous coordination at 20 Hz, 5-node topology with feedback edges
- **songbird discovery**: 2 player agents + raid server discovered by capability (`game.player_input`)
- **petalTongue `DataBinding`**: zone heatmap, health gauges, action timelines, fraud bar, inventory — all round-trip through JSON
- **End-to-end**: simulation → snapshot → dashboard → JSON → deserialize preserves all bindings

## Lysogeny — Open Recreation of Proprietary Game Mechanics (Track 17)

Recreates proprietary game mechanics from published scientific math, cross-validates
across biology and ecology, releases under AGPL-3.0. Every equation traces to a
published paper predating the proprietary implementation.

```bash
cargo run --release -p ludospring-exp055 -- validate  # 48/48 Usurper (Nemesis system)
cargo run --release -p ludospring-exp056 -- validate  # 47/47 Integrase (capture mechanics)
cargo run --release -p ludospring-exp057 -- validate  # 35/35 Symbiont (faction reputation)
cargo run --release -p ludospring-exp058 -- validate  # 40/40 Conjugant (roguelite meta-progression)
cargo run --release -p ludospring-exp059 -- validate  # 39/39 Quorum (emergent narrative)
cargo run --release -p ludospring-exp060 -- validate  # 28/28 Pathogen (gacha anti-pattern)
```

Key results:
- **Usurper**: persistent adaptive NPC hierarchy from replicator dynamics + spatial PD + Lotka-Volterra with memory. Maps 1:1 to antibiotic resistance populations.
- **Integrase**: capture probability from Wright-Fisher fixation, QS bond threshold, Markov evolution chains. The enzyme that integrates phage DNA into a host IS the cross-domain proof.
- **Symbiont**: multi-faction reputation from multi-species Lotka-Volterra competition coefficients. Factions = bacterial guilds, reputation = fitness contribution.
- **Conjugant**: roguelite meta-progression from horizontal gene transfer + Price equation. Dead runs release genes; survivors conjugate.
- **Quorum**: emergent procedural narrative from agent-based modeling + DAG causality. Quorum sensing threshold triggers collective phase transition = story event.
- **Pathogen**: defensive anti-pattern study quantifying gacha exploitation using operant conditioning + prospect theory. Measures and exposes, does not implement.

See `specs/LYSOGENY_CATALOG.md` for full citation tables and cross-domain mapping.

## Cross-Spring Provenance (exp062-066)

Five experiments extending the fermenting system (exp061) into cross-spring scaffolds:

```bash
cargo run --release -p ludospring-exp064                   # 39/39 BearDog-signed provenance chain
cargo run --release -p ludospring-exp062                   # 39/39 Field sample provenance (wetSpring scaffold)
cargo run --release -p ludospring-exp063                   # 35/35 Consent-gated medical access (healthSpring scaffold)
cargo run --release -p ludospring-exp065                   # 74/74 Cross-domain fraud unification
cargo run --release -p ludospring-exp066                   # 41/41 Radiating attribution calculator
```

Key results:
- **BearDog signing end-to-end**: Every vertex, certificate, and braid signed with Ed25519. Tamper detection at exact point.
- **Field sample lifecycle**: Collect → transport → store → extract → amplify → sequence → analyze → publish. 6 fraud types. DAG isomorphism with extraction shooter.
- **Consent-gated medical access**: Patient owns record (loamSpine cert). Provider gets scoped lending (consent cert). Every access logged. 5 fraud types. Zero-knowledge access proofs.
- **Cross-domain fraud unification**: Same `GenericFraudDetector` catches fraud across gaming, science, and medical with >80% structural similarity.
- **Radiating attribution**: sunCloud value distribution walks sweetGrass chains. Shares always sum to 1.0 (conservation). Decay models and role weighting.

Papers 21 (Sovereign Sample Provenance) and 22 (Zero-Knowledge Medical Provenance) are pending gen3 baseCamp write-up — the experimental validation is complete.

## Specs Paper Validation + Performance Benchmarks

Validates claims from the specs/ paper queue against live measurements:

```bash
cargo run --bin exp034_python_parity_bench           # 15/15 Python-vs-Rust math parity
cargo run --bin exp035_noise_throughput               # 10/10 BM-002 noise (0.93x fastnoise)
cargo run --bin exp036_raycaster_throughput           # 10/10 BM-003 raycaster (6,623 FPS)
cargo run --bin exp037_tick_budget                    # 10/10 tick budget (70% headroom)
```

Key results:
- **Python parity proven**: sigmoid, Fitts, Hick, LCG, dot, L2, Perlin all match Python within 1e-15
- **Faster than fastnoise-lite**: 0.93x ratio at 256x256 Perlin (we're faster, not just within 2x)
- **110x 60Hz headroom**: raycaster at 6,623 FPS on CPU alone
- **70% tick budget headroom**: 10K entities ticked in 910us (budget: 3,000us)

Both playable games now emit telemetry (NDJSON) during gameplay. After a session:

```bash
cargo run --bin exp026_game_telemetry -- analyze exp024_session_42.ndjson
```

## Portable Game Telemetry Protocol

Any game can emit NDJSON events; ludoSpring analyzes them. The protocol is the portability layer.

```bash
# Protocol validation (13 checks)
cargo run --bin exp026_game_telemetry -- validate

# Generate synthetic session + analyze
cargo run --bin exp026_game_telemetry -- generate session.ndjson
cargo run --bin exp026_game_telemetry -- analyze session.ndjson

# External game adapters
cargo run --bin exp027_veloren_adapter -- validate   # Veloren (SPECS ECS)
cargo run --bin exp028_fishfolk_adapter -- validate  # Fish Folk (Bevy)
cargo run --bin exp029_abstreet_adapter -- validate  # A/B Street (simulation)
```

13 event types, all `Serialize + Deserialize`. Any language that writes JSON is compatible:
Rust (direct lib call), Unity (C#), Godot (GDScript), web (JS).

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
cargo run --features ipc --bin ludospring_dashboard

# Live session: 120-tick streaming game simulation
cargo run --features ipc --bin ludospring_live_session

# Tufte dashboard: genre comparison, minimap analysis, cognitive load sweep
cargo run --features ipc --bin ludospring_tufte_dashboard
```

All binaries discover petalTongue automatically via Unix socket. If petalTongue is not running, scenarios are saved as JSON to `sandbox/`.

## Architecture

```
ludoSpring/
├── barracuda/             # Core library + 4 binaries
│   ├── src/
│   │   ├── game/          # Mechanics, raycaster, voxel, genre, state
│   │   ├── interaction/   # Fitts, Hick, Steering, GOMS, Flow, DDA
│   │   ├── procedural/    # Noise, WFC, L-systems, BSP
│   │   ├── metrics/       # Tufte, engagement, Four Keys to Fun
│   │   ├── tolerances/    # All constants with provenance (no magic numbers)
│   │   ├── validation/    # ValidationResult harness
│   │   ├── telemetry/     # Portable event protocol + analysis pipeline
│   │   ├── visualization/ # Data channels + PetalTonguePushClient
│   │   ├── ipc/           # JSON-RPC 2.0 server (capability-based discovery)
│   │   └── bin/           # ludospring, dashboard, live_session, tufte_dashboard
│   └── tests/             # python_parity.rs, validation.rs, determinism.rs
├── experiments/           # 66 experiments (22 validation + 3 playable + 4 telemetry + 4 compute + 4 benchmark + 3 control + 4 cross-spring + 3 RPGPT + 4 Games@Home + 1 Trio + 1 Extraction Shooter + 1 Composable Viz + 6 Lysogeny + 1 Fermenting + 5 Cross-Spring Provenance)
├── baselines/python/      # 7 Python reference implementations
├── benchmarks/            # Criterion benchmarks (noise, raycaster, ECS)
├── metalForge/forge/      # Hardware dispatch validation (7 checks)
├── specs/                 # 6 domain specifications
├── whitePaper/            # Local paper staging
└── wateringHole/          # Handoff documentation
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
# All tests (212 total: unit + determinism + parity + validation + forge + exp)
cargo test --features ipc --lib --tests

# Run a specific experiment
cargo run --bin exp017_bsp_level_generation

# Python baselines
python3 baselines/python/run_all_baselines.py

# UniBin server (biomeOS deployment)
cargo run --features ipc --bin ludospring -- server

# Quality checks
cargo fmt --check
cargo clippy --features ipc -p ludospring-barracuda -- -W clippy::pedantic
cargo doc --workspace --no-deps
```

## Quality

| Check | Result |
|-------|--------|
| `cargo fmt --check` | 0 diffs |
| `cargo clippy -W pedantic -W nursery` | 0 warnings (workspace-wide) |
| `cargo test --workspace` | 212 tests, 0 failures |
| `cargo doc --no-deps` | 0 warnings |
| 67 validation binaries | 1349 checks, 0 failures |
| 7 Python baselines | All pass (with embedded provenance: commit, date, Python version) |
| `#![forbid(unsafe_code)]` | All crate roots + all binaries |
| `llvm-cov` (library) | All 22 modules ≥ 90% (floor: 90.8% `interaction::flow`) |
| SPDX headers | All `.rs` + all `Cargo.toml` |
| Files > 1000 LOC | 0 |
| TODO/FIXME/HACK in source | 0 |

## Benchmark Gaps (Documented)

### Python Execution Timing
Python baselines (baselines/python/) validate **correctness parity** only — they
produce reference values that the Rust implementation must match. There are no
timed benchmarks that execute Python and compare wall-clock performance against
barracuda CPU. exp034 measures Rust-only throughput; the "inline-python" comparison
is Rust code that mirrors Python logic, not actual Python execution.

### Industry GPU Benchmarks
GPU validation (exp030) confirms CPU-vs-GPU **correctness parity** via wgpu/WGSL.
There are no benchmarks against industry GPU frameworks (Kokkos, CUDA, OpenCL,
cuDNN, oneMKL, rocBLAS). GPU performance parity against industry standards is
a toadStool/coralReef concern — ludoSpring validates the math, not the hardware.

## License

AGPL-3.0-or-later
