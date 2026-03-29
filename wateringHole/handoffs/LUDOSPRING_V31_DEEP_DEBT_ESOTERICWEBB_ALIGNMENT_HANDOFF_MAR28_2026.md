# ludoSpring V31 — Deep Debt + esotericWebb Alignment

**Date:** March 28, 2026
**From:** ludoSpring V31
**To:** barraCuda, toadStool, coralReef, petalTongue, biomeOS, primalSpring, Squirrel, esotericWebb, all springs
**Previous:** `archive/LUDOSPRING_V30_DEEP_EVOLUTION_MODERN_RUST_HANDOFF_MAR23_2026.md`
**Status:** Released — esotericWebb response shapes aligned, GPU IPC wired, workspace lints enforced

---

## Summary

V31 is a deep debt evolution focused on three axes: (1) aligning ludoSpring's
IPC surface with esotericWebb's LudoSpringClient expectations, (2) workspace-wide
lint enforcement across all 86 workspace members, and (3) completing implementation
of GPU compute handlers, visualization delegation, and MCP tool expansion.

80 files changed, 861 insertions, 72 deletions.

## What Changed

### esotericWebb Response Shape Alignment

Webb's `LudoSpringClient` (at `gardens/esotericWebb/webb/src/ipc/ludospring.rs`)
expects specific response types. V31 adds missing fields for compatibility:

| Method | Added Fields | Notes |
|--------|-------------|-------|
| `game.evaluate_flow` | `flow_score: f64`, `in_flow: bool` | Via new `flow_channel_metrics()` |
| `game.engagement` | `engagement_score`, `exploration_ratio` | Aliases for `composite` and `exploration_rate` |
| `game.difficulty_adjustment` | `reason: String` | Contextual recommendation explanation |
| `game.narrate_action` | `model`, `tokens` | From Squirrel metadata or defaults |
| `game.npc_dialogue` | `voice_notes: []`, `passive_checks_fired`, `degraded` | Webb `DialogueResponse` shape |

All existing fields preserved — backward compatible.

### Workspace Lint Enforcement

All 82 experiment `Cargo.toml` files and `benchmarks/Cargo.toml` now inherit
`[lints] workspace = true`. This means `unsafe_code = "forbid"`, `missing_docs = "deny"`,
and the full pedantic+nursery clippy profile apply uniformly across every workspace member.
Was 28/86; now 86/86.

### GPU IPC Handlers

New `barracuda/src/ipc/handlers/gpu.rs` (266 LOC) routes 4 `game.gpu.*` methods to
toadStool `compute.dispatch.submit` with structured CPU fallback when toadStool is unavailable:

- `game.gpu.fog_of_war` — accepts grid + viewer + sight params, forwards WGSL + uniforms
- `game.gpu.tile_lighting` — accepts grid + point lights array
- `game.gpu.pathfind` — accepts grid + start position + optional walls
- `game.gpu.perlin_terrain` — accepts grid + noise parameters + perm table

`game.gpu.batch_raycast` added to capability registry (27 total) but not yet handler-routed.

### Visualization Delegation

`neural.rs` stubs evolved from trivial `{ accepted: true }` responses to complete
petalTongue delegation via `VisualizationPushClient`. When petalTongue is discovered,
requests are forwarded. When unavailable, responses include structured degradation
metadata (`degraded: true`, `reason`, `delegated: false`).

### MCP Expansion

13 tools (was 8): added `game.begin_session`, `game.complete_session`,
`game.npc_dialogue`, `game.narrate_action`, `game.push_scene` with full JSON Schema
descriptors. Squirrel and any MCP client can now discover and invoke the full
storytelling surface.

### Fog of War WGSL

`fog_of_war.wgsl` now implements Bresenham line-of-sight checking the terrain buffer
for wall occlusion. The previously dead `terrain` binding is fully utilized.

### Other

- 3 `#[allow()]` → `#[expect(reason)]` (zero `#[allow()]` in production code)
- Dead import `Capability` removed from `metalForge/forge/routing.rs`
- Tolerance dedup: GPU raycaster tolerances re-export from validation module
- CI: forge tests + workspace-wide doc check added
- `validate_dispatch_routing` migrated to `ValidationHarness`
- Python parity provenance corrected (>= 3.10, not 3.12)

## Patterns for Team Absorption

### For primalSpring

- **STORYTELLING_EVOLUTION.md P0 is done.** ludoSpring V30 already had all 6 "missing" IPC
  methods (`game.narrate_action`, `game.npc_dialogue`, `game.voice_check`, `game.push_scene`,
  `game.begin_session`, `game.complete_session`). V31 aligns response shapes with Webb's
  `LudoSpringClient` types. Update the spec to reflect ludoSpring's actual state.
- **Graph capability alignment** (P1): `ludospring_validate.toml` expects `game.engine`,
  `game.physics`, `game.flow_state`, `game.tick_health` — none match actual IPC names.
  Align with real `game.evaluate_flow`, `game.engagement`, etc.
- **Deployment matrix storytelling cells**: ludoSpring binary is ready for plasmidBin.

### For barraCuda

- **WGSL absorption candidates** from exp030: `sigmoid.wgsl`, `relu.wgsl`, `dot_product.wgsl`,
  `reduce_sum.wgsl`, `softmax.wgsl`, `abs.wgsl`, `scale.wgsl`, `lcg.wgsl`,
  `engagement_batch.wgsl` — 9 generic math shaders suitable for `ops/`.
- **`flow_channel_metrics()` pattern**: Derives continuous score + boolean from a band
  comparison. Useful pattern for barraCuda's stats/analysis toolkit.
- **Tolerance re-export pattern**: Single source of truth with `pub use` aliases preserves
  domain-specific naming. Consider adopting in barraCuda tolerance modules.

### For toadStool

- ludoSpring now sends **4 real GPU compute jobs** via `compute.dispatch.submit` with typed
  params. The GPU handler builds uniforms + storage arrays into a `buffers` payload. If
  toadStool evolves the `compute.dispatch.submit` wire format, ludoSpring's
  `ipc/handlers/gpu.rs` needs to adapt.
- **`game.gpu.batch_raycast`** is registered but not handler-routed — next evolution step.

### For petalTongue

- Visualization methods (`visualization.render.*`) are now properly delegated, not stubbed.
  Responses include `peer` data when delegation succeeds and `degraded`/`reason` fields when
  petalTongue is unavailable.
- MCP now includes `game.push_scene` — AI agents can trigger scene pushes.

### For Squirrel

- MCP surface expanded from 8 to 13 tools. The 5 new tools (`begin_session`,
  `complete_session`, `npc_dialogue`, `narrate_action`, `push_scene`) give Squirrel the
  full storytelling pipeline.
- `game.narrate_action` response now includes `model` and `tokens` extracted from Squirrel's
  response metadata. If Squirrel evolves its response shape, update the extraction logic in
  `delegation.rs` `squirrel_chat_metadata()`.

### For esotericWebb

- Response shapes are now compatible with `LudoSpringClient` types. Extra fields on the
  ludoSpring side (`state`, `available`, `data`, etc.) are ignored by serde during
  deserialization into Webb's smaller structs.
- **Remaining gap**: Transport mismatch — ludoSpring is UDS-only, Webb is TCP-first. The
  `IpcServer` would need a TCP listener option or biomeOS must bridge.

### For all springs

- **Workspace lint enforcement pattern**: All members inherit `[lints] workspace = true`.
  This is the canonical way to ensure `unsafe_code = "forbid"` and `missing_docs = "deny"`
  uniformly. Springs with benchmark crates can layer domain-specific `[lints.clippy]`
  relaxations on top.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p ludospring-barracuda --features ipc --lib --tests
cargo test -p ludospring-forge --lib --tests
cargo doc --workspace --no-deps
cargo deny check
```

## Files Changed (from V30)

80 files changed, 861 insertions, 72 deletions:
- 58 experiment Cargo.toml + benchmarks Cargo.toml: `[lints] workspace = true`
- New: `barracuda/src/ipc/handlers/gpu.rs` (266 LOC)
- Modified: `results.rs`, `science.rs`, `delegation.rs`, `mcp.rs`, `neural.rs`, `mod.rs`,
  `params.rs`, `niche.rs`, `capability_domains.rs`, `gpu.rs` (tolerances), `flow.rs`,
  `noise.rs`, `fog_of_war.wgsl`, `validate_dispatch_routing.rs`, `routing.rs`,
  `provenance/mod.rs`, `ci.yml`, `README.md`, `python_parity.rs`, `validate.rs` (exp030)
