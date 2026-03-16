<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# ludoSpring V19 → barraCuda + toadStool Deep Debt Evolution Handoff

**Date:** March 16, 2026
**From:** ludoSpring V19 — 75 experiments, 1692 checks, 407 tests + 12 proptest
**To:** barraCuda team (math primitives), toadStool team (GPU dispatch)
**Supersedes:** V18 Niche Self-Knowledge + V18 barraCuda/toadStool Niche Absorption
**License:** AGPL-3.0-or-later

---

## Executive Summary

- V19 eliminates all remaining technical debt: magic numbers, clone abuse, production panics, monolith modules, clippy suppressions
- 24 IPC capabilities fully wired (Squirrel AI, NestGate storage, petalTongue scene push, provenance trio deep integration, GPU compute)
- provenance.rs decomposed from 773-line monolith into 3 focused submodules matching the trio architecture
- All tolerance constants centralized with provenance citations — zero hardcoded magic numbers
- Zero clippy warnings across pedantic + nursery in both default and `--features ipc` modes
- 407 tests pass, zero failures

---

## Part 1: What Changed (V18 → V19)

### Tolerance Constants (barraCuda relevance: P2)

ludoSpring now has zero magic numbers in production code. All numeric thresholds live in `tolerances/mod.rs` with citations. New constants added:

| Constant | Value | Used in | barraCuda action |
|----------|-------|---------|------------------|
| `RPC_TIMEOUT_SECS` | 5 | `push_client.rs` | Consider for shared IPC timeout |
| `PROBE_TIMEOUT_MS` | 500 | `push_client.rs` | Socket probe timeout |
| `CONNECT_PROBE_TIMEOUT_MS` | 200 | `push_client.rs` | Quick liveness check |
| `NPC_PROXIMITY_TILES` | 3 | `audio.rs` | Game-specific, not applicable |
| `AREA_DESCRIPTION_RANGE_TILES` | 5 | `audio.rs` | Game-specific, not applicable |
| `ITEM_PROXIMITY_TILES` | 3 | `audio.rs` | Game-specific, not applicable |
| `DEFAULT_VERTEX_QUERY_LIMIT` | 50 | `provenance/rhizocrypt.rs` | DAG query limit |
| `TARGET_FRAME_RATE_HZ` | 60.0 | `handlers.rs` | Frame budget calc |

**barraCuda team:** If barraCuda adopts a tolerance pattern, ludoSpring's `tolerances/mod.rs` is the template: constant name, citation, justification.

### Clone Reduction Pattern (barraCuda relevance: P1)

`JsonRpcError` and `JsonRpcResponse` constructors changed from taking `id: serde_json::Value` (owned) to `id: &serde_json::Value` (borrowed). This moves the single necessary clone into the constructor and eliminates 13 call-site `.clone()` calls.

**Pattern for barraCuda/toadStool JSON-RPC code:**

```rust
// Before (V18): clone at every call site
Err(JsonRpcError::internal(req.id.clone(), &msg))

// After (V19): reference, clone happens once inside
Err(JsonRpcError::internal(&req.id, &msg))
```

### Result Evolution Pattern (barraCuda relevance: P2)

`BlockPalette::register()` evolved from `unwrap_or_else(|| panic!())` to `Result<BlockId, String>`. This is the pattern for any `u16`/`u32` overflow that's not truly impossible — return `Result` instead of panicking.

**toadStool action:** Review any `panic!()` or `.unwrap()` in non-test code for similar evolution.

### Module Decomposition Pattern (barraCuda relevance: P1)

ludoSpring's `provenance.rs` grew to 773 lines as rhizoCrypt, loamSpine, and sweetGrass integrations deepened. Decomposition strategy:

```
provenance.rs (773 lines) →
  provenance/
    mod.rs        (~200 lines, session lifecycle + shared types)
    rhizocrypt.rs (~200 lines, DAG queries, Merkle proofs)
    loamspine.rs  (~230 lines, certificates, spines, loans)
    sweetgrass.rs (~150 lines, attribution braids, lineage)
```

All public items re-exported via `pub use *` so callers are unaffected. This is the correct Rust idiom: one module per external primal integration.

**barraCuda action:** If `dispatch.rs` or `shader.rs` grows past ~500 lines, decompose along the same axis.

---

## Part 2: GPU Compute Shaders (preserved from V18)

All 3 game-specific WGSL shaders remain ready for toadStool absorption:

| Shader | Path | Workgroup | Purpose |
|--------|------|-----------|---------|
| `fog_of_war.wgsl` | `barracuda/shaders/game/` | 64 | Per-tile visibility from viewer position |
| `tile_lighting.wgsl` | `barracuda/shaders/game/` | 64 | Point light propagation (1/d² falloff) |
| `pathfind_wavefront.wgsl` | `barracuda/shaders/game/` | 64 | BFS expansion (one ring per dispatch) |

Plus inherited from barraCuda ecosystem: `perlin_2d.wgsl`, `dda_raycast.wgsl`.

**toadStool action:** These shaders embed `include_str!` in `game/engine/gpu.rs`. When toadStool absorbs them, the `GpuOp` enum and dispatch types provide the integration surface.

---

## Part 3: 24 IPC Capabilities (preserved from V18)

ludoSpring exposes 24 capabilities via `niche.rs`:

| Domain | Capabilities |
|--------|-------------|
| Game science | `evaluate_flow`, `fitts_cost`, `engagement`, `analyze_ui`, `accessibility`, `wfc_step`, `difficulty_adjustment`, `generate_noise` |
| Provenance | `begin_session`, `record_action`, `complete_session`, `poll_telemetry` |
| AI (Squirrel) | `npc_dialogue`, `narrate_action`, `voice_check` |
| Visualization | `push_scene` |
| DAG (rhizoCrypt) | `query_vertices` |
| Certificates (loamSpine) | `mint_certificate` |
| Storage (NestGate) | `storage_put`, `storage_get` |
| GPU compute | `gpu.fog_of_war`, `gpu.tile_lighting`, `gpu.pathfind`, `gpu.perlin_terrain` |

Each capability has `operation_dependencies()` and `cost_estimates()` for biomeOS scheduling.

---

## Part 4: Absorption Opportunities (updated)

| ludoSpring module | Lines | What barraCuda gets | Priority | Notes |
|-------------------|-------|---------------------|----------|-------|
| `procedural::noise` | ~200 | Perlin 2D/3D + fBm | P1 | GPU-ready, Tier A |
| `procedural::wfc` | ~265 | Wave Function Collapse | P2 | Needs barrier sync for GPU |
| `procedural::lsystem` | ~200 | L-system string rewriting | P3 | Variable-length output |
| `procedural::bsp` | ~220 | BSP spatial partitioning | P2 | Recursive → iterative for GPU |
| `game::engine::gpu` | ~360 | 5 GpuOp dispatch types | P1 | Ready for toadStool integration |
| `tolerances/mod.rs` | ~290 | Tolerance pattern template | P2 | Citations + justifications |

---

## Part 5: Code Quality Metrics

| Metric | V18 | V19 |
|--------|-----|-----|
| Tests (--features ipc) | 349 | 407 |
| Clippy warnings | 0 | 0 |
| `#[allow()]` in production | 0 | 0 |
| Magic numbers in prod | ~8 | 0 |
| Production panics | 1 | 0 |
| Files > 700 LOC | 2 | 0 |
| Clippy suppressions | 1 | 0 |
| Clone-at-callsite (handlers) | 13 | 0 |

---

## License

AGPL-3.0-or-later
