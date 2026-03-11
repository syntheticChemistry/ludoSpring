# ludoSpring

The seventh ecoPrimals spring. The science of play, interaction, and game design.

**Date:** March 11, 2026
**Version:** V2 (22 experiments, 183 validation checks, 119 unit tests)
**License:** AGPL-3.0-or-later
**barraCuda:** v0.3.3 (standalone, 150+ primitives available)

## What is ludoSpring?

Where wetSpring validates bioinformatics, hotSpring validates nuclear physics, and healthSpring builds health applications, **ludoSpring treats game design as a rigorous science** — validated against published HCI research, with Python baselines and Rust parity tests proving the math.

Games are the most demanding real-time interactive systems humans build. They solve problems every primal needs: input handling, spatial navigation, physics simulation, procedural content generation, accessibility, and the deep question of what makes interaction *engaging*.

## Domains

| Module | What it studies | Key models | Status |
|--------|----------------|------------|--------|
| `game` | Mechanics, state, genre taxonomy | Raycasting (DDA), voxel worlds, session state | Validated |
| `interaction` | Input science, flow, accessibility | Fitts, Hick, Steering, GOMS, Flow, DDA | All 4 HCI laws validated |
| `procedural` | Content generation | Perlin noise, fBm, WFC, L-systems, BSP trees | All 4 PCG algorithms validated |
| `metrics` | Quantifying fun | Tufte-on-games, engagement curves, Four Keys to Fun | All 3 frameworks validated |

### Foundational Research Coverage

Every model from the spec is implemented, validated, and has Python parity:

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

## Experiments

22 validation binaries + 1 metalForge routing = 23 binaries, 183 total checks:

| # | Name | Track | Checks | What it validates |
|---|------|-------|--------|-------------------|
| 001 | Doom raycaster analysis | 1 | 6 | DDA, HUD Tufte, Fitts targeting |
| 002 | Procedural molecule gen | 1 | 5 | Noise→voxel chemistry world |
| 003 | Tufte game UI | 1 | 6 | Genre UI comparison (FPS/RTS/sandbox) |
| 004 | Folding adversarial | 1 | 5 | Player vs AI with DDA + flow |
| 005 | Fitts device sweep | 2 | 9 | Mouse/gamepad/gaze/voice devices |
| 006 | Hick menu depth | 2 | 6 | Flat vs hierarchical vs radial menus |
| 007 | Steering tunnel | 2 | 5 | Tunnel navigation D/W scaling |
| 008 | WFC crystal lattice | 3 | 7 | NaCl adjacency rules, propagation |
| 009 | Noise molecular density | 3 | 9 | fBm statistics, spatial coherence |
| 010 | Engagement curves | 5 | 14 | Flow states, DDA, player archetypes |
| 011 | GOMS task completion | 2 | 8 | KLM operator sequences |
| 012 | Flow channel calibration | 2 | 13 | Channel width sweep, 5-state coverage |
| 013 | L-system protein backbone | 3 | 15 | Fibonacci, Koch, turtle geometry |
| 014 | Hybrid noise+WFC | 3 | 5 | Noise-seeded WFC, determinism |
| 015 | Accessibility motor-limited | 4 | 9 | Eye-gaze, head-pointer, switch, sip-puff |
| 016 | Cognitive load Tufte | 4/5 | 7 | Minimal→maximal HUD sweep |
| 017 | BSP level generation | 3 | 10 | Area conservation, spatial query |
| 018 | Four Keys to Fun | 5 | 10 | Archetype classification (6 games) |
| 019 | Composite interaction cost | 2 | 6 | Fitts+Hick+Steering+GOMS pipeline |
| 020 | Difficulty-skill balance | 5 | 7 | DDA adaptation, trend detection |
| 021 | Retention reward curves | 5 | 7 | Fixed/variable/intrinsic rewards |
| 022 | Small multiples minimap | 1 | 7 | Doom/RTS/RPG minimap Tufte analysis |

## Python Baselines

7 reference implementations in `baselines/python/` (stdlib only, no dependencies):

| Script | Models | Parity tests |
|--------|--------|-------------|
| `perlin_noise.py` | Perlin 2D/3D, lattice zeros | 3 |
| `interaction_laws.py` | Fitts, Hick, Steering | 4 |
| `flow_engagement.py` | Flow state classification | 1 |
| `goms_model.py` | KLM operator times | 5 |
| `lsystem_growth.py` | Algae, Koch, protein backbone | 3 |
| `bsp_partition.py` | BSP area conservation | 3 |
| `fun_keys_model.py` | Four Keys classification | 6 |

All baselines produce JSON output consumed by `barracuda/tests/python_parity.rs`.

## Architecture

```
ludoSpring/
├── barracuda/             # Core library (31 source files, 66 unit tests)
│   ├── src/
│   │   ├── game/          # Mechanics, raycaster, voxel, genre, state
│   │   ├── interaction/   # Fitts, Hick, Steering, GOMS, Flow, DDA
│   │   ├── procedural/    # Noise, WFC, L-systems, BSP
│   │   ├── metrics/       # Tufte, engagement, Four Keys to Fun
│   │   ├── tolerances/    # All constants with provenance (no magic numbers)
│   │   ├── validation/    # ValidationResult harness
│   │   ├── visualization/ # Data channels for any viz consumer
│   │   └── ipc/           # JSON-RPC 2.0 server (capability-based discovery)
│   └── tests/
│       └── python_parity.rs  # 22 cross-language parity tests
├── experiments/           # 22 hotSpring-pattern validation binaries
├── baselines/python/      # 7 Python reference implementations
├── benchmarks/            # Criterion benchmarks (noise, raycaster, ECS)
├── metalForge/forge/      # Hardware dispatch validation (7 checks)
├── specs/                 # 4 domain specifications
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

## Evolution Path

```
Python baseline → barraCuda CPU → GPU (WGSL) → sovereign pipeline (coralReef)
```

### GPU Shader Promotion Readiness

| Module | Tier | GPU target | Blocking |
|--------|------|-----------|----------|
| `procedural::noise` | A | Perlin/fBm compute shader | Nothing — pure math |
| `procedural::wfc` | A | Parallel constraint propagation | Nothing — grid-parallel |
| `procedural::bsp` | B | Recursive → iterative conversion | Stack elimination |
| `procedural::lsystem` | B | Parallel string rewriting | Variable-length output |
| `game::raycaster` | A | Per-column DDA (embarrassingly parallel) | Nothing |
| `metrics::engagement` | A | Batch evaluation | Nothing — dot product |
| `metrics::fun_keys` | A | Batch classification | Nothing — weighted sum |
| `interaction::flow` | A | Batch flow evaluation | Nothing — comparisons |

## Build

```bash
# All tests (119 unit + parity + validation + determinism)
cargo test --workspace

# Run a specific experiment
cargo run --bin exp017_bsp_level_generation

# Python baselines
python3 baselines/python/run_all_baselines.py

# Quality checks
cargo fmt --check
cargo clippy --workspace --all-targets -- -W clippy::pedantic
cargo doc --workspace --no-deps
```

## Quality

| Check | Result |
|-------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy --pedantic` | 0 warnings |
| `cargo test --workspace` | 119 tests, 0 failures |
| `cargo doc --no-deps` | Clean |
| 23 validation binaries | 183 checks, 0 failures |
| 7 Python baselines | All pass |
| `#![forbid(unsafe_code)]` | All crate roots |
| Files > 1000 LOC | None |
| TODO/FIXME/HACK in source | None |

## License

AGPL-3.0-or-later
