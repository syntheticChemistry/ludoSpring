# ludoSpring → barraCuda/toadStool GPU Evolution Handoff — March 11, 2026

**From:** ludoSpring V2
**To:** barraCuda team, toadStool team, coralReef team
**License:** AGPL-3.0-or-later
**Date:** March 11, 2026

---

## Executive Summary

- ludoSpring has **validated 13 mathematical models** against published HCI research (Fitts 1954, Hick 1952, Accot 1997, Card 1983, Csikszentmihalyi 1990, Hunicke 2005, Lazzaro 2004, Perlin 1985, Gumin 2016, Lindenmayer 1968, Fuchs 1980, Tufte 1983, Yannakakis 2018) with Python parity proving faithful port
- **8 modules are Tier A** — ready for WGSL shader promotion today
- **4 absorption candidates** provide new math primitives for the barraCuda library
- The game engine niche is the **first continuous consumer** — 60 Hz, not run-to-completion. This drives toadStool streaming evolution.

---

## Part 1: GPU-Ready Math Inventory

### 1.1 Perlin Noise (procedural::noise)

**CPU Reference:** `perlin_2d(x, y, seed)` — gradient noise via lattice hashing.

```
Inputs:  (x: f64, y: f64, seed: u64)
Output:  f64 in [-1.0, 1.0]
Math:    floor → gradient dot → fade → lerp
GPU ops: floor, fract, mix, dot (all GPU-native)
```

**WGSL shader sketch:**
```wgsl
@compute @workgroup_size(256)
fn perlin_2d(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let x = input[idx * 2u];
    let y = input[idx * 2u + 1u];
    // lattice coordinates
    let xi = floor(x);
    let yi = floor(y);
    let xf = x - xi;
    let yf = y - yi;
    // fade curves
    let u = xf * xf * xf * (xf * (xf * 6.0 - 15.0) + 10.0);
    let v = yf * yf * yf * (yf * (yf * 6.0 - 15.0) + 10.0);
    // gradient dot products + bilinear interpolation
    // ... (hash → gradient → dot → mix)
    output[idx] = mix(mix(n00, n10, u), mix(n01, n11, u), v);
}
```

**fBm extension:** Octave accumulation is a simple loop over `perlin_2d` with halving amplitude — maps to single shader with configurable octave count.

**barraCuda action:** Absorb as `procedural::perlin_2d_f64` compute shader (op candidate).
**toadStool action:** Wire dispatch for `(width × height)` grid — one invocation per texel.

### 1.2 Wave Function Collapse (procedural::wfc)

**CPU Reference:** Constraint propagation on grid with entropy-based cell selection.

```
Inputs:  (width: u32, height: u32, rules: &[AdjacencyRule], seed: u64)
Output:  Grid<TileId>
Math:    entropy calculation (log2), constraint propagation (bitset AND), collapse (weighted random)
```

**GPU parallelism:** The propagation step is massively parallel — each cell can check its neighbors independently. Barrier sync between propagation rounds.

**barraCuda action:** New `procedural::wfc_propagate_f64` shader — parallel constraint check per cell.
**toadStool action:** Multi-pass dispatch with barrier between propagation rounds.

### 1.3 DDA Raycaster (game::raycaster)

**CPU Reference:** Per-column Digital Differential Analyzer for grid traversal.

```
Inputs:  (player_pos: Vec2, player_angle: f64, fov: f64, map: &Grid, column: u32)
Output:  (distance: f64, wall_hit: WallHit)
Math:    sin, cos, floor, step (all GPU-native via trig)
```

**GPU parallelism:** Embarrassingly parallel — each screen column is independent.

**barraCuda action:** Absorb as `game::dda_raycast_f64` compute shader.
**toadStool action:** Dispatch 320+ columns per frame at 60 Hz. This is the first toadStool streaming workload for ludoSpring.

### 1.4 Engagement Metrics (metrics::engagement)

**CPU Reference:** Weighted dot product of metric vector × weight vector.

```
Inputs:  (metrics: [f64; N], weights: [f64; N])
Output:  f64 composite score
Math:    dot product → clamp
```

**GPU parallelism:** Trivial — `fused::map_reduce_f64` already in barraCuda.

**barraCuda action:** Wire `engagement_batch` as `batched_dot_f64` workload.
**toadStool action:** Batch dispatch for N player sessions.

### 1.5 Four Keys to Fun (metrics::fun_keys)

**CPU Reference:** Weighted classification into Hard Fun / Easy Fun / Altered States / People Factor.

```
Inputs:  (signals: FunSignals — 8 f64 behavioral inputs)
Output:  (dominant: FunKey, scores: [f64; 4])
Math:    weighted sum → argmax
```

**barraCuda action:** Absorb as `metrics::fun_classify_batch` — element-wise weighted sum.
**toadStool action:** Batch dispatch, same pattern as engagement.

### 1.6 BSP Trees (procedural::bsp)

**CPU Reference:** Recursive binary spatial partitioning with deterministic LCG splits.

**GPU challenge:** Recursion → iterative. Pattern: pre-allocate node array, use stack-free BVH traversal.

**barraCuda action:** Tier B — needs iterative `generate_bsp_iterative` reference first.
**toadStool action:** Deferred until iterative CPU reference is validated.

### 1.7 L-systems (procedural::lsystem)

**CPU Reference:** String rewriting with production rules, turtle interpretation.

**GPU challenge:** Variable-length output from rewriting step. Pattern: two-pass (count output length, then generate).

**barraCuda action:** Tier B — needs two-pass GPU reference.
**toadStool action:** Deferred until two-pass CPU reference is validated.

## Part 2: Precision Characteristics

All ludoSpring math uses f64 exclusively. Precision observations:

| Operation | Precision notes |
|-----------|----------------|
| Perlin noise | Fade curve `6t^5 - 15t^4 + 10t^3` — `mul_add` chains, no precision concern |
| Fitts's law | `log2(D/W + 1)` — well-conditioned for typical `D/W` ratios (1–100) |
| Hick's law | `log2(n + 1)` — integer `n`, exact in f64 for `n < 2^53` |
| Steering law | Linear `a + b·(D/W)` — trivial |
| Flow state | Comparison-only — no floating-point accumulation |
| Engagement | Dot product of 3–7 elements — `mul_add` chain, negligible error |
| BSP | Split ratio from LCG — deterministic, not precision-sensitive |
| WFC | Entropy via `log2` — well-conditioned for non-degenerate weights |
| L-system | Turtle angle via `sin`/`cos` — GPU transcendentals sufficient |
| Fun Keys | Weighted sum of 8 elements — `mul_add` chain |

**Conclusion:** No DF64 needed. All ludoSpring math is within f64 native precision. This means consumer GPUs (RTX 4070) can run ludoSpring shaders without the DF64 emulation layer that hotSpring/wetSpring require.

## Part 3: toadStool Streaming Evolution

The game engine niche introduces a new toadStool dispatch pattern: **continuous streaming**.

### Current toadStool patterns (run-to-completion)
```
CPU → submit shader → GPU → readback → CPU → next step
```

### Game engine pattern (60 Hz continuous)
```
Frame N:  Input → GameLogic(CPU) → Render(GPU) → Present
Frame N+1: Input → GameLogic(CPU) → Render(GPU) → Present
... at 16.67ms cadence
```

### What this means for toadStool

1. **Pipeline persistence:** GPU pipelines must survive across frames (no recreate per frame)
2. **Buffer ring:** Double/triple buffering for CPU→GPU data flow without stalls
3. **Async readback:** GPU results from frame N read back during frame N+1 setup
4. **Priority dispatch:** Noise generation (background) vs raycasting (latency-critical) need different queue priorities

### What this means for barraCuda

1. **Persistent pipeline cache:** Compiled shaders cached for frame reuse
2. **Staging buffer pool:** Reusable staging buffers instead of per-dispatch allocation
3. **Fused multi-op:** Noise→WFC→raycast in single `queue.submit()` where possible

## Part 4: Cross-Spring Lessons for barraCuda Evolution

### 4.1 Tolerance Architecture

ludoSpring centralized all magic numbers in `tolerances/mod.rs` with provenance:

```rust
/// MacKenzie (1992) Fitts's law: a + b·log2(D/W + 1)
pub const FITTS_A_MS: f64 = 50.0;
pub const FITTS_B_MS: f64 = 150.0;
```

**barraCuda team:** This pattern (named constants with doc-comment citations) should be the ecosystem standard. wetSpring has 180 named tolerances. ludoSpring has 20+. Each spring's tolerance module is a domain-specific constants library.

### 4.2 Python Parity Testing Pattern

ludoSpring's `baselines/python/run_all_baselines.py` produces JSON; `barracuda/tests/python_parity.rs` consumes it and asserts exact match. This is the same pattern wetSpring uses for 22 baseline checks.

**barraCuda team:** Consider absorbing this test pattern as `barracuda::testing::python_parity` — a standard harness for any spring to register Python baselines.

### 4.3 Capability-Based IPC

ludoSpring's IPC uses capability advertisement (`game.evaluate_flow`, `game.classify_fun`, etc.) with XDG socket path resolution and zero hardcoded primal names. This is the pattern all springs should follow.

### 4.4 Deterministic Procedural Generation

ludoSpring uses `barracuda::rng::lcg_step` (not `rand`) for deterministic PCG across CPU and GPU. Same LCG seed → same BSP tree on any hardware.

**barraCuda team:** The LCG primitive is critical for GPU determinism. Consider adding `lcg_step_u64` as a WGSL shader utility for use across springs.

## Part 5: Action Items

### barraCuda actions (absorption)

| Priority | Action | Lines |
|----------|--------|-------|
| P1 | `perlin_2d_f64` compute shader from ludoSpring reference | ~100 |
| P1 | `fbm_2d_f64` compute shader (octave loop over Perlin) | ~50 |
| P2 | `dda_raycast_f64` compute shader from ludoSpring raycaster | ~80 |
| P2 | `engagement_batch_f64` via existing `fused::map_reduce_f64` | ~30 |
| P3 | `fun_classify_batch_f64` element-wise weighted sum | ~40 |
| P3 | `flow_eval_batch_f64` comparison shader | ~30 |
| P3 | `lcg_step_u64` WGSL utility for deterministic PCG | ~10 |

### toadStool actions (dispatch)

| Priority | Action |
|----------|--------|
| P1 | Wire Perlin noise grid dispatch (width×height compute) |
| P1 | Wire DDA raycaster column dispatch (N columns compute) |
| P2 | Wire WFC propagation with barrier sync |
| P2 | Evaluate continuous pipeline persistence for 60 Hz dispatch |
| P3 | Ring buffer staging for frame-to-frame streaming |

### coralReef actions (compilation)

| Priority | Action |
|----------|--------|
| P1 | Ensure `log2` f64 lowering works for Fitts/Hick shaders |
| P1 | Validate `sin`/`cos` f64 for turtle interpretation shaders |
| P2 | Profile fused noise→raycast shader compilation |

### metalForge integration

ludoSpring game engine workloads map to metalForge stages:

```
Weather(CPU) → Noise(GPU) → WFC(GPU) → Raycast(GPU) → Engagement(CPU) → Present
```

The GPU stages (Noise→WFC→Raycast) should stay on-device with zero CPU round-trips, matching the pattern validated in healthSpring V19 (Exp087) and airSpring v0.6.4 (metalForge 7-stage).
