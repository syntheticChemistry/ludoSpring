# ludoSpring wateringHole — Cross-Project Handoffs

**Project:** ludoSpring (game science, HCI, procedural generation, cross-spring provenance)
**Last Updated:** May 16, 2026 — **V74 is current** (petalTongue scene composition + meta-tier validation. 910 tests, 10 scenarios, zero clippy, zero unsafe. All 16 gaps RESOLVED.)

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
| **V74** | [LUDOSPRING_V74_PETALTONGUE_COMPOSITION_HANDOFF_MAY16_2026.md](handoffs/LUDOSPRING_V74_PETALTONGUE_COMPOSITION_HANDOFF_MAY16_2026.md) | May 16 | **Current** — petalTongue scene composition, meta-tier validation, Neural API signals, primal evolution observations. 910 tests. |
| V71 | [LUDOSPRING_V71_NICHE_CONVERGENCE_HANDOFF_MAY13_2026.md](handoffs/LUDOSPRING_V71_NICHE_CONVERGENCE_HANDOFF_MAY13_2026.md) | May 13 | Niche convergence: MDA + BM-004/005, Tower Atomic reference, composition patterns for sibling springs. |
| V71 | [LUDOSPRING_V71_DEEP_DEBT_AUDIT_MAY13_2026.md](handoffs/LUDOSPRING_V71_DEEP_DEBT_AUDIT_MAY13_2026.md) | May 13 | Deep debt CLEAN, audit questions answered. 896 tests. |
| V70 | [Central: LUDOSPRING_V70_TOWER_ATOMIC_LIVE_VALIDATED_MAY13_2026.md](../../infra/wateringHole/handoffs/LUDOSPRING_V70_TOWER_ATOMIC_LIVE_VALIDATED_MAY13_2026.md) | May 13 | Tower Atomic LIVE: 6/6 capabilities pass, protocol corrections (base64, security.audit_log). |
| V68 | [Central: LUDOSPRING_V67_TIER2_CONVERGENCE_HANDOFF_MAY12_2026.md](../../infra/wateringHole/handoffs/LUDOSPRING_V67_TIER2_CONVERGENCE_HANDOFF_MAY12_2026.md) | May 13 | Tier 2 wire contract aligned to `LIVE_SCIENCE_API.md`. `list_workloads` wired. 858 tests. |

## Cross-Spring Context

```
ludoSpring (game science composition, 910 workspace tests, V74 — petalTongue scene composition, meta-tier validation, Tower Atomic LIVE, Tier 2 aligned)
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

In the composition model, petalTongue is composed as a primal node in the cell graph. The spring binary's UniBin subcommands (`dashboard`, `live-session`, `tufte-dashboard`) remain as development/validation tools — they are NOT deployed.

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

## Composition Gap Status (V68, updated May 13, 2026)

| GAP | Owner | Status | Notes |
|-----|-------|--------|-------|
| GAP-01: coralReef IPC client | **coralReef** | **WIRED** (V64) | `try_coralreef_compile` in GPU path, blocked on upstream SM rebuild |
| GAP-02: Domain method parity | **ludoSpring** | **ADVANCED** (V64) | `math.flow.evaluate` + `math.engagement.composite` registered |
| GAP-03: biomeOS game_logic orchestration | **biomeOS** | **RESOLVED** | biomeOS v3.51 `composition.deploy` route alias |
| GAP-04: provenance commit chain | **rhizoCrypt** | OPEN | Commit exists but deterministic replay not validated |
| GAP-05: Trio not in proto-nucleate | **primalSpring** | OPEN | Graph completeness |
| GAP-06: rhizoCrypt UDS transport | **rhizoCrypt** | **RESOLVED** | S66: UDS operational since S23 |
| GAP-09: Neural API registration | **biomeOS** | **RESOLVED** | biomeOS v3.51 `method.register` |
| GAP-12: Registry cross-sync | **primalSpring** | **RESOLVED** | V59: 28 `game.*` methods registered (451 total) |
| GAP-13: barraCuda build regression | **barraCuda** | **RESOLVED** | V57: `#[cfg(feature = "gpu")]` gate |
| GAP-14: Provenance commit hash | **ludoSpring** | OPEN | Multiple commit hashes across validators |
| GAP-15: Squirrel in graph | **ludoSpring** | **RESOLVED** | V56: node added |
| GAP-07: loamSpine startup panic | **loamSpine** | **RESOLVED** | PG-33 |
| GAP-08/11: Fitts/Hick formulation | **barraCuda** | **RESOLVED** | PG-38 |
| GAP-10: game.* graph identity | **ludoSpring** | **RESOLVED** | V53: pure composition |

**Score**: 130/141 (92.2%) composition checks. Remaining 11 low-severity (PG-47 perlin3d, PG-48 petalTongue threading, Squirrel inference routing, content ownership edge cases).

**guideStone:** 54/54 checks passed (L4 NUCLEUS validated). Standard v1.2.0.

See: [V67 Tier 2 convergence](../../infra/wateringHole/handoffs/LUDOSPRING_V67_TIER2_CONVERGENCE_HANDOFF_MAY12_2026.md), [V59 post-interstadial](handoffs/LUDOSPRING_V59_POST_INTERSTADIAL_HANDOFF_MAY10_2026.md).

## Archive

| Version | File | Superseded by |
|---------|------|---------------|
| V58 | `handoffs/archive/LUDOSPRING_V58_DEEP_DEBT_ZERO_WARNINGS_HANDOFF_MAY09_2026.md` | V59 Post-Interstadial |
| V57 | `handoffs/archive/LUDOSPRING_V57_EUKARYOTIC_EVOLUTION_HANDOFF_MAY09_2026.md` | V59 Post-Interstadial |
| V56 | `handoffs/archive/LUDOSPRING_V56_PHASE60_PARITY_HANDOFF_MAY08_2026.md` | V59 Post-Interstadial |
| V55 | `handoffs/archive/LUDOSPRING_V55_DEEP_DEBT_RESOLUTION_HANDOFF_APR27_2026.md` | V58 Deep Debt Zero Warnings |
| V53 | `handoffs/archive/LUDOSPRING_V53_COMPOSITION_EVOLUTION_HANDOFF_APR25_2026.md` | V55 Deep Debt Resolution |
| V52 | `handoffs/archive/LUDOSPRING_V52_COMPOSITION_LOOP_HANDOFF_APR25_2026.md` | V55 Deep Debt Resolution |
| V49 | `handoffs/archive/LUDOSPRING_V49_DEEP_DEBT_RESOLUTION_HANDOFF_APR25_2026.md` | V55 Deep Debt Resolution |
| V47 | `handoffs/archive/LUDOSPRING_V47_V0917_GUIDESTONE_V120_HANDOFF_APR20_2026.md` | V55 Deep Debt Resolution |
| V46 | `handoffs/archive/LUDOSPRING_V46_THREE_TIER_NUCLEUS_HANDOFF_APR20_2026.md` | V55 Deep Debt Resolution |
| V39 | Central: `infra/wateringHole/handoffs/LUDOSPRING_V39_NUCLEUS_COMPOSITION_PARITY_HANDOFF_APR10_2026.md` | V42 Composition Evolution |
| V45 | `handoffs/archive/LUDOSPRING_V45_GUIDESTONE_HANDOFF_APR18_2026.md` | V46 Three-Tier NUCLEUS |
| V44 | `handoffs/archive/LUDOSPRING_V44_PRIMAL_PROOF_HANDOFF_APR17_2026.md` | V45 guideStone |
| V43 | `handoffs/archive/LUDOSPRING_V43_THREE_LAYER_VALIDATION_HANDOFF_APR17_2026.md` | V44 Primal Proof |
| V42 | Central: `infra/wateringHole/handoffs/LUDOSPRING_V42_COMPOSITION_EVOLUTION_HANDOFF_APR11_2026.md` | V43 Three-Layer Validation |
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
