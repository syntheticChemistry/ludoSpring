# ludoSpring wateringHole — Cross-Project Handoffs

**Project:** ludoSpring (game science, HCI, procedural generation, cross-spring provenance)
**Last Updated:** March 15, 2026

---

## What This Is

Unidirectional handoff documents from ludoSpring to consuming teams. No reverse
dependencies. Receiving teams: barraCuda (math primitives), toadStool (GPU dispatch),
coralReef (shader compilation), biomeOS (orchestration), petalTongue (visualization),
wetSpring (sample provenance), healthSpring (medical access), BearDog (cryptographic signing).

## Conventions

**Naming:** `LUDOSPRING_V{VER}_{TOPIC}_HANDOFF_{MON}{DD}_{YYYY}.md`

**Structure:** Header → Executive Summary → Parts → Tables → Action Items

**Archive:** Superseded handoffs move to `handoffs/archive/`

## Active Handoffs

| Version | File | Date | Scope |
|---------|------|------|-------|
| **V18** | `LUDOSPRING_V18_NICHE_SELF_KNOWLEDGE_NEURALBRIDGE_HANDOFF_MAR15_2026.md` | Mar 15 | Niche self-knowledge: `niche.rs` single source of truth, `NeuralBridge` typed IPC client, platform-agnostic paths, socket centralization, capability deduplication |
| **V18** | `LUDOSPRING_V18_BARRACUDA_TOADSTOOL_NICHE_ABSORPTION_HANDOFF_MAR15_2026.md` | Mar 15 | barraCuda + toadStool absorption: 11 WGSL shaders, 4 PCG absorption targets, NeuralBridge pattern, GPU pipeline builder |

## Cross-Spring Context

```
ludoSpring (game science, 66 experiments, 1371 checks, 244 tests + 12 proptest, niche self-knowledge V18)
    │
    ├─→ barraCuda (absorb: Perlin, fBm, engagement batch, flow eval, fun classify)
    ├─→ toadStool (dispatch: noise fields, raycaster, WFC, metrics batch, NUCLEUS pipeline)
    ├─→ coralReef (compile: f64-canonical shaders, log2 only transcendental)
    ├─→ petalTongue (render: 7 GameChannelType channels, streaming sessions)
    ├─→ biomeOS (orchestrate: game_logic + metrics nodes, Tower Atomic validated)
    ├─→ wetSpring (cross-spring: exp062 field sample scaffold, Anderson QS explorer)
    ├─→ healthSpring (cross-spring: exp063 consent-gated medical scaffold)
    ├─→ BearDog (cross-spring: exp064 signing wire format, IPC validation)
    └─→ nestgate (data: NCBI E-utilities for QS genes, providers module needs wiring)
```

## petalTongue Integration

ludoSpring pushes live game science data to petalTongue via 3 binaries:

| Binary | Feature | Scenarios |
|--------|---------|-----------|
| `ludospring_dashboard` | `ipc` | 8 real-math scenarios (all 7 `GameChannelType` channels) |
| `ludospring_live_session` | `ipc` | 120-tick streaming session (append/set_value/replace) |
| `ludospring_tufte_dashboard` | `ipc` | 3 Tufte analyses (genre, minimap, cognitive load) |

Connection: `VisualizationPushClient::discover()` via capability-based discovery (`visualization.render`).
Fallback: JSON files in `sandbox/scenarios/`, `sandbox/tufte/`, `sandbox/sessions/`.

## barraCuda Primitive Consumption

| Primitive | Module | Used by |
|-----------|--------|---------|
| `activations::sigmoid` | `interaction::flow` | `DifficultyCurve` sigmoid replacement |
| `stats::dot` | `metrics::engagement` | Weighted engagement composite |
| `rng::lcg_step` | `procedural::bsp` | Deterministic BSP generation |
| `rng::state_to_f64` | `procedural::bsp` | Split ratio from LCG state |
| `validation::ValidationResult` | All 66 experiments | hotSpring-pattern check harness |

### Absorption Opportunities

| Module | Lines | What barraCuda gets | Priority |
|--------|-------|---------------------|----------|
| `procedural::noise` | ~200 | Perlin 2D/3D + fBm (GPU-ready) | P1 |
| `procedural::wfc` | ~265 | Wave Function Collapse (GPU-parallel) | P2 |
| `procedural::lsystem` | ~200 | L-system string rewriting | P3 |
| `procedural::bsp` | ~220 | BSP spatial partitioning | P2 |
| `GenericFraudDetector` (exp065) | ~300 | Domain-agnostic graph fraud analysis | P3 |
| `compute_distribution` (exp066) | ~200 | Weighted-sum attribution with decay | P3 |

## Archive

| Version | File | Superseded by |
|---------|------|---------------|
| V17 | `handoffs/archive/LUDOSPRING_V17_BARRACUDA_TOADSTOOL_DEEP_EVOLUTION_HANDOFF_MAR15_2026.md` | V18 Niche Self-Knowledge |
| V16 | `handoffs/archive/LUDOSPRING_V16_BARRACUDA_TOADSTOOL_ABSORPTION_HANDOFF_MAR15_2026.md` | V17 Deep Evolution |
| V16 | `handoffs/archive/LUDOSPRING_V16_NICHE_DEPLOYMENT_HANDOFF_MAR15_2026.md` | V17 Deep Evolution |
| V15 | `handoffs/archive/LUDOSPRING_V15_GPU_DISPATCH_BUILDOUT_HANDOFF_MAR14_2026.md` | V16 Niche Deployment |
| V14 | `handoffs/archive/LUDOSPRING_V14_DEEP_AUDIT_BARRACUDA_TOADSTOOL_HANDOFF_MAR14_2026.md` | V15 GPU Dispatch |
| V13 | `handoffs/archive/LUDOSPRING_V13_BARRACUDA_TOADSTOOL_CROSS_SPRING_PROVENANCE_HANDOFF_MAR13_2026.md` | V14 Deep Audit |

Older handoffs (V1–V3) archived in shared `ecoPrimals/wateringHole/handoffs/archive/`.

## License

AGPL-3.0-or-later
