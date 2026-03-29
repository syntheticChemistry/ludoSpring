# ludoSpring V32 — Comprehensive Audit + Deep Debt Evolution

**Date:** March 29, 2026
**From:** ludoSpring V32
**To:** barraCuda, toadStool, coralReef, petalTongue, biomeOS, primalSpring, Squirrel, esotericWebb, wetSpring, healthSpring, BearDog, all springs
**Previous:** `archive/LUDOSPRING_V31_DEEP_DEBT_ESOTERICWEBB_ALIGNMENT_HANDOFF_MAR28_2026.md`
**Status:** Released — full audit + systematic remediation across 110 files

---

## Summary

V32 is a comprehensive codebase audit followed by systematic deep debt
remediation. The audit covered completion status, code quality, validation
fidelity, barraCuda dependency health, GPU evolution readiness, test coverage,
ecosystem standards, and primal coordination. The remediation touched 110 files
with +1010 insertions / −1097 deletions.

Key outcomes: all provenance hashes aligned, all tolerances centralized to named
constants, all manual `eprintln!("FATAL:...")/exit(1)` migrated to `OrExit`,
exp030 rewritten to `ValidationHarness`, CI hardened, 3 deprecated binaries
removed, `specs/BARRACUDA_REQUIREMENTS.md` created.

---

## Part 1: For barraCuda Team

### Tolerance Pattern for Upstream Adoption

ludoSpring now has a complete tolerance architecture that should inform barraCuda's
own constant organization:

| Constant | Value | Module | Justification |
|----------|-------|--------|---------------|
| `ANALYTICAL_TOL` | `1e-10` | `tolerances::validation` | Python→Rust parity (deterministic algorithms) |
| `STRICT_ANALYTICAL_TOL` | `1e-15` | `tolerances::validation` | Bit-exact transcendental parity (sigmoid, log2) |
| `NUMERICAL_FLOOR` | `1e-9` | `tolerances::validation` | Division-by-zero prevention in metric normalization |
| `DDA_ADJUSTMENT_EPSILON` | `1e-6` | `tolerances::validation` | Near-zero threshold for DDA recommendation engine |
| `SPAN_FLOOR` | `1e-6` | `tolerances::validation` | Minimum span denominator for flow score normalization |
| `TRUST_EQUALITY_TOL` | `1e-12` | `tolerances::game` | NPC trust comparison in plane transitions |
| `GAME_STATE_TOL` | `0.01` | `tolerances::game` | Game state floating-point comparisons |

**Action:** Consider adopting a similar `tolerances/` module tree in barraCuda
for downstream consumers to import, rather than each spring reinventing constants.

### Upstream Evolution Requests (new spec)

`specs/BARRACUDA_REQUIREMENTS.md` documents:

1. **Perlin 2D/3D absorption** — ludoSpring's `procedural::noise` (~200 LOC) is
   Tier A GPU-ready. barraCuda should absorb this as `barraCuda::procedural::perlin`.
2. **DDA raycaster** — embarrassingly parallel per-column DDA. Tier A.
3. **f32 tensor path** — `TensorSession` currently forces f64. Game compute needs
   f32 for GPU throughput.
4. **LCG alignment** — ludoSpring's `lcg_step` uses `state * 6364136223846793005 + 1442695040888963407`.
   Ensure barraCuda's LCG matches exactly for deterministic cross-crate parity.

### Consumed Primitives (unchanged)

| Primitive | Consumer | Status |
|-----------|---------|--------|
| `activations::sigmoid` | `interaction::flow` | Active |
| `stats::dot` | `metrics::engagement` | Active |
| `rng::lcg_step` | `procedural::bsp` | Active |
| `rng::state_to_f64` | `procedural::bsp` | Active |

### TensorSession Status

`TensorSession` remains a future-only evolution target. No product code exercises
it. Game GPU ops route through custom WGSL via toadStool or exp030's direct wgpu.
`GpuContext::tensor_session()` is wired and tested behind `#[cfg(feature = "gpu")]`.

---

## Part 2: For toadStool Team

### GPU Dispatch Surface (unchanged from V31)

4 `game.gpu.*` methods delegate to toadStool `compute.dispatch.submit`:

| Method | WGSL Shader | Uniforms | CPU Fallback |
|--------|------------|----------|-------------|
| `game.gpu.fog_of_war` | `fog_of_war.wgsl` | grid, viewer, sight | Bresenham LOS |
| `game.gpu.tile_lighting` | `tile_lighting.wgsl` | grid, lights | Per-cell distance |
| `game.gpu.pathfind` | `pathfind_wavefront.wgsl` | grid, start, goal | BFS |
| `game.gpu.perlin_terrain` | `perlin_terrain.wgsl` | width, height, scale, seed | CPU Perlin |
| `game.gpu.batch_raycast` | `dda_raycast.wgsl` | map, player, rays | CPU DDA |

### Capability-Based Degradation

V32 made degradation messages primal-agnostic:
- Before: `"toadStool unavailable — use CPU implementation"`
- After: `"compute dispatch unavailable — CPU fallback active"`

This means toadStool could be replaced by any compute dispatch primal without
ludoSpring code changes.

### Shader Promotion Tiers

See `specs/BARRACUDA_REQUIREMENTS.md` for the full tier map:

| Tier | Modules | Blocking |
|------|---------|----------|
| A (direct rewire) | noise, raycaster, engagement, fun_keys, flow, input_laws, goms | Nothing — pure math |
| B (adapt) | wfc, bsp, lsystem | Barrier sync, stack elimination, variable-length output |
| C (new shader) | RPGPT trust dynamics | Domain-specific state management |

---

## Part 3: For coralReef Team

### Shader Inventory

11 WGSL shaders in `exp030/shaders/` remain candidates for sovereign compilation:
- `sigmoid.wgsl`, `relu.wgsl`, `abs.wgsl`, `scale.wgsl`, `softmax.wgsl` (Tier A)
- `perlin_2d.wgsl`, `fbm.wgsl`, `engagement_batch.wgsl`, `dda_raycast.wgsl` (Tier A)
- `fog_of_war.wgsl`, `tile_lighting.wgsl` (game-domain)

All use f64 canonical precision. Only transcendental is log2.

---

## Part 4: For petalTongue Team

### Visualization Integration (unchanged)

`VisualizationPushClient::discover()` uses capability-based discovery
(`visualization.render`). Socket path now uses `niche::ECOSYSTEM_SOCKET_DIR`
instead of hardcoded `"biomeos"`.

15 `GameChannelType` channels, 3 UniBin subcommands (`dashboard`, `live-session`,
`tufte-dashboard`).

---

## Part 5: For All Springs — Patterns Worth Adopting

### 1. OrExit Trait for Validation Binaries

27 experiment files now use `.or_exit("context")` instead of
`eprintln!("FATAL: ..."); std::process::exit(1)`. This is the groundSpring V112 /
wetSpring V123 pattern. If your spring still has manual FATAL patterns, adopt
`OrExit`.

### 2. Named Tolerance Constants

Zero inline numeric literals in test assertions. Every tolerance is a named
`pub const` with a justification comment. Springs should centralize tolerances
in a `tolerances/` module tree.

### 3. Provenance Hash Alignment

All 77 experiment `BaselineProvenance` blocks now point to the same baselines
commit (`4b683e3e`). When you regenerate baselines, update ALL provenance blocks
in a single pass — don't let them drift.

### 4. CI Baseline Drift Check

`.github/workflows/ci.yml` now includes a `baselines` job that runs
`check_drift.py`. Springs with Python baselines should add equivalent drift
detection.

### 5. deny.toml for cargo-deny 0.19

If your `deny.toml` has `unmaintained = "warn"`, change it to `"workspace"`.
cargo-deny 0.19 removed `"warn"` as a valid value.

### 6. Deprecated Binary Cleanup

V32 removed 3 deprecated binary stubs that were superseded by UniBin subcommands
in V30. If your spring has deprecated binaries with deprecation notices, consider
removing them after one version cycle.

---

## Part 6: For wetSpring + healthSpring — Cross-Spring Scaffolds

The experimental scaffolds remain unchanged:
- **exp062** (field sample provenance): 39/39 — wetSpring scaffold
- **exp063** (consent-gated medical access): 35/35 — healthSpring scaffold
- **exp064** (BearDog-signed chain): 39/39 — shared signing infrastructure
- **exp065** (cross-domain fraud unification): 74/74 — same `GenericFraudDetector`
- **exp066** (radiating attribution): 41/41 — sunCloud value distribution

Papers 21 (Sovereign Sample Provenance) and 22 (Zero-Knowledge Medical Provenance)
validation is complete. Write-up pending.

---

## Part 7: For esotericWebb

Response shapes aligned in V31 remain compatible. V32 changes are internal
(tolerance centralization, provenance alignment, error handling patterns) and
do not affect the IPC wire format.

---

## Metrics

| Metric | V31 | V32 |
|--------|-----|-----|
| Experiments | 82 | 82 |
| Barracuda tests | 675 | 674 |
| Python parity tests | 42 | 47 |
| IPC integration tests | 11 | 11 |
| Proptest | 19 | 19 |
| Coverage floor | 85% | 85% |
| `#[allow()]` | 0 | 0 |
| `unsafe` | 0 | 0 |
| TODO/FIXME | 0 | 0 |
| Manual FATAL patterns | ~27 | 0 |
| Hardcoded primal names | 0 | 0 |
| Deprecated binaries | 3 | 0 |
| Files changed (this session) | — | 110 |

---

## License

AGPL-3.0-or-later
