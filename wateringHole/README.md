# ludoSpring wateringHole — Cross-Project Handoffs

**Project:** ludoSpring (game science, HCI, procedural generation, cross-spring provenance)
**Last Updated:** April 10, 2026 (V39)

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
| **V39** | [Central: LUDOSPRING_V39_NUCLEUS_COMPOSITION_PARITY_HANDOFF_APR10_2026.md](../../infra/wateringHole/handoffs/LUDOSPRING_V39_NUCLEUS_COMPOSITION_PARITY_HANDOFF_APR10_2026.md) | Apr 10 | NUCLEUS composition parity (Python → Rust → IPC → Primal), ecoBin harvest to plasmidBin v0.9.0, exp100 (27 checks), capability_registry.toml, 8 primal gaps, actions for all primal + spring teams |

## Cross-Spring Context

```
ludoSpring (game science, 100 experiments, 745+ workspace tests, V39 NUCLEUS composition parity)
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
| `validation::ValidationHarness` | All 100 experiments | hotSpring-pattern check harness with pluggable `ValidationSink` |

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

## V39 Composition Gap Matrix

| GAP | Owner | Severity | Impact |
|-----|-------|----------|--------|
| GAP-06: No UDS transport | **rhizoCrypt** | CRITICAL | Blocks 4 experiments |
| GAP-07: Startup panic (runtime nesting) | **loamSpine** | CRITICAL | Blocks 1 experiment |
| GAP-08: Fitts/Hick formula mismatch | **barraCuda** | HIGH | -4 composition checks |
| Neural API capability registration | **biomeOS** | HIGH | -14 checks (exp087, 088) |
| GAP-01: coralReef IPC client not exercised | **coralReef** | MEDIUM | Shader pipeline |
| GAP-05: Trio not in proto-nucleate | **primalSpring** | MEDIUM | Graph completeness |
| Inter-primal discovery gap | **toadStool** | MEDIUM | -1 check |
| Perlin3D lattice invariant | **barraCuda** | MEDIUM | -1 check |

**Score**: 95/141 (67.4%) + exp099 13/13 + exp100 27 checks (NUCLEUS parity). Projected with all fixes: ~143/154 (92.9%).

See [V39 central handoff](../../infra/wateringHole/handoffs/LUDOSPRING_V39_NUCLEUS_COMPOSITION_PARITY_HANDOFF_APR10_2026.md) for full details per team.

## Archive

| Version | File | Superseded by |
|---------|------|---------------|
| V38 | Central: `infra/wateringHole/handoffs/LUDOSPRING_V38_COMPOSITION_VALIDATION_CHAIN_HANDOFF_APR10_2026.md` | V39 NUCLEUS Composition Parity |
| V37.1 | Central: `infra/wateringHole/handoffs/archive/` | V38 Composition Validation Chain |
| V35 | Central: `infra/wateringHole/handoffs/archive/` | V38 Composition Validation Chain |
| V34 | `handoffs/archive/LUDOSPRING_V34_NUCLEUS_NEST_HANDOFF_MAR29_2026.md` | V38 Composition Validation Chain |
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
