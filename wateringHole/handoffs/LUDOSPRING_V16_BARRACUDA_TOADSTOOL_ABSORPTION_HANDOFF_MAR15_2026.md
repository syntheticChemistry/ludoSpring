<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# ludoSpring V16 — barraCuda + toadStool Absorption Handoff

**Date**: March 15, 2026
**From**: ludoSpring (game science spring)
**To**: barraCuda team (math primitives), toadStool team (GPU dispatch)
**Status**: Complete — ludoSpring V16, 66 experiments, 1371 checks, 240 tests, niche-deployable

---

## Executive Summary

ludoSpring V16 has validated 13 foundational HCI/game science models through the
full Python → Rust CPU → GPU (WGSL) evolution pipeline, and evolved into a
deployable biomeOS niche citizen. This handoff details:

1. **What barraCuda should absorb** from ludoSpring's validated math
2. **What toadStool should dispatch** via GPU compute pipelines
3. **What we learned** about GPU parity, mixed hardware, and NUCLEUS coordination
4. **What coralReef needs** for shader compilation

---

## Part 1: barraCuda Absorption Candidates

### Currently consumed primitives

ludoSpring already uses barraCuda `v0.3.5`:

| Primitive | ludoSpring consumer | Purpose |
|-----------|-------------------|---------|
| `activations::sigmoid` | `interaction::flow::DifficultyCurve` | Difficulty-to-skill mapping |
| `activations::sigmoid_batch` | (ready for batch flow) | Batch difficulty curves |
| `stats::dot` | `metrics::engagement::compute_engagement` | Weighted engagement composite |
| `stats::mean` | (available) | Statistical aggregation |
| `stats::l2_norm` | (available) | Distance metrics |
| `rng::lcg_step` | `procedural::bsp::generate_bsp` | Deterministic BSP subdivision |
| `rng::state_to_f64` | `procedural::bsp::generate_bsp` | Float from LCG state |
| `rng::uniform_f64_sequence` | (available) | Stochastic generation seeds |

### Priority 1: Absorb Perlin noise into barraCuda core

**Source**: `ludospring_barracuda::procedural::noise` (~200 LOC)
**Why this matters**: Perlin noise is the most-consumed algorithm across springs.
wetSpring, hotSpring, and ludoSpring all independently implement Perlin. Centralizing
in barraCuda eliminates duplication and enables a single GPU shader path.

**What barraCuda gets**:
- `perlin_2d(x, y) -> f64` — gradient-based, Ken Perlin (2002) improved version
- `perlin_3d(x, y, z) -> f64` — 3D extension
- `fbm_2d(x, y, octaves, lacunarity, persistence) -> f64` — fractal Brownian motion
- `fbm_3d(x, y, z, octaves, lacunarity, persistence) -> f64` — 3D fBm
- 256-byte permutation table (Perlin's original)
- **All validated**: exp002, exp009, exp014 (Python parity), exp030 (GPU parity)
- **GPU-ready**: WGSL shader validated in exp030 (24/24 checks pass)

**barraCuda action**: Create `barracuda::noise` module. The permutation table and
gradient computation are pure math — no external dependencies. Absorb the WGSL
shader from exp030 as a reference implementation for the GPU path.

### Priority 2: Wave Function Collapse

**Source**: `ludospring_barracuda::procedural::wfc` (~265 LOC)
**What barraCuda gets**:
- `WfcGrid` — constraint propagation grid
- `AdjacencyRules` — tile adjacency specification
- `propagate()` — iterative constraint removal
- **Tier B GPU**: Needs barrier sync for parallel propagation

**barraCuda action**: Absorb as `barracuda::wfc`. The propagation loop can be
parallelized per-cell for GPU, with barrier sync between propagation rounds.
toadStool needs workgroup barriers.

### Priority 3: BSP Trees

**Source**: `ludospring_barracuda::procedural::bsp` (~220 LOC)
**What barraCuda gets**:
- `BspNode`, `generate_bsp()` — binary space partitioning
- Uses `rng::lcg_step` for deterministic splits (already a barraCuda primitive)
- **Tier B GPU**: Needs recursive → iterative conversion

### Priority 4: L-systems

**Source**: `ludospring_barracuda::procedural::lsystem` (~200 LOC)
**What barraCuda gets**:
- `LSystem`, `apply_rules()` — Lindenmayer string rewriting
- **Tier B GPU**: Variable-length output complicates dispatch

### Priority 5: Domain-agnostic algorithms (from experiments)

| Source | Lines | What | Priority |
|--------|-------|------|----------|
| `GenericFraudDetector` (exp065) | ~300 | Graph-based fraud detection (>80% structural similarity across gaming/science/medical) | P3 |
| `compute_distribution` (exp066) | ~200 | Weighted-sum attribution with configurable decay models | P3 |

---

## Part 2: toadStool GPU Dispatch Targets

### Tier A — Ready now (pure math, embarrassingly parallel)

| Module | WGSL Status | Validated | Dispatch pattern |
|--------|------------|-----------|-----------------|
| `procedural::noise` | **Shader exists** (exp030) | 24/24 GPU parity | `compute.submit` with coords buffer → noise buffer |
| `game::raycaster` | **Shader exists** (exp030) | 24/24 GPU parity | Per-column DDA, 1 thread per screen column |
| `metrics::engagement` | **Shader exists** (exp030) | 24/24 GPU parity | Batch dot product over snapshots |
| `metrics::fun_keys` | Not yet | — | Weighted sum classification, trivial shader |
| `interaction::flow` | Not yet | — | Batch comparisons (challenge vs skill) |
| `interaction::input_laws` | Not yet | — | `log2()` only transcendental, batch Fitts/Hick/Steering |
| `interaction::goms` | Not yet | — | Sum of operation times |

**toadStool action**: The three WGSL shaders in exp030 (`PERLIN_2D_WGSL`,
`ENGAGEMENT_BATCH_WGSL`, `DDA_RAYCAST_WGSL`) are validated reference implementations.
Absorb into `toadstool-runtime-gpu` shader library. They use `f32` on GPU with
documented `1e-3` tolerance vs `f64` CPU values.

### Tier B — Adapt (needs barrier sync or restructuring)

| Module | Barrier need | toadStool requirement |
|--------|-------------|----------------------|
| `procedural::wfc` | Per-propagation-round sync | Workgroup barriers in WGSL |
| `procedural::bsp` | Stack elimination | Iterative rewrite or BVH pattern |
| `procedural::lsystem` | Variable-length output | Prefix-sum output allocation |

### GPU f64 parity finding

**Critical learning**: GPU shaders use `f32`. CPU implementations use `f64`.
The measured maximum error is:
- Perlin noise: `< 1e-3`
- Engagement batch: `< 1e-4`
- Raycaster distance: `< 1e-2` (expected for iterative DDA)

These tolerances are documented and justified. coralReef's f64 emulation path
(df64 double-float) could reduce these if higher precision is needed.

---

## Part 3: Mixed Hardware + NUCLEUS Learnings

### metalForge capability-based routing (exp032)

ludoSpring V15 built a capability-based routing system in `metalForge/forge/`:

```rust
pub fn route(workload: &GameWorkloadProfile, substrates: &[SubstrateInfo]) -> Option<Decision>
```

**Key findings**:
- GPU > NPU > CPU priority chain works for game workloads
- `SubstrateKind`, `Capability`, `SubstrateInfo` — portable hardware abstraction
- `TransferPath::Direct` (PCIe peer-to-peer NPU→GPU) saves ~50% vs `TransferPath::ViaCpu`
- `fallback_chain()` provides graceful degradation

**toadStool action**: The metalForge routing model is a reference implementation
for toadStool's own substrate routing. Absorb the `Capability` enum and
`SubstrateInfo` struct as a common vocabulary.

### NUCLEUS atomics (exp033)

- `TowerAtomic` — security (BearDog) + discovery (Songbird)
- `NodeAtomicV2` — Tower + capability-based compute dispatch via `route()`
- `NestAtomic` — Node + provenance recording

**Key validation**: ToadStool JSON-RPC 2.0 wire format (`compute.submit`,
`compute.status`) validated via serialization round-trip (exp033).

### Deploy graph integration

ludoSpring now ships local deploy graphs:
- `graphs/ludospring_deploy.toml` — 5-phase Sequential deploy
- `graphs/ludospring_gaming_niche.toml` — ludoSpring + petalTongue composition

**toadStool** appears in Phase 2 as an optional accelerator (`fallback = "skip"`).
When available, ludoSpring's noise generation and raycasting dispatch to GPU via
the `compute.execute` capability.

---

## Part 4: Neural API Integration

### What ludoSpring provides for the Pathway Learner

1. **Passive metrics** — every dispatch emits structured JSON to stderr:
   ```json
   {"primal":"ludospring","op":"game.evaluate_flow","latency_us":4,"ok":true}
   ```

2. **Operation dependencies** — `capability.list` includes:
   ```json
   "game.complete_session": { "requires": ["session_id"], "depends_on": ["game.begin_session"] }
   ```

3. **Cost estimates** — `capability.list` includes:
   ```json
   "game.generate_noise": { "typical_latency_us": 100, "cpu_intensity": "medium", "memory_bytes": 1024 }
   ```

**toadStool action**: When scheduling GPU dispatch for ludoSpring workloads,
use cost estimates to determine whether GPU overhead (buffer copy, shader compile)
is worth it for the given workload size. For single Perlin evaluations, CPU is
faster. For batch (>1000 samples), GPU wins.

---

## Part 5: coralReef Requirements

Shaders validated in exp030 use:
- `f32` arithmetic only (no `f64`, no `u64`)
- `log2()` as the only transcendental function (Fitts/Hick)
- No dynamic memory allocation
- Fixed-size buffers

**coralReef action**: These shaders are compilation candidates for SASS/RDNA ISA
via coralReef's shader compilation pipeline (`shader.compile.wgsl`). The `f32`
constraint means no df64 emulation is needed for Tier A game science shaders.

---

## Part 6: Test Coverage for Absorbers

| Algorithm | Unit tests | Integration tests | Validation checks | Python parity | GPU parity |
|-----------|-----------|-------------------|-------------------|---------------|------------|
| Perlin 2D/3D | 6 | exp002, exp009 | 14 | Yes (exp034) | Yes (exp030) |
| fBm 2D/3D | 3 | exp009, exp014 | 8 | Yes (exp034) | Yes (exp030 host-side) |
| WFC | 4 | exp008, exp014 | 12 | — | — |
| BSP | 3 | exp017 | 8 | — | — |
| L-systems | 3 | exp013 | 6 | — | — |
| DDA raycaster | 4 | exp001 | 6 | — | Yes (exp030) |
| Engagement | 5 | exp010, exp021 | 15 | — | Yes (exp030) |
| Fitts's law | 4 | exp005, exp019 | 9 | Yes (exp034) | — |
| Hick's law | 3 | exp006, exp019 | 6 | Yes (exp034) | — |
| Flow state | 4 | exp010, exp012 | 13 | — | — |

**barraCuda action**: When absorbing, carry the unit tests with the code. The
Python parity and GPU parity tests remain in ludoSpring as cross-spring validation.

---

## Summary of Actions

### barraCuda team

1. **P1**: Absorb `procedural::noise` (Perlin 2D/3D + fBm) → `barracuda::noise`
2. **P2**: Absorb `procedural::wfc` → `barracuda::wfc`
3. **P2**: Absorb `procedural::bsp` → `barracuda::bsp`
4. **P3**: Absorb `procedural::lsystem` → `barracuda::lsystem`
5. **P3**: Evaluate `GenericFraudDetector` and `compute_distribution` for core

### toadStool team

1. Absorb 3 WGSL shaders from exp030 into `toadstool-runtime-gpu`
2. Add `compute.submit` support for ludoSpring game workloads
3. Use metalForge `Capability` enum as common hardware vocabulary
4. Implement budget-aware scheduling using ludoSpring cost estimates
5. Add barrier sync for Tier B WFC dispatch

### coralReef team

1. Compile Tier A WGSL shaders to SASS/RDNA ISA
2. No df64 needed for game science (all `f32`)
3. `log2()` is the only transcendental

---

*"We validate the math. You accelerate it."*
