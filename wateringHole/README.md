# ludoSpring wateringHole — Cross-Project Handoffs

**Project:** ludoSpring (game science, HCI, procedural generation, cross-spring provenance)
**Last Updated:** March 13, 2026

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
| **V13** | `LUDOSPRING_V13_BARRACUDA_TOADSTOOL_CROSS_SPRING_PROVENANCE_HANDOFF_MAR13_2026.md` | Mar 13 | Cross-spring provenance: exp062-066 (228 checks), BearDog signing, fraud unification, attribution, GPU promotion map, action items for barraCuda/toadStool/coralReef |

## Cross-Spring Context

```
ludoSpring (game science, 66 experiments, 1349 checks)
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

Connection: `PetalTonguePushClient::discover()` via XDG Unix socket.
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
| V3 | `handoffs/archive/LUDOSPRING_V3_FULL_VALIDATION_HANDOFF_MAR11_2026.md` | V13 Cross-Spring Provenance |
| V3 | `handoffs/archive/LUDOSPRING_V3_BARRACUDA_TOADSTOOL_EVOLUTION_HANDOFF_MAR11_2026.md` | V13 Cross-Spring Provenance |
| V2 | `handoffs/archive/LUDOSPRING_V2_FULL_VALIDATION_HANDOFF_MAR11_2026.md` | V3 Full Validation |
| V2 | `handoffs/archive/LUDOSPRING_V2_BARRACUDA_TOADSTOOL_EVOLUTION_HANDOFF_MAR11_2026.md` | V3 GPU Evolution |
| V1 | `handoffs/archive/LUDOSPRING_V1_SCAFFOLD_HANDOFF_MAR_2026.md` | V2 Full Validation |

## License

AGPL-3.0-or-later
