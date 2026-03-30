# Changelog

All notable changes to ludoSpring are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project does not use SemVer — versions are session-sequential (V1–V35).

## [V35.1] — 2026-03-30

### Revalidated — Primal Evolution Confirmed

Pulled and rebuilt evolved primals (barraCuda v0.3.11 local, biomeOS v2.79 local,
coralReef Iter70 plasmidBin, toadStool S168 plasmidBin). Reran all 5 composition
experiments. Total: **5/47 → 21/50 (42%)**.

| Experiment | Before | After | Key change |
|-----------|--------|-------|------------|
| exp084 | 0/12 | 4/15 | barraCuda binary alive, 4 math methods pass |
| exp085 | 2/8 | 7/8 | coralReef raw JSON-RPC fixed, compile+dispatch work |
| exp086 | 0/10 | 5/10 | tensor.create + matmul + read pass |
| exp087 | 1/7 | 3/7 | health.liveness pass, 40 graphs listed |
| exp088 | 2/10 | 2/10 | Socket naming mismatch blocks forwarding |

### Resolved (by primal teams)

- P0: barraCuda binary exists with 30 JSON-RPC methods
- P1: coralReef speaks raw newline-delimited JSON-RPC on UDS
- P2: biomeOS continuous executor wired with capability routing
- P2: biomeOS graph.save + nucleus/runtime tier separation
- P3: biomeOS health.liveness implemented

### Remaining gaps

- barraCuda tensor element-wise ops: registered but handler not dispatching
- Socket naming: biomeOS expects `{primal}-{family}.sock` vs actual `{primal}.sock`
- Composition graphs need `graph.save` API deployment
- Domain-level methods (fitts, hick, flow) not on barraCuda IPC

## [V35] — 2026-03-30

### Added — Primal Composition Gap Discovery (Track 26)

- **5 new composition experiments** (exp084–exp088): probe whether ludoSpring's
  validated game science can be replicated using ONLY primal composition
  (no ludoSpring binary in the loop)
- **4 deploy graphs** (`graphs/composition/*.toml`): math pipeline, shader
  dispatch chain, engagement pipeline, 60Hz continuous game loop — biomeOS-
  compatible `[graph]` header with `[[nodes]]` and `by_capability` routing
- **exp084 (barraCuda math IPC)**: 12 math methods probed over JSON-RPC
- **exp085 (shader dispatch chain)**: coralReef compile → toadStool dispatch E2E
- **exp086 (tensor composition)**: engagement scoring via tensor element-wise ops
- **exp087 (Neural API pipeline)**: graph.execute, pipeline, continuous orchestration
- **exp088 (60Hz game loop)**: full storytelling loop at continuous tick rate

### Fixed — Build Infrastructure

- Fixed barraCuda path dependency in `barracuda/Cargo.toml`, `metalForge/forge/Cargo.toml`,
  `exp030_cpu_gpu_parity/Cargo.toml` (incorrect relative paths)
- Fixed provenance trio path dependencies in 7 experiments (exp052–054, exp061–064):
  `phase2/` → `primals/` for rhizoCrypt, loamSpine, sweetGrass
- Graph TOMLs use `[graph]` header (not `[metadata]`) for biomeOS compatibility
- `#![expect(missing_docs)]` → `#![allow(missing_docs)]` in binary crates

### Validated — Live Primal Composition (Mar 30, 2026)

Ran all 5 experiments against live primals from plasmidBin (v2026.03.25):

| Experiment | Result | Key finding |
|-----------|--------|-------------|
| exp084 | 0/12 | barraCuda not in plasmidBin release |
| exp085 | 2/8 | coralReef HTTP-wrapped JSON-RPC vs raw UDS |
| exp086 | 0/10 | barraCuda missing — tensor API unreachable |
| exp087 | 1/7 | Neural API discovered; primals not registered |
| exp088 | 2/10 | Sub-ms capability.call latency confirmed |

### Documented — 5 Critical Gaps for Primal Evolution

- P0: barraCuda not in plasmidBin (blocks all math composition)
- P0: Running primals not auto-registered with Neural API
- P1: coralReef HTTP vs raw JSON-RPC on UDS
- P1: barraCuda math/stats/noise methods not on JSON-RPC
- P2: biomeOS continuous executor stub + nucleus graph bundling

### Handoff

- Central: `LUDOSPRING_V35_PRIMAL_COMPOSITION_GAP_DISCOVERY_HANDOFF_MAR30_2026.md`
  (wateringHole) — full gap analysis with per-team actions

## [V34] — 2026-03-29

### Added — Full NUCLEUS Nest Atomic Composition

- **exp083_neural_api_e2e v2**: Evolved from 10 to **13 checks** — full Nest Atomic validation:
  - BearDog: Blake3, SHA3-256, ChaCha20-Poly1305 roundtrip, Ed25519 sign
  - Songbird: `discovery.peers` via Neural API
  - ToadStool: `compute.dispatch.capabilities` via Neural API (RTX 4060)
  - NestGate: `storage.store`/`storage.retrieve` roundtrip via Neural API
  - Squirrel: `ai.list_providers` + `tool.list` (25 tools) via Neural API
  - **Cross-domain provenance chain**: hash(BearDog) → sign(BearDog) → store(NestGate) → verify
  - Capability registry completeness (5 domains verified)
- **barracuda discovery refactor**: Split `discovery.rs` (652 lines) into module:
  - `discovery/mod.rs`: registry, probing, scanning (~280 lines)
  - `discovery/capabilities.rs`: 6-format parser, semantic aliases, base injection (~230 lines)
- **IPC timeouts now env-configurable**: `LUDOSPRING_RPC_TIMEOUT_SECS`,
  `LUDOSPRING_PROBE_TIMEOUT_MS`, `LUDOSPRING_CONNECT_PROBE_TIMEOUT_MS`
- **Stale test fix**: `gpu_fog_of_war_degrades_without_toadstool` assertion aligned to evolved message

### Validated — Full NUCLEUS (5 Primals)

- BearDog v0.9.0 (crypto): Blake3, SHA3-256, ChaCha20-Poly1305, Ed25519
- Songbird v0.2.1 (discovery): peer listing via Neural API
- ToadStool (compute): GPU dispatch capabilities (Vulkan/CUDA on RTX 4060)
- NestGate v2.1.0 (storage): store/retrieve/list via Unix socket JSON-RPC
- Squirrel v0.1.0 (AI/MCP): 25 tools, AI provider routing (abstract socket bridged)
- biomeOS neural-api: Coordinated Mode, 60+ capabilities across 5 primals
- Cross-domain provenance: hash → sign → store → retrieve verified end-to-end

### Test counts

- barracuda lib: 424 (discovery 24 incl. capabilities submodule)
- metalForge/forge: 26
- esotericWebb: 341
- Experiments: 83 (82 structural + exp083 live 13/13)
- Total workspace (excl exp032 pre-existing): **734**

## [V33] — 2026-03-29

### Added — Neural API E2E Pipeline

- **exp083_neural_api_e2e**: 10-check validation of full biomeOS Neural API pipeline
  - Blake3 + SHA3-256 hashing via `capability.call` → BearDog
  - ChaCha20-Poly1305 encrypt/decrypt roundtrip through Neural API routing
  - Songbird network primal reachability via Neural API
  - Capability listing verification (crypto, network domains)
  - Sub-200ms latency assertion for routed crypto calls
- **barracuda discovery evolution**: multi-probe fallback (`lifecycle.status` → `health.check` + `capabilities.list`)
  - Format E: BearDog `provided_capabilities` (type + methods objects)
  - Format F: Songbird flat capability arrays
  - Semantic alias generation (`crypto` → `crypto.hash`, `crypto.encrypt`, `crypto.sign`)
  - Auto-injection of `system.ping`, `health.check`, `health.liveness` for responsive primals
  - 4 new unit tests for new formats (discovery tests 15→19)
- **esotericWebb Neural API wiring**: `PrimalBridge.neural_api` field, `neural_api_call()` method,
  `resilient_call()` fallback to `capability.call` when direct domain client absent
  - `resolve_neural_api_socket()` in niche.rs now consumed by bridge `discover()`
  - 322 esotericWebb tests pass with zero regressions

### Validated — Live Primal E2E

- BearDog v0.9.0 server on Unix socket (real Blake3, SHA3-256, ChaCha20-Poly1305, Ed25519)
- Songbird v0.2.1 server on Unix socket (network discovery, federation)
- biomeOS neural-api with 16 registered capabilities routing to live primals
- ToadStool compute dispatch through Neural API (RTX 4060 detected, DRM mode)
- exp042 9/9 against live BearDog + Songbird (first real-primal run)
- exp083 10/10 full Neural API pipeline validation

### Test counts

- barracuda lib: 343 (discovery 19)
- metalForge/forge: 26
- esotericWebb: 322
- Experiments: 83 (82 structural + exp083 live)

## [V32.2] — 2026-03-29

### Added

- Game shader CPU–GPU parity in exp030 — fog-of-war, tile lighting, pathfind wavefront (checks 24→32)
- `GPU_LIGHTING_ABS_TOL` tolerance constant
- `Substrate::Npu`, `recommend_substrate_full()`, `GameWorkload::QuantizedInference`
- `BandTarget::NpuCompute`, `BandTarget::NpuToGpuTransfer` pipeline bands
- `HardwareProfile::mixed_gpu_npu()`, `npu_to_gpu_transfer_ms()`
- NPU/GPU budget fields on `BudgetEstimate`
- Seven new metalForge tests — NPU routing, mixed pipeline, PCIe bypass, budget (forge 19→26)
- Three new Forge integration checks in exp032 (20→23)
- Eight new checks in exp033 — NUCLEUS mixed pipeline + biomeOS NPU graph (19→27)
- V32.2 handoff: `wateringHole/handoffs/LUDOSPRING_V32_2_COMPUTE_EVOLUTION_HANDOFF_MAR29_2026.md`

### Changed

- Validation matrix: all 82 experiments validated (81 green + 1 live-IPC)

### Fixed

- exp003 — RTS HUD bounds + Tufte assertion direction
- exp004 — engagement session params for composite threshold
- exp052 — `Arc<str>` API drift in sweetGrass braid metadata
- exp062 — mislabel fraud detection (`inject_collect_event_for_test` now records `sample_type`)
- Seven trio experiments — `#![allow(missing_docs)]` for workspace lint inheritance

## [V32] — 2026-03-29

### Added

- `specs/BARRACUDA_REQUIREMENTS.md` — consumed/unused modules, shader promotion tiers, upstream evolution requests
- 5 new Python parity tests: `fun_keys_zero_scores`, `fun_keys_max_scores`, `fbm_3d_lattice_zero`, `lsystem_turtle_ff_end`, `lsystem_turtle_square_dist`
- CI baseline drift check job in `.github/workflows/ci.yml` (`baselines` job with `check_drift.py`)
- CI workspace-wide `cargo check` and full workspace `cargo clippy`
- `STRICT_ANALYTICAL_TOL`, `NUMERICAL_FLOOR`, `DDA_ADJUSTMENT_EPSILON`, `SPAN_FLOOR` to `tolerances::validation`
- `TRUST_EQUALITY_TOL` to `tolerances::game`
- `niche::ECOSYSTEM_SOCKET_DIR` constant replacing hardcoded `"biomeos"` socket path

### Changed

- All 77 experiment provenance blocks aligned to current baselines commit (`4b683e3e`)
- 34 analytical experiments populated with commit hashes and dates (was `"N/A"`)
- exp030 rewritten from legacy `ValidationResult` to `ValidationHarness` with GPU-skip via `EXIT_SKIPPED` (525-line rewrite)
- 27 experiment files migrated from manual `eprintln!("FATAL:..."); exit(1)` to `.or_exit("context")` — zero manual FATAL patterns remain
- All test `1e-10` literals replaced with `ANALYTICAL_TOL` across 6 library modules (23 instances)
- `1e-6` in `flow.rs` and `science.rs` handlers replaced with named constants (`SPAN_FLOOR`, `DDA_ADJUSTMENT_EPSILON`)
- `f64::EPSILON` in `transition.rs` replaced with `TRUST_EQUALITY_TOL`
- GPU degradation messages made primal-agnostic (no primal name in `DEGRADE_REASON`)
- MCP tool descriptions reference capabilities not primal names
- `deny.toml`: `unmaintained = "warn"` (invalid for cargo-deny 0.19) → `"workspace"`
- `deploy/ludospring.toml`: added `game.gpu.batch_raycast`, capability count 26→27
- Makefile coverage floor: 80%→85% (aligns with CONTEXT.md)
- IPC integration test: hardcoded `/tmp/ludospring-register.sock` → dynamic tempdir
- `push_client.rs`: hardcoded `"biomeos"` → `niche::ECOSYSTEM_SOCKET_DIR`
- `TensorSession` documented as future-only with shader promotion roadmap reference
- Deprecated binaries removed: `ludospring_dashboard`, `ludospring_live_session`, `ludospring_tufte_dashboard` (consolidated in UniBin V30)

### Removed

- 3 deprecated binary stubs (`ludospring_dashboard.rs`, `ludospring_live_session.rs`, `ludospring_tufte_dashboard.rs`) and their `Cargo.toml` entries — superseded by `ludospring dashboard|live-session|tufte-dashboard` since V30

## [V31] — 2026-03-28

V31 changelog is in the README.md (inlined for historical reasons). See git log for details.

## [V30] — 2026-03-23

### Added

- `thiserror` 2.x for all error types — `IpcError` and all handler errors now `#[derive(thiserror::Error)]`
- MCP `tools.list` and `tools.call` JSON-RPC methods — 8 science tool descriptors with JSON Schema input specs
- `tarpc-ipc` optional feature with `LudoSpringService` typed RPC trait mirroring JSON-RPC surface
- CI pipeline: `.github/workflows/ci.yml` with fmt, clippy, test, doc, cargo deny gates
- `CONTEXT.md` per `PUBLIC_SURFACE_STANDARD`
- `deploy/ludospring.toml` — primalSpring deploy graph fragment (26 capabilities, optional trio + viz deps)
- `LICENSE-ORC` and `LICENSE-CC-BY-SA` — scyBorg triple license files
- Mock IPC test harness `IpcTestServer` — spawns real `IpcServer` for integration tests
- Neural handler methods: `lifecycle.register`, `capability.deregister`, `capability.discover`, `capability.call`
- 273 new tests across IPC handlers, provenance trio, external clients, chaos/fault injection

### Changed

- Handler refactor: `ipc/handlers.rs` (1208 LOC) → `ipc/handlers/` directory with 5 submodules (`lifecycle`, `science`, `delegation`, `mcp`, `neural`) — all under 300 LOC
- UniBin consolidation: dashboard, live-session, tufte-dashboard merged as `ludospring` subcommands (7 total); old binaries deprecated
- Coverage: 80.2% → 91.27% line coverage (85% floor enforced, was 80%)
- Tests: 402 → 675 barracuda tests (587 unit + 42 integration + 3 doctests + 19 proptest + extras)
- Clippy strictness: added `cast_possible_truncation = "deny"`, `cast_sign_loss = "deny"`, `cast_precision_loss = "warn"`
- `health.liveness` returns `{"status": "alive"}` per SEMANTIC_METHOD_NAMING_STANDARD v2.1
- All 14 broken rustdoc intra-doc links fixed
- Provenance trio coverage: ~40% → ~84% (param builders, response mappers, serde round-trips)
- External client coverage: squirrel 49% → 84%, toadstool 47% → 90%, nestgate 52% → 81%
- Handler test coverage: 70% → 95%
- Makefile: `CARGO_TARGET_DIR` + `CARGO_HOME` overrides for `noexec` mount environments

## [V29] — 2026-03-23

### Added

- `GpuContext` module (`game::engine::gpu_context`) — shared `WgpuDevice` lifecycle with `TensorSession` access, behind `#[cfg(feature = "gpu")]`
- Shader absorption handoff for barraCuda: `LUDOSPRING_V29_PERLIN_DDA_SHADER_ABSORPTION_HANDOFF_MAR23_2026.md`
- `cargo-llvm-cov` gating: `make coverage` at 80% floor, `cargo coverage` / `cargo coverage-html` aliases in `.cargo/config.toml`
- `make coverage-report` target for summary-only output
- 7 new experiments: exp076 (Pong), exp077 (Spacewar), exp078 (Tetris), exp079 (Civilization), exp080 (Diablo Loot), exp081 (Procedural Generation), exp082 (Symphony Pipeline)
- `PERLIN_SAFE_BOUND` and `BSP_AREA_CONSERVATION_TOL` in `tolerances::validation`
- Python parity tests expanded from 25 to 42 (fun_keys, Doom Fitts, Hick sweep, flow, engagement, GOMS extended, BSP offset)

### Changed

- barraCuda dependency: `default-features = false` (upstream feature-gating bug fixed — `plasma_dispersion` and `spectral::stats` now properly gated behind `gpu`)
- `barcuda_math` re-exports expanded from 8 to 22 CPU primitives
- `metalForge/forge` refactored from monolithic `lib.rs` (911 LOC) into 4 domain modules: `substrate`, `workload`, `routing`, `pipeline` (19 tests)
- `baselines/python/run_all_baselines.py` relaxed Python requirement from 3.12 to 3.10+ (matches existing provenance)
- All Python baseline scripts: provenance headers updated to "CPython 3.10+"
- `combined_baselines.json` regenerated with `content_sha256` field
- Hardcoded test socket paths replaced with `temp_dir()` + process-unique slugs in `ipc/neural_bridge.rs` and `ipc/discovery.rs`
- Coverage target: 80% floor enforced (80.2% actual, bins excluded)
- Experiments provenance: exp061, exp067–exp075 now include `BaselineProvenance` with specs/ references

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
