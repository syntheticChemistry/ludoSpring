# ludoSpring V15 → barraCuda/toadStool GPU Dispatch Buildout Handoff — March 14, 2026

**From:** ludoSpring V15
**To:** barraCuda team, toadStool team, coralReef team
**License:** AGPL-3.0-or-later
**Date:** March 14, 2026
**Supersedes:** V14 Deep Audit Handoff (March 14, 2026)

---

## Executive Summary

ludoSpring V15: **66 experiments, 1371 checks, 218 tests** — all green.

V15 is a **GPU dispatch and mixed hardware buildout** release. Four phases:

1. **Tier A WGSL shaders** in exp030 — Perlin 2D, fBm, engagement batch, DDA raycaster with CPU parity validation (+8 checks, 24 total)
2. **metalForge capability routing** — SubstrateKind (Cpu/Gpu/Npu), Capability enum, route(), fallback_chain() (+6 tests, 9 total)
3. **Mixed hardware transfer model** in exp032 — TransferPath (Direct PCIe P2P, ViaCpu roundtrip), 4-stage CPU→NPU→GPU→CPU pipeline (+6 checks, 18 total)
4. **NUCLEUS atomics evolution** in exp033 — NodeAtomicV2 with capability routing, toadStool JSON-RPC 2.0 dispatch client, biomeOS DeploymentGraph (+8 checks, 19 total)

Quality gates unchanged: 0 clippy warnings (pedantic + nursery), fmt clean, `#![forbid(unsafe_code)]` everywhere.

---

## Part 1: Tier A WGSL Shaders (exp030)

### New shaders (inline WGSL, `bytemuck` for buffer casts)

| Shader | Bindings | Workgroup | CPU Reference |
|--------|----------|-----------|---------------|
| `PERLIN_2D_WGSL` | perm[512] + coords[N*2] → output[N] | 64 | `noise::perlin_2d` |
| `ENGAGEMENT_BATCH_WGSL` | components[N*5] + weights[5] → output[N] | 64 | `metrics::engagement::compute_engagement` |
| `DDA_RAYCAST_WGSL` | map[W*H] + params[6] + angles[N] → distances[N] | 64 | `game::raycaster::cast_ray` |

fBm is computed by calling `PERLIN_2D_WGSL` per octave on the host (multi-pass).

### New GPU helpers

| Function | Bindings | Purpose |
|----------|----------|---------|
| `gpu_run_perlin()` | 3 (perm, coords, output) | Perlin noise field |
| `gpu_run_engagement_batch()` | 3 (components, weights, output) | Batch engagement scoring |
| `gpu_run_raycaster()` | 4 (map, params, angles, distances) | DDA raycasting |

### New validation checks (+8)

| Check | Tolerance | What it proves |
|-------|-----------|---------------|
| `perlin_gpu_parity` | 1e-3 (f32 vs f64) | GPU Perlin matches CPU |
| `perlin_gpu_range_bounded` | exact | All outputs in [-1.1, 1.1] |
| `perlin_gpu_deterministic` | exact | Same seed = same output |
| `engagement_gpu_parity` | 1e-4 | GPU engagement matches CPU |
| `fbm_gpu_parity` | 0.01 | Multi-octave fBm via Perlin |
| `raycaster_gpu_parity` | 0.5 | DDA distance matches CPU |
| `raycaster_gpu_hit_match` | exact | Wall hit/miss agreement |
| `batch_speedup_nonnegative` | 10ms grace | GPU not slower than CPU |

### Absorption candidates for barraCuda

These shaders are validation artifacts. barraCuda should absorb:
- Perlin 2D WGSL shader → `barracuda::gpu::noise` module
- Engagement batch shader → `barracuda::gpu::metrics` module
- DDA raycaster shader → `barracuda::gpu::game` module

---

## Part 2: metalForge Capability-Based Routing

### New types

| Type | Purpose | Example |
|------|---------|---------|
| `SubstrateKind` | Cpu, Gpu, Npu | Hardware category |
| `Capability` | F64Compute, F32Compute, ShaderDispatch, SimdVector, PcieTransfer, QuantizedInference{bits} | Hardware capability |
| `SubstrateInfo` | Rich substrate descriptor (kind, name, capabilities, flops) | `SubstrateInfo::default_gpu()` |
| `GameWorkloadProfile` | Workload requirements (required caps, preferred substrate) | `GameWorkloadProfile::noise_generation()` |
| `Decision<'a>` | Routing decision (substrate reference + reason) | `Decision { substrate, reason }` |

### New functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| `route()` | `(workload, substrates) -> Option<Decision>` | Capability-filtered routing: GPU > NPU > CPU |
| `fallback_chain()` | `(substrates) -> Vec<&SubstrateInfo>` | Ordered fallback: GPU > NPU > CPU |

### Workload profiles (7)

`noise_generation()`, `raycasting()`, `physics_tick()`, `wfc_step()`, `metrics_batch()`, `ui_analysis()`, `fraud_batch()`, `quantized_inference()`

### Backward compatibility

`recommend_substrate(GameWorkload, bool)` preserved — builds synthetic substrates and calls `route()` internally.

### Absorption path for toadStool

metalForge's `SubstrateInfo`/`Capability`/`route()` pattern matches the sibling spring pattern (groundSpring, hotSpring, airSpring). toadStool can absorb this directly as its game workload routing layer.

---

## Part 3: Mixed Hardware Transfer Model (exp032)

### New transfer model

| Type | Variants | Purpose |
|------|----------|---------|
| `TransferPath` | `Direct(BandwidthTier)`, `ViaCpu(BandwidthTier, BandwidthTier)`, `Local` | Transfer cost modeling |
| `MixedPipelineStageV2` | + `transfer_path: Option<TransferPath>` | Per-stage transfer override |
| `score_substrate_v2()` | Takes `TransferPath` instead of `BandwidthTier` | Cost-aware scoring |

### Key validation results

- Direct NPU→GPU PCIe P2P = half the time of CPU-mediated roundtrip
- 4-stage pipeline (CPU→NPU→GPU→CPU) completes with mixed transfer paths
- NPU→GPU bypass saves measurable time vs CPU roundtrip
- Transfer path cost ordering: Local < Direct < ViaCpu

---

## Part 4: NUCLEUS Atomics + toadStool Dispatch (exp033)

### NodeAtomicV2

Uses `route()` from metalForge instead of `recommend_substrate()`. Holds `Vec<SubstrateInfo>` (discovered substrates) instead of `bool gpu_available`. Dispatch log records substrate kind + routing reason.

### toadStool JSON-RPC 2.0 wire format

| Type | Fields | Purpose |
|------|--------|---------|
| `ToadStoolDispatchRequest` | jsonrpc, method ("compute.submit"), id, params | Job submission |
| `ToadStoolParams` | job_type, priority, vram_required_mb, shader_source | Job parameters |
| `ToadStoolDispatchResponse` | jsonrpc, id, result | Job result |
| `ToadStoolResult` | job_id, status, substrate_kind | Completion info |

Serialize/deserialize roundtrip validated. This is the wire format toadStool should implement.

### biomeOS DeploymentGraph

| Node Type | Budget (us) | Role |
|-----------|-------------|------|
| Tower | 500 | BearDog + Songbird |
| Node | 2000 | Compute dispatch |
| Nest | 1000 | Provenance recording |
| Compute | 8000 | GPU workloads |
| Viz | 3000 | petalTongue rendering |

Total: 14,500us fits in 16,667us (60Hz) frame budget.

### Graceful degradation

CPU-only NodeAtomicV2 (no GPU/NPU substrates) falls back to CPU for all workloads. No panics, no failures.

---

## Action Items

### For barraCuda team

1. **Absorb WGSL shaders** from exp030 into `barracuda::gpu::*` modules
2. **Absorb metalForge routing types** — `SubstrateKind`, `Capability`, `SubstrateInfo` as core types
3. **Add `noise::perlin_2d_f32()`** — f32 variant alongside existing f64 for GPU parity

### For toadStool team

1. **Implement `compute.submit` JSON-RPC handler** matching `ToadStoolDispatchRequest` wire format
2. **Absorb `route()` logic** from metalForge for game workload dispatch
3. **Wire GPU job queue** for noise, raycaster, engagement batch workloads
4. **Implement NPU→GPU direct PCIe transfer** using the `TransferPath::Direct` model

### For coralReef team

1. **Compile Perlin 2D WGSL** to native (SASS/RDNA ISA) for production performance
2. **Compile DDA raycaster WGSL** — embarrassingly parallel, high throughput target
3. **Profile f32 precision loss** — Perlin tolerance is 1e-3 (f32 vs f64); if tighter needed, dual-precision

---

## Files Modified

| File | Lines Changed | What |
|------|---------------|------|
| `experiments/exp030_cpu_gpu_parity/src/main.rs` | +1177 | WGSL shaders + GPU helpers + 8 checks |
| `metalForge/forge/src/lib.rs` | +343 | Capability-based routing system |
| `experiments/exp032_mixed_hardware/src/main.rs` | +272 | TransferPath + V2 pipeline + 6 checks |
| `experiments/exp033_nucleus_pipeline/src/main.rs` | +345 | NodeV2 + toadStool dispatch + biomeOS graph + 8 checks |
| `experiments/exp033_nucleus_pipeline/Cargo.toml` | +2 | serde + serde_json deps |

---

## License

AGPL-3.0-or-later
