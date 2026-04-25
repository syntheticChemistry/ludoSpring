# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring — Context

**Last updated:** April 25, 2026 (V53 — Binary to composition evolution: ludospring binary removed from plasmidBin; game science served by composing primals via NUCLEUS cell graph; `ludospring_cell.toml` evolved to 12-node pure composition; GAP-10 resolved; 817 tests, zero clippy)

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
- **Transport**: XDG-compliant socket path resolution, capability-based discovery
- **No cross-primal Rust imports**: all coordination via runtime IPC
- **No deployed binary**: game science is served by composing primals (barraCuda,
  petalTongue, Squirrel, provenance trio) via the cell graph

## Capabilities (30 total in `niche.rs`: 27 game + 3 infrastructure — `health.check`, `lifecycle.status`, `capability.list`; MCP tools)

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

- **Tests**: 817 workspace `#[test]` functions (V53)
- **Experiments**: 100 total (83 science + 5 composition gap discovery + 5 science-via-composition + 5 NUCLEUS game engine composition + 2 composition validation)
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
| exp045 doc link fixed (rustdoc warning eliminated) | `experiments/exp045` |

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

**Composition experiments score: 95/141 (67.4%)** — 5 experiments fully PASS.

### Live results

| Exp | Target | Pass/Total | Key finding |
|-----|--------|------------|-------------|
| 084 | barraCuda math IPC | 12/15 | All math methods work. Neural API routing gap |
| 085 | Shader dispatch chain | 7/8 | coralReef compile works. toadStool↔coralReef discovery gap |
| 086 | Tensor composition | **10/10** | ALL tensor ops confirmed |
| 087 | Neural API pipeline | 3/8 | capability.call not routing to primals |
| 088 | Continuous game loop | 2/10 | Neural API capability registration gap |
| 089 | Psychomotor (Fitts/Hick/Steering) | 4/8 | barraCuda Fitts/Hick formula mismatch |
| 090 | GameFlow tensor | **13/13** | Flow, engagement, DDA all correct |
| 091 | PCG/Noise | 7/8 | perlin3d lattice invariant broken |
| 092 | Composite pipeline | **8/8** | GOMS, Four Keys, stats all correct |
| 093 | Continuous session | **6/6** | 60Hz loop, 0.18ms max tick, deterministic |
| 094 | Session lifecycle | 3/8 | BearDog+NestGate work. rhizoCrypt: no UDS |
| 095 | Content ownership | 0/8 | rhizoCrypt no UDS + loamSpine startup panic |
| 096 | NPC dialogue | 5/10 | barraCuda math works. rhizoCrypt/Squirrel/petalTongue missing |
| 097 | Population dynamics | **10/10** | Replicator, Markov, Wright-Fisher all correct |
| 098 | NUCLEUS game session | 5/6 | Full 10-tick loop. Only rhizoCrypt provenance missing |
| 099 | Composition validation | 13/13* | Rust library == IPC parity (all 8 science methods) |

### Primal gap matrix

**11 primal gaps (GAP-01–GAP-11)** — canonical registry and remediation detail: [`docs/PRIMAL_GAPS.md`](docs/PRIMAL_GAPS.md) (**GAP-09:** nest_atomic stubs; **GAP-10:** `game.*` identity; **GAP-11:** barraCuda formulation divergence). The table below summarizes composition-experiment impact (live genomeBin NUCLEUS / exp084–098 + guideStone); IDs in the doc may order topics differently.

| Gap | Owner | Severity | Checks gained when fixed |
|-----|-------|----------|--------------------------|
| TCP-only transport (no UDS) | rhizoCrypt | CRITICAL | +9 |
| Startup panic (runtime nesting) | loamSpine | CRITICAL | +6 |
| Fitts/Hick formula mismatch | barraCuda | HIGH | +4 |
| No capability registration | biomeOS Neural API | HIGH | +14 |
| No x86_64 binary in genomeBin | barraCuda | HIGH | deployment |
| Perlin3D lattice invariant | barraCuda | MEDIUM | +1 |
| Inter-primal discovery | toadStool↔coralReef | MEDIUM | +1 |

**Projected with all fixes: 130/141 (92.2%)**

### What works today

- barraCuda tensor/stats/noise/activation math via UDS IPC
- BearDog crypto (blake3_hash, sign_ed25519) via base64 params
- NestGate storage (store/retrieve with family_id) via UDS
- sweetGrass attribution via UDS (available, not fully tested — blocked by rhizoCrypt/loamSpine)
- Songbird discovery via UDS
- 60Hz composition loops under 0.54ms per tick
- biomeOS graph deployment and health probing

### Composition graphs

- `graphs/composition/science_validation.toml` — sequential barraCuda math pipeline
- `graphs/composition/nucleus_game_session.toml` — continuous 60Hz NUCLEUS game tick
- `graphs/composition/session_provenance.toml` — session lifecycle via Nest Atomic + Trio
- `graphs/composition/math_pipeline.toml`, `engagement_pipeline.toml`, `shader_dispatch_chain.toml`, `game_loop_continuous.toml`

### Handoff

[V53 central handoff](../../infra/wateringHole/handoffs/LUDOSPRING_V53_COMPOSITION_EVOLUTION_HANDOFF_APR25_2026.md)
[V49 deep debt handoff](../../infra/wateringHole/handoffs/LUDOSPRING_V49_DEEP_DEBT_COMPOSITION_PATTERNS_APR25_2026.md)
