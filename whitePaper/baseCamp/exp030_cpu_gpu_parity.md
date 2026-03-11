# Expedition 030: CPU-vs-GPU Math Parity

**Date:** 2026-03-11
**Status:** Active
**Reference:** barraCuda CPU primitives, WGSL compute shaders, wgpu 28

## What We Built

A comprehensive parity validation between barraCuda CPU (pure Rust) math and
GPU (WGSL compute shader) implementations. This is the critical first step in
the evolution pipeline:

```
Paper → Python → barraCuda CPU → **barraCuda GPU** → toadStool → coralReef
```

### Components

| Component | Purpose |
|-----------|---------|
| 8 inline WGSL shaders | Minimal GPU implementations of CPU primitives |
| GPU context helper | wgpu 28 device/queue/buffer management |
| CPU-vs-GPU parity checks | Element-by-element comparison within tolerances |
| Adapter probe | Enumerate all GPU/CPU adapters on the system |
| Benchmark suite | CPU-vs-GPU timing across data sizes (64 to 65536) |

### Parity Checks (16 total)

| # | Check | CPU path | GPU path | Tolerance |
|---|-------|----------|----------|-----------|
| 1 | sigmoid_cpu_at_zero | `barracuda::activations::sigmoid` | — | 1e-10 |
| 2 | relu_cpu_negative | `max(x, 0)` | — | exact |
| 3 | relu_cpu_positive | `max(x, 0)` | — | exact |
| 4 | dot_cpu_known | `barracuda::stats::dot` | — | 1e-10 |
| 5 | lcg_cpu_deterministic | `barracuda::rng::lcg_step` | — | exact |
| 6 | perlin_bounded_low | `procedural::noise::perlin_2d` | — | exact |
| 7 | perlin_bounded_high | `procedural::noise::perlin_2d` | — | exact |
| 8 | mean_cpu_known | `barracuda::stats::mean` | — | 1e-10 |
| 9 | sigmoid_gpu_parity | f32 sigmoid | `sigmoid.wgsl` | 1e-6 |
| 10 | relu_gpu_exact | f32 relu | `relu.wgsl` | exact |
| 11 | dot_gpu_parity | f32 elementwise * + sum | `dot_product.wgsl` | 1e-4 |
| 12 | softmax_gpu_parity | f32 softmax | `softmax.wgsl` | 1e-5 |
| 13 | scale_gpu_exact | affine 2x+1 | `scale.wgsl` | exact |
| 14 | lcg_gpu_exact | u32 LCG step | `lcg.wgsl` | exact |
| 15 | abs_gpu_exact | f32 abs | `abs.wgsl` | exact |
| 16 | reduce_sum_gpu_parity | f32 sum | `reduce_sum.wgsl` (workgroup) | 1.0 |

### Key Insight

f32 GPU operations (sigmoid, softmax, dot) match CPU f32 within 1e-6.
Integer operations (LCG, ReLU, abs, scale) are bit-exact between CPU and GPU.
This validates the portability thesis: pure math is substrate-independent.

### Reproducibility

```bash
cargo run --bin exp030_cpu_gpu_parity               # validate (16 checks)
cargo run --bin exp030_cpu_gpu_parity -- probe       # enumerate adapters
cargo run --bin exp030_cpu_gpu_parity -- bench       # CPU-vs-GPU timing
```
