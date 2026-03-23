# Symphony Architecture — CPU/GPU Concurrent Pipeline

**Date**: March 18, 2026
**Status**: Active — foundational spec for metalForge compute evolution
**License**: AGPL-3.0-or-later
**Depends on**: `LOCAL_HARDWARE_PROFILE.md`, `forge/src/lib.rs`,
`GPU_FIXED_FUNCTION_SCIENCE_REPURPOSING.md` (wateringHole)

---

## The Problem with Request-Response

The current compute model (exp030-033) treats GPU dispatch as synchronous
request-response: CPU prepares data, transfers over PCIe, waits for GPU,
transfers result back, continues. This is the HTTP of compute.

```
CPU: [prepare]--[wait]--[wait]--[wait]--[process result]--[prepare]--...
GPU:            [compute]                                 [compute]
PCIe:      [up]           [down]                     [up]          [down]
```

Total frame time = CPU_prepare + PCIe_up + GPU_compute + PCIe_down + CPU_process.
On our hardware (RTX 4060, PCIe 4 x8), exp030 showed GPU dispatch overhead of
~1.5ms minimum, making CPU faster for all workloads under 65K elements.

This is wrong. Not because the measurements are wrong, but because the
architecture is wrong. Real video games don't work this way.

---

## The Symphony Model

A symphony orchestra doesn't have each section wait for the others to finish.
Strings, brass, woodwinds, and percussion play simultaneously, on different
parts of the same piece, at different tempos, producing a unified output.

The CPU and GPU are sections of the same orchestra:

- **CPU** (strings): sequential, expressive, handles game logic, AI, state
  machines, DAG operations — the melody
- **GPU compute** (brass): parallel, powerful, handles batch math — noise
  fields, physics integration, pathfinding wavefronts — the harmony
- **GPU render** (percussion): rhythmic, regular, produces visual output
  at fixed cadence — the beat
- **PCIe** (conductor's baton): synchronization points, minimal, at section
  boundaries — not between every note
- **Human player** (audience): perceives the unified output at ~60 Hz,
  responds at ~200ms, closes the loop as memory

The key insight: **the human closing the loop is unidirectional**. The monitor
emits photons. The player perceives. The player acts. The input arrives next
frame. There is no synchronous round-trip — the pipeline flows forward, and
latency is hidden by perception.

---

## Pipeline Topology

### Frame Pipeline (60 Hz = 16.67ms budget)

The symphony runs 2-3 frames deep. At any moment:

- **Frame N-1**: GPU is rendering this frame to the display
- **Frame N**: GPU compute is crunching physics/noise for this frame
- **Frame N+1**: CPU is running game logic, preparing next frame's workloads

```
Time:     0ms        4ms        8ms        12ms       16ms
          |----------|----------|----------|----------|
CPU:      [Logic N+1][Prepare N+1 GPU work][Logic N+2]
GPU Comp: [Physics N ][Noise N   ][Engage N]
GPU Rend: [Render N-1                      ][Present]
PCIe:     [Upload N]              [Download N]
```

Total effective frame time = max(CPU_work, GPU_work), not the sum.
This is how AAA games achieve 60+ FPS with complex simulations.

### Budget Partitioning

Given 16.67ms per frame on our hardware:

| Domain | Budget | What Runs | Constraint |
|--------|--------|-----------|-----------|
| CPU game logic | 4ms | AI decisions, state transitions, DAG ops | Sequential, branchy |
| CPU prepare | 2ms | Build GPU command buffers, uniforms | Must finish before GPU needs them |
| GPU compute | 4ms | Physics batch, noise fBm, pathfinding | Parallel, throughput-bound |
| GPU render | 8ms | Scene draw, post-processing, present | Fixed cadence, vsync-bound |
| PCIe transfer | 1ms | Upload uniforms (~16KB), download results (~64KB) | 15.8 GB/s at PCIe 4 x8 |
| **Overlap savings** | **-6ms** | CPU and GPU run in parallel | Budget = max, not sum |
| **Effective total** | **~10ms** | Fits in 16.67ms with 6ms headroom | |

### Double Buffering

Two sets of GPU buffers exist simultaneously:

- **Buffer A**: GPU is reading this (computing frame N)
- **Buffer B**: CPU is writing this (preparing frame N+1)

After frame N completes, the buffers swap. The CPU never waits for the GPU to
finish reading before writing the next frame's data. This eliminates the
PCIe round-trip stall entirely.

```
Frame N:    CPU writes Buffer B    |    GPU reads Buffer A
Frame N+1:  CPU writes Buffer A    |    GPU reads Buffer B
Frame N+2:  CPU writes Buffer B    |    GPU reads Buffer A
```

---

## barracuda as Universal Math

barracuda provides identical math on CPU and GPU (validated: exp030, 24/24
parity checks on RTX 4060). The symphony uses both simultaneously:

| barracuda Function | CPU Purpose | GPU Purpose |
|-------------------|-------------|-------------|
| `sigmoid` | NPC decision evaluation (1 at a time) | Engagement batch (10K players) |
| `perlin_2d/fbm_2d` | Single-point terrain query | Full chunk generation (1024x1024) |
| `lcg_step` | Procedural seed for one NPC | Batch RNG for particle systems |
| `dot` | Single vector comparison | Mass dot-product for physics solver |
| `cast_ray` | LOS check for one NPC | Full-screen raycaster (320+ columns) |

The math is identical. The scale determines which instrument plays it.
CPU handles the soloist parts. GPU handles the full orchestra.

---

## Silicon Exploitation

### The Fixed-Function Insight

From `GPU_FIXED_FUNCTION_SCIENCE_REPURPOSING.md` (wateringHole):

> "Every piece of fixed-function GPU hardware is a special-purpose computer
> that can be repurposed if you can map your problem to its input/output
> contract."

A consumer GPU (RTX 4060) has eight distinct hardware units. During a
compute-only dispatch, most sit idle. The symphony model assigns every unit
a role:

| GPU Unit | Silicon Area | Normal Use | Symphony Role |
|----------|-------------|-----------|---------------|
| Shader cores (3072 CUDA) | ~40% | Vertex/fragment/compute | Physics, noise, engagement |
| Tensor cores (4th gen) | ~15% | AI inference, DLSS | DF64 precision math, matrix solves |
| RT cores (3rd gen) | ~10% | Hardware ray tracing | Spatial queries, LOS, nearest-neighbor |
| TMUs (96 units) | ~10% | Texture sampling | Lookup tables, biome maps, curve eval |
| ROPs (48 units) | ~8% | Pixel blending/output | Histogram reduction, analytics |
| Rasterizer | ~5% | Triangle scan conversion | Voronoi diagrams, territory maps |
| L2 Cache (32MB) | ~8% | Data cache | Persistent frame state, double buffers |
| Memory controllers | ~4% | VRAM access | Bandwidth-limited streaming |

During compute-only dispatch, only shader cores are active (~40% of silicon).
The symphony model targets 70-80% silicon utilization by engaging compute +
render + RT simultaneously.

### DF64 Pattern (hotSpring Discovery)

Consumer GPUs have poor native fp64 throughput (RTX 4060: ~0.3 TFLOPS fp64
vs ~15 TFLOPS fp32). The DF64 pattern from hotSpring/coralReef uses
Dekker (1971) double-float arithmetic to emulate fp64 from fp32 pairs:

- Two fp32 values represent one fp64 value (hi + lo)
- Arithmetic uses error-free transformations (Knuth 1969)
- Throughput: 8-16x improvement over native fp64 on consumer GPUs

For ludoSpring, DF64 enables precision-critical game math (long-duration
simulations, large-world coordinates, financial systems) on consumer hardware
without requiring scientific GPUs.

---

## Forge Evolution: From route() to plan_frame()

### Current API (per-workload routing)

```rust
// Routes ONE workload to ONE substrate
let decision = route(&noise_profile, &substrates);
```

### Symphony API (frame-level planning)

```rust
// Routes ALL workloads for a frame, considering overlap
let plan = plan_frame(&frame_workloads, &substrates, &hardware);

// Predicts whether 60 Hz is achievable
let budget = estimate_budget(&plan, pipeline_depth);
```

The `FramePlan` groups workloads into concurrent execution bands:

```
Band 0 (CPU):   [game_logic, ai_decisions, state_update]
Band 1 (GPU):   [physics_batch, noise_generation]     // concurrent with Band 0
Band 2 (GPU):   [pathfinding, lighting]                // after Band 1
Band 3 (Render):[scene_draw, post_process, present]    // after Band 2
```

Each band has:
- Target substrate (CPU, GPU compute, GPU render)
- Estimated duration (from hardware profile)
- Data dependencies (which bands must complete first)
- PCIe transfer cost (for cross-substrate dependencies)

### Pipeline Depth

| Depth | Latency | Throughput | Use Case |
|-------|---------|-----------|----------|
| 1 | 1 frame (16ms) | Low — CPU waits for GPU | Turn-based games, tools |
| 2 | 2 frames (33ms) | Medium — double buffered | Most games |
| 3 | 3 frames (50ms) | High — AAA-style | Visually complex games |

The pipeline depth is a tunable parameter, not a hardcoded choice.
Lower latency matters for competitive games. Higher throughput matters
for visual fidelity.

---

## Cross-Spring Universality

The symphony model is not game-specific. The same architecture applies:

| Spring | CPU Role | GPU Compute Role | "Render" Role |
|--------|---------|-----------------|--------------|
| **ludoSpring** | Game logic, AI, DAG | Physics, noise, pathfinding | Frame render |
| **hotSpring** | Reactor control logic | Neutron transport, thermal hydraulics | Visualization |
| **wetSpring** | Molecular topology | Force evaluation, integration | Trajectory output |
| **healthSpring** | Patient state machine | Population simulation, risk scoring | Dashboard render |
| **primalSpring** | Test orchestration | Load generation, data processing | Report generation |

The math is universal (barracuda). The pipeline topology is universal
(symphony). The domain interpretation changes.

---

## What We Validate (exp082)

1. **Overlapping CPU/GPU work**: CPU submits GPU work, continues logic,
   collects results later. Effective frame time = max(CPU, GPU).
2. **Double buffering**: Two buffer sets, zero-stall swap. CPU writes
   buffer B while GPU reads buffer A.
3. **Frame budget model**: Given hardware profile, predict whether 60 Hz
   is achievable. Partition budget across domains.
4. **Persistent GPU state**: GPU retains data between frames (no re-upload).
   Only deltas transferred over PCIe.
5. **Crossover evolution**: GPU dispatch overhead amortized over persistent
   state. The "CPU always wins" observation from exp030 reverses when GPU
   is already hot and data is resident.

---

## References

- Carmack, J. (1999). Frame pipelining in Quake III Arena
- NVIDIA (2024). CUDA Programming Guide, Ch. 3: Asynchronous Concurrent
  Execution
- Dekker, T.J. (1971). A floating-point technique for extending the
  available precision. Numerische Mathematik, 18(3)
- ecoPrimals/wateringHole/GPU_FIXED_FUNCTION_SCIENCE_REPURPOSING.md —
  hotSpring silicon exploitation framework
