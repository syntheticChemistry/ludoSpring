# ludoSpring V14 → barraCuda/toadStool Deep Audit Handoff — March 14, 2026

**From:** ludoSpring V14
**To:** barraCuda team, toadStool team, coralReef team
**License:** AGPL-3.0-or-later
**Date:** March 14, 2026
**Supersedes:** V13 Cross-Spring Provenance Handoff (March 13, 2026)

---

## Executive Summary

ludoSpring V14: **66 experiments, 1349 checks, 212 tests** — all green.

V14 is a **deep audit and quality evolution** release. No new experiments. All changes
strengthen the existing codebase for handoff to barraCuda/toadStool:

- **+74 tests** (138 → 212): targeted coverage for noise, lsystem, raycaster, ruleset, telemetry modules
- **All 22 library modules ≥ 90% coverage** via `llvm-cov` (floor: 90.8% `interaction::flow`)
- **0 Clippy warnings** workspace-wide (`-W pedantic -W nursery`)
- **0 `cargo doc` warnings**, 0 `cargo fmt` diffs
- **0 files > 1000 LOC** (exp061 refactored: 1245 → 111 + 496 + 663 across 3 modules)
- **SPDX `AGPL-3.0-or-later` headers** on all `.rs` and `Cargo.toml` files
- **`#![forbid(unsafe_code)]`** on every crate root
- **Centralized tolerances** with citations in `barracuda/src/tolerances/mod.rs`
- **Python baselines** embed runtime provenance (git commit, Python version, date)
- **Zero-copy NDJSON streaming** in telemetry parser (buffer reuse, no per-line allocation)

---

## Part 1: barraCuda Primitive Consumption

### 1.1 Stable Primitives (unchanged)

| Primitive | Consumer | Status |
|-----------|---------|--------|
| `activations::sigmoid` | `interaction::flow::DifficultyCurve` | Stable |
| `stats::dot` | `metrics::engagement::compute_engagement` | Stable |
| `rng::lcg_step` | `procedural::bsp::generate_bsp` | Stable |
| `rng::state_to_f64` | `procedural::bsp::generate_bsp` | Stable |

### 1.2 Validation Harness

| Primitive | Consumer | Status |
|-----------|---------|--------|
| `validation::ValidationResult` | All 66 experiments | Stable — 1349 check calls |
| `tolerances::*` | Tests + experiments | 6 named constants with source citations |

### 1.3 Tolerance Constants (new)

V14 centralized all ad-hoc tolerances into `barracuda/src/tolerances/mod.rs`:

| Constant | Value | Source |
|----------|-------|--------|
| `ANALYTICAL_TOL` | `1e-10` | IEEE 754 f64 error accumulation for 4-5 chained ops |
| `RAYCASTER_DISTANCE_TOL` | `0.01` | DDA grid-stepping geometry: float step accumulation |
| `NOISE_COHERENCE_TOL` | `0.15` | Perlin/fBm coherence floor from gradient table resolution |
| `UI_DATA_INK_TOL` | `0.01` | Tufte data-ink ratio — 1% threshold for UI metric comparison |
| `UI_COVERAGE_TOL` | `0.05` | Tufte coverage measurement — 5% threshold for UI evaluation |
| `RAYCASTER_HIT_RATE_TOL` | `0.05` | DDA ray-wall intersection — 5% tolerance for hit-rate checks |

**barraCuda action**: These constants live in ludoSpring's barracuda crate. If
barraCuda standalone absorbs ludoSpring math, bring the tolerance definitions with
provenance comments intact.

---

## Part 2: GPU Shader Promotion Map (unchanged from V13)

### Tier A — Ready Now (8 modules)

| Module | WGSL Shader | Blocking |
|--------|-------------|----------|
| `procedural::noise::perlin_2d` | `perlin_2d.wgsl` | Nothing — pure math |
| `procedural::noise::fbm_2d` | `fbm_2d.wgsl` | Nothing |
| `game::raycaster::cast_rays` | `dda_raycast.wgsl` | Nothing — embarrassingly parallel |
| `metrics::engagement::compute_engagement` | `engagement_batch.wgsl` | Nothing — dot product |
| `metrics::fun_keys::classify_fun` | `fun_classify.wgsl` | Nothing — weighted sum |
| `interaction::flow::evaluate_flow` | `flow_eval.wgsl` | Nothing — comparisons |
| `interaction::input_laws::*` | `interaction_laws.wgsl` | Nothing — log2 only |
| `interaction::goms::task_time` | `goms_batch.wgsl` | Nothing — sum of ops |

### Tier B — Adapt (2 modules)

| Module | Challenge | Approach |
|--------|-----------|----------|
| `procedural::wfc` | Barrier sync for constraint propagation | Workgroup-level sync |
| `procedural::bsp` | Recursive → iterative | Stack elimination |

### Tier A Candidates (deferred)

| Module | Why | Priority |
|--------|-----|----------|
| `GenericFraudDetector` (exp065) | Batch graph adjacency checks | P3 (at scale) |
| `compute_distribution` (exp066) | Batch weighted-sum with decay | P3 (at scale) |

---

## Part 3: What V14 Deep Audit Evolved

### 3.1 Coverage (138 → 212 tests)

| Module | Before | After | Key additions |
|--------|--------|-------|---------------|
| `procedural::noise` | 89.9% | 98.2% | `perlin_3d` edge cases, `grad2`/`grad3` branches, fBm octaves |
| `procedural::lsystem` | 89.9% | 98.3% | Dragon curve, turtle push/pop, half-turn, empty stack |
| `game::raycaster` | ~85% | ~95% | Edge-case ray angles, map boundary, hit-rate |
| `game::ruleset` | 87.9% | 100% | FATE/Cairn presets, `DegreeOfSuccess`, `DiceSystem::D6Pool`/`D100` |
| `telemetry::mapper` | ~80% | ~95% | Float comparison, format edge cases |
| `telemetry::report` | ~80% | ~95% | Float comparison fixes |

### 3.2 Idiomatic Rust Evolution

| Pattern | Before | After |
|---------|--------|-------|
| Float comparison in tests | `assert_eq!(x, 0.0)` | `assert!(x.abs() < f64::EPSILON)` |
| Clippy nursery | Not checked | 0 warnings workspace-wide |
| `mul_add()` | Scattered | Used where Clippy recommends |
| NDJSON parsing | `line.trim().to_string()` per line | Buffer reuse, zero per-line allocation |
| `map().unwrap_or()` | Several instances | `map_or()` throughout |
| Format strings | Some `format!("{}", x)` | Direct interpolation `format!("{x}")` |

### 3.3 exp061 Refactoring (1245 → 3 modules)

`experiments/exp061_fermenting/src/main.rs` was 1245 lines. Refactored to:

| File | Lines | Responsibility |
|------|-------|----------------|
| `main.rs` | 111 | Entry point, module declarations, CLI dispatch |
| `validate_objects.rs` | 496 | Cosmetic schema, certificate lifecycle, object memory, ownership |
| `validate_systems.rs` | 663 | Trading protocol, trio integration, full scenario, composable deployment |

All 89 validation checks preserved with zero behavioral change.

### 3.4 Provenance Enhancement

- Python baselines (`baselines/python/run_all_baselines.py`) now embed:
  - `git_commit` (HEAD at generation time)
  - `python_version` and `python_implementation`
  - `date` (UTC ISO-8601)
  - `command` and `dependencies`
- Rust parity tests (`barracuda/tests/python_parity.rs`) reference centralized tolerances
- Determinism tests (`barracuda/tests/determinism.rs`) document bitwise-identity methodology

### 3.5 License Compliance

- All 15 experiment `Cargo.toml` files that were missing SPDX headers now have them
- `barracuda/src/bin/ludospring.rs` corrected from `AGPL-3.0-only` to `AGPL-3.0-or-later`
- 4 experiment binaries (exp026-029) gained SPDX headers in source

---

## Part 4: External Dependency Audit

All external dependencies are pure Rust (no C/C++ FFI, no `unsafe`):

| Dependency | Usage | Sovereignty |
|-----------|-------|-------------|
| `serde` + `serde_json` | Serialization (telemetry, IPC, baselines) | Pure Rust, Apache-2.0/MIT |
| `uuid` | Event and session IDs | Pure Rust, Apache-2.0/MIT |
| `blake3` | Content-addressed hashing (DAG vertices) | Pure Rust, Apache-2.0 |
| `wgpu` | GPU dispatch (exp030 parity validation) | Pure Rust WebGPU, Apache-2.0/MIT |
| `ratatui` + `crossterm` | Terminal rendering (exp024, exp025) | Pure Rust, MIT |
| `ureq` | HTTP (NCBI E-utilities in exp041/043) | Pure Rust, Apache-2.0/MIT |
| `fastnoise-lite` | Control group benchmark comparison only | Pure Rust, MIT |
| `noise` | Control group benchmark comparison only | Pure Rust, Apache-2.0 |
| `bracket-pathfinding` | Control group (exp038 external roguelike) | Pure Rust, MIT |
| `regex` | Telemetry log parsing | Pure Rust, Apache-2.0/MIT |

**No `unsafe` code in the dependency tree for ludoSpring's core path.** `blake3` and
`wgpu` use platform-optimized `unsafe` internally but are well-audited.

---

## Part 5: Action Items for barraCuda Team

### Absorption Candidates (ludoSpring → barraCuda)

| # | Module | Why absorb | Priority |
|---|--------|------------|----------|
| 1 | `procedural::noise` (Perlin 2D/3D, fBm) | Cross-spring (wetSpring Anderson QS), 98% coverage, GPU-ready | P1 |
| 2 | `metrics::engagement` | Cross-spring (science viz quality), pure dot product | P1 |
| 3 | `interaction::input_laws` (Fitts, Hick, Steering) | Cross-spring (healthSpring UI), pure math | P2 |
| 4 | `tolerances/mod.rs` constants | Provenance-sourced, used across test suite | P2 |
| 5 | `GenericFraudDetector` (exp065 pattern) | 3-domain validated, graph analysis | P3 |

### What NOT to Absorb

- Telemetry (NDJSON protocol, adapters) — ludoSpring-specific domain logic
- Game state (raycaster, voxel, session) — game-engine-specific
- Playable prototypes (exp024/025) — application code, not primitives
- Experiment validation harnesses — remain in experiment crates

---

## Part 6: Action Items for toadStool Team

| # | Action | Priority | Source |
|---|--------|----------|--------|
| 1 | GPU dispatch for 8 Tier A modules | P1 | exp030 parity proven on CPU/GPU |
| 2 | coralReef sovereign compile for WGSL shaders | P2 | Shader stubs in promotion map |
| 3 | BearDog live IPC for exp064 signing | P2 | Wire format validated, model sigs ready to swap |
| 4 | Batch fraud analysis shader (if absorbed) | P3 | exp065 graph patterns at scale |

---

## Part 7: Quality Gates

| Check | Result |
|-------|--------|
| `cargo fmt --check` | 0 diffs |
| `cargo clippy -W pedantic -W nursery` | 0 warnings (workspace-wide) |
| `cargo test --workspace` | 212 tests, 0 failures |
| `cargo doc --workspace --no-deps` | 0 warnings |
| 67 validation binaries | 1349 checks, 0 failures |
| `llvm-cov` (library) | All 22 modules ≥ 90% |
| `#![forbid(unsafe_code)]` | All crate roots |
| SPDX headers | All `.rs` + `Cargo.toml` |
| Files > 1000 LOC | 0 |
| TODO/FIXME/HACK | 0 |

---

## Part 8: Files of Interest

| Path | What |
|------|------|
| `barracuda/src/tolerances/mod.rs` | Centralized tolerance constants with provenance |
| `barracuda/tests/python_parity.rs` | Python baseline parity (22 tests, centralized tolerances) |
| `barracuda/tests/determinism.rs` | Bitwise-identity determinism tests (8 tests) |
| `barracuda/src/telemetry/mod.rs` | Zero-copy NDJSON streaming parser |
| `baselines/python/run_all_baselines.py` | Python baselines with embedded provenance |
| `experiments/exp061_fermenting/src/validate_objects.rs` | Refactored from monolith |
| `experiments/exp061_fermenting/src/validate_systems.rs` | Refactored from monolith |
| `experiments/exp065_cross_domain_fraud/src/unified.rs` | Domain-agnostic fraud detector |
