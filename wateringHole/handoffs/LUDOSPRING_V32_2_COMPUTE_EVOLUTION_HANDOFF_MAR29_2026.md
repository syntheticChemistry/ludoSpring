# ludoSpring V32.2 — Compute Evolution: GPU Parity + NPU Dispatch + NUCLEUS

**Date:** March 29, 2026
**From:** ludoSpring V32.2
**To:** barraCuda, toadStool, coralReef, metalForge, biomeOS, all springs
**Previous:** `LUDOSPRING_V32_COMPREHENSIVE_AUDIT_DEEP_DEBT_HANDOFF_MAR29_2026.md`
**Status:** Released — game shader parity, NPU routing, NUCLEUS graph coordination

---

## Summary

V32.2 extends the V32 audit remediation with active compute evolution:
- **exp030**: CPU-GPU parity expanded from 24 to 32 checks — now validates all 3 production game shaders (fog-of-war, tile lighting, pathfind wavefront) with CPU reference implementations
- **metalForge**: Full NPU dispatch support — `Substrate::Npu`, `recommend_substrate_full()`, NPU compute bands, direct NPU→GPU PCIe transfer modeling
- **exp032**: Forge-integrated mixed hardware validation (20→23 checks)
- **exp033**: NUCLEUS Tower/Node/Nest deepened with Forge mixed pipeline (19→27 checks)
- **All 82 experiments green** (81 standalone + exp042 requires live IPC)

---

## Part 1: For barraCuda Team

### Game Shader Parity Validated

exp030 now validates CPU-GPU parity for all game shaders, not just math primitives:

| Shader | CPU Reference | Tolerance | Status |
|--------|--------------|-----------|--------|
| `fog_of_war.wgsl` | `cpu_fog_of_war()` — per-tile visibility | Exact u32 match | PASS |
| `tile_lighting.wgsl` | `cpu_tile_lighting()` — multi-light 1/r² | `GPU_LIGHTING_ABS_TOL` (1e-5) | PASS |
| `pathfind_wavefront.wgsl` | `cpu_pathfind_step()` — BFS expansion | Exact u32 match | PASS |
| `perlin_2d.wgsl` | `noise::perlin_2d` | `GPU_PERLIN_ABS_TOL` (1e-3) | PASS |
| `dda_raycast.wgsl` | `raycaster::cast_ray` | `GPU_RAYCASTER_DISTANCE_ABS_TOL` | PASS |

**New tolerance constant:** `GPU_LIGHTING_ABS_TOL = 1e-5` in `barracuda/src/tolerances/gpu.rs`.

**Absorption opportunity:** The CPU reference implementations (`cpu_fog_of_war`, `cpu_tile_lighting`, `cpu_pathfind_step`) are candidates for barraCuda CPU fallback implementations when toadStool dispatch is unavailable.

### Shader Inventory for Upstream Absorption

Full WGSL inventory with promotion tiers (unchanged from V32, now fully parity-validated):

| Tier | Shader | barraCuda Status |
|------|--------|-----------------|
| A | sigmoid, relu, abs, scale, softmax | Direct `ops::*` mapping |
| A | perlin_2d | `procedural::noise` mapping |
| A | engagement_batch | `interaction::engagement` mapping |
| B | fog_of_war, tile_lighting | Adapt existing — need game-specific uniform structs |
| B | dda_raycast | `game::raycaster` mapping |
| C | pathfind_wavefront | New — BFS with atomics, no existing barraCuda primitive |

---

## Part 2: For toadStool Team

### Dispatch Evolution

- **NPU support now modeled**: `GameWorkload::QuantizedInference` routes to NPU via `recommend_substrate_full(workload, gpu, npu)`
- **Direct NPU→GPU PCIe transfer**: `npu_to_gpu_transfer_ms()` bypasses CPU roundtrip. Validated faster than CPU roundtrip in both forge unit tests and exp032
- **Frame pipeline with NPU**: `plan_frame()` now creates `BandTarget::NpuCompute` and `BandTarget::NpuToGpuTransfer` bands when NPU substrates are discovered

**Wire format validated**: exp033 confirms toadStool `compute.submit` JSON-RPC roundtrip for request/response serialization.

### Mixed Hardware Profile

New `HardwareProfile::mixed_gpu_npu()` with:
- NPU inference budget: 2ms/frame
- Direct NPU↔GPU bandwidth: 12 GB/s (PCIe 4.0 x8 direct)
- 60 Hz budget fits with all 3 substrate types

---

## Part 3: For metalForge / biomeOS Teams

### Forge API Evolution

```rust
// New: NPU-aware routing
pub fn recommend_substrate_full(workload, gpu_available, npu_available) -> Substrate;

// New: NPU bands in frame pipeline
pub enum BandTarget { Cpu, GpuCompute, GpuRender, PcieTransfer, NpuCompute, NpuToGpuTransfer }

// New: Mixed hardware profile
pub fn HardwareProfile::mixed_gpu_npu() -> Self;

// New: Direct transfer estimation
pub fn npu_to_gpu_transfer_ms(hardware, bytes) -> f64;
```

### biomeOS Deployment Graph with NPU

exp033 validates a 5-node deployment graph including NPU:
- Tower → Node_GPU (control flow)
- Tower → Node_NPU (control flow)
- Node_NPU → Node_GPU (data flow, 256KB — direct PCIe)
- Node_GPU → Nest (data flow, 4MB)
- Nest → Viz (data flow, 2MB)

Total budget: 10.5ms < 16.67ms (60 Hz) — fits with headroom.

---

## Part 4: For All Springs

### Experiment Validation Summary

| Metric | V32 | V32.2 |
|--------|-----|-------|
| exp030 checks | 24 | **32** |
| exp032 checks | 20 | **23** |
| exp033 checks | 19 | **27** |
| Forge tests | 19 | **26** |
| Library tests | 422 | 422 |
| Total experiments green | 81/82 | **81/82** |
| Total parity checks | ~1,200 | **~1,240** |

### Adopted Pattern: Game Shader CPU References

For each GPU shader, maintain a CPU reference implementation for:
1. Parity validation (exact match or documented tolerance)
2. CPU fallback when toadStool is unavailable
3. Deterministic testing (GPU may vary across adapters)

Pattern:
```rust
fn cpu_fog_of_war(grid_w, grid_h, viewer_x, viewer_y, ...) -> Vec<u32> { ... }
let gpu_result = gpu_run_fog_of_war(ctx, ...);
h.check_bool("fog_of_war_gpu_parity", gpu_result == cpu_result);
```

---

## Provenance

- Commit: V32.2 (see git log)
- Date: March 29, 2026
- Command: `cargo test --workspace && cargo run --bin exp030_cpu_gpu_parity`
- All checks pass on Vulkan adapter (RTX series)
