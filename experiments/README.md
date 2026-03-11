# ludoSpring Experiments

**Date:** March 11, 2026
**Total:** 44 experiments (22 validation + 3 playable + 4 telemetry + 4 compute + 4 benchmark + 3 control + 4 cross-spring), 410 checks, 0 failures
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
| 030 | `exp030_cpu_gpu_parity` | 16 | PASS | barraCuda CPU, WGSL shaders, wgpu 28 | CPU-vs-GPU parity (sigmoid, relu, dot, softmax, LCG, reduce) |
| 031 | `exp031_dispatch_routing` | 10 | PASS | toadStool substrate, wgpu adapter API | Hardware discovery, workload routing |
| 032 | `exp032_mixed_hardware` | 12 | PASS | PCIe specs, barraCuda unified_hardware | Transfer cost, mixed pipelines, NPU mock, scoring |
| 033 | `exp033_nucleus_pipeline` | 11 | PASS | biomeOS nucleus_complete.toml | Tower/Node/Nest atomic coordination |

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

### metalForge Dispatch

| Binary | Checks | Status | Modules Validated |
|--------|--------|--------|-------------------|
| `validate_dispatch_routing` | 7 | PASS | GPU/CPU workload routing for noise, WFC, raycaster |

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
cargo run --bin exp030_cpu_gpu_parity                 # CPU-vs-GPU parity (16 checks)
cargo run --bin exp031_dispatch_routing               # dispatch routing (10 checks)
cargo run --bin exp032_mixed_hardware                 # mixed hardware (12 checks)
cargo run --bin exp033_nucleus_pipeline               # NUCLEUS pipeline (11 checks)

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

# Run all tests
cargo test --features ipc --lib --tests
```

## Validation Pattern

Every experiment follows the hotSpring validation pattern:

```rust
let result = ValidationResult::new("test_name")
    .expected(expected_value)
    .actual(actual_value)
    .tolerance(tolerance)
    .evaluate();
```

- Hardcoded expected values from documented Python baselines
- Explicit pass/fail with tolerance justification
- Exit code 0 = all pass, exit code 1 = any failure
- Summary printed to stdout with check counts
