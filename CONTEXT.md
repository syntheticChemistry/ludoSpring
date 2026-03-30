# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring — Context

**Last updated:** March 30, 2026 (V35)

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

## Capabilities (27 JSON-RPC methods + MCP tools)

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

MCP: `tools.list` (13 tool descriptors: 8 science + 5 delegation), `tools.call` (dispatch to handlers)

Optional: `tarpc-ipc` feature provides `LudoSpringService` typed RPC trait mirroring JSON-RPC.

## Code quality

- **Tests**: 424 barracuda lib + 26 forge + 47 Python parity + 2 doctests = 734 workspace total
- **Experiments**: 88 total (83 science + 5 primal composition gap discovery)
- **Coverage**: 91.27% line coverage (85% floor enforced via `cargo-llvm-cov`; target 90%+)
- **Error handling**: `thiserror` 2.x — all error types derive `thiserror::Error`
- **Handler layout**: `ipc/handlers/{lifecycle, science, delegation, mcp, neural}.rs`
- **Discovery**: `ipc/discovery/{mod, capabilities}.rs` — 6-format capability parser, semantic aliases
- **IPC timeouts**: env-configurable via `LUDOSPRING_RPC_TIMEOUT_SECS`, `LUDOSPRING_PROBE_TIMEOUT_MS`
- **CI**: `.github/workflows/ci.yml` — fmt, clippy, test, doc, cargo deny
- **Lints**: `clippy::pedantic`, `clippy::nursery`, `-D warnings`, `unsafe_code = "forbid"`, `missing_docs = "deny"`

## Build

```sh
cargo test --workspace
cargo clippy --workspace --all-features -- -D warnings
cargo doc --workspace --all-features --no-deps
cargo llvm-cov -p ludospring-barracuda --features ipc --lib --tests \
    --ignore-filename-regex bin/ --fail-under-lines 85
```

## Key standards followed

- wateringHole `STANDARDS_AND_EXPECTATIONS.md`
- wateringHole `SEMANTIC_METHOD_NAMING_STANDARD.md` v2.1
- wateringHole `PRIMAL_IPC_PROTOCOL.md` v3
- wateringHole `SPRING_AS_NICHE_DEPLOYMENT_STANDARD.md`
- wateringHole `SPRING_CROSS_EVOLUTION_STANDARD.md` v1.0
- **esotericWebb alignment** — IPC response shapes compatible with esotericWebb `LudoSpringClient` (gen4 product integration)

## V35: Primal Composition Validation (Track 26)

ludoSpring's next evolution tier: replicate validated game science using ONLY
primal composition — no ludoSpring binary in the loop. Experiments probe live
primals and document gaps.

**V35.1 revalidation (Mar 30, 2026 — barraCuda local build + biomeOS v2.79 + coralReef Iter70 + toadStool S168 + beardog):**

| Exp | Target | V35 | V35.1 | Key finding |
|-----|--------|-----|-------|-------------|
| 084 | barraCuda math IPC | 0/12 | **4/15** | barraCuda alive! sigmoid, log2, mean, std_dev PASS. Domain methods (fitts, hick, flow) still -32601 |
| 085 | Shader dispatch chain | 2/8 | **7/8** | coralReef raw JSON-RPC fixed! compile + dispatch work. Only readback fails (no sovereign driver) |
| 086 | Tensor composition | 0/10 | **5/10** | tensor.create + matmul + read PASS. Element-wise ops (add/scale/clamp/reduce) still method_not_found |
| 087 | Neural API pipeline | 1/7 | **3/7** | health.liveness PASS, graph.list returns 40 graphs. Composition graphs need graph.save deployment |
| 088 | Continuous game loop | 2/10 | **2/10** | Socket naming mismatch blocks forwarding. Sub-ms latency still confirmed |

**Total: 5/47 → 21/50 (42%) — evolution is real and substantial.**

**Composition graphs**: `graphs/composition/*.toml` — `[graph]` header (biomeOS-compatible), `[[nodes]]` with `by_capability`.

**V35 gaps RESOLVED by primal evolution:**
1. ~~barraCuda missing from plasmidBin~~ — binary built, IPC server responds (30 methods)
2. ~~coralReef HTTP JSON-RPC~~ — raw newline-delimited on UDS (Iter 70)
3. ~~biomeOS continuous executor stub~~ — node dispatch wired with capability routing (v2.79)
4. ~~biomeOS nucleus vs runtime graphs~~ — `graph.save` implemented, tier separation (v2.79)
5. ~~health.liveness missing~~ — implemented on Neural API (v2.79)

**Remaining gaps (V35.1):**
1. barraCuda tensor element-wise ops (add/scale/clamp/reduce/sigmoid) registered but not dispatched
2. Socket naming convention mismatch: biomeOS expects `{primal}-{family}.sock`, primals bind as `{primal}.sock`
3. Composition graphs need deployment via `graph.save` API (not filesystem)
4. barraCuda domain-level methods (fitts, hick, flow, engagement) not on IPC — these are ludoSpring science, need either upstream absorption or experiment refactoring to use barraCuda primitives
