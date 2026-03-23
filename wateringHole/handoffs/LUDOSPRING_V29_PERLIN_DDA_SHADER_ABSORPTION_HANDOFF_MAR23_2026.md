# ludoSpring v0.29 — Perlin 2D & DDA Raycast Shader Absorption Handoff

**Date**: March 23, 2026
**From**: ludoSpring
**To**: barraCuda, toadStool, coralReef
**Status**: Ready for absorption
**Previous**: [V28 toadStool/barraCuda Deep Evolution](LUDOSPRING_V28_TOADSTOOL_BARRACUDA_DEEP_EVOLUTION_HANDOFF_MAR18_2026.md)

---

## Summary

Two WGSL compute shaders — Perlin 2D noise and DDA grid raycasting — have
been validated against barraCuda CPU implementations (exp030 CPU/GPU parity)
and are ready for upstream absorption into barraCuda's shader catalog. Both
shaders are currently embedded in ludoSpring via `include_str!` and dispatched
through the game engine's `GpuOp` enum. After absorption, ludoSpring will
lean on the upstream barraCuda ops and remove its local copies.

---

## Shaders for Absorption

### 1. Perlin 2D Noise (`perlin_2d.wgsl`)

- **Location**: `barracuda/shaders/game/validated/perlin_2d.wgsl`
- **Duplicate**: `experiments/exp030_cpu_gpu_parity/shaders/perlin_2d.wgsl`
- **CPU parity target**: `barracuda::procedural::noise::perlin_2d`
- **Algorithm**: Ken Perlin (2002) improved Perlin noise, 2D
- **Precision**: f32 GPU vs f64 CPU, measured tolerance < 1e-3
- **Workgroup size**: 64
- **Bindings**: permutation table (storage read), coordinates (storage read), output (storage rw)
- **Validation**: exp030 runs identical inputs on CPU (`perlin_2d`) and GPU, asserts
  max absolute difference < `tolerances::gpu::PERLIN_2D_F32_TOL` (1e-3)

**Suggested barraCuda module**: `barracuda::shaders::procedural::perlin_2d` or
absorb into existing `procedural::noise` as a GPU dispatch path.

### 2. DDA Grid Raycast (`dda_raycast.wgsl`)

- **Location**: `barracuda/shaders/game/validated/dda_raycast.wgsl`
- **Duplicate**: `experiments/exp030_cpu_gpu_parity/shaders/dda_raycast.wgsl`
- **CPU parity target**: `barracuda::game::raycaster::cast_ray`
- **Algorithm**: DDA (Digital Differential Analyzer) 2D grid traversal
- **Precision**: f32 GPU vs f64 CPU, measured tolerance < 0.5 (iterative DDA accumulation)
- **Workgroup size**: 64
- **Bindings**: map data (storage read), params [player pos, map dims, max depth, n_rays]
  (storage read), ray angles (storage read), distances (storage rw)
- **Validation**: exp030 CPU/GPU parity, max distance deviation < `tolerances::gpu::DDA_F32_TOL` (0.5)

**Suggested barraCuda module**: `barracuda::shaders::game::dda_raycast` or a new
`spatial` shader family.

---

## Additional Validated Shaders (Lower Priority)

These exp030 shaders have also been validated but map to existing barraCuda ops
and should use the upstream `TensorSession` ops instead of custom WGSL:

| Shader | barraCuda Op | Status |
|--------|-------------|--------|
| `sigmoid.wgsl` | `TensorSession::sigmoid` / `activations::sigmoid` | Tier A: direct rewire |
| `relu.wgsl` | `TensorSession::relu` / `activations::relu` | Tier A: direct rewire |
| `softmax.wgsl` | `TensorSession::softmax` | Tier A: direct rewire |
| `dot_product.wgsl` | `TensorSession::mul` + reduce | Tier A: direct rewire |
| `reduce_sum.wgsl` | (host reduce / future `TensorSession::reduce`) | Tier B: adapt |
| `scale.wgsl` | `TensorSession::fma` | Tier A: direct rewire |
| `lcg.wgsl` | `rng::lcg_step` (CPU) | Tier B: GPU RNG path |
| `abs.wgsl` | (no barraCuda GPU abs yet) | Tier C: new op |
| `engagement_batch.wgsl` | Weighted dot → `TensorSession::fma` chain | Tier B: fused pipeline |

---

## Game Engine Shaders (Not for Absorption)

These are domain-specific game engine shaders that belong in ludoSpring, not
barraCuda. They use game-specific bindings (terrain grids, entity state):

| Shader | Purpose |
|--------|---------|
| `fog_of_war.wgsl` | Per-tile visibility from viewer position |
| `tile_lighting.wgsl` | Point light propagation on 2D tile grid |
| `pathfind_wavefront.wgsl` | BFS wavefront expansion on grid |

---

## Absorption Protocol

Following the Write → Validate → Handoff → Absorb → Lean cycle:

1. **Write** — Complete (shaders written and embedded in ludoSpring)
2. **Validate** — Complete (exp030 CPU/GPU parity tests pass)
3. **Handoff** — This document
4. **Absorb** — barraCuda adds shaders to its catalog with:
   - `ComputeDispatch` integration (shader → pipeline → dispatch)
   - Feature-gated under `gpu`
   - Unit tests with the same tolerance bounds
   - Documentation linking back to ludoSpring exp030 provenance
5. **Lean** — ludoSpring removes local `validated/` copies and imports from
   `barracuda::shaders::*`, updating `GpuOp::wgsl_source()` to delegate

---

## Feature-Gating Fix (Completed)

As part of this handoff preparation, the upstream barraCuda feature-gating bug
that prevented `default-features = false` compilation has been fixed:

- `special::plasma_dispersion` — module and re-exports gated behind `#[cfg(feature = "gpu")]`
- `spectral::stats::analyze_weight_matrix` — function and types gated behind `#[cfg(feature = "gpu")]`

ludoSpring now compiles with `barracuda = { ..., default-features = false }`,
pulling only CPU math by default and enabling GPU via `features = ["gpu"]`.

---

## Current State

| Metric | Value |
|--------|-------|
| ludoSpring tests | 402 passing (barracuda) + 19 (forge) |
| Python parity tests | 42 (all pass) |
| Baseline SHA-256 | `8c404eab...e181fc` |
| Clippy | 0 warnings (pedantic + nursery) |
| Format | Clean |
| Unsafe | 0 (`forbid(unsafe_code)`) |
| barraCuda default-features | false (CPU-only default) |

---

## For barraCuda

- The two validated shaders (`perlin_2d.wgsl`, `dda_raycast.wgsl`) are
  self-contained WGSL with no ludoSpring-specific dependencies
- Tolerance constants are documented and justified in the shader headers
- Both shaders use workgroup size 64 — barraCuda may want to make this
  configurable via `optimal_workgroup_size()` at dispatch time
- The permutation table for Perlin is the standard 512-element doubled
  permutation — same as `procedural::noise::PERM`

## For toadStool

- These shaders are already registered as `GpuOp` shader names:
  `perlin_2d` and `dda_raycast`
- After barraCuda absorption, toadStool's `compute.submit` can reference
  the upstream shader catalog instead of receiving inline WGSL

## For coralReef

- Both shaders are candidates for native compilation via
  `shader.compile.wgsl` once coralReef supports compute shader AOT
- No special intrinsics needed — standard WGSL compute only

---

## What's Next

- barraCuda absorbs Perlin 2D and DDA shaders into upstream catalog
- ludoSpring wires `TensorSession` for engagement batch pipeline (Tier A)
- ludoSpring removes local validated shader copies after upstream absorption
- exp030 evolves to test against upstream barraCuda shader dispatch
