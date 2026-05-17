# ludoSpring V74 — petalTongue Scene Composition + Neural API Handoff

**Date:** May 16, 2026
**From:** ludoSpring (Tower Atomic specialist — electron)
**To:** petalTongue team, primalSpring (coordination), upstream primals, sibling springs
**Version:** V74 (910 tests, 10 scenarios, zero clippy, zero unsafe, zero debt)

---

## Executive Summary

ludoSpring now provides the first structured, testable petalTongue integration
from a domain spring. The new scene composition module maps all 15 game science
channel types to petalTongue's `DataBinding` wire format, and the meta-tier
validation harness validates rendering intent without requiring a live renderer.
This creates the typed contract between game science data production and
visualization rendering that `projectFOUNDATION/expressions/GAMING_CREATIVE_SCIENCE.md`
(Thread 9) requires.

Additionally, V72-V73 absorbed Neural API Signal Elevation (primalSpring Wave 17):
`primal.announce`, `health.version`/`health.drain`, and 8 signal dispatch constants.

---

## Part 1: petalTongue Scene Composition (V74)

### What Was Built

| Module | Purpose | Location |
|--------|---------|----------|
| `visualization::scene` | Maps `GameChannelType` → petalTongue `DataBinding` wire format | `barracuda/src/visualization/scene.rs` |
| `visualization::meta_validation` | Meta-tier validation using `RenderIntent` declarations | `barracuda/src/visualization/meta_validation.rs` |
| `push_composed_scene()` | IPC method serializing `ScenePayload` → JSON-RPC push | `barracuda/src/visualization/push_client.rs` |

### DataBinding Wire Format Mapping

| GameChannelType | → petalTongue binding_type | Use Case |
|-----------------|---------------------------|----------|
| `EngagementCurve` | `timeseries` | Player engagement over session time |
| `DifficultyProfile` | `timeseries` | Difficulty vs game progress |
| `FlowTimeline` | `bar` | Flow state duration per phase |
| `UiAnalysis` | `heatmap` | Tufte data-ink analysis grid |
| `InteractionCostMap` | `heatmap` | Fitts/Hick cost surface |
| `GenerationPreview` | `fieldmap` | 2D procedural gen preview |
| `AccessibilityReport` | `gauge` | 0–100 accessibility score |
| `DialogueTree` | `gamescene` | Branching NPC dialogue graph |
| `CharacterSheet` | `gauge` | Multi-stat character gauges |
| `CombatGrid` | `fieldmap` | Tactical spatial layout |
| `VoiceDisplay` | `bar` | Internal voice priority stack |
| `NpcStatus` | `gauge` | Disposition/trust meters |
| `DiceResult` | `gauge` | Roll result vs target |
| `ExplorationMap` | `fieldmap` | Fog-of-war spatial map |
| `NarrationStream` | `timeseries` | Text event timeline |

### Meta-Tier Validation Pattern

The `meta_validation` module uses `meta.observe`/`meta.intent` signal patterns
to validate rendering properties structurally:

```
RenderIntent::AnalyticalDense { min_data_ink: 0.7 }
RenderIntent::Interactive { min_targets: 5 }
RenderIntent::StatusGauge { show_thresholds: true }
RenderIntent::SpatialMap { min_entities: 3 }
RenderIntent::NarrativeSequence { min_nodes: 4 }
```

Each intent is validated against the composed `ScenePayload` without needing a
live petalTongue renderer. This allows CI validation of the rendering contract.

### Action Items for petalTongue Team

1. **Validate wire format**: Our `ScenePayload` serializes to JSON matching your
   `DataBinding` enum variants. Confirm the field names align with
   `petal-tongue-types/src/data_channel.rs`.
2. **`gamescene` variant**: We use this for dialogue trees (nodes + edges). Confirm
   this maps to your `GameScene` DataBinding variant and that `nodes` / `edges`
   field names are correct.
3. **`fieldmap` entity format**: We send `(x: f64, y: f64, label: String)` tuples.
   Confirm your `FieldMap` variant's entity structure.
4. **Tufte metric feedback**: Our `estimate_data_ink()` heuristic mirrors your
   golden test Tufte analysis. If you expose actual computed data-ink ratios via
   IPC response, we can upgrade from heuristic to live validation.

---

## Part 2: Neural API Signal Elevation (V72-V73)

### What Was Absorbed

| Item | Version | Impact |
|------|---------|--------|
| `primal.announce` method constant + dispatch alias | V73 | Single-call registration replacing 3-call pattern |
| `primal.info` method constant | V73 | Read-only identity query |
| 8 signal dispatch constants | V73 | `nest.store/commit/retrieve`, `tower.publish/authenticate/discover`, `meta.observe/intent` |
| `health.version` handler | V72 | Returns primal name, version, build target |
| `health.drain` handler | V72 | Acknowledges graceful shutdown intent |
| Registry sync 418→451 | V73 | All docs reference current ecosystem method count |

### Composition Pattern: `primal.announce` Backward Compatibility

We wired `primal.announce` as a dispatch *alias* in `dispatch_lifecycle`:

```rust
methods::primal::ANNOUNCE => neural::handle_lifecycle_register(params).await,
```

Both old (`lifecycle.register`) and new (`primal.announce`) paths dispatch to the
same handler. This means upstream can migrate callers at their own pace.

### Signal Dispatch Candidates Identified

For biomeOS orchestration collapse, ludoSpring identified these multi-call
sequences that map cleanly to single signals:

| Signal | Replaces | Context |
|--------|----------|---------|
| `nest.store` | begin_session + record_action + write_vertex | Game session provenance |
| `nest.commit` | complete_session + mint_certificate | Session certification |
| `tower.publish` | compose + validate + deploy | NUCLEUS graph publishing |
| `tower.authenticate` | beardog.sign + verify_chain | Session integrity |
| `meta.observe` | poll_interaction + evaluate_flow | Real-time game analytics |
| `meta.intent` | RenderIntent declaration | Visualization targeting |

### Action Items for primalSpring / biomeOS

1. The signal constants are declared but not yet routed to handlers (they're
   composition-level collapse, not spring-level). biomeOS should absorb these
   as orchestration primitives.
2. `primal.announce` backward compat pattern is validated — recommend as standard
   migration approach for all springs.

---

## Part 3: Composition Patterns for NUCLEUS Deployment

### ludoSpring Cell Graph (`ludospring_cell.toml`)

12-node composition: barraCuda (math), petalTongue (viz), Squirrel (AI),
provenance trio (rhizoCrypt + loamSpine + sweetGrass), Tower Atomic
(bearDog + songbird + skunkBat), and the spring validation binary itself.

### Deployment Model

ludoSpring does NOT deploy as a primal binary. It is a "pure composition" spring:
- The Rust binary is the tier 2 validation target
- Game science is served by composing primals via NUCLEUS cell graph
- biomeOS deploys the graph; the spring binary validates it

### Key Lessons for Other Springs

1. **Capability-first discovery works**: Zero hardcoded primal names since V55.
   All IPC uses capability-based discovery with `hint_name` fallback.
2. **IPC-first default is viable**: `default-features = []` since V62. Library
   users opt in with `--features local`. IPC works without library deps.
3. **Three-tier validation is the gold standard**: Python baselines (correctness) →
   Rust port (performance + fidelity) → Primal composition (deployment contract).
4. **Meta-tier validation**: You can validate rendering/computation intent without
   running the full pipeline. `meta.observe` + `meta.intent` enable this pattern.
5. **Signal dispatch constants prepare for orchestration collapse**: Define them
   now even if biomeOS doesn't route them yet. Springs declare intent; biomeOS
   absorbs routing at its own pace.

---

## Part 4: Primal Evolution Observations

### What We Learned About Each Primal

| Primal | Observation | Upstream Action |
|--------|-------------|-----------------|
| **petalTongue** | Rich scene graph API (`DataBinding` → `GrammarExpr` → `SceneGraph`). 18 crates. Ready for cross-spring DataBinding consumers. | Validate our wire format mapping. Consider exposing Tufte metrics via IPC response. |
| **bearDog** | Tower Atomic validation revealed base64 wire encoding requirement not in original docs. | Document encoding requirements in method spec. |
| **songbird** | Discovery + capability listing work flawlessly over IPC. | No action needed — pattern is solid. |
| **skunkBat** | Uses `security.audit_log` (not `security.audit`) for audit submission. | Document canonical method name in registry. |
| **barraCuda** | v0.4.0 stable. `optional = true` works well. GPU dispatch behind feature gate. | Consider publishing IPC-first guidance to other springs. |
| **coralReef** | IPC wired on ludoSpring side but blocked on upstream deployment. | Deploy to plasmidBin so springs can validate shader compilation over IPC. |
| **primalSpring** | Wave 17 `primal.announce` is clean. 451 registry is stable. | Absorb V74 scene composition as reference for other spring→petalTongue integrations. |

### Atomic Instantiation Pattern

Tower Atomic (bearDog + songbird + skunkBat) was validated live in V70:
- Signing works over IPC with base64 payload encoding
- Discovery returns structured capability lists
- Audit logging uses `security.audit_log` method
- All 6 capabilities pass within 50ms per call

This pattern should be replicated for Node Atomic and Nest Atomic once those
primals ship their IPC surfaces.

---

## Metrics

| Metric | Value |
|--------|-------|
| Workspace tests | 910 |
| Validation scenarios | 10 |
| Clippy warnings | 0 |
| Unsafe code | 0 |
| TODO/FIXME/HACK | 0 |
| Ecosystem methods synced | 451 |
| Capabilities | 32 |
| Primal gaps | 16/16 RESOLVED |
| guideStone readiness | 4 (NUCLEUS validated) |

---

## Files Changed (V72-V74)

- `barracuda/src/visualization/scene.rs` — NEW: 15-variant scene composition
- `barracuda/src/visualization/meta_validation.rs` — NEW: meta-tier validation harness
- `barracuda/src/visualization/push_client.rs` — Added `push_composed_scene()`
- `barracuda/src/visualization/mod.rs` — Re-exports for scene + meta_validation
- `barracuda/src/ipc/methods.rs` — `primal::ANNOUNCE/INFO`, 8 `signal::*` constants
- `barracuda/src/ipc/handlers/mod.rs` — `primal.announce` dispatch routing
- `barracuda/src/ipc/handlers/lifecycle.rs` — `handle_version`, `handle_drain`
- `barracuda/src/niche.rs` — `health.version`/`health.drain` capabilities
- `barracuda/src/capability_domains.rs` — Updated capability count
- `config/capability_registry.toml` — Updated health section

---

*Prepared for primalSpring audit. Push via SSH. Upstream will review gaps and
route to respective primal teams.*
