<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# ludoSpring V17 — barraCuda + toadStool Deep Evolution Handoff

**Date**: March 15, 2026
**From**: ludoSpring (game science spring)
**To**: barraCuda team (math primitives), toadStool team (GPU dispatch)
**Covers**: V16 → V17 (deep audit, code evolution, shader extraction, proptest, tracing)
**Status**: Complete — 66 experiments, 1371 checks, 234 tests + 12 proptest, 0 clippy warnings

---

## Executive Summary

V17 is a deep audit and evolution pass on V16's validated foundation. Key deliverables
for the upstream teams:

- **11 standalone WGSL shaders** extracted to `.wgsl` files — ready for toadStool absorption
- **GPU pipeline boilerplate consolidated** — shared helpers eliminate 600+ LOC of duplicate wgpu setup
- **12 proptest invariants** validating structural properties (BSP area conservation, noise bounds, WFC entropy)
- **Structured tracing** replaces all library `eprintln!` — biomeOS Pathway Learner can now scrape structured spans
- **Capability-based visualization discovery** — no primal name hardcoding anywhere in the codebase
- **Baseline drift checker** — automated Python re-run and diff (zero drift confirmed)
- **Zero `#[allow()]`** in production code — all clippy lints centralized in `Cargo.toml`

---

## Part 1: WGSL Shaders Ready for toadStool Absorption

### Extracted shaders (exp030/shaders/)

All 11 shaders are now standalone `.wgsl` files with SPDX headers, validation provenance
comments, and documented f32-vs-f64 tolerance expectations:

| Shader | Workgroup | Bindings | CPU reference | GPU tolerance |
|--------|-----------|----------|---------------|--------------|
| `sigmoid.wgsl` | 64 | 2 (in, out) | `barraCuda::activations::sigmoid` | < 1e-6 |
| `relu.wgsl` | 64 | 2 | `f32::max(x, 0.0)` | exact |
| `dot_product.wgsl` | 256 | 3 (a, b, result) | `barraCuda::stats::dot` | < 1e-4 |
| `reduce_sum.wgsl` | 256 | 2 (in, out) | `Iterator::sum` | < 1.0 |
| `softmax.wgsl` | 1 | 2 | numerically stable softmax | < 1e-5 |
| `scale.wgsl` | 64 | 2 | `x * 2.0 + 1.0` | exact |
| `lcg.wgsl` | 64 | 2 (seeds, out) | `barraCuda::rng::lcg_step` (u32) | exact |
| `abs.wgsl` | 64 | 2 | `f32::abs` | exact |
| `perlin_2d.wgsl` | 64 | 3 (perm, coords, out) | `procedural::noise::perlin_2d` | < 1e-3 |
| `engagement_batch.wgsl` | 64 | 3 (components, weights, out) | `metrics::engagement::compute_engagement` | < 1e-4 |
| `dda_raycast.wgsl` | 64 | 4 (map, params, angles, distances) | `game::raycaster::cast_ray` | < 0.5 |

**toadStool action:** These shaders validate against barraCuda CPU primitives. The absorption
path is: ludoSpring validates → toadStool absorbs shader → barraCuda provides `dispatch::run_shader()` → ludoSpring leans on upstream.

### coralReef requirements (unchanged from V16)

All shaders use:
- `f32` arithmetic only (no `f64`)
- `exp()`, `log2()`, `cos()`, `sin()`, `abs()`, `max()`, `clamp()`, `floor()`
- No dynamic memory allocation
- Fixed-size workgroup shared memory (`var<workgroup>` in reduce_sum only)
- No texture/sampler bindings

---

## Part 2: GPU Pipeline Consolidation Learnings

### Before (V16): copy-paste boilerplate

Each `gpu_run_*` function duplicated ~130 lines of buffer creation, bind group layout,
pipeline compilation, dispatch, staging, and readback. Total: 1032 lines for 7 runners.

### After (V17): shared helper pattern

| Helper | Responsibility | LOC saved per caller |
|--------|---------------|---------------------|
| `storage_entry(binding, read_only)` | Bind group layout entry (now `const fn`) | ~10 |
| `create_storage_buf(ctx, label, size, writable)` | Buffer creation with usage flags | ~6 |
| `build_pipeline(ctx, label, shader_src, entries)` | Shader → pipeline in one call | ~35 |
| `dispatch_and_read_f32(ctx, pipeline, bg, output, n, wg_size, default)` | Encode, submit, readback | ~25 |
| `read_staging_f32` / `read_staging_u32` | Map-async + channel readback | ~15 |

Result: 1032 → 413 LOC (60% reduction). Each runner is now a focused 20-30 line orchestrator.

**toadStool action:** This helper pattern could become a `toadStool::compute::Pipeline` builder.
The shared buffer layout definitions (`storage_entry`) are the same across all springs.

---

## Part 3: Proptest Invariants for Absorption Targets

Added 12 property-based tests that complement the fixed-value determinism tests.
These catch edge cases across random inputs:

| Property | Algorithm | Invariant |
|----------|-----------|-----------|
| `bsp_area_conserved` | BSP | Leaf areas sum to bounds area |
| `bsp_leaves_nonempty` | BSP | All leaves have positive area |
| `perlin_2d_bounded` | Perlin 2D | Output in [-1.5, 1.5] for any input |
| `perlin_3d_bounded` | Perlin 3D | Output in [-2.0, 2.0] for any input |
| `fbm_2d_bounded` | fBm | Output in [-2.0, 2.0] for 1-8 octaves |
| `perlin_2d_zero_at_integers` | Perlin 2D | Zero at integer lattice points |
| `wfc_collapse_reduces_entropy` | WFC | Entropy never increases on collapse |
| `engagement_composite_bounded` | Engagement | Composite always in [0.0, 1.0] |
| `fitts_mt_positive` | Fitts's law | Movement time always positive |
| `fitts_id_positive` | Fitts's law | Index of difficulty always positive |
| `hick_rt_monotone` | Hick's law | Reaction time monotonically increases with choices |
| `flow_state_always_defined` | Flow | Always returns a valid state (5 variants) |

**barraCuda action:** When absorbing Perlin/BSP, carry these proptest invariants upstream.
They provide regression coverage beyond the fixed-seed tests.

---

## Part 4: Code Quality Evolution

### Lint configuration (Cargo.toml only, no file-level overrides)

```toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
expect_used = "deny"
unwrap_used = "deny"
doc_markdown = "allow"
module_name_repetitions = "allow"
must_use_candidate = "allow"
return_self_not_must_use = "allow"
```

Previously allowed (`missing_errors_doc`, `missing_panics_doc`) are now enforced.
All `# Errors` sections have been added to public Result-returning functions.

### Structured tracing

Library IPC code now uses `tracing` spans instead of `eprintln!`:

```rust
info!(path = %self.socket_path.display(), "IPC listening");
warn!(error = %e, "IPC connection error");
info!(primal = PRIMAL_NAME, op = %req.method, latency_us, ok, "dispatch");
info!(domain = GAME_DOMAIN, capabilities = GAME_CAPABILITIES.len(), "registered domain");
```

**toadStool action:** The structured format enables automatic metrics scraping via
the biomeOS Pathway Learner. Consider adopting the same `tracing` pattern for
dispatch latency in toadStool's compute scheduler.

### Capability-based discovery (no hardcoded primal names)

`VisualizationPushClient` (formerly `PetalTonguePushClient`) now discovers any
primal advertising `visualization.render` via `ipc::discovery::discover_primals()`.
The name-based fallback has been removed. This is the correct pattern for all
cross-primal communication.

---

## Part 5: Tolerance Tightening

| Tolerance | V16 | V17 | Justification |
|-----------|-----|-----|---------------|
| `RAYCASTER_HIT_RATE_TOL` | 20.0 | 5.0 | exp030 GPU parity: 8×8 room, 64 rays, f32/f64 DDA |

All other 19 tolerances remain unchanged — they were already properly justified
with provenance citations in `tolerances/mod.rs`.

---

## Part 6: Baseline Provenance

### Drift checker

`baselines/python/check_drift.py` re-runs all 7 Python baselines and compares
against the stored `combined_baselines.json`. Reports drift at 1e-12 tolerance.
Current status: **zero drift** across all baselines.

### Python baselines (stdlib only, no numpy/scipy)

| Script | What it computes | Checks |
|--------|-----------------|--------|
| `perlin_noise.py` | Perlin 2D/3D, fBm | Lattice zeros, bounds, determinism |
| `interaction_laws.py` | Fitts, Hick, Steering | Known values |
| `flow_engagement.py` | Flow state, engagement composite | Analytical |
| `goms_model.py` | KLM task time | Known values |
| `lsystem_growth.py` | L-system string growth | Iteration counts |
| `bsp_partition.py` | BSP area conservation | Analytical |
| `fun_keys_model.py` | Four Keys classification | Category assignment |

---

## Action Items

### For barraCuda team

1. **Absorb Perlin noise** (P1): `perlin_2d`, `perlin_3d`, `fbm_2d` — most-consumed algorithm across springs
2. **Absorb WFC core** (P2): `WfcGrid`, `AdjacencyRules`, `collapse`, `propagate`
3. **Absorb BSP** (P2): `generate_bsp`, `Rect`, `BspNode`
4. **Carry proptest invariants** upstream with each absorption
5. **Consider L-systems** (P3): `apply_rules`, `string_to_turtle` — lower priority, less cross-spring demand

### For toadStool team

1. **Absorb 11 WGSL shaders** from `exp030/shaders/` — validated, standalone, documented tolerances
2. **Consider `Pipeline` builder** based on exp030's consolidated helper pattern
3. **Adopt structured `tracing`** for dispatch latency (enables Pathway Learner scraping)
4. **GPU dispatch targets** (unchanged from V16):
   - Tier A (direct rewire): noise, raycaster, engagement, fun_keys, flow, input_laws, GOMS
   - Tier B (adapt): WFC (barrier sync), BSP (stack elimination), L-systems (variable output)

### For coralReef team

1. **f32 arithmetic**, `log2()`, no dynamic memory, fixed-size buffers (all shaders comply)
2. **reduce_sum.wgsl** uses `var<workgroup>` — needs coralReef support for shared memory

---

## Files Changed (V16 → V17)

| File | Change |
|------|--------|
| `barracuda/src/ipc/provenance.rs` | doc_markdown, `# Errors`, `map_or_else`, `let...else`, `PRIMAL_NAME` |
| `barracuda/src/ipc/server.rs` | `tracing` replaces `eprintln!` |
| `barracuda/src/ipc/handlers.rs` | `tracing` structured metrics |
| `barracuda/src/biomeos/mod.rs` | `tracing`, removed redundant `#[allow]` |
| `barracuda/src/telemetry/mapper.rs` | `#[must_use]`, doc_markdown fix |
| `barracuda/src/visualization/push_client.rs` | Capability-based discovery (full rewrite) |
| `barracuda/src/visualization/mod.rs` | Re-export `VisualizationPushClient` |
| `barracuda/src/lib.rs` | Removed redundant lint attributes |
| `barracuda/src/tolerances/mod.rs` | `RAYCASTER_HIT_RATE_TOL` 20.0 → 5.0 |
| `barracuda/src/bin/*.rs` | Removed redundant lint attributes (4 files) |
| `barracuda/Cargo.toml` | Removed `missing_errors_doc`/`missing_panics_doc` allows, added `proptest` dev-dep |
| `barracuda/tests/proptest_invariants.rs` | NEW — 12 property tests |
| `baselines/python/check_drift.py` | NEW — baseline drift detector |
| `experiments/exp030_cpu_gpu_parity/src/` | Refactored: 1 file → 4 modules |
| `experiments/exp030_cpu_gpu_parity/shaders/` | NEW — 11 standalone WGSL files |
