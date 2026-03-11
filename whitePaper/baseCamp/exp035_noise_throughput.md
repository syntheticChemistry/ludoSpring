# Expedition 035: BM-002 Noise Field Throughput

**Date:** 2026-03-11
**Status:** Active
**Reference:** OPEN_SYSTEMS_BENCHMARK_SPECIFICATION.md BM-002, fastnoise-lite

## What We Built

Formal throughput benchmark for Perlin noise and fBm field generation,
validating against the open systems benchmark specification target:
"CPU within 2x of noise-rs/FastNoiseLite."

### Results

| Field | Size | Time | Throughput |
|-------|------|------|------------|
| Perlin 2D | 1024x1024 | ~80ms | 13.1M samples/s |
| Perlin 3D | 64x64x64 | ~34ms | 7.7M samples/s |
| fBm 2D (4 oct) | 512x512 | ~82ms | 3.2M samples/s |
| fBm 3D (4 oct) | 32x32x32 | ~18ms | 1.8M samples/s |

### fastnoise-lite Comparison (256x256)

| Implementation | Time | Ratio |
|----------------|------|-------|
| ludoSpring Perlin | ~4.6ms | **0.93x** (faster) |
| fastnoise-lite | ~5.0ms | 1.0x (baseline) |

We beat the spec target: not only within 2x, but actually faster than
fastnoise-lite at equivalent field sizes.

### Reproducibility

```bash
cargo run --bin exp035_noise_throughput              # validate (10 checks)
cargo run --bin exp035_noise_throughput -- bench      # detailed sweep
```
