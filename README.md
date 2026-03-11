# ludoSpring — The Science of Play, Interaction, and Game Design

An ecoPrimals Spring. Treats game design with the same rigor that wetSpring treats bioinformatics and hotSpring treats nuclear physics: validated models, reproducible experiments, GPU-accelerated computation where it matters.

**Date:** March 11, 2026
**Version:** V3 (29 experiments, 236 validation checks, 133 tests)
**License:** AGPL-3.0-or-later
**MSRV:** 1.87 (edition 2024)
**barraCuda:** v0.3.3 (standalone, 150+ primitives)

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
├── experiments/           # 29 experiments (22 validation + 3 playable + 4 telemetry)
├── baselines/python/      # 7 Python reference implementations
├── benchmarks/            # Criterion benchmarks (noise, raycaster, ECS)
├── metalForge/forge/      # Hardware dispatch validation (7 checks)
├── specs/                 # 4 domain specifications
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
# All tests (81 unit + 8 determinism + 22 parity + 12 validation)
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
| `cargo fmt --check` | Clean |
| `cargo clippy --pedantic` | 0 warnings (new code) |
| `cargo test` | 133 tests, 0 failures |
| `cargo doc --no-deps` | Clean |
| 30 validation binaries | 236 checks, 0 failures |
| 7 Python baselines | All pass |
| `#![forbid(unsafe_code)]` | All crate roots |
| Files > 1000 LOC | None |
| TODO/FIXME/HACK in source | None |

## License

AGPL-3.0-or-later
