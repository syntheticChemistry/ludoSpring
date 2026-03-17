# ludoSpring Experiments

**Date:** March 16, 2026
**Total:** 75 experiments, 1692 checks, 0 failures, 394 tests + 12 proptest + 6 IPC integration (V23)
**Pattern:** hotSpring validation + baseCamp expeditions

---

## Experiment Index

### Track 1: Core Game Systems

| # | Binary | Checks | Status | Baseline | Modules Validated |
|---|--------|--------|--------|----------|-------------------|
| 001 | `exp001_doom_raycaster_analysis` | 6 | PASS | Python raycaster | `game::raycaster`, `metrics::tufte_gaming`, `interaction::input_laws` |
| 002 | `exp002_procedural_molecule_gen` | 5 | PASS | Python Perlin | `procedural::noise`, `game::voxel` |
| 003 | `exp003_tufte_game_ui` | 6 | PASS | Tufte (1983) | `metrics::tufte_gaming` |
| 004 | `exp004_folding_adversarial` | 5 | PASS | — | `interaction::difficulty`, `interaction::flow` |

### Track 2: Interaction Models

| # | Binary | Checks | Status | Baseline | Modules Validated |
|---|--------|--------|--------|----------|-------------------|
| 005 | `exp005_fitts_device_sweep` | 9 | PASS | MacKenzie (1992) | `interaction::input_laws` |
| 006 | `exp006_hick_menu_depth` | 6 | PASS | Hyman (1953) | `interaction::input_laws` |
| 007 | `exp007_steering_tunnel` | 5 | PASS | Accot & Zhai (1997) | `interaction::input_laws` |
| 011 | `exp011_goms_task_completion` | 8 | PASS | Card et al. (1983) | `interaction::goms` |
| 012 | `exp012_flow_channel_calibration` | 13 | PASS | Chen (2007) | `interaction::flow` |
| 019 | `exp019_composite_interaction_cost` | 6 | PASS | All 4 HCI laws | `interaction::input_laws`, `interaction::goms` |

### Track 3: Procedural Generation

| # | Binary | Checks | Status | Baseline | Modules Validated |
|---|--------|--------|--------|----------|-------------------|
| 008 | `exp008_wfc_crystal_lattice` | 7 | PASS | Gumin (2016) | `procedural::wfc` |
| 009 | `exp009_noise_molecular_density` | 9 | PASS | Perlin (2002) | `procedural::noise` |
| 013 | `exp013_lsystem_protein_backbone` | 15 | PASS | Lindenmayer (1968) | `procedural::lsystem` |
| 014 | `exp014_hybrid_noise_wfc` | 5 | PASS | — | `procedural::noise`, `procedural::wfc` |
| 017 | `exp017_bsp_level_generation` | 10 | PASS | Fuchs et al. (1980) | `procedural::bsp` |

### Track 4: Accessibility & Cognitive Load

| # | Binary | Checks | Status | Baseline | Modules Validated |
|---|--------|--------|--------|----------|-------------------|
| 015 | `exp015_accessibility_motor_limited` | 9 | PASS | IGDA/XAG | `interaction::accessibility`, `interaction::input_laws` |
| 016 | `exp016_cognitive_load_tufte` | 7 | PASS | Tufte (1983) | `metrics::tufte_gaming` |

### Track 5: Fun & Engagement Metrics

| # | Binary | Checks | Status | Baseline | Modules Validated |
|---|--------|--------|--------|----------|-------------------|
| 010 | `exp010_engagement_curves` | 14 | PASS | Yannakakis (2018) | `metrics::engagement`, `interaction::flow`, `interaction::difficulty` |
| 018 | `exp018_four_keys_fun` | 10 | PASS | Lazzaro (2004) | `metrics::fun_keys` |
| 020 | `exp020_difficulty_skill_balance` | 7 | PASS | Hunicke (2005) | `interaction::difficulty` |
| 021 | `exp021_retention_reward_curves` | 7 | PASS | — | `metrics::engagement`, `metrics::fun_keys` |
| 022 | `exp022_small_multiples_minimap` | 7 | PASS | Tufte (1983) | `metrics::tufte_gaming` |

### Track 6: baseCamp Expeditions (Playable Prototypes)

| # | Binary | Checks | Status | Reference | Modules Validated |
|---|--------|--------|--------|-----------|-------------------|
| 023 | `exp023_open_systems_benchmark` | 16 | PASS | fastnoise-lite, WFC crate, Bevy | `procedural::noise`, `procedural::wfc`, `procedural::bsp`, `game::state` |
| 024 | `exp024_doom_terminal` | — | Playable | Doom (1993), Wolfenstein 3D | `game::raycaster`, `procedural::bsp`, `metrics::tufte_gaming` |
| 025 | `exp025_roguelike_explorer` | — | Playable | Caves of Qud, Brogue, NetHack | `procedural::bsp`, `procedural::noise`, `interaction::difficulty`, `interaction::flow`, `metrics::engagement`, `metrics::fun_keys` |

### Track 7: Telemetry Protocol + External Game Adapters

| # | Binary | Checks | Status | Reference | Modules Validated |
|---|--------|--------|--------|-----------|-------------------|
| 026 | `exp026_game_telemetry` | 13 | PASS | OpenTelemetry, Unity Analytics | `telemetry::events`, `telemetry::mapper`, `telemetry::report` |
| 027 | `exp027_veloren_adapter` | 9 | PASS | Veloren (GPL-3.0) | `telemetry` (SPECS ECS log parser) |
| 028 | `exp028_fishfolk_adapter` | 7 | PASS | Fish Folk (MIT/Apache-2.0) | `telemetry` (Bevy plugin pattern) |
| 029 | `exp029_abstreet_adapter` | 8 | PASS | A/B Street (Apache-2.0) | `telemetry` (simulation-as-game) |

### Track 8: Compute Dispatch + metalForge

| # | Binary | Checks | Status | Reference | Modules Validated |
|---|--------|--------|--------|-----------|-------------------|
| 030 | `exp030_cpu_gpu_parity` | 24 | PASS | barraCuda CPU, WGSL shaders, wgpu 28 | CPU-vs-GPU parity (sigmoid, relu, dot, softmax, LCG, reduce, Perlin, fBm, engagement, raycaster) |
| 031 | `exp031_dispatch_routing` | 10 | PASS | toadStool substrate, wgpu adapter API | Hardware discovery, workload routing |
| 032 | `exp032_mixed_hardware` | 18 | PASS | PCIe specs, barraCuda unified_hardware | Transfer cost, mixed pipelines, NPU→GPU direct PCIe, 4-stage mixed pipeline |
| 033 | `exp033_nucleus_pipeline` | 19 | PASS | biomeOS nucleus_complete.toml | Tower/Node/Nest atomics + capability routing + toadStool dispatch + biomeOS graph |

### Track 9: Specs Paper Validation + Performance Benchmarks

| # | Binary | Checks | Status | Reference | Modules Validated |
|---|--------|--------|--------|-----------|-------------------|
| 034 | `exp034_python_parity_bench` | 15 | PASS | Python baselines, barraCuda CPU | Sigmoid, Fitts, Hick, LCG, dot, mean, L2, Perlin parity + timing |
| 035 | `exp035_noise_throughput` | 10 | PASS | BM-002, fastnoise-lite | Perlin 2D/3D, fBm throughput, fastnoise comparison |
| 036 | `exp036_raycaster_throughput` | 10 | PASS | BM-003, Lodev DDA reference | DDA 320/640-col cast, 60Hz sustainability, determinism |
| 037 | `exp037_tick_budget` | 10 | PASS | GAME_ENGINE_NICHE_SPEC budget table | game_logic 3ms, metrics 1ms, 10K entities at 60Hz |

### Track 10: External Control Groups

| # | Binary | Checks | Status | Reference | Modules Validated |
|---|--------|--------|--------|-----------|-------------------|
| 038 | `exp038_external_roguelike_control` | 12 | PASS | bracket-pathfinding (A*, FOV), drunkard's walk | Metrics on foreign content: engagement, flow, fun, DDA |
| 039 | `exp039_noise_cross_validation` | 12 | PASS | noise-rs, fastnoise-lite (C) | 3-way noise comparison: values, stats, game metrics, timing |
| 040 | `exp040_quality_discrimination` | 12 | PASS | 5 archetypes x 2 quality levels | Flow discriminates quality, fun keys classify archetypes |

### Track 11: Cross-Spring Experiments (NCBI, NUCLEUS, Anderson QS)

| # | Binary | Checks | Status | Reference | Modules Validated |
|---|--------|--------|--------|-----------|-------------------|
| 041 | `exp041_ncbi_qs_integration` | 12 | PASS | NCBI E-utilities, nestgate | NCBI esearch/esummary: luxI/luxS/agrB genes, SRA metagenomes, proteins |
| 042 | `exp042_tower_atomic_local` | 10 | PASS | biomeOS tower_atomic_bootstrap.toml | BearDog crypto.hash (Blake3, SHA3-256), Songbird IPC, JSON-RPC 2.0 |
| 043 | `exp043_qs_gene_fetch` | 10 | PASS | NCBI gene/protein databases | QS gene families (luxI/luxS/agrB/luxR/lasI/rhlI) × 20 gut genera |
| 044 | `exp044_anderson_qs_explorer` | 12 | PASS | wetSpring Exp356 (W model) | `procedural::noise`, `interaction::flow`, `metrics::engagement`, `metrics::fun_keys`, `interaction::difficulty` |

### Track 12: RPGPT — Sovereign RPG Engine (Paper 18, Architecture Defined)

Paper 18 defines a sovereign RPG engine where any open ruleset (Pathfinder 2e under ORC,
FATE Core under CC-BY, Cypher, PbtA) is ingested as a loamSpine certificate and combined
with any world to produce a playable RPG. The anti-cheat = chain-of-custody isomorphism
means the same rhizoCrypt DAG that tracks game item lineage tracks biological sample lineage.

Build phases:
1. Ruleset-as-certificate format (loamSpine)
2. Turn-based session DAG (rhizoCrypt)
3. AI narration loop (Squirrel + ludoSpring quality metrics)
4. Creative attribution (sweetGrass + sunCloud)

See `specs/RPGPT_DEEP_SYSTEM_DESIGN.md` and `gen3/baseCamp/18_rpgpt_sovereign_rpg_engine.md`.

| # | Package | Checks | Status | Reference | What it proves |
|---|---------|--------|--------|-----------|----------------|
| 045 | `ludospring-exp045` | 49 | PASS | PF2e (ORC), FATE (CC-BY), Cairn (CC-BY-SA) | Generic `Ruleset` trait abstracts 3 structurally different RPG systems |
| 046 | `ludospring-exp046` | 33 | PASS | Classic text adventures | DAG-based text adventure eliminates "guess the verb"; item provenance tracked |
| 047 | `ludospring-exp047` | 23 | PASS | MTG game rules | MTG game actions as DAG; physical card provenance via loamSpine cert pattern |

### Track 13: Games@Home — Distributed Human Computation (Paper 19)

Paper 19 proves that human gameplay is distributed computation. Stack resolution is protein folding (same components, different order → different outcomes). Every game produces novel data (game tree is provably infinite). Game tree complexity is a measurable design metric. The Folding@Home isomorphism maps 1:1 across 12 concepts. Seven cross-domain transfer paths validated. AR card gaming anchors digital in physical.

| # | Package | Checks | Status | Reference | What it proves |
|---|---------|--------|--------|-----------|----------------|
| 048 | `ludospring-exp048` | 36 | PASS | MTG stack rules, protein folding | Same cards, different stack order → opposite outcomes. Card text = genotype, resolution order = phenotype. Stack as DAG. |
| 049 | `ludospring-exp049` | 33 | PASS | Shannon (1950), Churchill et al. (2019) | Game tree ~10^358 (conservative). Birthday bound ~10^179. Every game is novel. 100B novel vertices/year. |
| 050 | `ludospring-exp050` | 30 | PASS | Wikipedia Game Complexity, Churchill et al. (2019) | Go explained (10^505). MTG is 2^ℵ₀ (Turing complete). Commander hypothesis: format ×216, designed cards ×0.036. Enzymatic shortcut model. |
| 051 | `ludospring-exp051` | 28 | PASS | Pande (F@H), von Ahn (2006) | F@H ↔ Games@Home isomorphism (12 concepts). 200× more compute units. Zero cost. 7 cross-domain transfers (avg 76%). Feedback loop model. |

See `gen3/baseCamp/19_games_at_home_distributed_human_computation.md`.

### Track 14: Provenance Trio Integration (rhizoCrypt + loamSpine + sweetGrass + biomeOS)

First experiment to directly import and exercise the three provenance primals
from ludoSpring. The trio lives among the biomeOS atomics — rhizoCrypt provides
ephemeral DAG workspace, loamSpine anchors permanent certificates, sweetGrass
attributes creative contributions. biomeOS deploys them as a Continuous 60 Hz
graph via the rootpulse niche.

| # | Package | Checks | Status | Reference | What it proves |
|---|---------|--------|--------|-----------|----------------|
| 052 | `ludospring-exp052` | 37 | PASS | rhizoCrypt v0.13, loamSpine v0.6, sweetGrass v0.1, biomeOS rootpulse niche | Cross-primal DAG+certificate+attribution round-trip; biomeOS graph topology fits 60 Hz tick |

### Track 15: Extraction Shooter Provenance + Fraud Detection

Models an extraction shooter (Tarkov/DMZ) where every bullet, loot pickup,
kill, and extract is tracked as a rhizoCrypt DAG vertex. Every item (weapon,
armor, key, barter) is a loamSpine certificate. Fraud detection reduces to
provenance chain analysis: orphan items, duplicate certificates, speed hacks,
impossible kills, unattributed container loot, and aimbot headshot anomalies.

| # | Package | Checks | Status | Reference | What it proves |
|---|---------|--------|--------|-----------|----------------|
| 053 | `ludospring-exp053` | 65 | PASS | Extraction shooter (Tarkov model), rhizoCrypt, loamSpine | 12 fraud types, zone topology, spatial fraud (spoof/ghost/wallhack/teleport), per-round provenance, consumable lifecycle |
| 054 | `ludospring-exp054` | 40 | PASS | petalTongue (viz IPC), biomeOS (graph), songbird (discovery) | Composable primal architecture: DataBinding JSON, DeploymentGraph TOML, 2-player coordination, zero chimeric deps |

**Key results (exp053):**
- Honest raid produces zero fraud detections across 13 DAG vertices
- 12 fraud types across 3 tiers: basic (orphan/dupe/speed/range/unattributed/aimbot), consumable (phantom rounds/overconsumption), spatial (identity spoof/ghost action/through-wall shot/teleport)
- Zone topology model with adjacency + line-of-sight enables spatial fraud without position telemetry
- Identity spoof caught via zone/tick mismatch in the claimed shooter's DAG timeline
- Ghost actions caught by comparing actor zone at action time vs Spawn/Move vertex history
- Through-wall shots caught by shooter/target zone LoS check against map topology
- Teleportation caught by non-adjacent zone transitions with no intermediate Move vertices
- Every bullet individually tracked: 6 rounds spawn → 3 fired (consumed) → 3 intact → 4 found → loaded to magazine
- Medical and food items have full consumption lifecycle — certs persist as proof even after use
- Chain-of-custody isomorphism: same code path catches item duplication in games AND sample tampering in genomics

**Key results (exp054):**
- Zero chimeric dependencies — infrastructure primals (biomeOS, songbird, petalTongue) interact via JSON-RPC 2.0 protocol only
- `DeploymentGraph` with `Continuous` coordination at 20 Hz: 5 nodes (2 inputs → raid server → fraud detect → viz push)
- songbird registration/discovery protocol validated: 2 player agents + raid server + viz provider
- biomeOS lifecycle messages: `lifecycle.register`, `capability.register` with correct JSON-RPC 2.0 envelopes
- petalTongue `DataBinding` JSON: zone heatmap, player health gauges, action timelines, fraud bar chart, inventory bars
- Streaming protocol: `append` (timeline) and `set_value` (gauge) operations round-trip through JSON
- End-to-end: simulation → snapshot → dashboard → JSON → deserialize preserves all bindings

### Track 17: Lysogeny — Open Recreation of Proprietary Game Mechanics

| # | Package | Checks | Status | Target | Open Math |
|---|---------|--------|--------|--------|-----------|
| 055 | `ludospring-exp055` | 48 | PASS | Usurper (Nemesis System) | Replicator dynamics + spatial PD + Lotka-Volterra with memory |
| 056 | `ludospring-exp056` | 47 | PASS | Integrase (Capture/Bonding) | Wright-Fisher fixation + QS threshold + Markov chains |
| 057 | `ludospring-exp057` | 35 | PASS | Symbiont (Faction/Reputation) | Multi-species Lotka-Volterra + frequency-dependent fitness |
| 058 | `ludospring-exp058` | 40 | PASS | Conjugant (Roguelite Meta-Progression) | HGT + Wright-Fisher + Price equation + Red Queen |
| 059 | `ludospring-exp059` | 39 | PASS | Quorum (Emergent Narrative) | Agent-based + Markov + DAG causality + QS threshold |
| 060 | `ludospring-exp060` | 28 | PASS | Pathogen (Gacha Anti-Pattern) | Operant conditioning + prospect theory + parasitism |

**Strategy**: Identify proprietary game mechanics, trace the underlying math to published open research predating patents/trade secrets, recreate from first principles under AGPL-3.0, cross-validate in biology/ecology.

### Track 18: Fermenting — Memory-Bound Digital Objects

| # | Package | Checks | Status | Description |
|---|---------|--------|--------|-------------|
| 061 | `ludospring-exp061` | 89 | PASS | Full fermenting lifecycle: mint, trade, loan, return, consume, achievements, cosmetic schema, atomic swap, ownership enforcement, trio integration, composable IPC wire format |

**Concept**: A "ferment" is a digital object whose value accumulates through use — NFT without crypto. loamSpine certificates provide ownership, rhizoCrypt DAGs track history, sweetGrass braids provide attribution. The fermenting system proves the full lifecycle works both as direct library integration and as composable IPC over JSON-RPC 2.0.

**Key results:**
- Every equation traces to published research predating proprietary implementations (Fisher 1930, Wright 1931, Gause 1934, Skinner 1938, Lotka 1925, Maynard Smith 1982, Nowak & May 1992)
- Cross-domain mapping tables prove game mechanics are general population dynamics math
- Usurper: orc captain ↔ antibiotic-resistant bacterial strain, 1:1 isomorphism
- Integrase: capture probability ↔ phage lysogeny MOI, the enzyme name IS the proof
- Pathogen: defensive anti-pattern study that MEASURES and EXPOSES gacha exploitation

See `specs/LYSOGENY_CATALOG.md` for full citation tables.

### Track 19: BearDog-Signed Provenance Chain

| # | Package | Checks | Status | Description |
|---|---------|--------|--------|-------------|
| 064 | `ludospring-exp064` | 39 | PASS | BearDog Ed25519 signing wired into all trio operations: vertex, certificate, braid. Chain verification detects tampered items at exact position. IPC wire format for `crypto.sign_ed25519` / `crypto.verify_ed25519` / `crypto.blake3_hash`. |

### Track 20: Cross-Spring Provenance — Field Sample (wetSpring Scaffold)

| # | Package | Checks | Status | Description |
|---|---------|--------|--------|-------------|
| 062 | `ludospring-exp062` | 39 | PASS | Full biological sample lifecycle: collect → transport → store → extract → amplify → sequence → analyze → publish. 6 fraud types (PhantomSample, DuplicateAccession, BrokenColdChain, UnauthorizedAccess, MislabeledSpecimen, ContaminationGap). DAG isomorphism with extraction shooter exp053. IPC wire format. |

**What wetSpring gets**: `SampleCertificate` and `SampleDag` patterns mapping directly to field genomics architecture. Fraud detectors become the QC pipeline.

### Track 21: Cross-Spring Provenance — Consent-Gated Medical Access (healthSpring Scaffold)

| # | Package | Checks | Status | Description |
|---|---------|--------|--------|-------------|
| 063 | `ludospring-exp063` | 35 | PASS | Patient-owned medical records via DID-based loamSpine certs. Consent certificates as scoped lending. 5 fraud types (UnauthorizedAccess, ExpiredConsent, ScopeViolation, PhantomAccess, ConsentForgery). BearDog zero-knowledge access proofs. PROV-O audit trail. |

**What healthSpring gets**: consent/access model mapping to clinical tracks (PK/PD, microbiome, biosignal, TRT). Zero-knowledge access proof pattern via BearDog.

### Track 22: Cross-Domain Fraud Unification + Radiating Attribution

| # | Package | Checks | Status | Description |
|---|---------|--------|--------|-------------|
| 065 | `ludospring-exp065` | 74 | PASS | Same `GenericFraudDetector` catches fraud across gaming (exp053), science (exp062), medical (exp063). 5 generic fraud types, 3 vocabularies. Structural similarity >80%. |
| 066 | `ludospring-exp066` | 41 | PASS | sunCloud radiating attribution: walk sweetGrass chain, compute proportional credit. Decay models, role weighting. Conservation proven (shares sum to 1.0). Game/science/medical scenarios. |

**Key insight**: The universality claim from Paper 18 (anti-cheat = chain-of-custody) is not just conceptual — it is the same code path. exp065 proves it with identical fraud detections across all 3 domain vocabularies.

### Track 23: RPGPT Deep System — Dialogue Engine (Phase 1)

The Dialogue Engine is Phase 1 of the RPGPT Deep System Design: NPC personality
certificates, knowledge bounds, internal voices (Disco Elysium model), passive
checks, trust model, and NPC memory as DAG subgraph. These experiments validate
the mechanical substrate before any AI narration is connected.

biomeOS deploy graph: `graphs/rpgpt_dialogue_engine.toml`
BYOB niche composition: `niches/rpgpt-dialogue.yaml`
Core types: `barracuda/src/game/rpgpt/` (56 unit tests)

| # | Package | Checks | Status | Reference | What it proves |
|---|---------|--------|--------|-----------|----------------|
| 067 | `ludospring-exp067` | 38 | PASS | RPGPT_NPC_PERSONALITY_SPEC.md | Knowledge bounds classify queries: knows / suspects / lies_about / does_not_know / unbound. Lies take priority. Case insensitive. Multi-NPC (Maren, Sheriff Marsh, Professor Armitage). |
| 068 | `ludospring-exp068` | 21 | PASS | RPGPT_DIALOGUE_PLANE_EXPERIMENTS.md | Passive voice checks detect lies at DCs. Higher skill = higher rate. Higher DC = lower rate. Tells reveal behavioral cues, NOT the truth. Perception vs Empathy detection. |
| 069 | `ludospring-exp069` | 75 | PASS | RPGPT_INTERNAL_VOICES_SPEC.md | 10 voices distinct. Temperature ranges valid (Composure coldest, Inland Empire warmest). Opposing voice pairs symmetric. Token limits bounded. Selection by priority then roll. |
| 070 | `ludospring-exp070` | 25 | PASS | RPGPT_INTERNAL_VOICES_SPEC.md | Max 3 voices per action. Priority ordering: critical > high > medium > low. Tie-breaking by roll. Edge cases: zero max, empty input, all same priority. Check gating (only passing checks produce output). |
| 071 | `ludospring-exp071` | 26 | PASS | RPGPT_DEEP_SYSTEM_DESIGN.md | NPC memory assembler: recent window verbatim, promises always included, trust milestones always included, routine summarized. Cumulative trust correct. Empty and few-interaction edge cases. Secret reveals excluded from routine. |
| 072 | `ludospring-exp072` | 45 | PASS | RPGPT_NPC_PERSONALITY_SPEC.md | Trust accumulates from defined actions, gates information access at level thresholds. Betrayal asymmetric (-5 vs +1). Arc phases (conformity->internal_conflict->revelation) with triggers. Quorum threshold for collective NPC events. Full trust history with running totals. |
| 073 | `ludospring-exp073` | 34 | PASS | RPGPT_PLANES_SCHEMA.md (Dialogue) | D6 pool resolution: threshold 4+, pool sizing from skill+modifiers. 5-degree resolution (CritFail/Fail/Partial/Success/CritSuccess). Statistical distribution matches expected binomial (10K trials). Modifier stacking (trust/env/emotional/knowledge). Pool minimum 1. |
| 074 | `ludospring-exp074` | 26 | PASS | Csikszentmihalyi, Hick, Hunicke | Flow/DDA/Hick integration with dialogue: Flow detected when balanced, Anxiety when challenge>skill, Boredom when skill>challenge. Hick's law flags >6 options. DDA suggests easier/harder adjustments. DialogueFlowTracker evolves skill estimate. Stall detection. |
| 075 | `ludospring-exp075` | 31 | PASS | RPGPT_DEEP_SYSTEM_DESIGN.md | Plane transition: Dialogue<->Tactical round-trip. Inventory preserved. NPC dispositions unchanged. Conditions mapped (Frightened->Frightened+decay, Exhausted->Fatigued, Wounded persists). Cross-plane knowledge carries. HP preserved. Verification detects tampering. |

**Key results (Phase 1 — exp067-071):**
- 185 validation checks, all passing
- Knowledge bounds enforce the four-quadrant NPC knowledge model
- Lies have detection DCs and behavioral tells — passive checks reveal tells, not truth
- Internal voices have distinct personality parameters that constrain AI inference
- Priority system ensures maximum 3 voices per action with deterministic ordering
- NPC memory assembly is graph-aware, not context-window-aware

**Key results (Phase 2 — exp072-075):**
- 136 validation checks, all passing
- Trust model gates secrets and drives NPC arc progression through defined phases
- D6 pool system matches expected binomial distribution at all pool sizes
- ludoSpring Flow/DDA/Hick integration detects and responds to dialogue pacing issues
- Plane transitions preserve complete world state with verified condition mapping
- Round-trip (Dialogue->Tactical->Dialogue) demonstrates ruleset-agnostic state preservation

### metalForge Dispatch (Capability-Based Routing)

| Binary | Checks | Status | Modules Validated |
|--------|--------|--------|-------------------|
| `validate_dispatch_routing` | 7 | PASS | GPU/CPU workload routing for noise, WFC, raycaster |

metalForge forge library: 9 unit tests — `SubstrateKind` (Cpu/Gpu/Npu), `Capability` enum (F64/F32/Shader/SIMD/PCIe/QuantizedInference), `route()` with capability filtering, `fallback_chain()` (GPU>NPU>CPU).

### petalTongue Dashboards

| Binary | Scenarios | What it visualizes |
|--------|-----------|-------------------|
| `ludospring_dashboard` | 8 | All 7 `GameChannelType` channels from validated math |
| `ludospring_live_session` | 1 (streaming) | 120-tick game session with DDA, flow, engagement |
| `ludospring_tufte_dashboard` | 3 | Genre comparison, minimap multiples, cognitive load sweep |

## Running

```bash
# Run a specific validation experiment
cargo run --bin exp017_bsp_level_generation

# Run metalForge dispatch
cargo run --bin validate_dispatch_routing

# Run petalTongue dashboards
cargo run --features ipc --bin ludospring_dashboard
cargo run --features ipc --bin ludospring_live_session
cargo run --features ipc --bin ludospring_tufte_dashboard

# Run baseCamp expeditions
cargo run --bin exp023_open_systems_benchmark    # benchmark (16 checks)
cargo run --bin exp024_doom_terminal             # playable Doom walker
cargo run --bin exp025_roguelike_explorer        # playable roguelike

# Run telemetry protocol + adapters
cargo run --bin exp026_game_telemetry -- validate    # telemetry protocol (13 checks)
cargo run --bin exp027_veloren_adapter -- validate   # Veloren adapter (9 checks)
cargo run --bin exp028_fishfolk_adapter -- validate  # Fish Folk adapter (7 checks)
cargo run --bin exp029_abstreet_adapter -- validate  # A/B Street adapter (8 checks)

# Run compute dispatch experiments
cargo run --bin exp030_cpu_gpu_parity                 # CPU-vs-GPU parity (24 checks)
cargo run --bin exp031_dispatch_routing               # dispatch routing (10 checks)
cargo run --bin exp032_mixed_hardware                 # mixed hardware (18 checks)
cargo run --bin exp033_nucleus_pipeline               # NUCLEUS pipeline (19 checks)

# Generate + analyze telemetry pipeline
cargo run --bin exp026_game_telemetry -- generate session.ndjson
cargo run --bin exp026_game_telemetry -- analyze session.ndjson

# Run external control groups
cargo run --bin exp038_external_roguelike_control         # external roguelike (12 checks)
cargo run --bin exp039_noise_cross_validation             # 3-way noise validation (12 checks)
cargo run --bin exp040_quality_discrimination             # quality discrimination (12 checks)

# Run specs paper validation + benchmarks
cargo run --bin exp034_python_parity_bench               # Python-Rust parity (15 checks)
cargo run --bin exp035_noise_throughput                   # BM-002 noise throughput (10 checks)
cargo run --bin exp036_raycaster_throughput               # BM-003 raycaster throughput (10 checks)
cargo run --bin exp037_tick_budget                        # tick budget validation (10 checks)

# Run cross-spring experiments
cargo run --release -p ludospring-exp041 -- validate      # NCBI QS integration (12 checks)
cargo run --release -p ludospring-exp042 -- validate      # Tower Atomic local (10 checks)
cargo run --release -p ludospring-exp043 -- validate      # QS gene dataset (10 checks)
cargo run --release -p ludospring-exp044 -- validate      # Anderson QS explorer (12 checks)

# Run RPGPT experiments (Track 12)
cargo run --release -p ludospring-exp045 -- validate      # Ruleset control systems (49 checks)
cargo run --release -p ludospring-exp046 -- validate      # Text adventure DAG (33 checks)
cargo run --release -p ludospring-exp047 -- validate      # MTG card provenance (23 checks)

# Run Games@Home experiments (Track 13)
cargo run --release -p ludospring-exp048 -- validate      # Stack resolution as folding (36 checks)
cargo run --release -p ludospring-exp049 -- validate      # Novel data combinatorics (33 checks)
cargo run --release -p ludospring-exp050 -- validate      # Game tree design metric (30 checks)
cargo run --release -p ludospring-exp051 -- validate      # Games@Home distributed human computation (28 checks)

# Run Provenance Trio integration (Track 14)
cargo run --release -p ludospring-exp052 -- validate      # Trio integration (37 checks)

# Run Extraction Shooter fraud detection (Track 15)
cargo run --release -p ludospring-exp053 -- validate      # Extraction shooter provenance (65 checks)
cargo run --release -p ludospring-exp054 -- validate      # Composable raid visualization (40 checks)

# Run Lysogeny — Open recreation of proprietary game mechanics (Track 17)
cargo run --release -p ludospring-exp055 -- validate      # Usurper: Nemesis system (48 checks)
cargo run --release -p ludospring-exp056 -- validate      # Integrase: capture mechanics (47 checks)
cargo run --release -p ludospring-exp057 -- validate      # Symbiont: faction reputation (35 checks)
cargo run --release -p ludospring-exp058 -- validate      # Conjugant: roguelite meta-progression (40 checks)
cargo run --release -p ludospring-exp059 -- validate      # Quorum: emergent narrative (39 checks)
cargo run --release -p ludospring-exp060 -- validate      # Pathogen: gacha anti-pattern (28 checks)

# Run Fermenting — Memory-bound digital objects (Track 18)
cargo run --release -p ludospring-exp061 -- validate      # Fermenting: full lifecycle (89 checks)

# Run BearDog-Signed Provenance Chain (Track 19)
cargo run --release -p ludospring-exp064                   # BearDog signing (39 checks)

# Run Cross-Spring Provenance — Field Sample (Track 20)
cargo run --release -p ludospring-exp062                   # Field sample provenance (39 checks)

# Run Cross-Spring Provenance — Medical Access (Track 21)
cargo run --release -p ludospring-exp063                   # Consent-gated medical access (35 checks)

# Run Cross-Domain Fraud + Radiating Attribution (Track 22)
cargo run --release -p ludospring-exp065                   # Cross-domain fraud unification (74 checks)
cargo run --release -p ludospring-exp066                   # Radiating attribution calculator (41 checks)

# Run RPGPT Deep System — Dialogue Engine (Track 23)
cargo run -p ludospring-exp067                             # NPC knowledge bounds (38 checks)
cargo run -p ludospring-exp068                             # Lie detection passive checks (21 checks)
cargo run -p ludospring-exp069                             # Internal voice personality (75 checks)
cargo run -p ludospring-exp070                             # Voice priority/concurrency (25 checks)
cargo run -p ludospring-exp071                             # NPC memory DAG retrieval (26 checks)
cargo run -p ludospring-exp072                             # Trust dynamics/arc progression (45 checks)
cargo run -p ludospring-exp073                             # Dialogue skill checks D6 pool (34 checks)
cargo run -p ludospring-exp074                             # Dialogue flow monitoring (26 checks)
cargo run -p ludospring-exp075                             # Plane transition continuity (31 checks)

# Run all tests
cargo test --features ipc --lib --tests
```

## Validation Pattern

Every experiment follows the hotSpring validation pattern using `ValidationHarness`:

```rust
const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/interaction_laws.py",
    commit: "abc1234",
    date: "2026-03-10",
    python: "3.12.1",
    command: "python3 baselines/python/interaction_laws.py",
};

fn main() {
    let mut h = ValidationHarness::new("exp001_raycaster");
    h.print_provenance(&PROVENANCE);
    h.check_abs("hit_rate", actual, expected, tolerances::RAYCASTER_HIT_RATE_TOL);
    h.check_bool("walls_only", actual_bool);
    h.finish();
}
```

- `ValidationHarness<S: ValidationSink>` with pluggable output (default: stderr)
- `BaselineProvenance` records script, commit, date, Python version, exact command
- Named tolerances from `tolerances/` submodules (zero magic numbers)
- Hardcoded expected values from documented Python baselines
- Explicit pass/fail with tolerance justification
- Exit code 0 = all pass, exit code 1 = any failure
- Summary printed to stdout with check counts

Legacy experiments use `ValidationResult::check()` — migration to `ValidationHarness` is incremental (exp001 complete, remainder follows same pattern).
