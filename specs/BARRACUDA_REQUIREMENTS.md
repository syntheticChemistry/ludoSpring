# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring — barraCuda Requirements

Follows the sibling-spring pattern (`wetSpring`, `hotSpring`, `primalSpring`).
Documents which barraCuda modules ludoSpring depends on, which it intends to
adopt, and what upstream evolution is needed.

## Current dependency

```toml
barracuda = { path = "../../../primals/barraCuda/crates/barracuda", default-features = false }
```

- **Path dependency** with `default-features = false` (lean CPU build).
- `gpu` feature enables `barracuda/gpu` + optional `wgpu` / `tokio`.

## Modules consumed (CPU — `barcuda_math` re-export)

| barraCuda module | ludoSpring usage |
|------------------|------------------|
| `activations` | `sigmoid` (flow.rs), `relu`, `gelu`, `softmax`, batch variants, `mish`, `swish`, `leaky_relu`, `softplus` |
| `rng` | `lcg_step`, `state_to_f64`, `uniform_f64_sequence` (bsp.rs) |
| `stats` | `dot` (engagement.rs), `l2_norm`, `mae`, `mean`, `percentile`, `rmse`, `covariance`, `pearson_correlation`, `std_dev`, `variance` |

## Modules not currently used

| barraCuda module | Reason |
|------------------|--------|
| `linalg` | No matrix/linear algebra needed in current game-science domain |
| `spectral` | No FFT/spectral analysis in game metrics |
| `nautilus` | No numerical integration needed |
| `nn` | No neural network layers in game engine |
| `special` | No special functions (erf, gamma) needed |

## Intentional CPU-parallel implementations

| ludoSpring module | barraCuda equivalent | Justification |
|-------------------|---------------------|---------------|
| `procedural::noise` (Perlin, fBm) | `ops::procedural::perlin_noise` | CPU implementation for Python parity tests and IPC `game.generate_noise`. GPU path uses WGSL shaders validated in exp030. Duplication is evolutionary, not accidental — upstream absorption pending. |
| exp030 `cpu_softmax_f32` | `ops::softmax` (f64 tensor) | Intentional f32 reference to match WGSL shader precision for GPU parity validation. Different precision domain. |

## GPU modules (behind `feature = "gpu"`)

| barraCuda GPU module | ludoSpring status |
|---------------------|-------------------|
| `TensorSession` | Exposed via `GpuContext::tensor_session()`, not yet wired to product code. Future target for fused unary/softmax/dot/reduce pipelines. |
| `dispatch` | Not used directly; GPU dispatch goes through toadStool IPC or exp030's direct wgpu. |
| `ops` | Not used directly; candidate for Tier A shader absorption (sigmoid, relu, abs, scale, softmax). |

## Upstream evolution requests

1. **Perlin 2D absorption**: `barracuda/shaders/game/validated/perlin_2d.wgsl` → barraCuda `ops::procedural::perlin_noise`. Perm table layout and coordinate conventions need alignment.
2. **DDA raycaster**: `barracuda/shaders/game/validated/dda_raycast.wgsl` — game-domain-specific; may stay ludoSpring-local or become a named barraCuda game op.
3. **f32 tensor path**: `TensorSession` operates at f64; ludoSpring GPU shaders use f32. Need explicit f32 kernel support or documented precision bridge for engagement/softmax parity.
4. **LCG GPU alignment**: exp030 `lcg.wgsl` uses 32-bit LCG constants; barraCuda `rng::lcg_step` uses 64-bit. Need documented 32-bit variant or explicit downcast contract.

## Shader promotion tiers

| Tier | Shaders | Direction |
|------|---------|-----------|
| **A** (rewire to existing op) | sigmoid, relu, abs, scale, softmax | `TensorSession` unary/fused ops |
| **B** (adapt existing) | perlin_2d, dot_product, reduce_sum, engagement_batch, lcg | Bind layout + precision alignment |
| **C** (game-domain, stay local) | fog_of_war, tile_lighting, pathfind_wavefront, dda_raycast | ludoSpring-specific semantics |

## Version tracking

- **Current pin**: path dependency (tracks monorepo HEAD)
- **Sibling pattern**: wetSpring uses path dep; hotSpring uses git+rev pin
- **Recommendation**: Add `rev` pin if reproducible CI builds are needed without monorepo checkout
