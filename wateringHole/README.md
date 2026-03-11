# ludoSpring wateringHole

**Date:** March 11, 2026
**Purpose:** Spring-local handoff documents to `barraCuda`/`toadStool` and cross-spring provenance records.

---

## Active Handoffs

| Version | File | Date | Scope |
|---------|------|------|-------|
| **V2** | `handoffs/LUDOSPRING_V2_FULL_VALIDATION_HANDOFF_MAR11_2026.md` | Mar 11 | Full validation: 22 experiments (183 checks), 119 tests, all foundational research implemented. GPU shader promotion map, barraCuda primitive consumption, Python parity. |
| **V1** | `handoffs/LUDOSPRING_V1_SCAFFOLD_HANDOFF_MAR_2026.md` | Mar 10 | Initial scaffold: 4 modules, 4 experiments, 45 tests, biomeOS integration, game engine niche architecture. |

## Cross-Spring Documents

| File | Purpose |
|------|---------|
| `handoffs/LUDOSPRING_BARRACUDA_TOADSTOOL_EVOLUTION_HANDOFF_MAR11_2026.md` | GPU evolution targets: 8 Tier A modules ready for WGSL shader promotion, primitive consumption inventory, absorption requests |

## barraCuda Primitive Consumption

ludoSpring consumes these `barraCuda` primitives (validated via Python parity):

| Primitive | Module | Used by |
|-----------|--------|---------|
| `activations::sigmoid` | `interaction::flow` | `DifficultyCurve` sigmoid replacement |
| `stats::dot` | `metrics::engagement` | Weighted engagement composite |
| `rng::lcg_step` | `procedural::bsp` | Deterministic BSP generation |
| `rng::state_to_f64` | `procedural::bsp` | Split ratio from LCG state |

### Absorption Opportunities

ludoSpring has validated pure-math modules ready for upstream absorption:

| Module | Lines | What barraCuda gets |
|--------|-------|--------------------|
| `procedural::noise` | ~200 | Perlin 2D/3D + fBm (GPU-ready) |
| `procedural::wfc` | ~250 | Wave Function Collapse (GPU-parallel) |
| `procedural::lsystem` | ~200 | L-system string rewriting |
| `procedural::bsp` | ~220 | BSP spatial partitioning |

## Convention

Following hotSpring/wetSpring naming pattern:
`LUDOSPRING_{VERSION}_{TOPIC}_HANDOFF_{DATE}.md`

Handoffs flow: ludoSpring → barraCuda (math) and ludoSpring → toadStool (hardware).
No reverse dependencies.
