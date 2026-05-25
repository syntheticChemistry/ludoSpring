# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring — Context

**Last updated:** May 25, 2026 (V79 — Wave 50 covalent HPC. 12/12 NUCLEUS ALIVE. GAP-01 coralReef LIVE VALIDATED. Songbird mesh seeded. Cross-gate state sync graph. 982 tests, zero clippy.)

## Gate Deployment

| **Gate** | ironGate |
| **Hardware** | i9-14900K (24c/32t), RTX 5070, 96GB DDR5 |
| **Composition** | Full NUCLEUS (13/13 primals) |
| **Status** | **operational** — 11/11 proto-nucleate PASS |
| **Federation** | Songbird TCP :7700 (LAN covalent mesh) |
| **Cell graph** | `plasmidBin/cells/ludospring_cell.toml` |

## What is this?

ludoSpring is an ecoSprings spring — the science of play, interaction, and game
design. It treats games with the same rigor that wetSpring treats bioinformatics
and hotSpring treats computational physics: validated models, reproducible
experiments, and GPU-accelerated computation where it matters.

## Ecosystem position

- **Type**: Spring (domain science) — NOT a primal
- **Domain**: `game` — ludology, procedural generation, HCI, engagement metrics
- **Deployment**: Pure composition via NUCLEUS cell graph (`ludospring_cell.toml`)
- **Validation**: Rust binary is tier 2 validation target (Python → Rust → Composition)
- **Parent**: ecoPrimals / ecoSprings
- **License**: AGPL-3.0-or-later (scyBorg triple: AGPL + ORC + CC-BY-SA-4.0)

## Architecture

- **Main crate**: `ludospring-barracuda` (library + IPC binaries for validation)
- **GPU math**: `barraCuda` (path dependency, `default-features = false`)
- **IPC**: JSON-RPC 2.0 over Unix domain sockets (newline-delimited)
- **Transport**: XDG-compliant socket path resolution, capability-based discovery, shared `RpcClient` for all UDS JSON-RPC operations
- **No cross-primal Rust imports**: all coordination via runtime IPC
- **No deployed binary**: game science is served by composing primals (barraCuda,
  petalTongue, Squirrel, provenance trio) via the cell graph

## Capabilities (32 total in `niche.rs`: 27 game + 5 infrastructure — `health.check`, `health.version`, `health.drain`, `lifecycle.status`, `capability.list`; MCP tools)

Game science: `game.evaluate_flow`, `game.fitts_cost`, `game.engagement`,
`game.generate_noise`, `game.wfc_step`, `game.analyze_ui`,
`game.accessibility`, `game.difficulty_adjustment`

Provenance trio: `game.begin_session`, `game.record_action`,
`game.complete_session`, `game.mint_certificate`, `game.query_vertices`

AI (via Squirrel): `game.npc_dialogue`, `game.narrate_action`, `game.voice_check`

Coordination: `game.poll_telemetry`, `game.push_scene`, `game.storage_put`,
`game.storage_get`

GPU (via toadStool delegation, CPU fallback): `game.gpu.fog_of_war`,
`game.gpu.tile_lighting`, `game.gpu.pathfind`, `game.gpu.perlin_terrain`,
`game.gpu.batch_raycast`

Health/lifecycle: `health.check`, `health.liveness`, `health.readiness`,
`lifecycle.status`, `capability.list`

MCP: `tools.list` (15 tool descriptors: 8 science + 7 delegation), `tools.call` (dispatch to handlers)

Optional: `tarpc-ipc` feature provides `LudoSpringService` typed RPC trait mirroring JSON-RPC.

## Code quality

- **Tests**: 982 workspace `#[test]` functions (V76, Schell Lenses + CPU/GPU parity + NUCLEUS atomics + Bartle + Deterding + petalTongue scenes + Tier 4 IPC-first)
- **Experiments**: 100 total (fossilized to `fossilRecord/`; 10 validation scenarios absorbed into `validation/scenarios/` with `ScenarioMeta`)
- **Coverage**: 90%+ line coverage (enforced via `cargo-llvm-cov` in CI and local `make coverage`)
- **Error handling**: `thiserror` 2.x — all error types derive `thiserror::Error`
- **Handler layout**: `ipc/handlers/{lifecycle, science, delegation, mcp, neural, gpu}.rs` — three-tier dispatch (lifecycle → infrastructure → science)
- **Discovery**: `ipc/discovery/{mod, capabilities}.rs` — 6-tier tiered discovery (`DiscoveryTier`, `DiscoveryResult`), 6-format capability parser, semantic aliases
- **IPC errors**: `IpcErrorPhase` + `PhasedIpcError` with `is_retriable()` / `is_recoverable()` / `is_method_not_found()` classification (primalSpring pattern)
- **Method normalization**: `normalize_method()` strips spring/primal prefixes before dispatch (biomeOS routing compat)
- **Composition validation**: `ipc/composition.rs` — `CompositionReport` probes all 11 niche dependencies at runtime
- **Niche dependencies**: `NicheDependency` table in `niche.rs` — 11 typed proto-nucleate entries
- **IPC timeouts**: env-configurable via `LUDOSPRING_RPC_TIMEOUT_SECS`, `LUDOSPRING_PROBE_TIMEOUT_MS`
- **CI**: `.github/workflows/ci.yml` — fmt, clippy, test, doc, cargo deny, llvm-cov 90% floor
- **Lints**: `clippy::pedantic`, `clippy::nursery`, `-D warnings`, `unsafe_code = "forbid"`, `missing_docs = "deny"`

## Build

```sh
cargo test --workspace
cargo clippy --workspace --all-features -- -D warnings
cargo doc --workspace --all-features --no-deps
cargo llvm-cov -p ludospring-barracuda --features ipc --lib --tests \
    --ignore-filename-regex bin/ --fail-under-lines 90
```

## Key standards followed

- wateringHole `STANDARDS_AND_EXPECTATIONS.md`
- wateringHole `SEMANTIC_METHOD_NAMING_STANDARD.md` v2.1
- wateringHole `PRIMAL_IPC_PROTOCOL.md` v3
- wateringHole `SPRING_AS_NICHE_DEPLOYMENT_STANDARD.md`
- wateringHole `SPRING_CROSS_EVOLUTION_STANDARD.md` v1.0
- **esotericWebb alignment** — IPC response shapes compatible with esotericWebb `LudoSpringClient` (gen4 product integration)

## V42: Composition Evolution — Rust+Python validate NUCLEUS patterns (April 11, 2026)

Python validated Rust. Now Rust and Python are validation targets for ecoPrimal
NUCLEUS composition patterns. V42 completes the evolution from validation spring
to composition spring — `lifecycle.composition` is externally callable, discovery
is capability-first, and fragments declare the full atomic surface.

### V42 changes

| Change | Module |
|--------|--------|
| `lifecycle.composition` handler — runtime composition probe via JSON-RPC | `ipc/handlers/lifecycle.rs` |
| Capability-first discovery (`by_capability` → name fallback) | `ipc/composition.rs` |
| `nest_atomic` in declared fragments (aspirational until GAP-06/07 resolve) | `ipc/composition.rs`, `config/capability_registry.toml` |
| Provenance unified to single commit `19e402c0` (88 files) | all experiments, `validation.rs` |
| ecoBin banned-crate enforcement (8 C deps) | `deny.toml` |
| fog_of_war.wgsl README reconciled with shader body | `barracuda/shaders/game/README.md` |
| exp045 doc link fixed (rustdoc warning eliminated) | `fossilRecord/experiments_prokaryotic_may2026/exp045` |

### Metrics

- **Tests**: 781 → **791** (composition parity, WFC, accessibility tests)
- **Clippy**: zero warnings (workspace-wide)
- **Fragments**: `tower_atomic`, `node_atomic`, `nest_atomic`, `meta_tier`
- **GAP-09**: updated to RESOLVED (nest_atomic declared, trio `required: false`)

## V41: Composition Evolution — Absorbed primalSpring patterns (April 11, 2026)

V41 absorbs 9 hardened composition patterns from primalSpring, plasmidBin, and
wateringHole `SPRING_COMPOSITION_PATTERNS.md`.

### Metrics

- **Tests**: 733 → **779** (+46 new composition pattern tests)
- **Clippy**: zero warnings (ludoSpring workspace)
- **Gaps**: 10 tracked, nest_atomic documented as aspirational
- **plasmidBin**: metadata bumped to V41, `--port` flag aligned

## V40: Audit & documentation cleanup (April 11, 2026)

Workspace-wide quality pass and doc alignment with `docs/PRIMAL_GAPS.md`:

- **Clippy:** 207 warnings → 0 (`cargo clippy --workspace --all-features -- -D warnings`)
- **`deny.toml`:** Migrated for current `cargo-deny` / workspace policy
- **Tests:** Authoritative counts — 605 barracuda lib + 102 `--tests` targets + 26 forge = **733** total
- **Gaps:** **10** primal gaps (GAP-01–GAP-10); see `docs/PRIMAL_GAPS.md` — notably **GAP-09** (nest_atomic stubs), **GAP-10** (`game.*` identity)
- **`ipc/handlers/neural.rs`:** Split for maintainability (was one large module; now sub–100 LOC units)
- **Experiments exp030–exp100:** Clippy-clean; `load_baseline_f64` coverage verified

## V39: NUCLEUS Composition Parity (April 11, 2026)

V39 evolves ludoSpring from Layer 2 (Rust→IPC) into full Layer 3 (IPC→NUCLEUS)
validation. Python validated Rust; now both Python and Rust validate the primal
composition patterns.

### Key changes

- **exp100 — NUCLEUS Composition Parity**: Four-layer validator (niche integrity,
  health probes, capability discovery, science parity, golden chain Python→Rust→IPC→primal proof)
- **Coverage enforced in CI**: `cargo-llvm-cov --fail-under-lines 90` added to
  `.github/workflows/ci.yml`
- **`config/capability_registry.toml`**: Machine-readable SSOT for capabilities,
  semantic mappings, external dependencies, and proto-nucleate references
- **Shared HUD fixtures**: Extracted `hud_fixtures.rs` from duplicated dashboard code
- **Dialogue constants centralized**: `D6_SUCCESS_THRESHOLD` and `DIALOGUE_EMA_ALPHA`
  moved to `tolerances::game`
- **Stale provenance fixed**: `python_parity.rs` commit hash and
  `specs/BARRACUDA_REQUIREMENTS.md` barraCuda path updated
- **Forge naming**: `fraud_batch` → `anti_cheat_batch`
- **Makefile parity**: `make test` now includes forge (matches CI)

## V38: Composition Validation Chain (April 11, 2026)

ludoSpring ships a UniBin (`ludospring`) with `server`, `status`, `version`,
and visualization subcommands for local IPC deployment. The ecoBin is now
harvested to `infra/plasmidBin/` (v0.10.0, 3.1M PIE binary, sha256-verified).

### Four-layer validation chain

```text
Python baseline → validates → Rust library code       (Layer 1: established)
Rust library    → validates → IPC composition          (Layer 2: NEW in V38)
IPC composition → validates → NUCLEUS deployment       (Layer 3: experiments)
```

**Layer 2 artifacts:**
- `baselines/rust/composition_targets.json` — golden targets from Rust library
- `baselines/rust/generate_composition_targets.rs` — generator (cargo example)
- 7 composition parity tests in `barracuda/tests/ipc_integration.rs`
- exp099 — standalone composition validation experiment (13 checks)

**Composition experiments score: 130/141 (92.2%) — projected** — 9 experiments fully PASS.

Previous blockers (all **RESOLVED** upstream, May 10, 2026):
- GAP-06 (rhizoCrypt UDS): RESOLVED (S66)
- GAP-07 (loamSpine panic): RESOLVED (PG-33)
- GAP-08/11 (barraCuda formula): RESOLVED (PG-38)
- GAP-03 (biomeOS composition.deploy): RESOLVED (v3.51)
- GAP-09 (biomeOS method.register): RESOLVED (v3.51)
- JH-11 (token federation): RESOLVED (May 10)

### Live results (post-resolution projection)

| Exp | Target | Pass/Total | Key finding |
|-----|--------|------------|-------------|
| 084 | barraCuda math IPC | **15/15** | All math methods work. Neural API routing fixed (GAP-09) |
| 085 | Shader dispatch chain | **8/8** | coralReef compile works. toadStool discovery fixed |
| 086 | Tensor composition | **10/10** | ALL tensor ops confirmed |
| 087 | Neural API pipeline | **8/8** | capability.call routing via method.register (GAP-09 resolved) |
| 088 | Continuous game loop | **10/10** | Neural API capability registration fixed (GAP-09) |
| 089 | Psychomotor (Fitts/Hick/Steering) | **8/8** | barraCuda formula parity (PG-38 resolved) |
| 090 | GameFlow tensor | **13/13** | Flow, engagement, DDA all correct |
| 091 | PCG/Noise | 7/8 | perlin3d lattice invariant (PG-47 still open) |
| 092 | Composite pipeline | **8/8** | GOMS, Four Keys, stats all correct |
| 093 | Continuous session | **6/6** | 60Hz loop, 0.18ms max tick, deterministic |
| 094 | Session lifecycle | **8/8** | BearDog+NestGate+rhizoCrypt all working (GAP-06 resolved) |
| 095 | Content ownership | **6/8** | rhizoCrypt UDS + loamSpine both work. 2 edge cases remain |
| 096 | NPC dialogue | 8/10 | barraCuda + rhizoCrypt work. Squirrel/petalTongue partial |
| 097 | Population dynamics | **10/10** | Replicator, Markov, Wright-Fisher all correct |
| 098 | NUCLEUS game session | **6/6** | Full 10-tick loop with rhizoCrypt provenance |
| 099 | Composition validation | **13/13** | Rust library == IPC parity (all 8 science methods) |

### Remaining gap matrix

| Gap | Owner | Severity | Checks remaining |
|-----|-------|----------|------------------|
| Perlin3D lattice invariant | barraCuda (PG-47) | LOW | 1 |
| Squirrel inference routing | Squirrel/biomeOS | LOW | 2 |
| petalTongue musl threading | petalTongue (PG-48) | LOW | 2 |
| Content ownership edge cases | loamSpine | LOW | 2 |
| stats.entropy not implemented | barraCuda (PG-47) | LOW | 4 |

**Remaining: 11 checks across 5 low-severity upstream issues**
**Target: 141/141 once PG-47/PG-48 resolve upstream**

### What works today (post-interstadial)

- barraCuda tensor/stats/noise/activation math via UDS IPC (Tier 4, optional library dep)
- BearDog crypto (blake3_hash, sign_ed25519) via base64 params
- NestGate storage (store/retrieve with family_id) via UDS
- rhizoCrypt DAG provenance via UDS (GAP-06 resolved)
- loamSpine certificates (GAP-07 resolved)
- sweetGrass attribution via UDS
- Songbird discovery + NAT traversal via UDS
- skunkBat audit logging (deploy graphs + Rust IPC module: audit_log, audit_session, audit_certification, audit_validation, query_audit_trail)
- biomeOS composition.status + method.register (v3.51)
- biomeOS graph deployment and health probing
- 60Hz composition loops under 0.54ms per tick
- JH-11 token federation (cross-primal auth)

### Composition graphs

- `graphs/composition/science_validation.toml` — sequential barraCuda math pipeline
- `graphs/composition/nucleus_game_session.toml` — continuous 60Hz NUCLEUS game tick
- `graphs/composition/session_provenance.toml` — session lifecycle via Nest Atomic + Trio
- `graphs/composition/math_pipeline.toml`, `engagement_pipeline.toml`, `shader_dispatch_chain.toml`, `game_loop_continuous.toml`

### Handoff

[V63 deep debt + SPDX handoff](../../infra/wateringHole/handoffs/LUDOSPRING_V63_DEEP_DEBT_SPDX_HANDOFF_MAY11_2026.md)
[V61 deep debt handoff](../../infra/wateringHole/handoffs/LUDOSPRING_V61_DEEP_DEBT_HANDOFF_MAY11_2026.md)
[V60 skunkBat + foundation handoff](../../infra/wateringHole/handoffs/LUDOSPRING_V60_SKUNKBAT_FOUNDATION_HANDOFF_MAY11_2026.md)
