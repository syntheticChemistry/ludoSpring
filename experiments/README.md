# ludoSpring Experiments

**Date:** March 11, 2026
**Total:** 22 experiments, 183 validation checks, 0 failures
**Pattern:** hotSpring validation — hardcoded expected values, explicit pass/fail, exit code 0/1

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
# Run a specific experiment
cargo run --bin exp017_bsp_level_generation

# Run metalForge dispatch
cargo run --bin validate_dispatch_routing

# Run petalTongue dashboards
cargo run --features ipc --bin ludospring_dashboard
cargo run --features ipc --bin ludospring_live_session
cargo run --features ipc --bin ludospring_tufte_dashboard

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
