# ludoSpring V29 — Deep Evolution & Cross-Ecosystem Absorption

**Date**: March 23, 2026
**From**: ludoSpring
**To**: barraCuda, toadStool, coralReef, biomeOS, petalTongue, Squirrel, all springs
**Status**: Released
**Previous**: [V28 Capability-Based Discovery](archive/LUDOSPRING_V28_TOADSTOOL_BARRACUDA_DEEP_EVOLUTION_HANDOFF_MAR18_2026.md)

---

## Summary

V29 resolves the barraCuda upstream feature-gating bug, wires `TensorSession` for
GPU compute, expands Python parity coverage to 42 tests, refactors `metalForge/forge`
into 4 domain modules, adds 7 game-history revalidation experiments (exp076–082),
establishes `cargo-llvm-cov` at 80% floor, and regenerates all baselines with
`content_sha256` provenance. The shader absorption handoff for Perlin 2D and DDA
raycast is published separately.

---

## Changes

### Added

- **`GpuContext` module** — `game::engine::gpu_context` behind `#[cfg(feature = "gpu")]`.
  Wraps `Arc<WgpuDevice>`, exposes `tensor_session()` → `TensorSession::with_device()`.
  Mirrors neuralSpring's `Dispatcher::tensor_session()` pattern.
- **Shader absorption handoff** — `LUDOSPRING_V29_PERLIN_DDA_SHADER_ABSORPTION_HANDOFF_MAR23_2026.md`
  documents 2 validated WGSL shaders (Perlin 2D, DDA raycast) ready for barraCuda absorption,
  plus 9 additional exp030 shaders with tier classification.
- **`cargo-llvm-cov` gating** — `make coverage` at 80% floor, `cargo coverage` / `cargo coverage-html`
  aliases in `.cargo/config.toml`. Bins excluded from measurement.
- **7 new experiments** — exp076 (Pong), exp077 (Spacewar), exp078 (Tetris), exp079 (Civilization),
  exp080 (Diablo Loot), exp081 (Procedural Generation), exp082 (Symphony Pipeline).
- **`PERLIN_SAFE_BOUND`** and **`BSP_AREA_CONSERVATION_TOL`** in `tolerances::validation`.
- **Python parity expansion** — 25 → 42 tests: fun_keys scores, Doom Fitts scenarios, Hick choice
  sweep, flow states, engagement snapshots, GOMS extended operators, BSP offset area.

### Changed

- **barraCuda `default-features = false`** — upstream feature-gating fixed: `special::plasma_dispersion`
  and `spectral::stats::analyze_weight_matrix` now properly gated behind `#[cfg(feature = "gpu")]`.
  ludoSpring pulls CPU-only math by default; GPU opt-in via `features = ["gpu"]`.
- **`barcuda_math` re-exports** — expanded from 8 to 22 CPU primitives (all activations,
  RNG, stats, correlation).
- **`metalForge/forge` refactored** — monolithic `lib.rs` (911 LOC) → 4 domain modules:
  `substrate.rs`, `workload.rs`, `routing.rs`, `pipeline.rs`. 19 tests, backward-compatible
  via re-exports.
- **Baselines regenerated** — `combined_baselines.json` includes `content_sha256`, `python_version`,
  `git_commit`, `dependencies` in `_provenance`.
- **Test socket paths** — hardcoded `/tmp/*.sock` in `ipc/neural_bridge.rs` and `ipc/discovery.rs`
  replaced with `temp_dir()` + process-unique slugs.

---

## Current State

| Metric | Value |
|--------|-------|
| Experiments | 82 |
| Tests (barracuda) | 402 (326 lib + 8 IPC + 12 validation + 42 parity + 12 proptest + 2 doc) |
| Tests (forge) | 19 |
| Python baselines | 7 scripts, 42 parity tests, all pass |
| Baseline SHA-256 | `8c404eab5fb6eeb2679b75e65f58590c0ab43c9217a875641fcf7cf737e181fc` |
| Clippy | 0 warnings (pedantic + nursery) |
| Format | Clean |
| Docs | 0 warnings |
| Unsafe | 0 (`forbid(unsafe_code)`) |
| `#[allow()]` | 0 (all `#[expect(reason)]`) |
| Line coverage | 80.2% (library, bins excluded, 80% floor enforced) |
| barraCuda default-features | false (CPU-only default) |
| TODO/FIXME/HACK | 0 |
| Files > 1000 LOC | 0 |
| SPDX headers | All `.rs` + all `Cargo.toml` |

---

## For barraCuda

1. **Feature-gating fix applied upstream** — `special/mod.rs` (plasma_dispersion gated),
   `spectral/stats.rs` (WeightMatrixAnalysis/analyze_weight_matrix gated),
   `spectral/mod.rs` (re-exports split). Enables `--no-default-features` compilation.
   Please review and merge.

2. **Shader absorption ready** — Perlin 2D and DDA raycast shaders validated against CPU
   implementations. See separate handoff document for full details, suggested module paths,
   and tolerance specifications.

3. **`barcuda_math` consumption** — ludoSpring now uses 22 CPU primitives from barraCuda
   (activations, stats, RNG, correlation). Any API changes to these should be coordinated.

4. **`TensorSession` wired** — `GpuContext::tensor_session()` mirrors neuralSpring's pattern.
   Ready for engagement batch pipeline once barraCuda absorbs the shaders.

## For toadStool

1. **Shader names registered** — `GpuOp` enum registers 5 shader names for `compute.submit`:
   `fog_of_war`, `tile_lighting`, `pathfind_wavefront`, `perlin_2d`, `dda_raycast`.
   After barraCuda absorption, these should be resolvable in toadStool's shader catalog.

2. **`GpuAvailability` probe** — ludoSpring probes for in-process GPU and toadStool IPC
   independently. When both are available, the engine can choose the optimal dispatch path.

3. **metalForge routing** — 4 domain modules (substrate, workload, routing, pipeline) with
   19 tests. The routing logic uses capability-based substrate scoring that matches
   toadStool's hardware discovery model.

## For coralReef

1. **5 WGSL compute shaders** ready for sovereign compilation via `shader.compile.wgsl`.
   All use standard WGSL compute — no extensions, no intrinsics. Workgroup size 64.

2. **DDA raycast** and **Perlin 2D** are the highest-priority candidates for AOT compilation
   (both are hot paths in the game engine).

## For petalTongue

1. **3 dashboard binaries** push game science data via JSON-RPC over Unix socket.
   All use capability-based discovery (no hardcoded primal names).
   Output paths configurable via `LUDOSPRING_OUTPUT_DIR`.

2. **13 telemetry event types** — the portable game telemetry protocol is stable and
   works with external game adapters (Veloren, Fish Folk, A/B Street).

## For biomeOS

1. **ludoSpring is a deployable niche** — UniBin binary, deploy graph, niche YAML,
   26 capabilities registered via Neural API.

2. **Provenance Trio wired at graph level** — all nodes `fallback = "skip"`.

3. **No chimeric deps** — infrastructure primals (biomeOS, songbird, petalTongue) are
   NOT Cargo dependencies. Communication is JSON-RPC 2.0 over Unix sockets.

## For All Springs

1. **Pattern: `default-features = false` on barraCuda** — if your spring imports barraCuda
   and doesn't need GPU at compile time, add `default-features = false` to your `Cargo.toml`.
   This is now safe after the upstream feature-gating fix.

2. **Pattern: `GpuContext` for shared device** — if you need both custom WGSL shaders and
   `TensorSession`, wrap `WgpuDevice` in a shared context to avoid duplicate adapter init.
   See `barracuda/src/game/engine/gpu_context.rs` for the reference implementation.

3. **Pattern: `cargo-llvm-cov` gating** — add `--fail-under-lines` to your coverage command
   to prevent regression. Use `--ignore-filename-regex bin/` to exclude binary entry points
   that have 0% coverage by design with `--lib --tests`.

4. **Baseline provenance with `content_sha256`** — `combined_baselines.json` now includes
   a SHA-256 hash of the combined output, enabling drift detection at the content level.

---

## Breaking Changes

None. All changes are backward-compatible.

---

## What's Next

- barraCuda absorbs Perlin 2D and DDA shaders
- `TensorSession` engagement batch pipeline (Tier A — after shader absorption)
- Coverage push toward 90% (IPC modules, game session, audio)
- exp076–082 validation check counts to baseCamp README
- coralReef AOT compilation of validated WGSL shaders
