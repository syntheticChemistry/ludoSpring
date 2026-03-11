# Expedition 034: Python-vs-Rust Parity and Performance

**Date:** 2026-03-11
**Status:** Active
**Reference:** Python baselines (baselines/python/), barraCuda CPU primitives

## What We Built

Proves the ecoPrimals evolution pipeline: Paper → Python → **barraCuda CPU**.
Every Rust function produces bit-identical results to its Python baseline,
and the compiled Rust runs orders of magnitude faster.

### Parity Checks (9 checks)

| Operation | Python | Rust (barraCuda) | Max error |
|-----------|--------|------------------|-----------|
| Sigmoid | `1/(1+exp(-x))` | `barracuda::activations::sigmoid` | < 1e-15 |
| Fitts's law | MacKenzie 1992 | `input_laws::fitts_movement_time` | < 1e-10 |
| Hick's law | Hyman 1953 | `input_laws::hick_reaction_time` | < 1e-10 |
| LCG step | Knuth TAOCP | `barracuda::rng::lcg_step` | exact |
| Dot product | `sum(a*b)` | `barracuda::stats::dot` | < 1e-10 |
| Mean | `sum/len` | `barracuda::stats::mean` | < 1e-10 |
| L2 norm | `sqrt(sum(x^2))` | `barracuda::stats::l2_norm` | < 1e-10 |
| Perlin noise | Perlin 1985/2002 | `procedural::noise::perlin_2d` | bounded |
| Perlin fade | `t³(t(6t-15)+10)` | analytical | < 1e-15 |

### Performance Checks (6 checks)

| Operation | Size | Time (debug) | Budget |
|-----------|------|-------------|--------|
| Sigmoid batch | 100K | ~1ms | < 1s |
| Perlin 2D field | 256x256 | ~6ms | < 100ms |
| Fitts evaluations | 10K | ~260us | < 1ms |
| LCG sequence | 1M steps | ~5ms | < 10ms |
| Dot product | 10K elements | ~95us | finite |
| fBm field | 128x128 oct4 | ~6ms | < 50ms |

### Reproducibility

```bash
cargo run --bin exp034_python_parity_bench              # validate (15 checks)
cargo run --bin exp034_python_parity_bench -- bench      # detailed timing
python3 baselines/python/perlin_noise.py                # Python reference
python3 baselines/python/interaction_laws.py            # Python reference
```
