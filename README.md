# ludoSpring — The Science of Play, Interaction, and Game Design

An ecoPrimals Spring. Treats game design with the same rigor that wetSpring treats bioinformatics and hotSpring treats nuclear physics: validated models, reproducible experiments, GPU-accelerated computation where it matters.

**Date:** April 27, 2026
**Version:** V54 (Composition library absorption — `nucleus_composition_lib.sh` + `composition_nucleus.sh` from primalSpring TTT reference; `ludo_composition.sh` domain-specific interactive game science composition (Fitts pointing, reaction/Hick, DAG sandbox); exploration lane: interaction fidelity, real-time feedback, multi-input routing. Pure primal composition via 12-node cell graph. **817** workspace tests.)
**Spring alignment table:** The ludoSpring row in sibling `../primalSpring/wateringHole/NUCLEUS_SPRING_ALIGNMENT.md` uses the same workspace test total as this README (**817** as of V54); if they diverge, treat this README and `cargo test --workspace` as canonical.
**License:** AGPL-3.0-or-later (scyBorg triple: AGPL + ORC + CC-BY-SA-4.0)
**MSRV:** 1.87 (edition 2024)
**barraCuda:** v0.3.11 (standalone, default-features = false — CPU-only default, GPU opt-in)
**ecoBin:** Pure Rust application code. One `-sys` dep: `renderdoc-sys` (transitive via `wgpu-hal`, GPU feature only — infrastructure C per ecoBin v3.0 guidance). `deny.toml` enforces ecoBin v3.0 banned-crate list (openssl-sys, ring, aws-lc-sys, native-tls, zstd-sys, lz4-sys, libsqlite3-sys, cryptoki-sys). Harvested to genomeBin v5.1 (46 binaries, 6 target triples).
**Deployment model:** Pure composition — no spring binary in plasmidBin. Game science is served by composing primals via `ludospring_cell.toml` (12 NUCLEUS nodes: barraCuda for math/science, petalTongue for viz/interaction, Squirrel for AI, provenance trio, Tower Atomic). The ludospring binary is the Rust validation target (tier 2); it validates science locally but does not deploy as a primal. 30 capabilities across 11 composed primals. `lifecycle.composition` handler for runtime proto-nucleate validation.
**Audit Status:** Complete — zero hardcoded primal names (capability-based discovery), zero hardcoded paths, **zero hardcoded method strings in dispatch/push paths** (V51: `ipc::methods` constants for viz/interaction/lifecycle/capability), zero `#[allow()]` in application code, zero `unsafe`, **zero `Result<_, String>` in entire IPC layer** (V50: all 43 functions evolved to `IpcError`, `classify_io_error` absorbed from primalSpring v0.9.17), zero external deps removable (base64 inlined), zero clippy warnings (workspace-wide), zero TODO/FIXME, all experiments use `ValidationHarness` + `BaselineProvenance` (provenance unified to `19e402c0`), all tolerances centralized (named constants with citations, v1.2.0 ordering invariant: 7 constants), `GpuContext` + `TensorSession` wired behind `gpu` feature, CI pipeline with baseline drift check + three-tier validation (LOCAL_CAPABILITIES→IPC-WIRED→FULL NUCLEUS) + `validate_composition` + `validate_primal_proof` + `ludospring_guidestone` + `cargo-llvm-cov` gated at 90% floor. Fragments: `tower_atomic`, `node_atomic`, `nest_atomic`, `meta_tier`. **817** workspace tests, **11** primal gaps documented (GAP-01–GAP-11 in `docs/PRIMAL_GAPS.md`). guideStone readiness 4 (three-tier: bare + IPC + NUCLEUS cross-atomic). GAP-02 guideStone wired. BTSP relay wired (`ipc/btsp.rs`, typed `IpcError`). `interaction.poll` wired. `is_skip_error` graceful degradation. Honest `push_scene` error reporting. `game.tick` composite handler (V52). MCP surface complete (15/15 tools). Conforms to guideStone Composition Standard v1.2.0. Cell graph ready (`ludospring_cell.toml`). All upstream blockers resolved.

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
cargo run --bin exp030_cpu_gpu_parity               # 32/32 CPU-vs-GPU parity (Tier A WGSL shaders; fog-of-war, tile lighting, pathfind wavefront)
cargo run --bin exp031_dispatch_routing              # 10/10 real hardware discovery
cargo run --bin exp032_mixed_hardware                # 23/23 PCIe + mixed pipelines + NPU→GPU direct + Forge integration
cargo run --bin exp033_nucleus_pipeline              # 27/27 NUCLEUS atomics + toadStool dispatch + biomeOS graph + mixed pipeline / NPU
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
cargo run --release -p ludospring-exp042 -- validate  # Tower Atomic (capability-based crypto+IPC discovery)
cargo run --release -p ludospring-exp043 -- validate  # 10/10 QS gene dataset (6 families × 20 genera)
cargo run --release -p ludospring-exp044 -- validate  # 12/12 Anderson QS interactive explorer
```

Key results:
- **Live NCBI integration**: luxI/luxS/agrB gene search, SRA metagenomes, protein databases via E-utilities
- **Biological validation**: gut microbes use AI-2 (luxS) not AHL (luxI) — NCBI data confirms published biology
- **Tower Atomic boot**: Crypto primal discovered by `crypto.hash` capability, validated via JSON-RPC over Unix sockets
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

See `specs/RPGPT_DEEP_SYSTEM_DESIGN.md` (planes architecture, NPC personality, internal voices) and `gen3/baseCamp/18_rpgpt_sovereign_rpg_engine.md`.

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

## RPGPT Dialogue Plane (exp067-075)

Nine experiments validating the Dialogue Plane of the RPGPT system — NPC personality,
knowledge bounds, internal voices, trust dynamics, and plane transition continuity:

```bash
cargo run --release -p ludospring-exp067 -- validate  # NPC knowledge bounds enforcement
cargo run --release -p ludospring-exp068 -- validate  # Lie detection via passive checks
cargo run --release -p ludospring-exp069 -- validate  # Internal voice personality consistency
cargo run --release -p ludospring-exp070 -- validate  # Voice priority and concurrency
cargo run --release -p ludospring-exp071 -- validate  # NPC memory DAG retrieval
cargo run --release -p ludospring-exp072 -- validate  # Trust dynamics and NPC arc progression
cargo run --release -p ludospring-exp073 -- validate  # Dialogue plane skill check resolution
cargo run --release -p ludospring-exp074 -- validate  # Dialogue plane flow monitoring
cargo run --release -p ludospring-exp075 -- validate  # Plane transition continuity (Dialogue <-> Tactical)
```

Key results:
- **NPC personality certificates**: loamSpine-anchored personality + knowledge bounds — NPCs know what they know, refuse what they don't
- **Internal voices**: Disco Elysium-style skill-as-perspective via constrained Squirrel AI calls (10 voices: Logic, Empathy, Rhetoric, Perception, Endurance, Authority, Composure, Imagination, History, Esotericism)
- **Trust dynamics**: Multi-factor disposition (faction + personal + relationship + debt), trust gates on knowledge sharing
- **Plane transitions**: State preserved across Exploration ↔ Dialogue ↔ Tactical ↔ Investigation ↔ Political ↔ Crafting ↔ Card/Stack

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
├── barracuda/             # Core library + 4 binaries
│   ├── src/
│   │   ├── game/          # Mechanics, raycaster, voxel, genre, state
│   │   ├── interaction/   # Fitts, Hick, Steering, GOMS, Flow, DDA
│   │   ├── procedural/    # Noise, WFC, L-systems, BSP
│   │   ├── metrics/       # Tufte, engagement, Four Keys to Fun
│   │   ├── tolerances/    # 6 submodules (game, interaction, ipc, metrics, procedural, validation)
│   │   ├── validation/    # ValidationHarness<S: ValidationSink> + BaselineProvenance
│   │   ├── telemetry/     # Portable event protocol + analysis pipeline
│   │   ├── visualization/ # Data channels + VisualizationPushClient (capability-based)
│   │   ├── ipc/           # JSON-RPC 2.0 server + handlers/{lifecycle,science,delegation,mcp,neural} + typed clients + discovery/{mod,capabilities}
│   │   ├── biomeos/       # Niche deployment: domain, registration, Neural API
│   │   └── bin/           # ludospring UniBin (7 subcommands) + commands/ modules
│   └── tests/             # python_parity, validation, determinism, proptest_invariants, ipc_integration
├── experiments/           # 100 experiments
├── baselines/python/      # 7 Python reference implementations
├── benchmarks/            # Criterion benchmarks (noise, raycaster, ECS)
├── metalForge/forge/      # Capability-based routing (26 tests, 4 domain modules, GPU>NPU>CPU)
├── graphs/                # Deploy graphs (ludospring_deploy.toml, gaming_niche.toml)
├── niches/                # Niche YAML (ludospring-game.yaml)
├── deploy/                # primalSpring deploy graph fragment
├── specs/                 # 14 domain specifications
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
# All tests (817 workspace total: barracuda lib + barracuda --tests incl. ipc integration + forge + 100 experiments)
cargo test --workspace

# Run a specific experiment
cargo run --bin exp017_bsp_level_generation

# Python baselines + drift check
python3 baselines/python/run_all_baselines.py
python3 baselines/python/check_drift.py

# UniBin server (biomeOS niche deployment)
cargo run --features ipc --bin ludospring -- server

# Quality checks
cargo fmt --check
cargo clippy -p ludospring-barracuda --all-features -- -D warnings
cargo doc -p ludospring-barracuda --all-features --no-deps
cargo llvm-cov -p ludospring-barracuda --features ipc --lib --tests \
    --ignore-filename-regex bin/ --fail-under-lines 90
```

## Quality

| Check | Result |
|-------|--------|
| `cargo fmt --check` | 0 diffs |
| `cargo clippy --all-features -D warnings` | 0 warnings (pedantic + nursery) |
| `cargo test --workspace` | 817 total (barracuda lib + barracuda `--tests` incl. 23 ipc integration + forge + 100 experiments), 0 failures |
| `cargo doc --all-features --no-deps` | 0 warnings |
| 100 validation binaries | All checks pass, 0 failures (exp032 22/23 pre-existing) |
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

## V32 Comprehensive Audit + Deep Debt Evolution (March 29, 2026)

Full codebase audit + systematic remediation across 110 files:

- **Provenance integrity** — all 77 experiment provenance blocks aligned to current baselines commit (`4b683e3e`); 34 analytical experiments populated with commit hashes and dates
- **Tolerance centralization** — 6 new named constants (`STRICT_ANALYTICAL_TOL`, `NUMERICAL_FLOOR`, `DDA_ADJUSTMENT_EPSILON`, `SPAN_FLOOR`, `TRUST_EQUALITY_TOL`); all test `1e-10` literals replaced with `ANALYTICAL_TOL` across 6 library modules
- **exp030 harness migration** — 525-line rewrite from legacy `ValidationResult` to `ValidationHarness<S>` with GPU-skip via `EXIT_SKIPPED`
- **OrExit adoption** — 27 experiment files migrated from manual `eprintln!("FATAL:..."); exit(1)` to `.or_exit("context")`; zero manual FATAL patterns remain
- **Capability-based evolution** — GPU degradation messages made primal-agnostic; MCP tool descriptions reference capabilities not names; `"biomeos"` socket dir extracted to `niche::ECOSYSTEM_SOCKET_DIR`
- **Deploy manifest fix** — added missing `game.gpu.batch_raycast`, corrected 26→27 capability count
- **deny.toml fix** — `unmaintained = "warn"` (invalid for cargo-deny 0.19) → `"workspace"`
- **CI hardening** — added baseline drift check job, workspace-wide `cargo check`, full workspace clippy
- **Coverage floor aligned** — Makefile 80%→85%→90% matching CI
- **Python parity expansion** — 5 new tests: fun_keys zero/max, fBm 3D lattice, L-system turtle geometry
- **TensorSession documented** — future-only status with shader promotion roadmap in `specs/BARRACUDA_REQUIREMENTS.md`
- **`specs/BARRACUDA_REQUIREMENTS.md`** — new: consumed/unused modules, shader tiers, upstream evolution requests

## V32.2 Compute Evolution — GPU Parity + NPU Dispatch + NUCLEUS (March 29, 2026)

Builds on the V32 audit with active compute-path evolution and broader validation:

- **exp030 game shader parity** — CPU↔GPU checks expanded **24→32**: fog-of-war, tile lighting, and pathfind wavefront parity alongside existing Tier A WGSL coverage
- **metalForge NPU evolution** — forge tests **19→26**: `Substrate::Npu`, `recommend_substrate_full`, NPU pipeline bands (`BandTarget::NpuCompute`, `BandTarget::NpuToGpuTransfer`), direct NPU→GPU PCIe transfer paths, mixed hardware profiles, and budget fields
- **exp032 Forge integration** — mixed-hardware validation **20→23** checks (Forge routing + pipeline integration)
- **exp033 NUCLEUS deepening** — atomic coordination checks **19→27** (mixed pipeline + biomeOS NPU graph)
- **Experiment matrix** — all **82 experiments validated** (**81 green + 1 live-IPC**)

## V31 Deep Debt + esotericWebb Alignment (March 28, 2026)

- **Workspace lint enforcement** — all 82 experiments + benchmarks now inherit `[lints] workspace = true` (was 24/82)
- **esotericWebb response alignment** — FlowResult, EngagementResult, DifficultyAdjustmentResult, DialogueResponse, NarrationResponse shapes compatible with Webb's LudoSpringClient
- **GPU IPC handlers** — 4 `game.gpu.*` methods routed to toadStool delegation with CPU fallback
- **MCP expansion** — 13 tools (was 8): added session, dialogue, narration, scene push
- **Visualization delegation** — neural.rs stubs evolved to complete petalTongue delegation with degraded responses
- **Fog of war LOS** — WGSL shader implements Bresenham line-of-sight with terrain wall occlusion
- **Tolerance dedup** — GPU raycaster tolerances re-export from validation module (single source of truth)
- **`#[allow()]` elimination** — 3 remaining instances migrated to `#[expect(reason)]`; dead import removed
- **CI expansion** — forge tests + workspace-wide doc check added to pipeline
- **ValidationHarness adoption** — forge validation binary migrated from manual counters
- **27 capabilities** — `game.gpu.batch_raycast` added to registry with semantic mapping and cost estimate
- **Python version fix** — corrected provenance claim from >= 3.12 to >= 3.10

## V30 Deep Evolution — Modern Rust, 91% Coverage, MCP (March 23, 2026; superseded by V31 above)

- **Handler refactor** — `ipc/handlers.rs` (1208 LOC) split into 5 domain submodules: `lifecycle`, `science`, `delegation`, `mcp`, `neural` — all under 300 LOC each
- **UniBin consolidation** — Dashboard, live-session, and tufte-dashboard merged as `ludospring` subcommands (7 total); old binaries deprecated
- **MCP tools support** — `tools.list` returns 8 science tool descriptors with JSON Schema; `tools.call` dispatches to existing handlers
- **tarpc optional feature** — `tarpc-ipc` feature with `LudoSpringService` typed RPC trait mirroring JSON-RPC surface
- **thiserror migration** — All error types now `#[derive(thiserror::Error)]`, eliminating manual `Display`/`Error` impls
- **Coverage push** — 80.2% → 91.27% line coverage (+273 tests): provenance trio 40% → 84%, external clients 48% → 84%, handler tests 70% → 95%
- **CI pipeline** — `.github/workflows/ci.yml` with fmt, clippy, test, doc, cargo deny gates
- **Stricter clippy** — `cast_possible_truncation = deny`, `cast_sign_loss = deny`, `cast_precision_loss = warn`
- **Deploy graph** — `deploy/ludospring.toml` primalSpring fragment: 27 capabilities, optional deps
- **Rustdoc cleanup** — All 14 broken intra-doc links fixed
- **CONTEXT.md** — Created per `PUBLIC_SURFACE_STANDARD`
- **Triple license** — `LICENSE-ORC` + `LICENSE-CC-BY-SA` files, README triple license section
- **Mock IPC harness** — `IpcTestServer` for integration tests exercising connected code paths
- **Neural handler** — `lifecycle.register`, `capability.deregister`, `capability.discover`, `capability.call` routed through dispatch

## V28 Capability-Based Discovery + Deep Code Quality (March 18, 2026)

- **Capability-based discovery** — exp042 evolved from hardcoded `"beardog"`/`"songbird"` to `discover_primals()` → `registry.find("crypto.hash")` / `registry.find("system.ping")`; exp054 parameterized `viz_register(primal_id, ...)` removing hardcoded `"petaltongue"`
- **Configurable output paths** — 3 dashboard binaries evolved from `Path::new("sandbox/...")` to `LUDOSPRING_OUTPUT_DIR` env var with fallback
- **IPC integration test fixes** — 3 pre-existing failures fixed (field name mismatch, response structure, test isolation race condition)
- **Doc completeness** — `# Errors` sections added to 2 public `Result`-returning functions in `ipc/envelope.rs`

## V27 Deep Debt Sprint (March 18, 2026)

- **Zero `#[allow()]`** — all 9 remaining instances migrated to `#[expect(reason)]` with curated dictionary
- **Zero `.expect()` in validation** — 4 calls migrated to `OrExit` pattern
- **Workspace lint centralization** — 16 experiment `Cargo.toml` files migrated to `[lints] workspace = true`
- **Smart refactoring** — `exp062/sample.rs` monolithic `detect_sample_fraud` extracted into 6 focused functions with structural type tracking

## V26 Full Harness Migration (March 18, 2026)

- **Full `ValidationHarness` migration** — all 71 validation experiments use `ValidationHarness` + `BaselineProvenance`
- **GPU tolerance centralization** — 14 named constants in `tolerances::gpu`
- **Shader dedup audit** — 7 upstream absorption candidates documented, 2 domain-specific retained

## V25 Deep Debt Sprint (March 18, 2026)

- **`ValidationHarness` migration** — exp002–exp010 migrated from legacy `ValidationResult`
- **GPU tolerance centralization** — `tolerances::gpu` module (10 constants)
- **`exit_skipped` pattern** — `exit(2)` for unavailable hardware
- **`check_abs_or_rel`** — compound tolerance for multi-order GPU parity
- **`load_baseline_f64`** — runtime JSON loader for Python baselines
- **Proptest tuning** — Fitts, Hick, flow state bumped to 1024 cases

## V24 Ecosystem Absorption Sprint (March 17, 2026)

Absorbed 8 patterns from 7 sibling springs and 5 infrastructure primals. Adds
resilience, fuzz testing, health probes, and structured dispatch classification:

- **`OrExit<T>` trait** — `.or_exit("context")` on `Result`/`Option` replaces `let Ok else { eprintln!; exit(1) }` boilerplate (groundSpring V112, wetSpring V123)
- **`DispatchOutcome<T>` enum** — `Ok` / `ProtocolError` / `ApplicationError` classification for RPC responses with `classify()` and `into_result()` (groundSpring V112, petalTongue V166)
- **4-format capability parsing** — `extract_capabilities()` handles flat arrays, object arrays, nested, and double-nested formats + `result` wrapper (airSpring v0.8.7, rhizoCrypt S17)
- **`health.liveness` + `health.readiness` probes** — Kubernetes-style probes registered as capabilities; 26 total (was 24) (healthSpring V32, coralReef Iter 51)
- **Resilient provenance trio IPC** — circuit breaker (5s cooldown) + exponential backoff (50ms base, 2 retries) wrapping all trio calls; graceful degradation when trio unavailable (healthSpring V32)
- **JSON-RPC proptest fuzz** — 7 property-based tests covering `extract_rpc_result`, `DispatchOutcome`, and `extract_capabilities` with arbitrary JSON (airSpring v0.8.7)
- **`deny.toml` evolution** — `yanked = "deny"` (was `"warn"`) for supply chain hardening (toadStool S157b)
- **Leverage guide** — cross-primal composition catalog: standalone, trio combos, wider primal combos, 6 novel cross-spring compositions

### V23 Cross-Ecosystem Deep Debt (preserved)

- Zero `#[allow()]` anywhere — `#[expect(reason)]` curated dictionary (wetSpring V122)
- Zero-panic validation binaries — 14 experiments (groundSpring V109)
- Centralized `extract_rpc_result()` (healthSpring V29)
- `deny.toml wildcards=deny` (barraCuda Sprint 6)
- XDG socket resolution, named unit constants
- Large file review — `handlers.rs`, `session.rs`, `mapper.rs` confirmed coherent

### V22 Ecosystem Absorption (preserved)

- toadStool `compute.dispatch.*` — 3 direct dispatch methods for low-latency game GPU compute
- Dual-format capability discovery — array and nested-object response formats (neuralSpring S156 fix)
- Python tolerance mirror — 46 constants mirroring Rust tolerances (wetSpring V121 pattern)
- Write→Absorb→Lean documentation on `procedural::noise`
- Deploy graph evolution — `compute.dispatch.submit/result/capabilities` capabilities added
- 4 new discovery tests

### V21 Deep Debt Evolution (preserved)

- **Session decomposition** — `GameSession::resolve()` extracted into per-command methods (`resolve_wait`, `resolve_end_turn`, `resolve_use_item`, `resolve_custom`, etc.), eliminating `#[allow(clippy::too_many_lines)]`
- **Typed transition verification** — `TransitionVerification` booleans replaced with `TransitionIssue` enum (`InventoryLost`, `DispositionChanged`, `KnowledgeLost`, `ConditionMismatch`, `HpChanged`) + `Vec<TransitionIssue>`, eliminating `#[allow(clippy::struct_excessive_bools)]`
- **Pluggable validation output** — `ValidationSink` trait with `StderrSink` (default) and `BufferSink` (testing); `ValidationHarness<S>` generic over sink, replacing hardcoded `eprintln!`
- **Typed toadStool IPC client** — `ipc/toadstool.rs` with `ComputeResult`, `SubstrateCapabilities`, typed methods (`submit_workload`, `workload_status`, `query_capabilities`), graceful degradation when Neural API unavailable
- **IPC integration tests** — 6 tests in `barracuda/tests/ipc_integration.rs`: lifecycle status, capability list, game method evaluation, error handling, health check
- **`#[expect]` evolution** — `#[allow(dead_code)]` replaced with `#[expect(dead_code, reason = "...")]` for justified IPC wire types (edition 2024 pattern)
- **Platform-agnostic paths** — hardcoded `/tmp/biomeos/` and `/tmp/petaltongue/` replaced with `std::env::temp_dir().join(...)` in test fixtures
- **Centralized game tolerance** — `GAME_STATE_TOL` constant in `tolerances/game.rs`, replacing inline `0.01` across 4 experiments
- **ValidationHarness adoption** — `exp001` fully rewritten from legacy `ValidationResult` to `ValidationHarness` + `BaselineProvenance`
- **75 .rs files, 19,302 lines** — net +544 lines (typed clients, integration tests, extracted methods)

### V20 Deep Primal Integration (preserved)

- IPC method alignment: 19 external methods aligned to canonical JSON-RPC specs
- Capability domains registry: 24 capabilities (10 local, 14 external)
- Tolerance decomposition: 6 domain-specific submodules
- Typed provenance pipeline: `DehydrationSummary` + `TrioStage`
- Game engine core: `RulesetCert` validation, concrete `apply()`, `GridMap` bridge
- `discover_by_capability()` runtime peer lookup
- 394 tests pass, zero clippy warnings

### V19 Foundation (preserved)

- Magic numbers eliminated — 9 tolerance constants with provenance citations
- Clone abuse eliminated — `&serde_json::Value` constructors
- Production panic eliminated — `BlockPalette::register()` → `Result`
- Provenance decomposed — 773-line monolith → 3 focused submodules
- Audio narration refactored — 5 focused functions

### V18 Foundation (preserved)

- `niche.rs` single source of truth — 24 capabilities, semantic mappings, cost estimates
- `NeuralBridge` typed IPC client for all inter-primal communication
- Platform-agnostic paths, XDG-compliant socket chain
- Squirrel AI, NestGate storage, petalTongue scene push, provenance trio all wired
- GPU compute: fog of war, tile lighting, pathfinding, Perlin terrain via toadStool/barraCuda

### V17 Foundation (preserved)

- Zero `#[allow()]` in production code
- 11 WGSL shaders extracted for toadStool absorption
- 12 proptest invariants
- Structured `tracing` in all IPC/biomeOS code
- Capability-based viz discovery

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
