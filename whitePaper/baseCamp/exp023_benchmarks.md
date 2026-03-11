# Expedition 023: Open-Systems Benchmark — ludoSpring vs Rust Ecosystem

**Date:** March 11, 2026
**Status:** Complete — 16/16 checks pass
**Pattern:** OPEN_SYSTEMS_BENCHMARK_SPECIFICATION (study → scaffold → evolve → shed)

---

## What We Compared

| Benchmark | ludoSpring | Ecosystem crate | Result |
|-----------|-----------|-----------------|--------|
| BM-Noise | `procedural::noise::perlin_2d` | `fastnoise-lite` v1.1.1 (MIT) | 1.08x speed ratio, both correct |
| BM-WFC | `procedural::wfc` | `wave-function-collapse` v0.3.0 (MIT) | API comparison only |
| BM-ECS | `game::state` | Bevy `bevy_ecs` (MIT/Apache-2.0) | Pattern study, no dependency |

## BM-Noise: Perlin/fBm

**Both correct.** ludoSpring and fastnoise-lite both satisfy the fundamental Perlin
properties: lattice zeros, bounded output, spatial coherence. Performance is
within 8% (ludoSpring f64 vs fastnoise-lite f32 default).

**What we learn:**
- fastnoise-lite provides domain warping and cellular (Voronoi) noise that
  ludoSpring doesn't have — absorption candidates for future expeditions
- fastnoise-lite supports `no_std` — useful if ludoSpring noise moves to embedded
- ludoSpring advantage: f64 precision, deterministic seeding, GPU shader sketches

**Recommendation:** Keep ludoSpring's Perlin/fBm as the validated reference.
Study fastnoise-lite's domain warping for future absorption into barraCuda.

## BM-WFC: Wave Function Collapse

**API comparison** (no performance benchmark — different interfaces):
- ludoSpring WFC: manual collapse + propagate loop, `BTreeSet` options, no `rand`
- wave-function-collapse crate: sequential/random/entropic search, image-based WFC

**What we learn:**
- Entropic search (pick lowest-entropy cell, collapse, propagate) is a standard
  pattern that ludoSpring should implement as a convenience method
- Image-based WFC (learn adjacency rules from example images) is a powerful
  feature for level design that ludoSpring could scaffold from

**Recommendation:** Study the entropic search pattern. Keep deterministic LCG core.
Consider adding `WfcGrid::step_entropic(seed)` convenience method.

## BM-ECS: Bevy Patterns

**Pattern study only** — Bevy is not a dependency.

| Bevy Concept | ludoSpring Equivalent | Gap |
|-------------|----------------------|-----|
| Component | struct fields | No composition |
| Entity | (none) | No entity system |
| System | `fn update()` | No scheduling |
| Resource | `GridMap`, `TickBudget` | Manual globals |
| Query | (none) | Manual iteration |
| Event | `InputRecord` | Only replay |
| Plugin | (none) | Monolithic |

**What we learn:**
- Entity-Component composition is essential for game objects beyond prototypes
- System scheduling gives deterministic update order without manual orchestration
- ludoSpring's strengths (validated math, science annotations, replay, no unsafe)
  are orthogonal to ECS — they complement, not compete

**Recommendation:** Study Bevy ECS for entity management patterns. Don't depend on
Bevy. ludoSpring stays focused on validation + game logic. Rendering stays in
petalTongue. Physics stays in barraCuda.

## Cross-Spring Impact

- **barraCuda:** fastnoise-lite's domain warping → absorption candidate for
  `procedural::domain_warp_f64` compute shader
- **toadStool:** Noise performance comparison validates that CPU-side Perlin
  is fast enough for development; GPU dispatch adds value only at scale (>1M samples)
- **petalTongue:** Benchmark output tables → push to petalTongue as Bar channel

## Reproducibility

```bash
cargo run --bin exp023_open_systems_benchmark
```
