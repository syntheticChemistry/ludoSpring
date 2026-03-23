# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring — Context

## What is this?

ludoSpring is an ecoSprings spring — the science of play, interaction, and game
design. It treats games with the same rigor that wetSpring treats bioinformatics
and hotSpring treats computational physics: validated models, reproducible
experiments, and GPU-accelerated computation where it matters.

## Ecosystem position

- **Primal type**: Spring (domain science)
- **Domain**: `game` — ludology, procedural generation, HCI, engagement metrics
- **Parent**: ecoPrimals / ecoSprings
- **License**: AGPL-3.0-or-later (scyBorg triple: AGPL + ORC + CC-BY-SA-4.0)

## Architecture

- **Main crate**: `ludospring-barracuda` (library + IPC binaries)
- **GPU math**: `barraCuda` (path dependency, `default-features = false`)
- **IPC**: JSON-RPC 2.0 over Unix domain sockets (newline-delimited)
- **Transport**: XDG-compliant socket path resolution, capability-based discovery
- **No cross-primal Rust imports**: all coordination via runtime IPC

## Capabilities (26 JSON-RPC methods + MCP tools)

Game science: `game.evaluate_flow`, `game.fitts_cost`, `game.engagement`,
`game.generate_noise`, `game.wfc_step`, `game.analyze_ui`,
`game.accessibility`, `game.difficulty_adjustment`

Provenance trio: `game.begin_session`, `game.record_action`,
`game.complete_session`, `game.mint_certificate`, `game.query_vertices`

AI (via Squirrel): `game.npc_dialogue`, `game.narrate_action`, `game.voice_check`

Coordination: `game.poll_telemetry`, `game.push_scene`, `game.storage_put`,
`game.storage_get`

Health/lifecycle: `health.check`, `health.liveness`, `health.readiness`,
`lifecycle.status`, `capability.list`

MCP: `tools.list` (8 science tool descriptors), `tools.call` (dispatch to science handlers)

Optional: `tarpc-ipc` feature provides `LudoSpringService` typed RPC trait mirroring JSON-RPC.

## Code quality

- **Tests**: 675 barracuda + 19 forge + 42 Python parity + 3 doctests
- **Coverage**: 91.27% line coverage (85% floor enforced via `cargo-llvm-cov`)
- **Error handling**: `thiserror` 2.x — all error types derive `thiserror::Error`
- **Handler layout**: `ipc/handlers/{lifecycle, science, delegation, mcp, neural}.rs`
- **CI**: `.github/workflows/ci.yml` — fmt, clippy, test, doc, cargo deny
- **Lints**: `clippy::pedantic`, `clippy::nursery`, `-D warnings`, `unsafe_code = "forbid"`, `missing_docs = "deny"`

## Build

```sh
cargo test -p ludospring-barracuda --features ipc --lib --tests
cargo clippy -p ludospring-barracuda --all-features -- -D warnings
cargo doc -p ludospring-barracuda --all-features --no-deps
cargo llvm-cov -p ludospring-barracuda --features ipc --lib --tests \
    --ignore-filename-regex bin/ --fail-under-lines 85
```

## Key standards followed

- wateringHole `STANDARDS_AND_EXPECTATIONS.md`
- wateringHole `SEMANTIC_METHOD_NAMING_STANDARD.md` v2.1
- wateringHole `PRIMAL_IPC_PROTOCOL.md` v3
- wateringHole `SPRING_AS_NICHE_DEPLOYMENT_STANDARD.md`
- wateringHole `SPRING_CROSS_EVOLUTION_STANDARD.md` v1.0
