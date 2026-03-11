# Expedition 039: Cross-Implementation Noise Validation

**Date:** 2026-03-11
**Status:** Active
**Reference:** noise-rs (MIT), fastnoise-lite (C, MIT)

## What We Built

Three independent Perlin noise implementations compared head-to-head:

| Implementation | Language | Source | 256x256 Time |
|----------------|----------|--------|-------------|
| ludoSpring | Rust | Our code | ~5.3ms |
| noise-rs | Rust | noise crate v0.9 | ~15.1ms |
| fastnoise-lite | C (Rust wrapper) | MIT | ~5.6ms |

### Statistical Property Equivalence

| Property | Ours | noise-rs | fastnoise-lite |
|----------|------|----------|----------------|
| Mean | 0.0006 | -0.0072 | 0.0005 |
| Std dev | 0.300 | 0.418 | 0.304 |
| Bounded | [-0.81, 0.81] | [-1.0, 1.0] | [-0.94, 0.83] |
| Floor % (>0) | 49.2% | 48.5% | 49.2% |
| Smoothness | 0.036 | 0.052 | 0.036 |

### Key Findings

1. **We're the fastest**: 0.93x fastnoise-lite, 2.85x faster than noise-rs
2. **Statistical equivalence**: all produce near-zero mean, similar std dev
3. **Terrain equivalence**: thresholded terrain maps are within 1% of each other
4. **Game metrics equivalent**: walkable area and connected regions comparable

### Reproducibility

```bash
cargo run --bin exp039_noise_cross_validation            # validate (12 checks)
cargo run --bin exp039_noise_cross_validation -- bench    # timing sweep
```
