# ludoSpring V26 — toadStool/barraCuda Absorption & Evolution Handoff

**Date:** March 18, 2026
**From:** ludoSpring (V26)
**To:** toadStool team, barraCuda team
**License:** AGPL-3.0-or-later
**Covers:** V25–V26 deep debt sprint + full harness migration

---

## Executive Summary

ludoSpring V26 completes its internal validation infrastructure evolution. This handoff documents:

1. **What ludoSpring produces** that barraCuda/toadStool should absorb
2. **What ludoSpring consumes** from barraCuda and how the dependency evolves
3. **GPU shader absorption candidates** with provenance and validation status
4. **Tolerance patterns** that are generic enough for upstream
5. **Learnings** from the V25–V26 sprint relevant to ecosystem evolution

---

## Part 1: What ludoSpring Produces for Upstream Absorption

### 1.1 ValidationHarness Pattern (P0 — already absorbed)

`ValidationHarness<S: ValidationSink>` is used by 3+ springs (ludoSpring, wetSpring, hotSpring). It lives in barraCuda `validation` module. ludoSpring V26 confirms the pattern works at scale: 71 experiments, 1692 checks, zero legacy usage remaining.

**V26 additions to the pattern:**
- `check_abs_or_rel(desc, measured, expected, abs_tol, rel_tol)` — compound check for GPU parity where values span orders of magnitude
- `exit_skipped(reason)` — exit code 2 for hardware-unavailable experiments (wetSpring V123 pattern, now validated at scale)
- `load_baseline_f64(json_path, key_path)` — runtime JSON baseline loader for Python provenance

**barraCuda action:** Absorb `check_abs_or_rel`, `exit_skipped`, and `load_baseline_f64` into the upstream `validation` module. These are generic — not ludoSpring-specific.

### 1.2 GPU Tolerance Constants (P1)

`barracuda/src/tolerances/gpu.rs` (14 constants) documents the empirical f32-vs-f64 delta bounds for all validated GPU operations. These tolerances are generic — they describe WGSL compute shader numerical behavior, not game science.

| Constant | Value | What It Bounds |
|----------|-------|----------------|
| `GPU_UNARY_ABS_TOL` | 1e-6 | Element-wise f32 ops (sigmoid, relu, abs) |
| `GPU_PERLIN_ABS_TOL` | 1e-3 | Perlin 2D noise (accumulated rounding) |
| `GPU_FBM_ABS_TOL` | 2e-3 | Fractal Brownian motion (octave accumulation) |
| `GPU_FBM_ABS_TOL_LOOSE` | 0.01 | fBm on higher-variance GPU hardware |
| `GPU_REDUCTION_ABS_TOL` | 1e-4 | Parallel reduction (dot product, sum) |
| `GPU_SOFTMAX_ABS_TOL` | 1e-5 | Numerically stable softmax |
| `GPU_ENGAGEMENT_REL_TOL` | 1e-4 | Weighted composite (relative) |
| `GPU_ENGAGEMENT_ABS_TOL` | 1e-4 | Weighted composite (absolute) |
| `GPU_RAYCASTER_HIT_RATE_PP` | 5.0 | DDA raycaster hit rate (percentage points) |
| `GPU_RAYCASTER_DISTANCE_ABS_TOL` | 0.5 | DDA raycaster wall distance |
| `GPU_REDUCE_SUM_ABS_TOL` | 1.0 | Parallel reduce-sum (large arrays, f32 catastrophic cancellation) |
| `GPU_LCG_ABS_TOL` | 1e-10 | LCG PRNG (integer arithmetic, should be exact) |

**barraCuda action:** Absorb these into `barracuda::tolerances::gpu` upstream. All constants are documented with their physical meaning and validation provenance (exp030_cpu_gpu_parity).

### 1.3 Validated WGSL Shaders (P1–P2)

Seven exp030 shaders are generic math primitives that barraCuda should absorb as f32 variants alongside existing f64/DF64 shaders:

| exp030 Shader | Lines | barraCuda Equivalent | Gap |
|---------------|-------|---------------------|-----|
| `sigmoid.wgsl` | 12 | `activation/sigmoid_df64.wgsl` | No f32 variant |
| `relu.wgsl` | 8 | `activation/relu_f64.wgsl` | No f32 variant |
| `abs.wgsl` | 8 | `math/abs_f64.wgsl` | No f32 variant |
| `softmax.wgsl` | 30 | `activation/softmax_f64.wgsl` | No single-workgroup f32 variant |
| `reduce_sum.wgsl` | 25 | `reduce/sum_reduce.wgsl` | Binding layout differs |
| `dot_product.wgsl` | 10 | `linalg/gemm_f64.wgsl` (elementwise) | No standalone f32 elementwise_mul |
| `lcg.wgsl` | 8 | inline in `pharma/vpc_simulate_f64.wgsl` | No standalone lcg_u32 shader |

**barraCuda action:** For each, either:
- Add the f32 variant to the shader library (use exp030 as reference implementation)
- Or document why f32 is not needed (if all downstream is f64/DF64)

Two domain-specific shaders stay in ludoSpring:
- `engagement_batch.wgsl` — 5-component weighted engagement (ludoSpring metrics)
- `scale.wgsl` — trivial `2x+1` parity test (not worth upstreaming)

Two game shaders already deduplicated into ludospring-barracuda:
- `perlin_2d.wgsl` → `barracuda/shaders/game/validated/perlin_2d.wgsl`
- `dda_raycast.wgsl` → `barracuda/shaders/game/validated/dda_raycast.wgsl`

### 1.4 Capability Domains Pattern (P1)

`capability_domains.rs` (~100 LOC) provides structured `Domain` / `Method` introspection — a typed capability catalog that maps to JSON-RPC method names. This pattern is reusable across all primals.

**barraCuda action:** Consider absorbing as a trait/macro for capability self-description. Every primal needs this; centralizing prevents divergence.

### 1.5 Procedural Algorithms (P2–P3)

| Module | LOC | What barraCuda Gets | GPU-Ready |
|--------|-----|---------------------|-----------|
| `procedural::noise` | ~200 | Perlin 2D/3D + fBm reference impl | Yes (exp030 validated) |
| `procedural::wfc` | ~265 | Wave Function Collapse | Yes (embarrassingly parallel) |
| `procedural::bsp` | ~220 | BSP spatial partitioning | Partially (tree structure is sequential) |
| `procedural::lsystem` | ~200 | L-system string rewriting | No (sequential by nature) |

barraCuda already has `ops::procedural::perlin_noise` (CPU+GPU) and `perlin_2d_f64.wgsl`. ludoSpring's implementation is the authoritative provenance reference. No duplication needed — just confirm parity.

---

## Part 2: What ludoSpring Consumes from barraCuda

### 2.1 Current Consumption

```rust
pub mod barcuda_math {
    pub use barracuda::activations::{sigmoid, sigmoid_batch};
    pub use barracuda::rng::{lcg_step, state_to_f64, uniform_f64_sequence};
    pub use barracuda::stats::{dot, l2_norm, mean};
}
```

This is a thin re-export layer. 75 experiments use it via `ludospring_barracuda::barcuda_math`.

### 2.2 Evolution Path

```
Current:  ludospring-barracuda → barracuda (path dep, CPU math only)
Near:     ludospring-barracuda → barracuda (CPU + GPU ops via TensorSession)
Future:   ludospring-barracuda → barracuda (CPU) + toadStool IPC (GPU dispatch)
Terminal: ludospring-barracuda → barracuda (CPU) + coralReef IPC (sovereign shaders)
```

**Key blocker:** ludoSpring does not use `TensorSession` yet. The 5 `GpuOp` variants (`FogOfWar`, `TileLighting`, `PathfindStep`, `PerlinTerrain`, `BatchRaycast`) have WGSL sources and dispatch params, but actual GPU execution goes through exp030's direct wgpu usage, not the `TensorSession` pipeline.

**toadStool action:** When `TensorSession` supports WGSL source injection (compile-and-dispatch), ludoSpring can wire its 5 GpuOps through the standard pipeline instead of raw wgpu.

### 2.3 GpuOp Catalog

| GpuOp | Shader | Workgroup | Dispatch |
|-------|--------|-----------|----------|
| `FogOfWar` | `fog_of_war.wgsl` | 64 | In-process / toadStool IPC |
| `TileLighting` | `tile_lighting.wgsl` | 64 | In-process / toadStool IPC |
| `PathfindStep` | `pathfind_wavefront.wgsl` | 64 | In-process / toadStool IPC |
| `PerlinTerrain` | `perlin_2d.wgsl` (validated) | 64 | In-process / toadStool IPC |
| `BatchRaycast` | `dda_raycast.wgsl` (validated) | 64 | In-process / toadStool IPC |

---

## Part 3: Learnings for Ecosystem Evolution

### 3.1 ValidationHarness at Scale

Migrating 71 experiments confirmed:
- The `check_abs` / `check_bool` / `check_lower` / `check_upper` API covers 100% of validation patterns encountered
- `check_abs_or_rel` is needed for GPU parity (values spanning orders of magnitude)
- `exit_skipped` (exit 2) is essential for CI on machines without GPU hardware
- `BaselineProvenance` (script, commit, date, command) catches 100% of "where did this expected value come from?" questions
- The `BufferSink` test pattern (capture output without printing) works cleanly for unit testing validation logic

### 3.2 Tolerance Centralization

Before V25: inline magic numbers in 71 experiments. After V26: all tolerances are named, centralized, and documented.

The pattern `tolerances::domain::CONSTANT_NAME` with `f64` type and doc comments explaining the physical meaning is the right abstraction level. Sub-modules by domain (gpu, validation, noise, interaction) scale better than a flat file.

### 3.3 GPU Parity Strategy

exp030 validated that:
- f32 GPU shaders match f64 CPU within documented tolerances for all tested operations
- Tolerances vary by 5 orders of magnitude (1e-10 for LCG integer math to 5.0 for raycaster hit rates)
- `check_abs_or_rel` is the right check type — some operations are better tested with relative tolerance, others absolute
- Parallel reduction (`reduce_sum`) has the largest tolerance (1.0) due to f32 catastrophic cancellation in large arrays

### 3.4 Primal Discovery

`FAMILY_ID`-based socket path derivation works cleanly:
```rust
fn family_id() -> String {
    std::env::var("FAMILY_ID").unwrap_or_else(|_| "default".into())
}
```
All primals derive their socket paths from this, enabling multi-instance deployment without hardcoding.

### 3.5 Benchmark Gaps

ludoSpring has no:
- Python-vs-Rust CPU timing benchmarks (baselines prove correctness, not performance)
- Industry-standard GPU benchmarks (Kokkos, CUDA, cuBLAS, Galaxy)
- Cross-spring throughput comparisons

These are documented in the README as future work. They are ecosystem-level concerns, not ludoSpring-specific.

---

## Part 4: Recommended Absorption Timeline

| Priority | Item | Owner | Effort |
|----------|------|-------|--------|
| P0 | `check_abs_or_rel` + `exit_skipped` + `load_baseline_f64` | barraCuda | Small (3 functions) |
| P1 | GPU tolerance constants | barraCuda | Small (copy `tolerances/gpu.rs`) |
| P1 | f32 shader variants (sigmoid, relu, abs, softmax, reduce_sum, dot_product, lcg) | barraCuda | Medium (7 shaders) |
| P1 | `capability_domains` pattern | barraCuda or biomeOS | Small (trait + derive macro) |
| P2 | `TensorSession` WGSL source injection | toadStool | Medium (compile-and-dispatch API) |
| P2 | Perlin/fBm parity confirmation (ludoSpring vs barraCuda `ops::procedural`) | barraCuda | Small (diff + test) |
| P3 | WFC GPU parallelization | barraCuda | Large (new compute graph) |

---

## License

AGPL-3.0-or-later
