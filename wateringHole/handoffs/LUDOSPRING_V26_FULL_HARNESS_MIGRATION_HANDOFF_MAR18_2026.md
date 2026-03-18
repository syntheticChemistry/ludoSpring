# ludoSpring V26 — Full Harness Migration + Deep Debt Completion Handoff

**Date:** March 18, 2026
**From:** ludoSpring (V26)
**To:** All primals, all springs, biomeOS orchestration
**Supersedes:** V24 (Leverage Guide + Cross-Ecosystem Absorption) + V25 (Deep Debt Sprint)
**License:** AGPL-3.0-or-later
**Covers:** V25–V26

---

## Executive Summary

- **71 of 75 experiments** now use `ValidationHarness` + `BaselineProvenance` (4 non-validation crates exempted: exp024 playable Doom, exp025 playable roguelike, exp030 GPU parity crate, exp058 conjugant demo)
- **Zero** legacy `ValidationResult` usage remains anywhere in the codebase
- **14 GPU tolerance constants** centralized in `tolerances::gpu` — exp030 fully migrated from inline magic numbers
- **Workspace lints tightened**: `missing_errors_doc` and `missing_panics_doc` upgraded from `allow` → `warn`; per-crate overrides removed from 11 experiment Cargo.tomls
- **Shader dedup audit** complete — 2 already deduplicated, 7 upstream absorption candidates documented, 2 domain-specific kept
- **Zero clippy warnings**, zero compilation errors, zero formatting issues across full workspace

---

## Part 1: ValidationHarness Migration (56 experiments)

All validation experiments migrated from the legacy pattern:

```rust
// BEFORE (legacy)
let mut results = Vec::new();
let r = ValidationResult::check("exp", "desc", measured, expected, tol);
report(&r);
results.push(r);
// ... manual pass/fail/exit(1)

// AFTER (modern)
let mut h = ValidationHarness::new("expXXX_name");
h.print_provenance(&[&PROVENANCE]);
h.check_abs("desc", measured, expected, tol);
h.finish(); // deterministic exit 0/1
```

### Migration waves

| Wave | Experiments | Agent |
|------|-------------|-------|
| V25 | exp002–exp010 (9) | Initial template |
| V26-A | exp011–exp020 (10) | Batch migration |
| V26-B | exp021–exp029 (7, skip exp024/025/030) | Adapter + telemetry patterns |
| V26-C | exp031–exp045 (15) | Dispatch, pipeline, IPC patterns |
| V26-D | exp046–exp057, exp059 (13, skip exp058) | Provenance, lysogeny, composable patterns |
| V26-E | exp060–exp066 (7) | Cross-domain, fraud, attribution |

### What consuming teams get

- **Uniform exit codes**: every validation binary exits 0 (pass), 1 (fail), or 2 (skipped/no hardware)
- **Provenance tracking**: every experiment declares its `BaselineProvenance` (script, commit, date, command)
- **Pluggable sinks**: `ValidationHarness<S: ValidationSink>` — default `ConsoleSink`, test `BufferSink`, future `JsonSink`
- **Named tolerances**: all checks reference named constants from `tolerances::*` modules

---

## Part 2: GPU Tolerance Centralization

`barracuda/src/tolerances/gpu.rs` now contains 14 constants:

| Constant | Value | Domain |
|----------|-------|--------|
| `GPU_UNARY_ABS_TOL` | 1e-6 | Sigmoid, ReLU, abs — f32 vs f64 |
| `GPU_PERLIN_ABS_TOL` | 1e-3 | Perlin 2D noise |
| `GPU_FBM_ABS_TOL` | 2e-3 | Fractal Brownian motion |
| `GPU_FBM_ABS_TOL_LOOSE` | 0.01 | Higher-variance GPU fBm |
| `GPU_REDUCTION_ABS_TOL` | 1e-4 | Dot product, reduction |
| `GPU_SOFTMAX_ABS_TOL` | 1e-5 | Softmax |
| `GPU_ENGAGEMENT_REL_TOL` | 1e-4 | Engagement batch (relative) |
| `GPU_ENGAGEMENT_ABS_TOL` | 1e-4 | Engagement batch (absolute) |
| `GPU_RAYCASTER_HIT_RATE_PP` | 5.0 | Raycaster hit rate (percentage points) |
| `GPU_RAYCASTER_DISTANCE_ABS_TOL` | 0.5 | Raycaster distance |
| `GPU_REDUCE_SUM_ABS_TOL` | 1.0 | Parallel reduce-sum |
| `GPU_LCG_ABS_TOL` | 1e-10 | LCG PRNG |
| `ANALYTICAL_TOL` | 1e-10 | CPU analytical known-value checks |
| `NOISE_MEAN_TOL` | 0.05 | Perlin noise statistical mean |

---

## Part 3: Shader Dedup Audit

| Shader | Status | Location |
|--------|--------|----------|
| `perlin_2d.wgsl` | Deduplicated | `barracuda/shaders/game/validated/` |
| `dda_raycast.wgsl` | Deduplicated | `barracuda/shaders/game/validated/` |
| `sigmoid.wgsl` | Upstream candidate | exp030 (barraCuda has DF64 variant) |
| `relu.wgsl` | Upstream candidate | exp030 (barraCuda has f64 variant) |
| `abs.wgsl` | Upstream candidate | exp030 (barraCuda has f64 variant) |
| `softmax.wgsl` | Upstream candidate | exp030 (barraCuda has multi-pass f64) |
| `reduce_sum.wgsl` | Upstream candidate | exp030 (barraCuda has f32+f64) |
| `dot_product.wgsl` | Upstream candidate | exp030 (barraCuda has f64 gemm) |
| `lcg.wgsl` | Upstream candidate | exp030 (barraCuda has xorshift32) |
| `scale.wgsl` | Domain-specific | exp030 only (trivial parity test) |
| `engagement_batch.wgsl` | Domain-specific | exp030/ludoSpring (5-component engagement) |

**barraCuda action:** Add f32 variants of math primitives (sigmoid, relu, abs, softmax, reduce_sum, dot_product) for f32-vs-f64 parity testing. ludoSpring exp030 shaders are the reference implementations.

---

## Part 4: Lint Evolution

| Lint | Before | After |
|------|--------|-------|
| `missing_errors_doc` | `allow` | `warn` |
| `missing_panics_doc` | `allow` | `warn` |
| Per-crate overrides | 11 experiment Cargo.tomls | 0 (removed, inherit workspace) |
| Violations at time of tightening | 0 | 0 |

---

## Part 5: Quality Metrics

| Metric | Value |
|--------|-------|
| Experiments | 75 |
| Validation checks | 1692 |
| Tests | 450+ unit + 19 proptest + 6 IPC integration |
| Clippy warnings | 0 (workspace-wide, pedantic+nursery) |
| `unsafe` blocks | 0 (`#![forbid(unsafe_code)]`) |
| `#[allow()]` in prod | 0 |
| Legacy `ValidationResult` usage | 0 |
| Files > 1000 LOC | 0 |
| C dependencies | 1 transitive (`renderdoc-sys` via `wgpu-hal`, GPU feature only) |

---

## Action Items

**toadStool action:** No changes needed — all IPC contracts unchanged.

**barraCuda action:**
1. Add f32 shader variants for parity testing (sigmoid_f32, relu_f32, abs_f32, softmax_f32, reduce_sum_f32, dot_product_f32)
2. Consider absorbing `ValidationHarness` pattern into barraCuda validation module (already used by 3+ springs)
3. Consider absorbing `capability_domains` pattern (structured domain/method introspection)

**coralReef action:** No changes needed — shader sources unchanged.

**biomeOS action:** No changes needed — niche YAML, deploy graph, UniBin unchanged.

---

## License

AGPL-3.0-or-later
