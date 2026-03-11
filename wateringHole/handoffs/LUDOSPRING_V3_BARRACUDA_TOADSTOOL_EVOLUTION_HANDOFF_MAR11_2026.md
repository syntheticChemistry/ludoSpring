# ludoSpring → barraCuda/toadStool GPU Evolution Handoff V3 — March 11, 2026

**From:** ludoSpring V6
**To:** barraCuda team, toadStool team, coralReef team, metalForge
**License:** AGPL-3.0-or-later
**Date:** March 11, 2026
**Supersedes:** V2 GPU Evolution Handoff (archived)

---

## Executive Summary

- ludoSpring V6: **44 experiments, 410 checks, 144 tests** — all green
- **13 HCI models validated** with Python parity (unchanged from V2)
- **8 Tier A modules** ready for GPU shader promotion (unchanged from V2)
- **4 new cross-spring experiments** demonstrate that ludoSpring's game science generalizes to bioinformatics exploration (Anderson QS) and primal infrastructure (NUCLEUS atomics)
- **New finding**: the game metric suite (engagement, flow, fun, DDA) validates on scientific exploration sessions, not just game sessions — this means barraCuda batch shaders serve both game and science workloads

---

## Part 1: What barraCuda Should Absorb

### 1.1 Primitives Currently Consumed

| Primitive | Consumer | Status |
|-----------|---------|--------|
| `activations::sigmoid` | `interaction::flow::DifficultyCurve` | Stable |
| `stats::dot` | `metrics::engagement::compute_engagement` | Stable |
| `rng::lcg_step` | `procedural::bsp::generate_bsp` | Stable |
| `rng::state_to_f64` | `procedural::bsp::generate_bsp` | Stable |

### 1.2 New Absorption Candidates from V3

| Module | LOC | What barraCuda Gets | Priority |
|--------|-----|---------------------|----------|
| `procedural::noise` (perlin_2d, fbm_2d) | ~200 | GPU-ready Perlin noise — validated against fastnoise-lite (0.93×), 3-way cross-validated (exp039). Now also used as Anderson disorder landscape generator (exp044). | **P1** |
| `procedural::wfc` | ~265 | Wave Function Collapse — constraint propagation with entropy-based collapse. Tier B (needs barrier sync). | P2 |
| `procedural::bsp` | ~220 | BSP spatial partitioning — deterministic via LCG. Tier B (needs iterative conversion). | P2 |
| `procedural::lsystem` | ~200 | L-system string rewriting + turtle interpretation. Tier B (variable-length output). | P3 |

### 1.3 Perlin Noise Is Now Cross-Spring Critical

exp044 uses `noise::perlin_2d` to generate Anderson disorder landscapes for microbial community visualization. This means the Perlin noise GPU shader serves:

1. **Game terrain** (ludoSpring): procedural landscape generation at 60 Hz
2. **Scientific visualization** (wetSpring × ludoSpring): disorder parameter fields for Anderson localization
3. **Potential healthSpring use**: physiological parameter landscapes for patient monitoring

**barraCuda action**: Perlin noise should be a first-class `barracuda::procedural` primitive, not just a ludoSpring local module. The CPU reference is validated — promote to barraCuda core and add GPU shader.

### 1.4 Batch Metric Shaders Serve Science Too

exp044 proves that `engagement`, `flow`, `fun_keys`, and `difficulty` produce valid results on scientific exploration sessions (microbial community navigation). This doubles the consumer base for batch metric shaders:

| Shader | Game workload | Science workload |
|--------|--------------|-----------------|
| `engagement_batch.wgsl` | N player sessions | N exploration sessions |
| `flow_eval.wgsl` | Player flow state | Researcher flow state |
| `fun_classify.wgsl` | Game fun type | Exploration fun type |
| `dda_adjust.wgsl` | Game difficulty | Exploration difficulty |

**barraCuda action**: These batch shaders are higher priority than V2 estimated — they serve both domains.

## Part 2: GPU Shader Promotion Map (Updated)

### Tier A — Ready Now

| Module | WGSL Shader | Validated By | Cross-Spring? |
|--------|-------------|-------------|---------------|
| `procedural::noise::perlin_2d` | `perlin_2d.wgsl` | exp002, exp009, exp035, exp039, **exp044** | Yes (wetSpring Anderson QS) |
| `procedural::noise::fbm_2d` | `fbm_2d.wgsl` | exp002, exp009, exp035 | Potential |
| `game::raycaster::cast_rays` | `dda_raycast.wgsl` | exp001, exp024, exp036 | No |
| `metrics::engagement::compute_engagement` | `engagement_batch.wgsl` | exp010, exp021, exp038, **exp044** | Yes (science exploration) |
| `metrics::fun_keys::classify_fun` | `fun_classify.wgsl` | exp018, exp038, **exp044** | Yes (science exploration) |
| `interaction::flow::evaluate_flow` | `flow_eval.wgsl` | exp010, exp012, exp040, **exp044** | Yes (science exploration) |
| `interaction::input_laws::*` | `interaction_laws.wgsl` | exp005–007, exp019, exp034 | Potential (healthSpring UI) |
| `interaction::goms::task_time` | `goms_batch.wgsl` | exp011, exp019 | Potential |

### Tier B — Needs Adaptation (unchanged from V2)

| Module | Challenge | Path Forward |
|--------|-----------|-------------|
| `procedural::bsp` | Recursive → iterative | Stack-free BVH traversal |
| `procedural::lsystem` | Variable-length output | Two-pass: count then generate |
| `procedural::wfc` | Global constraint propagation | Parallel propagation with barriers |
| `interaction::difficulty` | `VecDeque` state | Ring buffer in storage buffer |

## Part 3: toadStool Dispatch Evolution

### 3.1 New Dispatch Pattern: Scientific Exploration (from exp044)

exp044 runs a complete scientific exploration pipeline:

```
Generate disorder landscape (Perlin 32×32)
  → Simulate QS propagation (BFS flood-fill)
  → Compute engagement snapshot
  → Evaluate flow state
  → Classify fun type
  → Suggest DDA adjustment
```

For toadStool, this maps to:

```
Noise(GPU) → Propagation(CPU/GPU) → MetricsBatch(GPU) → FlowEval(GPU) → DDA(CPU)
```

The GPU stages are independent per community — toadStool can dispatch N communities in parallel.

### 3.2 NUCLEUS Integration (validated by exp042)

exp042 validated the Tower Atomic pattern:
- BearDog: JSON-RPC `crypto.hash` over Unix socket — deterministic Blake3/SHA3-256
- Songbird: IPC reachability (socket path alignment needed)

For toadStool, this means game-science GPU dispatch can be coordinated through biomeOS NUCLEUS graphs:

```
Tower Atomic (BearDog + Songbird)
  └── Node Atomic (toadStool GPU dispatch)
       ├── perlin_2d.wgsl (noise fields)
       ├── engagement_batch.wgsl (metric evaluation)
       └── flow_eval.wgsl (flow classification)
```

### 3.3 Streaming Pattern (unchanged from V2)

Game engines need 60 Hz continuous dispatch. Key requirements:
1. Pipeline persistence across frames
2. Double/triple buffer ring for CPU→GPU data flow
3. Async readback (frame N results read during frame N+1)
4. Priority dispatch (noise = background, raycast = latency-critical)

## Part 4: Precision Characteristics (unchanged from V2)

All ludoSpring math uses f64. No DF64 emulation needed — consumer GPUs suffice.

Key operations: fade curves (polynomial), log2 (Fitts/Hick), sin/cos (L-systems), dot product (engagement), comparisons (flow). All well-conditioned in f64.

## Part 5: Cross-Spring Lessons for Evolution

### 5.1 NCBI Data Makes Exploration Richer

exp041/exp043 fetch real biological data. When GPU-accelerated:
- Perlin noise generates disorder landscape from real community diversity (H')
- QS gene density from NCBI scales signal strength
- The visualization becomes a real-time scientific instrument, not a demo

**toadStool action**: Plan for NestGate data fetch → toadStool GPU dispatch → petalTongue render pipeline.

### 5.2 Tolerance Architecture Proven Across Domains

ludoSpring's `tolerances/mod.rs` pattern (named constants with citations) works for both HCI constants (Fitts a=50ms, b=150ms) and biological parameters (W = 3.5·H' + 8.0·O₂). This should be the ecosystem standard.

**barraCuda action**: Consider `barracuda::tolerances` as an ecosystem-wide named-constant registry.

### 5.3 ValidationResult Pattern Works Everywhere

All 44 experiments use the same `ValidationResult::check(experiment, name, actual, expected, tolerance)` pattern. This is the same harness wetSpring and hotSpring use.

**barraCuda action**: The pattern is stable — no changes needed.

## Part 6: Prioritized Action Items

### barraCuda P1 (absorb now)

1. **Perlin noise** → `barracuda::procedural::perlin_2d_f64` (CPU + WGSL compute shader)
2. **fBm** → `barracuda::procedural::fbm_2d_f64` (octave loop)
3. **Engagement batch** → `barracuda::metrics::engagement_batch_f64` (weighted dot)
4. **Flow eval batch** → `barracuda::metrics::flow_eval_batch_f64` (comparison shader)

### barraCuda P2 (next cycle)

5. **DDA raycaster** → `barracuda::game::dda_raycast_f64`
6. **Fun classify batch** → `barracuda::metrics::fun_classify_batch_f64`
7. **LCG step WGSL utility** → deterministic PCG on GPU

### toadStool P1

1. Perlin noise grid dispatch (width × height compute)
2. DDA raycaster column dispatch (N columns at 60 Hz)
3. Batch metrics dispatch (N sessions × M metrics)

### toadStool P2

4. Continuous pipeline persistence for 60 Hz
5. NestGate → toadStool → petalTongue data pipeline
6. NUCLEUS graph integration for game-science workloads

### coralReef (unchanged)

1. `log2` f64 lowering for Fitts/Hick shaders
2. `sin`/`cos` f64 for turtle interpretation
3. Fused noise→raycast compilation

### metalForge integration

Game-science pipeline stages for cross-substrate dispatch:

```
NestGate(CPU) → Noise(GPU) → Propagation(GPU) → Metrics(GPU) → Engagement(CPU) → Present
```

GPU stages should stay on-device (zero CPU round-trips), matching hotSpring's validated metalForge pattern.
