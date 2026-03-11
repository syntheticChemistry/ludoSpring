# ludoSpring Experiments

**Date:** March 11, 2026
**Total:** 22 experiments, 183 validation checks, 0 failures
**Pattern:** hotSpring validation — hardcoded expected values, explicit pass/fail, exit code 0/1

## Experiment Index

### Track 1: Core Game Systems

| # | Binary | Checks | What it validates |
|---|--------|--------|-------------------|
| 001 | `exp001_doom_raycaster_analysis` | 6 | Doom raycaster: DDA ray-grid, HUD Tufte ratio, Fitts targeting |
| 002 | `exp002_procedural_molecule_gen` | 5 | Procedural molecule world: noise→voxel, element mapping |
| 003 | `exp003_tufte_game_ui` | 6 | Genre UI Tufte: FPS vs RTS vs sandbox UI patterns |
| 004 | `exp004_folding_adversarial` | 5 | Folding adversarial: player vs AI, DDA flow maintenance |

### Track 2: Interaction Models

| # | Binary | Checks | What it validates |
|---|--------|--------|-------------------|
| 005 | `exp005_fitts_device_sweep` | 9 | Fitts's law device sweep: mouse, gamepad, gaze, voice, touch |
| 006 | `exp006_hick_menu_depth` | 6 | Hick's law menu depth: flat, hierarchical, radial |
| 007 | `exp007_steering_tunnel` | 5 | Steering law tunnel: D/W scaling, constraint navigation |
| 011 | `exp011_goms_task_completion` | 8 | GOMS task completion: KLM operator sequences |
| 012 | `exp012_flow_channel_calibration` | 13 | Flow channel calibration: width sweep, 5-state coverage |
| 019 | `exp019_composite_interaction_cost` | 6 | Composite interaction cost: Fitts+Hick+Steering+GOMS pipeline |

### Track 3: Procedural Generation

| # | Binary | Checks | What it validates |
|---|--------|--------|-------------------|
| 008 | `exp008_wfc_crystal_lattice` | 7 | WFC crystal lattice: NaCl adjacency, propagation |
| 009 | `exp009_noise_molecular_density` | 9 | Noise molecular density: fBm statistics, spatial coherence |
| 013 | `exp013_lsystem_protein_backbone` | 15 | L-system protein backbone: Fibonacci, Koch, turtle geometry |
| 014 | `exp014_hybrid_noise_wfc` | 5 | Hybrid noise+WFC: noise-seeded WFC, determinism |
| 017 | `exp017_bsp_level_generation` | 10 | BSP level generation: area conservation, spatial query |

### Track 4: Accessibility & Cognitive Load

| # | Binary | Checks | What it validates |
|---|--------|--------|-------------------|
| 015 | `exp015_accessibility_motor_limited` | 9 | Accessibility motor-limited: eye-gaze, head-pointer, switch, sip-puff |
| 016 | `exp016_cognitive_load_tufte` | 7 | Cognitive load Tufte: minimal→maximal HUD sweep |

### Track 5: Fun & Engagement Metrics

| # | Binary | Checks | What it validates |
|---|--------|--------|-------------------|
| 010 | `exp010_engagement_curves` | 14 | Engagement curves: flow states, DDA, player archetypes |
| 018 | `exp018_four_keys_fun` | 10 | Four Keys to Fun: archetype classification (6 games) |
| 020 | `exp020_difficulty_skill_balance` | 7 | Difficulty-skill balance: DDA adaptation, trend detection |
| 021 | `exp021_retention_reward_curves` | 7 | Retention reward curves: fixed/variable/intrinsic rewards |
| 022 | `exp022_small_multiples_minimap` | 7 | Small multiples minimap: Doom/RTS/RPG minimap Tufte analysis |

### metalForge Dispatch

| Binary | Checks | What it validates |
|--------|--------|-------------------|
| `validate_dispatch_routing` | 7 | GPU/CPU workload routing for noise, WFC, raycaster |

## Running

```bash
# Run a specific experiment
cargo run --bin exp017_bsp_level_generation

# Run metalForge dispatch
cargo run --bin validate_dispatch_routing

# Run all tests (includes library + parity + validation)
cargo test --workspace
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
