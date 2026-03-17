<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# ludoSpring V23 → toadStool + barraCuda Absorption Handoff

**Date:** March 16, 2026
**From:** ludoSpring V23 — 75 experiments, 1692 checks, 394 tests + 12 proptest + 6 IPC integration
**To:** barraCuda team (math primitives absorption), toadStool team (GPU dispatch absorption)
**Covers:** ludoSpring V18–V23 evolution (game science, RPGPT dialogue plane, ecosystem deep debt)
**License:** AGPL-3.0-or-later

---

## Executive Summary

ludoSpring has matured through six major versions (V18–V23) since its deep primal
integration phase. This handoff documents:

1. **barraCuda primitives consumed** — what we use, how, and where the boundary is
2. **WGSL shaders ready for upstream absorption** — 14 game-domain shaders
3. **toadStool dispatch contract** — our typed client, tested patterns, and what we need next
4. **Patterns we evolved that are reusable** — validation harness, capability domains, tolerance architecture
5. **What we need from you** — 3D noise absorption, dispatch latency, shader compilation caching

---

## Part 1: barraCuda Primitives Consumed

### Active Usage

| Primitive | Consumer | Purpose | Hot Path? |
|-----------|----------|---------|-----------|
| `activations::sigmoid` | `interaction::flow::DifficultyCurve` | Flow state sigmoid transition | Yes — per-tick |
| `activations::sigmoid_batch` | `barcuda_math` re-export | Batch flow evaluation | GPU candidate |
| `stats::dot` | `metrics::engagement::compute_engagement` | Weighted engagement composite | Yes — per-tick |
| `stats::l2_norm` | `barcuda_math` re-export | Distance calculations | Moderate |
| `stats::mean` | `barcuda_math` re-export | Averaging | Moderate |
| `rng::lcg_step` | `procedural::bsp::generate_bsp` | Deterministic BSP splits | Once per level gen |
| `rng::state_to_f64` | `procedural::bsp::generate_bsp` | Float from LCG state | Once per level gen |
| `rng::uniform_f64_sequence` | `barcuda_math` re-export | Stochastic experiments | Test/benchmark |

### Consumption Pattern

ludoSpring imports barraCuda via path dependency (`../../barraCuda/crates/barracuda`)
and re-exports selected primitives through `barcuda_math` for experiment convenience.
All GPU promotion candidates are pure-math functions with no side effects.

### What We Do NOT Use (Yet)

| Primitive | Why Not | When We'd Use It |
|-----------|---------|-----------------|
| `linalg::gemm_*` | No matrix workloads in game science | RPGPT Phase 2 (NPC embedding similarity) |
| `nn::*` | Squirrel handles AI inference | If we internalize NPC personality inference |
| `spectral::*` | No signal processing in game domain | Audio analysis experiments |
| `nautilus::*` | No graph algorithms beyond DAG | World graph pathfinding at scale |

---

## Part 2: WGSL Shaders — Absorption Candidates

### Game Shaders (barracuda/shaders/game/) — 3 shaders

| Shader | LOC | Purpose | Absorption Priority |
|--------|-----|---------|-------------------|
| `fog_of_war.wgsl` | ~80 | Tile-based fog with sight radius | **P2** — game-specific, but reusable for any grid visibility |
| `tile_lighting.wgsl` | ~60 | Per-tile light attenuation | **P2** — standard point-light model |
| `pathfind_wavefront.wgsl` | ~100 | BFS wavefront on 2D grid | **P1** — generic graph BFS, reusable |

### CPU–GPU Parity Shaders (experiments/exp030/) — 11 shaders

| Shader | Purpose | Absorption Priority |
|--------|---------|-------------------|
| `perlin_2d.wgsl` | Perlin 2D noise | **Absorbed** — barraCuda has `perlin_2d_f64.wgsl` |
| `sigmoid.wgsl` | Sigmoid activation | **Absorbed** — barraCuda `activations` |
| `relu.wgsl` | ReLU activation | **Absorbed** — barraCuda `activations` |
| `softmax.wgsl` | Softmax activation | **Absorbed** — barraCuda `activations` |
| `dot_product.wgsl` | Dot product | **Absorbed** — barraCuda `stats` |
| `reduce_sum.wgsl` | Reduction sum | **Absorbed** — barraCuda `stats` |
| `abs.wgsl` | Absolute value | **Absorbed** — barraCuda `ops` |
| `scale.wgsl` | Scalar multiply | **Absorbed** — barraCuda `ops` |
| `lcg.wgsl` | LCG RNG | **Absorbed** — barraCuda `rng` |
| `dda_raycast.wgsl` | DDA raycaster | **P1** — per-column parallel, embarrassingly parallel |
| `engagement_batch.wgsl` | Batch engagement | **P1** — dot product + sigmoid composition |

**toadStool action:** The 3 game shaders + `dda_raycast.wgsl` + `engagement_batch.wgsl`
are good candidates for the shader catalog. They demonstrate game-domain GPU patterns
with validated CPU parity (exp030 proves bit-exact for all 11).

**barraCuda action:** `pathfind_wavefront.wgsl` could become a `graph::bfs_2d` op.
`dda_raycast.wgsl` could become an `ops::spatial::dda_raycast` op.

---

## Part 3: toadStool Dispatch Contract

### Our Typed Client (`barracuda/src/ipc/toadstool.rs`)

```
submit_workload(socket, shader, input_data) → ComputeResult
workload_status(socket, job_id) → ComputeResult
query_capabilities(socket) → SubstrateCapabilities
dispatch_submit(socket, shader_id, input_data) → ComputeResult
dispatch_result(socket, dispatch_id) → ComputeResult
dispatch_capabilities(socket) → SubstrateCapabilities
```

**Types:**
- `ComputeResult { available, job_id, status, output_hash, duration_us, error }`
- `SubstrateCapabilities { gpu_available, gpu_name, f64_supported, raw }`

### What Works

- Job queue path (`compute.submit/status/capabilities`) is tested
- Direct dispatch path (`compute.dispatch.*`) is wired but depends on toadStool support
- Graceful degradation: returns `available: false` when Neural API is unavailable
- `extract_rpc_result()` centralized error extraction (V23)

### What We Need from toadStool

| Need | Priority | Context |
|------|----------|---------|
| `compute.dispatch.submit` implementation | **P1** | Real-time game loop needs <3ms latency |
| Shader compilation caching | **P1** | Game shaders are loaded every session; cache by content hash |
| `f64_supported` accuracy | **P2** | Some GPUs demote f64; we need honest reporting for tolerance decisions |
| Batch dispatch | **P3** | Submit N shaders in one call for multi-pass rendering |

---

## Part 4: Reusable Patterns for Ecosystem

### 4.1 Validation Architecture

ludoSpring's `ValidationHarness<S: ValidationSink>` + `BaselineProvenance` pattern
has been adopted by wetSpring V122+ and groundSpring V109+:

```rust
ValidationHarness::new(EXP, StderrSink)
    .check("test_name", actual, expected, tolerance)
    .finish(); // exit 0 or 1
```

**For barraCuda:** Consider adopting for GPU parity tests where CPU reference
values are the "Python baseline equivalent."

### 4.2 Capability Domains Registry

`barracuda/src/capability_domains.rs` — structured capability introspection with:
- 24 capabilities (10 local, 14 external)
- Domain/subdomain classification
- Cost estimates (latency class, compute weight)
- Semantic mappings for Neural API routing

**For biomeOS:** This pattern gives the Neural API Pathway Learner structured
data for intelligent routing decisions.

### 4.3 Tolerance Architecture

`barracuda/src/tolerances/` — 6 domain-specific submodules with 50+ named constants:
- `game.rs` — frame rate, proximity, entity limits, unit conversions
- `interaction.rs` — Fitts/Hick/Steering analytical tolerances
- `ipc.rs` — timeouts, retry counts
- `metrics.rs` — engagement, flow, fun thresholds
- `procedural.rs` — noise, WFC, BSP tolerances
- `validation.rs` — analytical vs stochastic tolerances

Python mirror: `baselines/python/tolerances.py` (46 constants).

**For barraCuda:** This pattern prevents magic numbers in GPU parity assertions.
Recommend mirroring in barraCuda's test suite.

### 4.4 `deny.toml` Configuration (V23)

```toml
[bans]
wildcards = "deny"

[licenses]
allow = ["AGPL-3.0-or-later", "Apache-2.0", "MIT", "BSD-2-Clause", "BSD-3-Clause", ...]
```

**For all primals:** Follows barraCuda Sprint 6 pattern. Every primal should have a `deny.toml`.

### 4.5 `extract_rpc_result()` (V23)

Centralized JSON-RPC error extraction:

```rust
pub fn extract_rpc_result(response: &serde_json::Value) -> Result<serde_json::Value, String>
```

Handles `error.code` + `error.message` with safe defaults. Eliminates duplicated
check-for-error + extract-result patterns.

**For all primals:** Any primal parsing raw JSON-RPC responses should use a
single extraction point.

---

## Part 5: Write → Absorb → Lean Status

| ludoSpring Module | Status | barraCuda Absorption |
|-------------------|--------|---------------------|
| `procedural::noise` (Perlin 2D) | **Lean** | barraCuda has `perlin_2d_f64.wgsl` + CPU op |
| `procedural::noise` (Perlin 3D) | **Write** | 3D not yet in barraCuda — absorption candidate |
| `procedural::noise` (fBm) | **Write** | fBm not yet in barraCuda — absorption candidate |
| `procedural::wfc` | **Write** | WFC not in barraCuda; barrier sync needed for GPU |
| `procedural::bsp` | **Write** | BSP not in barraCuda; recursive → iterative needed |
| `procedural::lsystem` | **Write** | L-systems not in barraCuda; variable-length output |
| `game::raycaster` | **Write** | DDA raycaster could be `ops::spatial::dda` |
| `metrics::engagement` | **Write** | Could be a batch `stats::weighted_dot` variant |
| `interaction::input_laws` | **Write** | Fitts/Hick/Steering are pure log2 math |

**barraCuda action (P1):** Absorb Perlin 3D + fBm. ludoSpring wrote the reference
implementation, validated against Python baselines, and maintains the validation
chain. Once absorbed, ludoSpring leans on upstream.

**barraCuda action (P2):** Consider `ops::spatial::dda_raycast` for the DDA raycaster.
Embarrassingly parallel, validated to 6,623 FPS on CPU, ready for GPU.

---

## Part 6: Ecosystem Patterns Learned (V21–V23)

| Pattern | Source | What We Learned |
|---------|--------|-----------------|
| `#[expect(reason)]` dictionary | wetSpring V122 | Curated reasons prevent ad-hoc lint suppression; stale entries auto-detected |
| Zero-panic validation | groundSpring V109 | `let Ok/Some else exit(1)` gives CI clean signals, no stack traces |
| `extract_rpc_error()` | healthSpring V29 | Centralize once, rewire all callers |
| `deny.toml wildcards=deny` | barraCuda Sprint 6 | Supply chain hardening is 5 minutes of work for permanent protection |
| XDG socket resolution | biomeOS v2.46 | `$XDG_RUNTIME_DIR` with fallback, never hardcode `/run/user/` |
| Dual-format capabilities | neuralSpring S156 | Always handle both array and nested-object capability responses |
| Typed `IpcError` enum | coralReef Iter 52 | Next evolution: typed errors instead of `String` for IPC failures |

---

## Part 7: Next Steps (V24 Candidates)

| Candidate | Priority | Depends On |
|-----------|----------|------------|
| Typed `IpcError` enum (coralReef pattern) | P1 | Nothing — pure refactor |
| `CapabilityClient` SDK adoption (biomeOS v2.46) | P2 | Async runtime decision |
| Perlin 3D + fBm absorption into barraCuda | P1 | barraCuda team availability |
| DDA raycaster GPU shader via toadStool dispatch | P2 | `compute.dispatch.submit` working |
| metalForge integration tests with live GPU | P3 | GPU test infrastructure |

---

## License

AGPL-3.0-or-later
