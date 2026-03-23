# metalForge — Local Hardware Profile

**Date**: March 18, 2026
**Status**: Validated — all compute experiments green against local hardware
**License**: AGPL-3.0-or-later
**Purpose**: Documents the validated local hardware topology and its implications
for toadStool compute dispatch and mixed hardware pipeline design.

---

## Detected Hardware

### CPU

| Property | Value |
|----------|-------|
| Model | AMD Ryzen 7 5800X3D 8-Core Processor |
| Architecture | Zen 3 (V-Cache) |
| Cores / Threads | 8 / 16 |
| System RAM | 128 GB DDR4 |
| Notable | 96 MB L3 V-Cache — large cache benefits sequential math |

### GPU

| Property | Value |
|----------|-------|
| Model | NVIDIA GeForce RTX 4060 |
| Vendor ID | 0x10de |
| Device ID | 0x2882 |
| VRAM | 8 GB GDDR6 |
| Compute Capability | 8.9 (Ada Lovelace) |
| Max SM Clock | 3105 MHz |
| TDP | 115 W |
| wgpu Backend | Vulkan |
| wgpu Device Type | DiscreteGpu |
| Driver | nvidia 580.82.09 |

### PCIe Topology

| Property | Value |
|----------|-------|
| PCIe Generation | 4 |
| Link Width | x8 |
| Max Speed | 16.0 GT/s |
| Effective Bandwidth | ~15.8 GB/s (one-way) |
| Bandwidth Tier | PciE4x8 |

**Note**: The RTX 4060 supports PCIe 4.0 x8 natively (not x16 downgraded).
This is the card's design — smaller bus bandwidth traded for lower cost and
power. The bandwidth is equivalent to PCIe 3.0 x16.

### NPU

No dedicated NPU detected on this system (PCI class 0x12 not found).
NPU routing paths are validated analytically with simulated hardware profiles.

---

## Validated Experiment Results (Local Hardware)

| Experiment | Checks | What It Validates |
|-----------|--------|------------------|
| exp030 (CPU/GPU parity) | 24/24 | RTX 4060 shader output matches CPU math (sigmoid, ReLU, dot, softmax, LCG, abs, Perlin 2D, fBm, engagement, raycaster DDA) |
| exp031 (dispatch routing) | 11/11 | wgpu discovers RTX 4060 as DiscreteGpu via Vulkan. Forge routes noise/physics/raycasting to GPU, WFC/metrics/UI to CPU |
| exp032 (mixed hardware) | 20/20 | PCIe Gen 4 x8 detected at 15.8 GB/s. Transfer cost model validated. NPU→GPU bypass faster than CPU roundtrip (analytical) |
| exp034 (Python parity) | 15/15 | barraCuda Rust math == Python baselines |
| exp035 (noise throughput) | 10/10 | Perlin/fBm throughput >500K samples/s. Within 3x of fastnoise-lite |
| exp036 (raycaster throughput) | 10/10 | DDA raycaster >60 FPS at 320 cols. 1000 frames avg <1ms |

---

## Implications for toadStool Absorption

### GPU Dispatch Decision Boundary

The exp030 benchmark reveals the CPU/GPU crossover point on this hardware:

| Workload Size | CPU (us) | GPU (us) | Winner |
|--------------|----------|----------|--------|
| 64 elements | 10 | ~18,000 | CPU |
| 1,024 | 25 | ~1,500 | CPU |
| 16,384 | 235 | ~1,900 | CPU |
| 65,536 | 920 | ~3,900 | CPU |

For sigmoid on this hardware, CPU wins at all tested sizes due to GPU dispatch
overhead (~1.5ms minimum). The crossover requires either:

1. **Larger batches** (100K+ elements) to amortize kernel launch cost
2. **Fused multi-op pipelines** (noise → engagement → raycaster in one dispatch)
3. **Persistent compute patterns** (GPU already hot, data already resident)

toadStool's dispatch routing should incorporate these crossover points as
hardware-specific tuning parameters, not hardcoded thresholds.

### PCIe Transfer Budget

At 15.8 GB/s effective bandwidth (PciE4x8):

| Data Size | Transfer Time | Implication |
|-----------|--------------|-------------|
| 1 KB | 0.06 us | Negligible — always worth offloading |
| 1 MB | 63 us | ~4 frames at 60 Hz — must batch across frames |
| 10 MB | 634 us | >1 frame — requires async pipeline |
| 100 MB | 6.3 ms | Half a frame — must stream or pre-load |

### Mixed Hardware Pipeline Model

The validated pipeline architecture (exp032):

```
CPU preprocess → [PCIe transfer] → GPU compute → [PCIe transfer] → CPU postprocess
                   ~63 us/MB                       ~63 us/MB
```

For NPU→GPU direct (when NPU available):

```
CPU preprocess → [PCIe] → NPU inference → [PCIe direct] → GPU compute → [PCIe] → CPU
                                            ~63 us/MB
                                            (bypasses CPU roundtrip = 2x faster)
```

---

## Silicon Budget — RTX 4060 Per-Unit Inventory

The RTX 4060 (AD107, Ada Lovelace) has the following functional units.
During compute-only dispatch, only shader cores and memory subsystem are
active (~52% of die area). The symphony model targets 70-80% utilization
by engaging compute, render, and fixed-function units concurrently.

### Programmable Units

| Unit | Count | Throughput | Compute-Only Active | Symphony Role |
|------|-------|-----------|-------------------|--------------|
| **CUDA Cores** | 3072 (24 SMs × 128) | ~15 TFLOPS FP32 | Yes | Physics batch, noise fBm, engagement metrics, pathfinding wavefronts |
| **Tensor Cores** (4th gen) | 96 (24 SMs × 4) | ~242 TOPS INT8, ~121 TFLOPS FP16 | No (unused in pure compute) | DF64 emulation for precision-critical math, matrix solves, neural inference |
| **RT Cores** (3rd gen) | 24 (1 per SM) | ~33 RT TFLOPS | No | Spatial queries (nearest-NPC, line-of-sight, BVH traversal) |

### Fixed-Function Units

| Unit | Count | Normal Role | Symphony Role |
|------|-------|-----------|--------------|
| **TMUs** (Texture Mapping Units) | 96 | Bilinear/trilinear texture sampling | Lookup table evaluation: biome maps, damage curves, transfer functions |
| **ROPs** (Render Output Units) | 48 | Pixel blending, depth test, AA | Histogram/reduction for analytics, player engagement statistics |
| **Rasterizer** | 4 GPC × 1 | Triangle scan conversion | Voronoi diagram generation (faction territories), signed distance fields |
| **Geometry Engine** | 4 GPC × 1 | Primitive assembly, culling | Spatial hashing for broad-phase collision detection |

### Memory Subsystem

| Unit | Specification | Role |
|------|-------------|------|
| **L2 Cache** | 32 MB | Persistent frame state, double-buffer working set, hot data residence |
| **VRAM** (GDDR6) | 8 GB, 128-bit bus | 272 GB/s bandwidth. World state, texture atlas, compute buffers |
| **Memory Controllers** | 4 × 32-bit | Bandwidth distribution across channels |

### Silicon Area Budget

| Category | Est. % Die | Compute-Only | Symphony |
|----------|-----------|-------------|----------|
| Shader cores (CUDA) | 40% | Active | Active |
| Tensor cores | 15% | **Idle** | Active (DF64, matrix) |
| RT cores | 10% | **Idle** | Active (spatial queries) |
| TMUs | 10% | **Idle** | Active (LUT sampling) |
| ROPs | 8% | **Idle** | Active (reduction) |
| Rasterizer + geometry | 5% | **Idle** | Active (Voronoi, SDF) |
| L2 cache | 8% | Active | Active |
| Memory controllers | 4% | Active | Active |
| **Total utilization** | **100%** | **~52%** | **~100%** |

### Utilization Improvement

The symphony model nearly doubles effective silicon utilization from ~52%
(compute-only) to ~100% (all units assigned work). This does not mean 2x
performance — fixed-function units have specialized throughput that may not
match the bottleneck. But it means:

1. **Tensor cores free**: DF64 emulation runs on tensor cores while shader
   cores handle standard FP32 physics. No contention.
2. **RT cores free**: BVH traversal for spatial queries runs on RT cores
   while shader cores compute noise fields. Parallel, not sequential.
3. **TMUs free**: Lookup table evaluation (biome classification, damage
   curves) runs on TMUs during compute passes. Zero shader core cost.
4. **ROPs free**: Histogram accumulation for player analytics runs on ROPs
   during render passes. No additional dispatch needed.

### toadStool Implications

When toadStool absorbs the symphony model, dispatch should consider:

- **Workload affinity**: Map problems to the cheapest capable unit.
  Spatial query → RT cores, not shader cores.
- **Occupancy tracking**: Don't dispatch to shader cores if tensor cores
  can handle the workload and shader cores are already saturated.
- **Persistent allocation**: L2 cache (32 MB) can hold double-buffer
  working sets between frames. Avoid re-uploading static data.
- **Bandwidth budget**: 272 GB/s VRAM bandwidth is shared across all
  units. Heavy TMU + shader + ROP use can saturate the bus.

---

## Hardware Evolution Roadmap

| Phase | What Changes | toadStool Impact |
|-------|-------------|-----------------|
| Current | RTX 4060 (Ada, PCIe 4 x8) | Baseline validation target |
| Near-term | PCIe 5 GPU upgrade | 2x transfer bandwidth, wider dispatch window |
| NPU addition | Dedicated inference accelerator | int8 inference offload, PCIe peer-to-peer |
| Multi-GPU | SLI/NVLink topology | Cross-device dispatch, data locality routing |
| Cloud target | coralReef deployment | Network-attached GPU, latency-aware routing |
