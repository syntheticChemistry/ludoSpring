<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# ludoSpring V18 → barraCuda + toadStool Absorption Handoff

**Date**: March 15, 2026
**From**: ludoSpring V18 (game science spring)
**To**: barraCuda team (primitive absorption), toadStool team (GPU dispatch + shader absorption)
**Supersedes**: V17 Deep Evolution Handoff, V16 Absorption Handoff, V16 Niche Deployment, V14 Deep Audit
**Authority**: ludoSpring validates the math. barraCuda absorbs the primitives. toadStool dispatches the shaders.

---

## Executive Summary

ludoSpring V18 completes a niche self-knowledge evolution that makes absorption cleaner:

1. **`niche.rs`** centralizes all primal identity, capabilities, and metadata — absorption targets
   are clearly delineated from niche infrastructure
2. **`NeuralBridge`** typed IPC client provides the pattern for primal-to-primal communication —
   toadStool can adopt for dispatch coordination
3. **11 WGSL shaders** remain ready for toadStool absorption (unchanged from V17)
4. **12 proptest invariants** should accompany each absorbed primitive upstream
5. **Platform-agnostic paths** — all socket resolution via `std::env::temp_dir()` XDG chain,
   ready for cross-platform deployment

---

## Part 1: barraCuda Absorption Targets (Priority Order)

### P1: Perlin Noise (most consumed across all springs)

| Item | Location | Tests | Proptest |
|------|----------|-------|----------|
| `perlin_2d(x, y, perm)` | `barracuda/src/procedural/noise.rs` | 22 parity + determinism | `perlin_2d_bounded`, `perlin_2d_zero_at_integers` |
| `perlin_3d(x, y, z, perm)` | same | same | `perlin_3d_bounded` |
| `fbm_2d(x, y, perm, octaves, lacunarity, persistence)` | same | same | `fbm_2d_bounded` |
| `generate_permutation_table(seed)` | same | determinism tests | — |

**Cross-spring consumers**: wetSpring (disorder landscapes), airSpring (soil moisture), neuralSpring (weight initialization), ludoSpring (terrain, content generation).

**WGSL shader**: `perlin_2d.wgsl` (validated < 1e-3 vs CPU). toadStool absorbs shader alongside CPU primitive.

### P2: BSP Partition

| Item | Location | Tests | Proptest |
|------|----------|-------|----------|
| `generate_bsp(bounds, depth, seed)` | `barracuda/src/procedural/bsp.rs` | unit + determinism | `bsp_area_conserved`, `bsp_leaves_nonempty` |
| `BspNode` | same | — | — |

**Note**: Uses `barraCuda::rng::lcg_step` for deterministic splitting — dependency is internal.

### P2: WFC (Wave Function Collapse)

| Item | Location | Tests | Proptest |
|------|----------|-------|----------|
| `WfcGrid::new(width, height, tile_count)` | `barracuda/src/procedural/wfc.rs` | unit + determinism | `wfc_collapse_reduces_entropy` |
| `WfcGrid::collapse(x, y, tile)` | same | — | — |
| `WfcGrid::propagate(rules)` | same | — | — |

**GPU note**: WFC needs barrier synchronization for parallel constraint propagation (Tier B). Not a simple shader.

### P3: L-systems (lower priority, less cross-spring demand)

| Item | Location | Tests |
|------|----------|-------|
| `apply_rules(axiom, rules, iterations)` | `barracuda/src/procedural/lsystem.rs` | unit + determinism |
| `string_to_turtle(commands)` | same | — |

---

## Part 2: toadStool Shader Absorption

### 11 WGSL shaders (exp030/shaders/)

| Shader | Workgroup | CPU Reference | Tolerance | Priority |
|--------|-----------|---------------|-----------|----------|
| `perlin_2d.wgsl` | 64 | `procedural::noise::perlin_2d` | < 1e-3 | P1 (cross-spring) |
| `engagement_batch.wgsl` | 64 | `metrics::engagement::compute_engagement` | < 1e-4 | P1 (unique to ludoSpring) |
| `dda_raycast.wgsl` | 64 | `game::raycaster::cast_ray` | < 0.5 | P1 (game-critical) |
| `sigmoid.wgsl` | 64 | `barraCuda::activations::sigmoid` | < 1e-6 | P2 (already in barraCuda) |
| `dot_product.wgsl` | 256 | `barraCuda::stats::dot` | < 1e-4 | P2 (already in barraCuda) |
| `reduce_sum.wgsl` | 256 | `Iterator::sum` | < 1.0 | P2 (fundamental) |
| `softmax.wgsl` | 1 | numerically stable softmax | < 1e-5 | P2 |
| `lcg.wgsl` | 64 | `barraCuda::rng::lcg_step` | exact | P3 |
| `relu.wgsl` | 64 | `f32::max(x, 0.0)` | exact | P3 |
| `scale.wgsl` | 64 | linear transform | exact | P3 |
| `abs.wgsl` | 64 | `f32::abs` | exact | P3 |

All shaders: f32 only, no dynamic memory, no texture/sampler, fixed-size workgroup.
Only `reduce_sum.wgsl` uses `var<workgroup>` shared memory.

### GPU Pipeline Builder Pattern

exp030 consolidated 7 GPU runners from 1032 → 413 LOC using shared helpers:

| Helper | What it does |
|--------|-------------|
| `storage_entry(binding, read_only)` | Bind group layout entry |
| `create_storage_buf(ctx, label, size, writable)` | Buffer with usage flags |
| `build_pipeline(ctx, label, shader, entries)` | Shader → pipeline |
| `dispatch_and_read_f32(ctx, pipeline, bg, output, n, wg, default)` | Encode + submit + readback |

**Recommendation**: This pattern could become `toadStool::compute::Pipeline` or `toadStool::compute::Runner`.

---

## Part 3: NeuralBridge Pattern for toadStool

The `NeuralBridge` typed IPC client (ludoSpring V18) provides a reusable pattern for any
primal communicating with the biomeOS Neural API:

```rust
pub struct NeuralBridge {
    socket: PathBuf,
    timeout: Duration,
}

impl NeuralBridge {
    pub fn discover() -> Result<Self, Box<dyn Error>>  // XDG socket chain
    pub fn capability_call(&self, domain: &str, operation: &str, args: &Value) -> Result<Value, Box<dyn Error>>
    pub fn discover_capability(&self, domain: &str) -> Result<Value, Box<dyn Error>>
    pub fn register(&self, socket_path: &Path) -> Result<(), Box<dyn Error>>
    pub fn deregister(&self) -> Result<(), Box<dyn Error>>
}
```

**toadStool action**: If toadStool needs to coordinate with biomeOS (dispatch requests,
capability queries, registration), this pattern eliminates raw socket I/O. Consider a
shared `biomeos-client` crate if 3+ springs converge on this interface.

---

## Part 4: What ludoSpring Does NOT Need from barraCuda/toadStool

ludoSpring's consumption of barraCuda is minimal and intentional:

| Primitive | Current Use | Note |
|-----------|------------|------|
| `activations::sigmoid` | Flow difficulty curve | Only activation used |
| `activations::sigmoid_batch` | Re-exported but unused in ludoSpring | Available for consumers |
| `stats::dot` | Engagement composite score | Only stat used |
| `rng::lcg_step` | BSP partition splitting | Only RNG used |
| `rng::state_to_f64` | Float from LCG state | Paired with lcg_step |
| `rng::uniform_f64_sequence` | Re-exported | Available for consumers |
| `stats::l2_norm` | Re-exported | Available for consumers |
| `stats::mean` | Re-exported | Available for consumers |

ludoSpring does NOT need: tensor ops, spectral methods, surrogate models, staging,
unified hardware, SNN, lattice QCD, or any physics-specific primitives.

---

## Part 5: Quality at Handoff

| Check | Result |
|-------|--------|
| `cargo clippy --features ipc -p ludospring-barracuda` | 0 warnings (pedantic + nursery) |
| `cargo fmt --check` | 0 diffs |
| `cargo test --features ipc -p ludospring-barracuda` | 244 tests, 0 failures |
| TODO/FIXME/HACK in source | 0 |
| `unsafe` blocks | 0 (`#![forbid(unsafe_code)]`) |
| External C dependencies | 0 (ecoBin compliant) |
| Hardcoded `/tmp` | 0 |
| Hardcoded primal names | 0 |
| `#[allow()]` in production | 0 |

---

## Recommended Actions

### For barraCuda team

1. **Absorb Perlin noise** (P1) — most-consumed algorithm across all springs. Carry `perlin_2d_bounded` and `perlin_2d_zero_at_integers` proptest invariants upstream.
2. **Absorb BSP** (P2) — carry `bsp_area_conserved` proptest.
3. **Absorb WFC core** (P2) — carry `wfc_collapse_reduces_entropy` proptest.
4. **Consider L-systems** (P3) — lower cross-spring demand but clean absorption target.
5. **niche.rs pattern** — ludoSpring's `niche.rs` follows airSpring's pattern. Consider recommending it as the standard for all springs.

### For toadStool team

1. **Absorb 11 WGSL shaders** — all standalone, SPDX-licensed, tolerance-documented.
2. **Consider `Pipeline` builder** from exp030's consolidated helper pattern.
3. **Adopt NeuralBridge pattern** for biomeOS coordination if needed.
4. **GPU dispatch tiers** (unchanged):
   - Tier A (direct shader): noise, raycaster, engagement, fun_keys, flow, input_laws, GOMS
   - Tier B (needs adaptation): WFC (barrier sync), BSP (stack elimination), L-systems (variable output)

### For coralReef team

1. All shaders use f32 arithmetic, standard builtins, no dynamic memory.
2. `reduce_sum.wgsl` needs `var<workgroup>` support.

---

*We validate the math. You accelerate it. — ludoSpring V18*
