# ludoSpring baseCamp — Game Design as Rigorous Science

**Date:** March 11, 2026
**Paper:** #17 in ecoPrimals baseCamp (gen3)
**Status:** Validated + Playable + Telemetry + Compute + Benchmarks — 37 experiments, 330 checks, 2 playable prototypes, 3 game adapters

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

### Cross-Spring Provenance

- **Python baselines** (7 scripts, stdlib only) → `barracuda/tests/python_parity.rs` (22 tests)
- **barraCuda primitives** consumed: `sigmoid`, `dot`, `lcg_step`, `state_to_f64`
- **Tolerances** centralized with citations in `tolerances/mod.rs`
- **petalTongue** integration: 3 dashboard binaries, all 7 `GameChannelType` channels wired
- **GPU promotion**: 8 modules Tier A (pure math, embarrassingly parallel)

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
cargo test --features ipc --lib --tests              # 123 Rust tests
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
cargo run --features ipc --bin ludospring_dashboard  # petalTongue visualization
```

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
