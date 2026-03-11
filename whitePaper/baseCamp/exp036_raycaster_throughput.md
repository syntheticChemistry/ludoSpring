# Expedition 036: BM-003 Raycaster Throughput

**Date:** 2026-03-11
**Status:** Active
**Reference:** OPEN_SYSTEMS_BENCHMARK_SPECIFICATION.md BM-003, Lodev DDA

## What We Built

Formal throughput benchmark for the DDA raycaster, validating the spec
target: "320-column screen cast at 60 Hz. Match C reference within 1.5x."

### Results

| Configuration | Time | FPS | vs 60Hz |
|---------------|------|-----|---------|
| 320 cols, 64x64 | 164us | 6,100+ | 100x headroom |
| 640 cols, 64x64 | 305us | 3,300+ | 55x headroom |
| 320 cols, 128x128 | 271us | 3,700+ | 61x headroom |
| 320 cols, maze | 35us | 28,500+ | 475x headroom |
| 1000 frames avg | 151us | 6,623 | 110x headroom |

The raycaster runs at over 6,000 FPS on CPU alone — 110x the 60 Hz
requirement. This massive headroom means the raycaster node consumes
less than 1% of the frame budget, leaving room for physics, metrics,
rendering, and audio.

### Reproducibility

```bash
cargo run --bin exp036_raycaster_throughput              # validate (10 checks)
cargo run --bin exp036_raycaster_throughput -- bench      # detailed sweep
```
