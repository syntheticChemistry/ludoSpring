# Changelog

All notable changes to ludoSpring are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project does not use SemVer — versions are session-sequential (V1–V28).

## [V28] — 2026-03-18

### Changed

- Evolved exp042 (Tower Atomic) from hardcoded primal names (`"beardog"`, `"songbird"`) to capability-based discovery via `discovery::discover_primals()` — discovers by `crypto.hash` and `system.ping` capabilities at runtime
- Parameterized `coordination::viz_register()` in exp054 to accept `primal_id` argument, removing hardcoded `"petaltongue"` name
- Evolved 3 dashboard binaries from hardcoded `sandbox/` paths to `LUDOSPRING_OUTPUT_DIR` environment variable with fallback default
- Fixed `prop_assert!` format string conflict with `matches!` struct patterns in `ipc/envelope.rs` proptest
- Fixed IPC integration test isolation — unique socket paths per test via atomic counter (was shared PID-based path causing connection resets)

### Fixed

- `missing_errors_doc` warnings on `DispatchOutcome::into_result` and `extract_rpc_result` in `ipc/envelope.rs`
- IPC integration test `evaluate_flow` — expected `"flow_state"` / `"Flow"`, actual serialized field is `"state"` / `"flow"`
- IPC integration test `capability_list` — expected `"capabilities"` array, actual response uses `"total_capabilities"` + `"domains"`

### Added

- V28 handoff: toadStool/barraCuda deep evolution handoff with capability-based discovery patterns
- `ipc` feature dependency on exp042 `Cargo.toml` for runtime primal discovery

## [V27] — 2026-03-18

### Changed

- Migrated all 9 `#[allow()]` instances to `#[expect(reason = "...")]` across 5 experiment files (exp034, exp050, exp051, exp055, exp061)
- Migrated 4 `.expect()` calls to `OrExit` pattern in exp045, exp052, exp053, exp054 (vertex id computation, skill lookup)
- Migrated exp058 (conjugant) from hand-rolled validation to `ValidationHarness` + `BaselineProvenance`
- Centralized lint configuration: 16 experiment `Cargo.toml` files migrated from local `[lints.clippy]` to `[lints] workspace = true`
- Added `must_use_candidate` and `return_self_not_must_use` to workspace lint overrides
- Refactored `exp062_field_sample_provenance/src/sample.rs`: extracted monolithic `detect_sample_fraud` (180 lines, `too_many_lines` suppression) into 6 focused per-rule functions
- Replaced brittle string-parsing fraud detection (MislabeledSpecimen) with structural `collect_sample_types` tracking on `SampleSystem`
- Documented exp030 validation exemption (legacy `ValidationResult`, pending harness per-section skip support)

### Added

- `CHANGELOG.md` (this file) per SPRING_PRIMAL_PRESENTATION_STANDARD
- `NOISE_MEAN_TOL` to `baselines/python/tolerances.py` (was missing from Python mirror)
- `SampleSystem::collected_sample_type()` accessor for structural fraud detection

### Fixed

- V26 handoff claim "zero legacy ValidationResult usage" — now accurately documented as one exemption (exp030)

## [V26] — 2026-03-18

### Changed

- Migrated 71/75 experiments to `ValidationHarness` + `BaselineProvenance`
- Centralized 14 GPU tolerances in `tolerances::gpu` module
- Tightened `missing_errors_doc` and `missing_panics_doc` lints to warn
- Deduplicated `perlin_2d.wgsl` and `dda_raycast.wgsl` into `barracuda/shaders/game/validated/`
- Unified shader audit: 7 upstream absorption candidates, 2 domain-specific retained

### Added

- `ValidationSink` trait (pluggable output: `StderrSink`, `BufferSink`)
- `check_abs_or_rel` method on `ValidationHarness`
- V26 handoffs: full harness migration + toadStool/barraCuda absorption

## [V24] — 2026-03-17

### Added

- Leverage guide handoff for ecosystem coordination

## [V23] — 2026-03-16

### Changed

- Cross-ecosystem deep debt resolution
- toadStool/barraCuda absorption coordination

## [V22] — 2026-03-16

### Changed

- Ecosystem absorption: aligned with wetSpring V119+ patterns

## [V21] — 2026-03-16

### Changed

- Deep debt evolution for barraCuda/toadStool integration
- Workspace lint consolidation (`expect_used = "deny"`, `unwrap_used = "deny"`)

## [V20] — 2026-03-16

### Changed

- Deep primal integration: barraCuda/toadStool wiring

## [V19] — 2026-03-16

### Changed

- barraCuda/toadStool deep debt resolution

## [V18] — 2026-03-15

### Added

- Niche self-knowledge: `niche.rs` with 26 capabilities, socket resolution
- Neural Bridge IPC client
- biomeOS domain registration/deregistration

### Changed

- barraCuda/toadStool niche absorption

## [V17] — 2026-03-15

### Changed

- Deep evolution: barraCuda/toadStool coordination patterns

## [V16] — 2026-03-15

### Added

- Niche deployment: `ludospring_deploy.toml`, `ludospring-game.yaml`
- UniBin architecture: `ludospring server`, `--help`, `--version`

### Changed

- barraCuda/toadStool absorption

## [V15] — 2026-03-14

### Added

- GPU dispatch buildout: exp030 CPU-vs-GPU parity validation
- 11 WGSL shaders for game-domain compute
- `GpuOp` catalog (FogOfWar, TileLighting, PathfindStep, PerlinTerrain, BatchRaycast)

## [V14] — 2026-03-14

### Changed

- Deep audit: barraCuda/toadStool integration review

## [V13] — 2026-03-13

### Added

- Cross-spring provenance: rhizoCrypt, loamSpine, sweetGrass integration
- Provenance trio IPC clients
