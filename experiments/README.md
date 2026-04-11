# ludoSpring Experiments

**Date:** April 11, 2026
**Total:** 100 experiments, 781 workspace tests (V42)
**Pattern:** hotSpring validation + baseCamp expeditions + primal composition gap discovery + science-via-composition + NUCLEUS game engine composition + composition validation + NUCLEUS parity
**Lints:** All 100 experiment Cargo.toml files inherit `[lints] workspace = true`
**Live V42 results:** 95/141 (67.4%) composition checks passing against plasmidBin primals. exp099: 13/13 (Rust == IPC parity). exp100: 27-check NUCLEUS composition parity. `lifecycle.composition` handler wired — runtime proto-nucleate probe externally callable. Capability-first discovery. `nest_atomic` in fragments. Provenance unified to `19e402c0`.

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
| 030 | `exp030_cpu_gpu_parity` | 32 | PASS | barraCuda CPU, WGSL shaders, wgpu 28 | CPU-vs-GPU parity (sigmoid, relu, dot, softmax, LCG, reduce, Perlin, fBm, engagement, raycaster, fog-of-war, tile-lighting, pathfind) |
| 031 | `exp031_dispatch_routing` | 10 | PASS | toadStool substrate, wgpu adapter API | Hardware discovery, workload routing |
| 032 | `exp032_mixed_hardware` | 23 | PASS (22/23) | PCIe specs, barraCuda unified_hardware | Transfer cost, mixed pipelines, NPU→GPU direct PCIe, 4-stage mixed pipeline |
| 033 | `exp033_nucleus_pipeline` | 27 | PASS | biomeOS nucleus_complete.toml | Tower/Node/Nest atomics + capability routing + toadStool dispatch + biomeOS graph |

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

### Track 25: Neural API End-to-End Pipeline (V33→V34)

Full NUCLEUS Nest Atomic validation against 5 live primal processes through biomeOS Neural API routing.

| # | Package | Checks | Status | Reference | What it proves |
|---|---------|--------|--------|-----------|----------------|
| 083 | `ludospring-exp083` | 13 | PASS (live) | biomeOS capability_registry.toml | Full Nest Atomic: Blake3/SHA3-256 hash, ChaCha20-Poly1305 roundtrip, Ed25519 sign, Songbird discovery, ToadStool compute caps, NestGate store/retrieve, Squirrel AI/tool list, cross-domain provenance chain (hash→sign→store→verify), capability registry 5-domain completeness |

**Requires live primals**: BearDog, Songbird, ToadStool, NestGate, Squirrel + biomeOS neural-api. Not structural — talks to real processes.

**V34 evolution**: Expanded from Tower-only (10 checks) to full Nest Atomic (13 checks) with NestGate storage, Squirrel AI/MCP, and cross-domain provenance chain.

### Track 26: Primal Composition Validation — Gap Discovery (V35)

The next evolution tier: replicate ludoSpring's validated science using ONLY
primal composition (existing deployed primals composed via biomeOS graphs).
No ludoSpring binary participates — just barraCuda, toadStool, coralReef,
Squirrel, petalTongue, rhizoCrypt, and biomeOS Neural API.

Gaps discovered here become evolution pressure on primals and the Neural API.
esotericWebb learns from these composition graphs instead of depending on a
ludoSpring process. primalSpring validates the composition patterns work.

**Deploy graphs**: `graphs/composition/*.toml` — focused DAGs for each experiment.

**Evolution path**:
  Python baseline → Rust validation → barraCuda CPU → barraCuda GPU
  → **primal composition (THIS TRACK)** → esotericWebb absorbs patterns

| # | Package | Checks | V35 | V35.2 | **V37.1 live** | Composition Target |
|---|---------|--------|-----|-------|----------------|--------------------|
| 084 | `ludospring-exp084` | 15 | 0/12 | **12/15** | **12/15** | barraCuda math IPC + Neural API routing |
| 085 | `ludospring-exp085` | 8 | 2/8 | **7/8** | **7/8** | Sovereign shader dispatch: coralReef compile → toadStool dispatch |
| 086 | `ludospring-exp086` | 10 | 0/10 | **10/10** | **10/10** | Tensor API: all element-wise ops confirmed |
| 087 | `ludospring-exp087` | 8 | 1/7 | 3/7 | **3/8** | Neural API graph orchestration — capability routing gap |
| 088 | `ludospring-exp088` | 10 | 2/10 | 2/10 | **2/10** | 60Hz game loop — Neural API capability registration gap |

**Total trajectory**: 5/47 → 34/50 (68%) → **34/51 V37.1 live (67%)**

**Key insight**: These experiments are designed to FAIL. Each failure is a
documented gap with a specific primal owner and evolution target. When the
primals evolve to fill the gaps, these experiments will start passing —
proving that ludoSpring's science is replicable through composition alone.

**V35.3 ecosystem evolution — gaps resolved since V35.2**:

| Former Gap | Resolution | Version |
|-----------|------------|---------|
| biomeOS: no barraCuda domain in registry | Bootstrap graph has `register_barracuda` with 30+ method translations | v2.80 |
| biomeOS: `graph.save` parse error | Now accepts `{"toml": "..."}` format; experiments updated | v2.80 |
| biomeOS: bootstrap graph not bundled | `include_str!()` compiles graph into binary | v2.80 |
| barraCuda math not on IPC | 30 methods registered since Sprint 23 (confirmed V35.2) | Sprint 23+ |
| barraCuda tensor element-wise ops | All ops work: add, scale, clamp, reduce, sigmoid (confirmed V35.2) | Sprint 23+ |
| coralReef HTTP JSON-RPC | Raw newline-delimited on UDS (confirmed V35.1) | Iter 70 |

**V37.1 live gap matrix** (all failures below are primal evolution gaps, not local experiment debt):

| Gap | Owner | Severity | What's Needed | Experiments Blocked |
|-----|-------|----------|---------------|---------------------|
| No UDS transport | **rhizoCrypt** | **CRITICAL** | Add `--unix` / `XDG_RUNTIME_DIR` socket support (only binds TCP:9401) | exp094, exp095, exp096, exp098 |
| Startup panic (runtime nesting) | **loamSpine** | **CRITICAL** | Fix `block_on` inside async runtime in `infant_discovery.rs:233` | exp095 |
| Fitts formula mismatch | **barraCuda** | HIGH | `activation.fitts` returns 800 for (d=256,w=32,a=200,b=150); Python expects 708.85. Likely using `log2(D/W)` instead of Shannon `log2(2D/W+1)` | exp089 |
| Hick formula variant | **barraCuda** | HIGH | `activation.hick` returns 675.49 for (n=8,a=200,b=150); Python expects 650. Using `log2(n+1)` instead of `log2(n)` | exp089 |
| Perlin3D lattice invariant | **barraCuda** | MEDIUM | `noise.perlin3d(0,0,0)` returns -0.11 instead of 0 (lattice points must be zero by gradient noise definition) | exp091 |
| Neural API capability registration | **biomeOS** | HIGH | Running primals not auto-registering `math`, `tensor`, `compute`, `dag`, `visualization`, `crypto` capabilities with Neural API | exp087, exp088 |
| Sovereign GPU dispatch readback | **toadStool + coralReef** | MEDIUM | toadStool reports "coralReef not available" even though coralReef socket exists (inter-primal discovery gap) | exp085 |
| Domain-level math methods | **barraCuda** | LOW | `math.flow.evaluate`, `math.engagement.composite` (composable from existing primitives) | exp084 |
| No binary in plasmidBin | **barraCuda** | HIGH | Need published ecoBin for plasmidBin deployment | all science exps |
| JWT secret generation in start_primal.sh | **plasmidBin** | LOW | Script generates 25-byte secret, NestGate requires 32+ | NestGate startup |

**Score**: 95/141 checks passing (67.4%). With rhizoCrypt UDS + loamSpine fix alone → estimated 115/141 (81.5%). With barraCuda formula fixes → estimated 125/141 (88.7%).

### Track 27: Science via Primal Composition — HCI Model Validation (V36)

The next evolution step: validate each HCI model's math purely through primal
IPC composition. Each experiment calls barraCuda IPC methods (activation.*,
math.*, tensor.*, noise.*, stats.*) and compares results to the same Python
baselines used in exp001-034. No ludoSpring binary participates in the science.

**Deploy graph**: `graphs/composition/science_validation.toml`

**HCI Model → barraCuda IPC Mapping**:
- Fitts/Hick/Steering → `activation.fitts`, `activation.hick`, `math.log2`
- Flow/DDA → `math.sigmoid` (flow curve, performance-to-difficulty)
- Engagement → `stats.weighted_mean` (composite), `tensor.create` + `tensor.scale` + `tensor.reduce`
- Four Keys → `tensor.create` + `tensor.scale` + `tensor.reduce` (composite score)
- GOMS KLM → `stats.mean`, `stats.weighted_mean`
- Perlin → `noise.perlin2d`, `noise.perlin3d` (direct IPC)
- WFC → `tensor.create` + `tensor.reduce` (constraint propagation)

| # | Package | Checks | **V37.1 live** | Composition Target | Blocking Gaps |
|---|---------|--------|----------------|--------------------|---------------|
| 089 | `ludospring-exp089` | 8 | **4/8** | Fitts + Hick + Steering via barraCuda IPC | barraCuda: Fitts/Hick formula mismatch |
| 090 | `ludospring-exp090` | 13 | **13/13 PASS** | Flow + Engagement + DDA via tensor composition | — |
| 091 | `ludospring-exp091` | 8 | **7/8** | Perlin + WFC via noise/tensor composition | barraCuda: perlin3d lattice invariant |
| 092 | `ludospring-exp092` | 8 | **8/8 PASS** | GOMS + Four Keys via Pipeline composition | — |
| 093 | `ludospring-exp093` | 6 | **6/6 PASS** | Full game session via Continuous composition | — |

### Track 28: NUCLEUS Game Engine Composition (V37)

Full game engine patterns validated via NUCLEUS composition. V36 proved barraCuda
can compute the math. V37 proves the entire engine stack can be composed from
NUCLEUS primals — session lifecycle, content ownership, RPGPT dialogue, Lysogeny
game mechanics, and a full NUCLEUS game tick. Every `game.*` capability in
`niche.rs` gets a demonstrated primal composition equivalent.

**Deploy graphs**: `graphs/composition/nucleus_game_session.toml` (full stack),
`graphs/composition/session_provenance.toml` (session lifecycle)

**NUCLEUS layers exercised**:
- Tower: BearDog (crypto.blake3_hash, crypto.sign_ed25519)
- Node: barraCuda (math.sigmoid, stats.weighted_mean, tensor.*, activation.*)
- Nest: NestGate (storage.store, storage.retrieve)
- Trio: rhizoCrypt (provenance.*), loamSpine (certificate.*), sweetGrass (attribution.*)
- Meta-tier: Squirrel (ai.query), petalTongue (visualization.render.scene)

**Capability mapping** (game.* → primal decomposition):
- `game.begin_session` → rhizoCrypt `provenance.session_create` (exp094)
- `game.record_action` → rhizoCrypt `provenance.vertex_append` (exp094)
- `game.complete_session` → rhizoCrypt + BearDog + NestGate (exp094)
- `game.mint_certificate` → loamSpine `certificate.mint` (exp095)
- `game.npc_dialogue` → Squirrel `ai.query` (exp096)
- `game.voice_check` → Squirrel `ai.query` with temperature (exp096)
- `game.evaluate_flow` → barraCuda `math.sigmoid` (exp096, exp098)
- Population dynamics → barraCuda `tensor.*` + `stats.*` (exp097)

| # | Package | Checks | **V37.1 live** | Composition Target | Blocking Gaps |
|---|---------|--------|----------------|--------------------|---------------|
| 094 | `ludospring-exp094` | 8 | **3/8** | Session lifecycle via Nest Atomic (BearDog + rhizoCrypt + NestGate) | rhizoCrypt: no UDS (TCP-only) |
| 095 | `ludospring-exp095` | 8 | **0/8** | Content ownership via Provenance Trio (loamSpine + sweetGrass + BearDog) | rhizoCrypt: no UDS, loamSpine: startup panic |
| 096 | `ludospring-exp096` | 10 | **5/10** | NPC dialogue via NUCLEUS (Squirrel + barraCuda + rhizoCrypt + petalTongue) | rhizoCrypt: no UDS, Squirrel/petalTongue: not started |
| 097 | `ludospring-exp097` | 10 | **10/10 PASS** | Population dynamics as tensor composition (replicator, LV, WF, Markov) | — |
| 098 | `ludospring-exp098` | 6 | **5/6** | NUCLEUS Complete game session (full stack, 10-tick continuous) | rhizoCrypt: no UDS |

### Track 29: Composition Validation — Rust → IPC Parity (V38)

Three-layer validation chain proving that calling ludoSpring's science methods
via JSON-RPC IPC produces identical results to calling the Rust library directly.
Golden targets generated from direct library calls serve the same role for IPC
composition that Python baselines serve for Rust code.

**Evolution path**:
  Python baseline → Rust library → **IPC composition (THIS TRACK)** → NUCLEUS deployment

**Artifacts**:
- `baselines/rust/composition_targets.json` — golden targets from direct Rust calls
- `baselines/rust/generate_composition_targets.rs` — generator (cargo example)
- 7 composition parity tests in `barracuda/tests/ipc_integration.rs`

| # | Package | Checks | Status | What it validates |
|---|---------|--------|--------|-------------------|
| 099 | `ludospring-exp099` | 13 | PASS* | Flow x2, Fitts x2, Engagement x2, Noise, DDA x3, Accessibility x2, WFC — all within ANALYTICAL_TOL (1e-10) |

*Requires live ludoSpring server on UDS. Graceful dry-mode when no server running.

### Track 30: NUCLEUS Composition Parity — Python → Rust → IPC → Primal (V40)

Three-layer chain complete. Python validated Rust; now both Python and Rust
validate primal composition patterns through the full NUCLEUS graph.

**Evolution path**:
  Python baseline → Rust library → IPC composition → **NUCLEUS primal graph (THIS TRACK)**

| # | Package | Checks | Status | What it validates |
|---|---------|--------|--------|-------------------|
| 100 | `ludospring-exp100` | 27 | PASS* | Niche integrity (7), health probes (2), capability discovery (4), science parity via IPC (8), golden chain Python→Rust→IPC (6) |

*Requires live ludoSpring server on UDS. Exit code 2 = skip (primals not running).

### metalForge Dispatch (Capability-Based Routing)

| Binary | Checks | Status | Modules Validated |
|--------|--------|--------|-------------------|
| `validate_dispatch_routing` | 7 | PASS | GPU/CPU workload routing for noise, WFC, raycaster |

metalForge forge library: 26 unit tests — `SubstrateKind` (Cpu/Gpu/Npu), `Capability` enum (F64/F32/Shader/SIMD/PCIe/QuantizedInference), `route()` with capability filtering, `fallback_chain()` (GPU>NPU>CPU), pipeline routing, workload dispatch.

### petalTongue Dashboards (UniBin subcommands)

| Subcommand | Scenarios | What it visualizes |
|------------|-----------|-------------------|
| `ludospring dashboard` | 8 | All 7 `GameChannelType` channels from validated math |
| `ludospring live-session` | 1 (streaming) | 120-tick game session with DDA, flow, engagement |
| `ludospring tufte-dashboard` | 3 | Genre comparison, minimap multiples, cognitive load sweep |

## Running

```bash
# Run a specific validation experiment
cargo run --bin exp017_bsp_level_generation

# Run metalForge dispatch
cargo run --bin validate_dispatch_routing

# Run petalTongue dashboards (UniBin subcommands)
cargo run --features ipc --bin ludospring -- dashboard
cargo run --features ipc --bin ludospring -- live-session
cargo run --features ipc --bin ludospring -- tufte-dashboard

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
cargo run --bin exp030_cpu_gpu_parity                 # CPU-vs-GPU parity (32 checks)
cargo run --bin exp031_dispatch_routing               # dispatch routing (10 checks)
cargo run --bin exp032_mixed_hardware                 # mixed hardware (23 checks)
cargo run --bin exp033_nucleus_pipeline               # NUCLEUS pipeline (27 checks)

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

# Run Primal Composition Validation — Gap Discovery (Track 26)
# Requires live primals: barraCuda, coralReef, toadStool, biomeOS neural-api
cargo run -p ludospring-exp084                             # barraCuda math IPC gaps (13 checks)
cargo run -p ludospring-exp085                             # Shader dispatch chain (8 checks)
cargo run -p ludospring-exp086                             # Tensor composition (9 checks)
cargo run -p ludospring-exp087                             # Neural API pipeline (7 checks)
cargo run -p ludospring-exp088                             # Continuous game loop (10 checks)

# Run Science via Primal Composition — HCI Model Validation (Track 27, V36)
# Requires live primals: barraCuda (+ biomeOS neural-api for exp093)
cargo run -p ludospring-exp089                             # Fitts + Hick + Steering (8 checks)
cargo run -p ludospring-exp090                             # Flow + Engagement + DDA (10 checks)
cargo run -p ludospring-exp091                             # Perlin + WFC noise/tensor (8 checks)
cargo run -p ludospring-exp092                             # GOMS + Four Keys pipeline (8 checks)
cargo run -p ludospring-exp093                             # Full game session continuous (6 checks)

# Run NUCLEUS Game Engine Composition (Track 28, V37)
# Requires live primals: varies per experiment (see check list)
cargo run -p ludospring-exp094                             # Session lifecycle (8 checks)
cargo run -p ludospring-exp095                             # Content ownership (8 checks)
cargo run -p ludospring-exp096                             # NPC dialogue composition (10 checks)
cargo run -p ludospring-exp097                             # Population dynamics (10 checks)
cargo run -p ludospring-exp098                             # NUCLEUS Complete session (6 checks)

# Run Composition Validation (Track 29, V38)
# Requires live ludoSpring server
cargo run -p ludospring-exp099                             # Composition validation (13 checks)

# Run NUCLEUS Composition Parity (Track 30, V40)
# Requires live ludoSpring server (exit 2 = skip)
cargo run -p ludospring-exp100                             # NUCLEUS parity (27 checks)

# Run all tests
cargo test --features ipc --lib --tests
```

## Validation Pattern

Every experiment follows the hotSpring validation pattern using `ValidationHarness`:

```rust
const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/interaction_laws.py",
    commit: "19e402c0",
    date: "2026-04-10",
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

All experiments use `ValidationHarness` with `BaselineProvenance` for reproducible validation.
