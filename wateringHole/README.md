# ludoSpring wateringHole — Cross-Project Handoffs

**Project:** ludoSpring (game science, HCI, procedural generation)
**Last Updated:** March 11, 2026

---

## What This Is

Unidirectional handoff documents from ludoSpring to consuming teams. No reverse
dependencies. Receiving teams: barraCuda (math primitives), toadStool (GPU dispatch),
coralReef (shader compilation), biomeOS (orchestration), petalTongue (visualization).

## Conventions

**Naming:** `LUDOSPRING_V{VER}_{TOPIC}_HANDOFF_{MON}{DD}_{YYYY}.md`

**Structure:** Header → Executive Summary → Parts → Tables → Action Items

**Archive:** Superseded handoffs move to `handoffs/archive/`

## Active Handoffs

| Version | File | Date | Scope |
|---------|------|------|-------|
| **V2** | `LUDOSPRING_V2_FULL_VALIDATION_HANDOFF_MAR11_2026.md` | Mar 11 | Full validation: 22 experiments (183 checks), 123 tests, 13 models, petalTongue integration, GPU shader promotion map |
| **V2** | `LUDOSPRING_V2_BARRACUDA_TOADSTOOL_EVOLUTION_HANDOFF_MAR11_2026.md` | Mar 11 | GPU evolution: 8 Tier A shader targets, WGSL sketches, toadStool streaming, precision characteristics |

## Cross-Spring Context

```
ludoSpring (game science)
    │
    ├─→ barraCuda (absorb: Perlin, fBm, engagement batch, flow eval, fun classify)
    ├─→ toadStool (dispatch: noise fields, raycaster, WFC, engagement batch)
    ├─→ coralReef (compile: f64-canonical shaders, log2 only transcendental)
    ├─→ petalTongue (render: 7 GameChannelType channels, streaming sessions)
    └─→ biomeOS (orchestrate: game_logic + metrics nodes, needs Continuous mode)
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

### Absorption Opportunities

| Module | Lines | What barraCuda gets |
|--------|-------|--------------------|
| `procedural::noise` | ~200 | Perlin 2D/3D + fBm (GPU-ready) |
| `procedural::wfc` | ~265 | Wave Function Collapse (GPU-parallel) |
| `procedural::lsystem` | ~200 | L-system string rewriting |
| `procedural::bsp` | ~220 | BSP spatial partitioning |

## Archive

| Version | File | Superseded by |
|---------|------|---------------|
| V1 | `handoffs/archive/LUDOSPRING_V1_SCAFFOLD_HANDOFF_MAR_2026.md` | V2 Full Validation |

## License

AGPL-3.0-or-later
