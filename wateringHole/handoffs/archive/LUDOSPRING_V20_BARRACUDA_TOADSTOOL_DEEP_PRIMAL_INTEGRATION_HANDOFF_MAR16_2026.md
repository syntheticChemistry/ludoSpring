<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# ludoSpring V20 → barraCuda + toadStool Deep Primal Integration Handoff

**Date:** March 16, 2026
**From:** ludoSpring V20 — 75 experiments, 1692 checks, 394 tests + 12 proptest
**To:** barraCuda team (math primitives), toadStool team (GPU dispatch)
**Supersedes:** V19 Deep Debt Evolution
**License:** AGPL-3.0-or-later

---

## Executive Summary

- V20 aligns all 19 external IPC method names to canonical JSON-RPC specs (NestGate, Squirrel, rhizoCrypt, loamSpine, sweetGrass)
- New `capability_domains.rs` provides structured introspection of 24 capabilities (10 local, 14 external)
- Tolerance constants decomposed into 6 domain-specific submodules with backward-compatible re-exports
- Typed provenance pipeline: `DehydrationSummary` struct + `TrioStage` enum for 4-step session completion
- Game engine core deepened: `RulesetCert` command validation, concrete effect application, `GridMap` bridge
- `discover_by_capability()` enables runtime primal peer lookup
- Workspace dependencies consolidated (`serde`, `serde_json`, `uuid`, `proptest`)
- 394 tests pass, zero failures, zero clippy warnings

---

## Part 1: IPC Method Alignment (barraCuda relevance: P1)

All external primal calls now match canonical JSON-RPC method names. This is the authoritative mapping for any primal consuming these services:

| Primal | Old Method | New Method | Module |
|--------|-----------|------------|--------|
| NestGate | `storage.put` | `storage.store` | `ipc/nestgate.rs` |
| NestGate | `storage.get` | `storage.retrieve` | `ipc/nestgate.rs` |
| Squirrel | `ai.chat` | `ai.query` | `ipc/squirrel.rs` |
| Squirrel | `ai.text_generation` | `ai.suggest` | `ipc/squirrel.rs` |
| Squirrel | `ai.inference` | `ai.analyze` | `ipc/squirrel.rs` |
| rhizoCrypt | `dag.create_session` | `dag.session.create` | `ipc/provenance/mod.rs` |
| rhizoCrypt | `dag.append_vertex` | `dag.vertex.append` | `ipc/provenance/mod.rs` |
| rhizoCrypt | `dag.get_vertices` | `dag.vertex.list` | `ipc/provenance/rhizocrypt.rs` |
| rhizoCrypt | `dag.get_frontier` | `dag.frontier.get` | `ipc/provenance/rhizocrypt.rs` |
| rhizoCrypt | `dag.dehydrate` | `dag.session.dehydrate` | `ipc/provenance/mod.rs` |
| loamSpine | `spine.create_certificate` | `spine.certificate.create` | `ipc/provenance/loamspine.rs` |
| loamSpine | `spine.create_waypoint` | `spine.waypoint.create` | `ipc/provenance/loamspine.rs` |
| loamSpine | `spine.commit` | `spine.commit` | `ipc/provenance/mod.rs` |
| sweetGrass | `provenance.create_braid` | `braid.create` | `ipc/provenance/sweetgrass.rs` |
| sweetGrass | `provenance.lineage` | `provenance.graph` | `ipc/provenance/sweetgrass.rs` |
| sweetGrass | `provenance.attribution` | `attribution.chain` | `ipc/provenance/sweetgrass.rs` |
| sweetGrass | (new) | `provenance.record_dehydration` | `ipc/provenance/mod.rs` |

**barraCuda action:** If barraCuda has JSON-RPC clients for any of these primals, verify method names match.

**toadStool action:** Same — verify `compute.submit` and any primal calls match canonical specs.

---

## Part 2: Capability Domains Registry (barraCuda relevance: P1)

New `capability_domains.rs` provides a structured, introspectable registry:

```rust
pub const DOMAINS: &[Domain] = &[
    Domain {
        prefix: "game",
        description: "Game science — HCI models, flow, engagement, procedural generation",
        methods: &[
            Method { name: "evaluate_flow", fqn: "game.evaluate_flow", external: false },
            Method { name: "begin_session", fqn: "game.begin_session", external: true },
            // ... 24 total (10 local, 14 external)
        ],
    },
];
```

The `capability.list` RPC response now includes:
- Domain prefix and description
- Method FQN with local/external classification
- Operation dependencies from `niche.rs`
- Cost estimates for biomeOS scheduling

**barraCuda action:** Consider adopting this pattern for barraCuda's own `capability.list` response. The `Domain`/`Method` types are reusable.

**toadStool action:** The `external: true` flag tells biomeOS which capabilities require IPC routing — useful for toadStool dispatch planning.

---

## Part 3: Tolerance Decomposition Pattern (barraCuda relevance: P2)

The monolithic `tolerances/mod.rs` was decomposed into focused submodules:

```
tolerances/
  mod.rs          (re-exports for backward compat)
  game.rs         (sight radius, trigger range, NPC proximity, frame rate, entity limits)
  interaction.rs  (Fitts, Hick, Steering, Flow, DDA constants)
  ipc.rs          (RPC timeout, probe timeout, connect timeout)
  metrics.rs      (Tufte, engagement, UI analysis thresholds)
  procedural.rs   (raycaster, noise, chemistry constants)
  validation.rs   (analytical, raycaster, noise, UI tolerances)
```

New constants in `game.rs`:
- `DEFAULT_SIGHT_RADIUS: u32 = 5` — replaces magic number in `GameSession::new()`
- `TRIGGER_DETECTION_RANGE: u32 = 1` — fixes bug where range 0 missed adjacent triggers

**barraCuda action:** If barraCuda's tolerance/constants grow past ~200 lines, this decomposition pattern scales well. The re-export in `mod.rs` preserves backward compat.

---

## Part 4: Typed Provenance Pipeline (barraCuda relevance: P2)

Session completion is now a typed 4-step pipeline:

```
1. dag.session.dehydrate (rhizoCrypt) → DehydrationSummary
2. provenance.record_dehydration (sweetGrass) → acknowledgment
3. spine.commit (loamSpine) → commitment ID
4. braid.create (sweetGrass) → attribution braid
```

New types:

```rust
pub struct DehydrationSummary {
    pub merkle_root: String,
    pub frontier: Vec<String>,
    pub vertex_count: u64,
    pub raw: serde_json::Value,
}

pub enum TrioStage {
    Unavailable,  // rhizoCrypt not reachable
    Dehydrated,   // step 1 complete
    Committed,    // steps 1-3 complete
    Complete,     // all 4 steps complete
}
```

The pipeline handles partial completions gracefully — if sweetGrass is unavailable, the session still dehydrates and commits.

---

## Part 5: Game Engine Core Evolution

### RulesetCert Command Validation

`GameSession` now has an `active_ruleset: Option<RulesetCert>` field. When set, `process()` validates each `Command::verb()` against the ruleset's `available_actions`:

```rust
impl Command {
    pub fn verb(&self) -> &str {
        match self {
            Self::Move { .. } => "move",
            Self::Attack { .. } => "attack",
            Self::Talk { .. } => "talk",
            Self::Custom { verb, .. } => verb,
            // ... all 9 variants
        }
    }
}
```

Invalid commands return `ActionOutcome::NoEffect` — the ruleset enforces what actions are legal in each plane.

### Concrete Effect Application

`apply()` now handles three effects:
- `ItemAcquired { entity_id }` — despawns the item entity from the world
- `Damaged { target, amount }` — decrements `hp` property on target entity
- `Interacted { target }` — sets `last_interaction` property with current turn

### GridMap Bridge

`From<&TileWorld> for GridMap` bridges the high-level game world to the raycaster:

```rust
impl From<&TileWorld> for GridMap {
    fn from(world: &TileWorld) -> Self {
        // tiles that block_sight() → solid (true)
    }
}
```

---

## Part 6: GPU Compute Shaders (preserved from V19)

All shaders remain ready for toadStool absorption:

| Shader | Path | Workgroup | Purpose |
|--------|------|-----------|---------|
| `fog_of_war.wgsl` | `barracuda/shaders/game/` | 64 | Per-tile visibility from viewer position |
| `tile_lighting.wgsl` | `barracuda/shaders/game/` | 64 | Point light propagation (1/d² falloff) |
| `pathfind_wavefront.wgsl` | `barracuda/shaders/game/` | 64 | BFS expansion (one ring per dispatch) |

Plus from Tier A parity (exp030): `perlin_2d.wgsl`, `engagement_batch.wgsl`, `dda_raycast.wgsl`, `sigmoid.wgsl`, `dot_product.wgsl`, `reduce_sum.wgsl`, `softmax.wgsl`, `lcg.wgsl`, `relu.wgsl`, `scale.wgsl`, `abs.wgsl`.

---

## Part 7: Absorption Opportunities (updated)

| ludoSpring module | Lines | What barraCuda gets | Priority | V20 notes |
|-------------------|-------|---------------------|----------|-----------|
| `procedural::noise` | ~200 | Perlin 2D/3D + fBm | P1 | GPU-ready, Tier A |
| `procedural::wfc` | ~265 | Wave Function Collapse | P2 | Needs barrier sync |
| `procedural::bsp` | ~220 | BSP spatial partitioning | P2 | Recursive → iterative for GPU |
| `game::engine::gpu` | ~360 | 5 GpuOp dispatch types | P1 | RulesetCert + GridMap integration adds context |
| `capability_domains.rs` | ~100 | Domain/Method introspection pattern | P1 | Reusable for any primal |
| `tolerances/` (pattern) | ~300 | 6-submodule decomposition | P2 | Template for organized constants |

---

## Part 8: Code Quality Metrics

| Metric | V19 | V20 |
|--------|-----|-----|
| Tests (workspace) | 407 | 394 |
| Clippy warnings | 0 | 0 |
| `#[allow()]` in production | 0 | 0 |
| Magic numbers in prod | 0 | 0 |
| Production panics | 0 | 0 |
| IPC methods misaligned | 19 | 0 |
| Tolerance submodules | 1 (monolith) | 6 (domain-specific) |
| Capability introspection | basic | structured (Domain/Method) |
| Provenance pipeline | untyped | typed (DehydrationSummary/TrioStage) |
| Command validation | none | RulesetCert-backed |

---

## License

AGPL-3.0-or-later
