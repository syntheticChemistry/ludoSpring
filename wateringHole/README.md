# ludoSpring wateringHole — Cross-Project Handoffs

**Project:** ludoSpring (game science, HCI, procedural generation, cross-spring provenance)
**Last Updated:** March 30, 2026 (V35)

---

## What This Is

Unidirectional handoff documents from ludoSpring to consuming teams. No reverse
dependencies. Receiving teams: barraCuda (math primitives), toadStool (GPU dispatch),
coralReef (shader compilation), biomeOS (orchestration), petalTongue (visualization),
wetSpring (sample provenance), healthSpring (medical access), BearDog (cryptographic signing),
primalSpring (composition patterns), esotericWebb (gen4 product composition).

## Conventions

**Naming:** `LUDOSPRING_V{VER}_{TOPIC}_HANDOFF_{MON}{DD}_{YYYY}.md`

**Structure:** Header → Executive Summary → Parts → Tables → Action Items

**Archive:** Superseded handoffs move to `handoffs/archive/`

## Active Handoffs

| Version | File | Date | Scope |
|---------|------|------|-------|
| **V35** | [Central: LUDOSPRING_V35_PRIMAL_COMPOSITION_GAP_DISCOVERY_HANDOFF_MAR30_2026.md](../../infra/wateringHole/handoffs/LUDOSPRING_V35_PRIMAL_COMPOSITION_GAP_DISCOVERY_HANDOFF_MAR30_2026.md) | Mar 30 | Primal composition gap discovery: 5 experiments against live primals, 5 critical gaps documented (barraCuda missing, Neural API registration, coralReef transport, continuous executor, nucleus graphs) |
| **V34** | [LUDOSPRING_V34_NUCLEUS_NEST_HANDOFF_MAR29_2026.md](handoffs/LUDOSPRING_V34_NUCLEUS_NEST_HANDOFF_MAR29_2026.md) | Mar 29 | Dependency path fixes, workspace alignment |

## Cross-Spring Context

```
ludoSpring (game science, 88 experiments, 734 workspace tests, V35 primal composition gap discovery)
    │
    ├─→ barraCuda (absorb: Perlin, fBm, engagement batch, flow eval, fun classify, tolerance pattern, capability_domains pattern)
    ├─→ toadStool (dispatch: noise fields, raycaster, WFC, metrics batch, NUCLEUS pipeline, 3 game WGSL shaders, GPU dispatch for game.gpu.*)
    ├─→ coralReef (compile: f64-canonical shaders, log2 only transcendental)
    ├─→ petalTongue (render: 15 GameChannelType channels, scene push, dashboard, streaming sessions; delegation handlers for visualization / neural IPC)
    ├─→ Squirrel (AI: NPC dialogue, narration, internal voices via ai.query/analyze/suggest)
    ├─→ NestGate (storage: game state, NPC snapshots, rulesets via storage.store/retrieve)
    ├─→ biomeOS (orchestrate: game_logic + metrics nodes, Tower Atomic validated)
    ├─→ wetSpring (cross-spring: exp062 field sample scaffold, Anderson QS explorer)
    ├─→ healthSpring (cross-spring: exp063 consent-gated medical scaffold)
    ├─→ BearDog (cross-spring: exp064 signing wire format, IPC validation)
    └─→ nestgate (data: NCBI E-utilities for QS genes)
```

## petalTongue Integration

ludoSpring pushes live game science data to petalTongue through the unified `ludospring` binary: **UniBin subcommands** (`dashboard`, `live-session`, `tufte-dashboard`) replace the legacy separate per-feature binaries.

| Subcommand | Feature | Scenarios |
|------------|---------|-----------|
| `ludospring dashboard` | `ipc` | 8 real-math scenarios (all 7 `GameChannelType` channels) |
| `ludospring live-session` | `ipc` | 120-tick streaming session (append/set_value/replace) |
| `ludospring tufte-dashboard` | `ipc` | 3 Tufte analyses (genre, minimap, cognitive load) |

Connection: `VisualizationPushClient::discover()` via capability-based discovery (`visualization.render`).
Fallback: JSON files in `sandbox/scenarios/`, `sandbox/tufte/`, `sandbox/sessions/`.

## barraCuda Primitive Consumption

| Primitive | Module | Used by |
|-----------|--------|---------|
| `activations::sigmoid` | `interaction::flow` | `DifficultyCurve` sigmoid replacement |
| `stats::dot` | `metrics::engagement` | Weighted engagement composite |
| `rng::lcg_step` | `procedural::bsp` | Deterministic BSP generation |
| `rng::state_to_f64` | `procedural::bsp` | Split ratio from LCG state |
| `validation::ValidationHarness` | All 82 experiments | hotSpring-pattern check harness with pluggable `ValidationSink` |

### Absorption Opportunities

| Module | Lines | What barraCuda gets | Priority |
|--------|-------|---------------------|----------|
| `procedural::noise` | ~200 | Perlin 2D/3D + fBm (GPU-ready) | P1 |
| `procedural::wfc` | ~265 | Wave Function Collapse (GPU-parallel) | P2 |
| `procedural::lsystem` | ~200 | L-system string rewriting | P3 |
| `procedural::bsp` | ~220 | BSP spatial partitioning | P2 |
| `capability_domains.rs` | ~100 | Structured Domain/Method introspection | P1 |
| `validation/` (pattern) | ~400 | `ValidationSink` trait + `ValidationHarness<S>` — composable validation | P1 |
| `ipc/toadstool.rs` | ~80 | Typed toadStool client — first typed contract for compute dispatch | P0 |
| `ipc/handlers/delegation` (GPU path) | — | `game.gpu.*` → toadStool `compute.submit` with CPU fallback | P0 |
| `GenericFraudDetector` (exp065) | ~300 | Domain-agnostic graph fraud analysis | P3 |
| `compute_distribution` (exp066) | ~200 | Weighted-sum attribution with decay | P3 |

## V35 Composition Gap Summary

| Gap | Owner | Priority |
|-----|-------|----------|
| barraCuda not in plasmidBin | barraCuda | P0 |
| Primals not registered with Neural API | biomeOS | P0 |
| coralReef HTTP vs raw JSON-RPC on UDS | coralReef | P1 |
| barraCuda math not on JSON-RPC | barraCuda | P1 |
| Tensor element-wise ops not on IPC | barraCuda | P1 |
| Continuous executor stub | biomeOS | P2 |
| Nucleus vs runtime graph separation | biomeOS | P2 |

See central handoff for full details per team.

## Archive

| Version | File | Superseded by |
|---------|------|---------------|
| V32 | `handoffs/archive/LUDOSPRING_V32_COMPREHENSIVE_AUDIT_DEEP_DEBT_HANDOFF_MAR29_2026.md` | V34 Nucleus Nest |
| V31 | `handoffs/archive/LUDOSPRING_V31_DEEP_DEBT_ESOTERICWEBB_ALIGNMENT_HANDOFF_MAR28_2026.md` | V32 Comprehensive Audit + Deep Debt |
| V30 | `handoffs/archive/LUDOSPRING_V30_DEEP_EVOLUTION_MODERN_RUST_HANDOFF_MAR23_2026.md` | V31 Deep Debt + esotericWebb |
| V26 | `handoffs/archive/LUDOSPRING_V26_FULL_HARNESS_MIGRATION_HANDOFF_MAR18_2026.md` | V28 Deep Evolution |
| V26 | `handoffs/archive/LUDOSPRING_V26_TOADSTOOL_BARRACUDA_ABSORPTION_HANDOFF_MAR18_2026.md` | V28 Deep Evolution |
| V24 | `handoffs/archive/LUDOSPRING_V24_LEVERAGE_GUIDE_HANDOFF_MAR17_2026.md` | V26 Full Harness Migration + Absorption |
| V23 | `handoffs/archive/LUDOSPRING_V23_CROSS_ECOSYSTEM_DEEP_DEBT_HANDOFF_MAR16_2026.md` | V24 Leverage Guide + Absorption Sprint |
| V23 | `handoffs/archive/LUDOSPRING_V23_TOADSTOOL_BARRACUDA_ABSORPTION_HANDOFF_MAR16_2026.md` | V24 Leverage Guide + Absorption Sprint |
| V22 | `handoffs/archive/LUDOSPRING_V22_ECOSYSTEM_ABSORPTION_HANDOFF_MAR16_2026.md` | V23 Cross-Ecosystem Deep Debt |
| V21 | `handoffs/archive/LUDOSPRING_V21_BARRACUDA_TOADSTOOL_DEEP_DEBT_EVOLUTION_HANDOFF_MAR16_2026.md` | V22 Ecosystem Absorption |
| V20 | `handoffs/archive/LUDOSPRING_V20_BARRACUDA_TOADSTOOL_DEEP_PRIMAL_INTEGRATION_HANDOFF_MAR16_2026.md` | V21 Deep Debt Evolution |
| V19 | `handoffs/archive/LUDOSPRING_V19_BARRACUDA_TOADSTOOL_DEEP_DEBT_HANDOFF_MAR16_2026.md` | V20 Deep Primal Integration |
| V18 | `handoffs/archive/LUDOSPRING_V18_NICHE_SELF_KNOWLEDGE_NEURALBRIDGE_HANDOFF_MAR15_2026.md` | V19 Deep Debt |
| V18 | `handoffs/archive/LUDOSPRING_V18_BARRACUDA_TOADSTOOL_NICHE_ABSORPTION_HANDOFF_MAR15_2026.md` | V19 Deep Debt |
| V17 | `handoffs/archive/LUDOSPRING_V17_BARRACUDA_TOADSTOOL_DEEP_EVOLUTION_HANDOFF_MAR15_2026.md` | V18 Niche Self-Knowledge |
| V16 | `handoffs/archive/LUDOSPRING_V16_BARRACUDA_TOADSTOOL_ABSORPTION_HANDOFF_MAR15_2026.md` | V17 Deep Evolution |
| V16 | `handoffs/archive/LUDOSPRING_V16_NICHE_DEPLOYMENT_HANDOFF_MAR15_2026.md` | V17 Deep Evolution |
| V15 | `handoffs/archive/LUDOSPRING_V15_GPU_DISPATCH_BUILDOUT_HANDOFF_MAR14_2026.md` | V16 Niche Deployment |
| V14 | `handoffs/archive/LUDOSPRING_V14_DEEP_AUDIT_BARRACUDA_TOADSTOOL_HANDOFF_MAR14_2026.md` | V15 GPU Dispatch |
| V13 | `handoffs/archive/LUDOSPRING_V13_BARRACUDA_TOADSTOOL_CROSS_SPRING_PROVENANCE_HANDOFF_MAR13_2026.md` | V14 Deep Audit |

Older handoffs (V1–V3) archived in shared `ecoPrimals/wateringHole/handoffs/archive/`.

## License

AGPL-3.0-or-later
