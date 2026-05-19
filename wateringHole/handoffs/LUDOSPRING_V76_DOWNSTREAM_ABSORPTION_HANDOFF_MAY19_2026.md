# ludoSpring V76 — Downstream Absorption Guide

**Date:** May 19, 2026
**From:** ludoSpring (Tower Atomic, game science specialist)
**To:** projectNUCLEUS, projectFOUNDATION, esotericWebb, petalTongue
**Version:** V76 (982 tests, zero debt, all Priority 1 papers implemented)
**Wave:** 20 PM compliant (stability tiers, degradation contracts, cross-tier parity)

---

## Executive Summary

ludoSpring V76 completes three major subsystems ready for downstream consumption:

1. **Schell Game Design Lenses** — 20-lens structured quality evaluation for any
   game/interaction system (RPGPT planes validated; applicable to esotericWebb scenes)
2. **CPU/GPU Parity Framework** — Validates pure Rust math against GPU dispatch;
   ready for projectNUCLEUS gate-level execution and barraCuda absorption
3. **NUCLEUS Atomics Composition Validation** — Models and validates mixed-hardware
   composition graphs (Tower/Node/Nest, NPU→GPU DirectP2P bypass)

All Priority 1 papers from the queue are implemented. Thread 9 (Gaming) and
Thread 10 (Provenance) foundation targets are validated. The codebase is at
zero debt (zero warnings, zero unsafe, zero TODO/FIXME).

---

## Part 1: For projectNUCLEUS

### New Workload: metalForge Parity Validation

The existing 2 workloads (`ludospring-game-validation.toml`, `ludospring-composition-parity.toml`)
should be joined by a metalForge parity workload:

```toml
# ludospring-metalforge-parity.toml
[metadata]
name = "ludospring-metalforge-parity"
description = "CPU vs GPU parity + NUCLEUS atomics composition validation"
spring = "ludoSpring"
thread = 9
tags = ["parity", "gpu", "nucleus", "metalforge"]

[execution]
type = "native"
command = "$SPRINGS_ROOT/ludoSpring/target/release/ludospring validate --tier parity --format json"
working_dir = "$SPRINGS_ROOT/ludoSpring"

[output]
schema = "toadstool-validate-v1"
assertions = ["all_passed == true", "parity_checks >= 128"]
```

### Gate Dispatch Guidance

| Workload | Best Gate | Reason |
|----------|-----------|--------|
| `ludospring-game-validation` | flockgate / northgate | CPU-only, any hardware |
| `ludospring-composition-parity` | flockgate | Requires live primals for IPC |
| `ludospring-metalforge-parity` | **northgate (RTX 5090)** | GPU parity needs live GPU |

### NUCLEUS Atomics Validation

`metalForge/forge/src/nucleus.rs` provides `CompositionGraph::validate()` which
can be called from biomeOS graph builder during deploy-time validation:

```rust
use ludospring_forge::nucleus::{canonical_composition, CompositionGraph};

let graph = canonical_composition();
let result = graph.validate();
assert!(result.valid, "composition errors: {:?}", result.errors);
// result.suggestions contains optimization hints (e.g., DirectP2P for NPU→GPU)
```

**Integration point:** biomeOS `composition.deploy` could call this validation
before bringing up a mixed-hardware NUCLEUS. Transfer overhead estimation
(`result.transfer_overhead_ms`) feeds into frame budget planning.

---

## Part 2: For projectFOUNDATION

### Thread 9 — New Validated Targets

The following targets are now validated and can be added to
`data/targets/thread09_gaming_targets.toml`:

| Target | Validator | Paper | Status |
|--------|-----------|-------|--------|
| `schell_lens_evaluation` | `lenses::evaluate_plane` (all 7 planes) | Schell 2008 | PASS (V76) |
| `schell_lens_comparison` | `lenses::compare_planes` (differential) | Schell 2008 | PASS (V76) |
| `cpu_gpu_parity_perlin` | `parity::tier_a_parity_suite` | metalForge | PASS (V76) |
| `cpu_gpu_parity_fbm` | `parity::tier_a_parity_suite` | metalForge | PASS (V76) |
| `cpu_gpu_parity_sigmoid` | `parity::tier_a_parity_suite` | metalForge | PASS (V76) |
| `cpu_gpu_parity_raycaster` | `parity::tier_a_parity_suite` | metalForge | PASS (V76) |
| `nucleus_composition_valid` | `nucleus::canonical_composition` | metalForge | PASS (V76) |

### Updated Paper Count

Thread 9 now covers **16 foundational models** (was 15):
- Added: Schell (2008) — Game Design Lenses

### Literature Source Addition

For `data/sources/thread09_gaming.toml`:

```toml
[[sources]]
id = "schell_2008"
author = "Schell, Jesse"
year = 2008
title = "The Art of Game Design: A Book of Lenses"
publisher = "CRC Press"
isbn = "978-0123694966"
notes = "20 analytical lenses implemented in ludoSpring V76 (game/rpgpt/lenses.rs). Structural evaluation of 7 RPGPT planes. Cross-domain: applicable to any interaction system design."
spring_check = "game::rpgpt::lenses::tests::all_planes_evaluable"
```

---

## Part 3: For esotericWebb

### GAP-021 Evolution Path (Game-Science Primal)

esotericWebb's GAP-021 asks for a dedicated game-science primal. The evolution
path is:

```
Current:  ludoSpring local science (flow, engagement, DDA)
    ↓
Next:     barraCuda absorbs metalForge parity + procedural gen + interaction laws
    ↓
Future:   barraCuda exposes game-science capabilities via IPC
    ↓
Resolve:  esotericWebb consumes via capability.call("game_science", ...)
```

**What's ready now for esotericWebb to absorb locally (pattern-level):**

| Module | What It Provides | esotericWebb Use |
|--------|------------------|------------------|
| `game/rpgpt/lenses.rs` | 20-lens plane quality evaluation | Scene quality scoring for CRPG |
| `metrics/player_types.rs` | Bartle profile + population dynamics | NPC archetype assignment |
| `metrics/gamification.rs` | Engagement decay, half-life | Session pacing, reward timing |
| `metrics/mda.rs` | MDA aesthetic distribution | Scene aesthetic balance |
| `game/rpgpt/personality_dynamics.rs` | NPC personality drift under social pressure | Faction dynamics |

### GAP-002 Evolution Path (petalTongue Dialogue Trees)

ludoSpring's RPGPT system provides the data model for dialogue rendering:

```rust
// From barracuda/src/game/rpgpt/scene.rs — scene composition for petalTongue
GameScene {
    nodes: Vec<DialogueNode>,      // branching conversation tree
    active_npcs: Vec<NpcSnapshot>, // trust, knowledge, voice state
    plane: PlaneType,              // current game mode (Dialogue, etc.)
    narration: NarrationGuide,     // tone, pacing, vocabulary, perspective
}
```

**petalTongue absorption:** This structure maps directly to a
`DataBinding::DialogueTree` scene type. The Schell Lens evaluation provides
quality metadata that can render as a radar overlay on the dialogue UI.

### Composition Pattern: Local Science with IPC Shape

esotericWebb absorbed this pattern from ludoSpring V30-V32. The key principle:

```rust
// Local computation — ALWAYS available
let flow_score = science::evaluate_flow(skill, challenge);

// IPC shape — same interface, ready for future primal swap (GAP-021)
pub fn evaluate_flow(skill: f64, challenge: f64) -> FlowResult {
    // Today: local pure Rust math
    // Tomorrow: capability.call("game_science.evaluate_flow", {skill, challenge})
    FlowResult { zone: compute_zone(skill, challenge), ... }
}
```

---

## Part 4: For petalTongue

### Visualization Opportunities from V76

| Data Source | Visualization Type | DataBinding Shape |
|-------------|-------------------|-------------------|
| Schell Lens evaluation (20 axes) | Radar/spider chart | `RadarChart { axes: 20, scores: [f64; 20] }` |
| Lens comparison (2 planes) | Dual radar overlay | `RadarChart` × 2 with delta highlighting |
| CPU/GPU parity results | Error distribution heatmap | `Heatmap { x: workload, y: point_idx, value: error }` |
| NUCLEUS composition graph | Node/edge diagram | `Graph { nodes: Atomic[], edges: Signal[] }` |
| Transfer overhead budget | Stacked bar chart | `BarChart { bands: [cpu, gpu, npu, pcie, render] }` |
| Bartle population dynamics | 4-quadrant scatter | `ScatterPlot { x: action/interaction, y: world/player }` |
| Engagement decay curves | Line chart with half-life marker | `TimeSeries { decay_fn, half_life_marker }` |

### Priority for CRPG Dashboard (esotericWebb + petalTongue)

1. **Dialogue tree rendering** (GAP-002) — NPC conversation branching
2. **Lens radar chart** — Real-time scene quality during play
3. **Population dynamics scatter** — Faction balance visualization
4. **Engagement decay** — Session pacing indicator

---

## Part 5: Stability and Integration Contracts

### Method Stability (Wave 20 PM Compliant)

All 23 `game.*` methods are `stability = "stable"`. Consumers can depend on
these wire names indefinitely. The 5 `game.gpu.*` methods are `stability = "evolving"`
(names may change as toadStool GPU dispatch solidifies).

### Degradation Contract

ludoSpring's `domain_logic` is ALWAYS `fully_operational` regardless of primal
state. See `docs/DEGRADATION_BEHAVIOR.md` for the per-capability table.

### Cross-Tier Parity Contract

Python baselines → Rust library → IPC composition produce identical results
within `ANALYTICAL_TOL = 1e-10`. See `docs/CROSS_TIER_PARITY.md`.

### Trio Transaction Compliance

Provenance is enrichment, never a gate. Partial completion is reported via
`TrioStage` (Unavailable/Dehydrated/Committed/Complete). No rollback.
DAG sessions are append-only.

---

## Metrics

| Metric | Value |
|--------|-------|
| Workspace tests | 982 |
| Foundational models validated | 16 |
| Clippy warnings | 0 |
| Unsafe code | 0 |
| Deep debt markers | 0 |
| Stability: stable methods | 27 |
| Stability: evolving methods | 5 |
| metalForge parity checks | 128 |
| NUCLEUS validation checks | 9 |
| Thread 9 targets PASS | 13/13 |
| Thread 10 targets validated | 1/1 (game_session_provenance) |
| Priority 1 papers remaining | 0 |

---

## Action Items

| # | Item | Owner | Priority |
|---|------|-------|----------|
| 1 | Add metalForge parity workload TOML | projectNUCLEUS | P1 |
| 2 | Add Schell 2008 to thread09_gaming.toml sources | projectFOUNDATION | P2 |
| 3 | Add 7 new targets to thread09_gaming_targets.toml | projectFOUNDATION | P2 |
| 4 | Integrate CompositionGraph::validate() in biomeOS deploy | biomeOS / projectNUCLEUS | P1 |
| 5 | Scope DataBinding::DialogueTree scene type (GAP-002) | petalTongue | P2 |
| 6 | Scope DataBinding::RadarChart for Schell Lens viz | petalTongue | P3 |
| 7 | Absorb metalForge parity pattern into barraCuda (GAP-021 path) | barraCuda | P2 |
| 8 | Update ludospring workloads to reflect V76 test count | projectNUCLEUS | P3 |
