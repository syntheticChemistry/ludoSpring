# ludoSpring baseCamp — Game Design as Rigorous Science

**Date:** March 13, 2026
**Paper:** #17 in ecoPrimals baseCamp (gen3)
**Status:** Validated + Playable + Telemetry + Compute + Benchmarks + Controls + Cross-Spring + RPGPT + Games@Home + Provenance + Extraction Shooter + Composable Viz + Lysogeny + Fermenting + Cross-Spring Provenance — 66 experiments, 1349 checks, 138 tests, 3 playable prototypes, 3 game adapters, 3 external control groups, 4 cross-spring, 3 RPGPT, 4 Games@Home, 1 trio integration, 2 extraction shooter/viz, 6 lysogeny, 1 fermenting, 5 cross-spring provenance

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
| Compute Dispatch | CPU-GPU parity, routing, mixed hw, NUCLEUS | 030–033 | 49 |
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

### Cross-Spring Provenance

- **Python baselines** (7 scripts, stdlib only) → `barracuda/tests/python_parity.rs` (22 tests)
- **barraCuda primitives** consumed: `sigmoid`, `dot`, `lcg_step`, `state_to_f64`
- **Tolerances** centralized with citations in `tolerances/mod.rs`
- **petalTongue** integration: 3 dashboard binaries, all 7 `GameChannelType` channels wired
- **GPU promotion**: 8 modules Tier A (pure math, embarrassingly parallel)
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
| 030 | CPU-vs-GPU Parity | Pure Rust math matches GPU WGSL shaders within tolerance | `exp030_cpu_gpu_parity.md` |
| 031 | Dispatch Routing | Real wgpu adapter discovery + workload routing validation | `exp031_dispatch_routing.md` |
| 032 | Mixed Hardware | PCIe transfer cost, mixed pipelines, NPU mock, substrate scoring | `exp032_mixed_hardware.md` |
| 033 | NUCLEUS Pipeline | Tower/Node/Nest atomic coordination for game-science workloads | `exp033_nucleus_pipeline.md` |
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
cargo test --features ipc --lib --tests             # 138 Rust tests
cargo run --bin exp023_open_systems_benchmark        # benchmark: 16/16 checks
cargo run --bin exp024_doom_terminal                 # playable Doom walker
cargo run --bin exp025_roguelike_explorer            # playable roguelike
cargo run --bin exp026_game_telemetry -- validate   # telemetry protocol: 13/13 checks
cargo run --bin exp027_veloren_adapter -- validate  # Veloren adapter: 9/9 checks
cargo run --bin exp028_fishfolk_adapter -- validate # Fish Folk adapter: 7/7 checks
cargo run --bin exp029_abstreet_adapter -- validate # A/B Street adapter: 8/8 checks
cargo run --bin exp030_cpu_gpu_parity               # CPU-vs-GPU parity: 16/16 checks
cargo run --bin exp031_dispatch_routing              # dispatch routing: 10/10 checks
cargo run --bin exp032_mixed_hardware                # mixed hardware: 12/12 checks
cargo run --bin exp033_nucleus_pipeline              # NUCLEUS pipeline: 11/11 checks
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
cargo run --features ipc --bin ludospring_dashboard  # petalTongue visualization
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

**Full specification**: `ludoSpring/specs/RPGPT_ARCHITECTURE_SKETCH.md`
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
