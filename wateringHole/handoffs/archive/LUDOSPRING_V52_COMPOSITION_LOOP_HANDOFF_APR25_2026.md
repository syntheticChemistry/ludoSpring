# ludoSpring V52 — Composition Loop & Upstream Handoff

**From:** ludoSpring V52 (composition evolution partner)
**For:** primalSpring team, all spring teams, primal teams
**Date:** April 25, 2026
**Previous:** V51 (typed composition patterns), V50 (IpcError debt), V49 (deep debt resolution)

---

## Executive Summary

V50–V52 completes ludoSpring's absorption of primalSpring v0.9.17 composition
patterns and wires the full desktop game loop. ludoSpring is now a reference
implementation for how springs compose interactive systems via NUCLEUS.

**Headline numbers:** 30 capabilities, 817 tests, zero clippy, zero `unsafe`,
zero `Result<_, String>` in IPC, zero hardcoded method strings, zero hardcoded
primal names. Cell graph ready (14 NUCLEUS nodes, BTSP enforced).

---

## What Changed (V49→V52)

### V50: IpcError Debt Resolution

All 43 IPC client functions across 9 modules evolved from `Result<_, String>` to
`Result<_, IpcError>`. `classify_io_error` absorbed from primalSpring v0.9.17.
Query methods (`is_retriable`, `is_recoverable`, `is_method_not_found`,
`is_connection_error`, `is_protocol_error`) enable smart retry and routing logic.

### V51: Typed Composition Patterns

- **`ipc::methods` module**: 19 `&str` constants mirroring `primalspring::ipc::methods`
  for visualization, interaction, health, lifecycle, capability domains. Replaces
  ~30 hardcoded string literals in dispatch and push paths.
- **Handler dispatch**: `dispatch_lifecycle`, `dispatch_infrastructure`, and all 6
  `neural.rs` delegation functions match on constants instead of literals.
- **`VisualizationPushClient`**: All 8 RPC method strings replaced.
- **`IpcError::is_skip_error()`**: Mirrors `primalspring::composition::is_skip_error`.
  Returns `true` for connection errors and protocol errors — enables `call_or_skip`.

### V52: Game Tick Loop (this handoff)

Three new handlers complete the desktop gameplay composition:

| Handler | Method | What it does |
|---------|--------|--------------|
| `handle_game_tick` | `game.tick` | Composite: push scene → poll interactions → record action → engagement metrics. One RPC call per game loop tick. |
| `handle_subscribe_interaction` | `game.subscribe_interaction` | Subscribe to petalTongue input events. |
| `handle_poll_interaction` | `game.poll_interaction` | Poll petalTongue for pending input events. |

All three use `is_skip_error()` for graceful degradation when petalTongue is absent.

**`handle_push_scene` evolved**: Now classifies errors semantically — reports
`degraded: true` instead of opaque error strings.

**`ludospring_cell.toml`**: New cell graph defines the full NUCLEUS deployment —
14 nodes (Tower, Node, Nest, Springs, AI, Store) with BTSP on every node.

---

## Composition Patterns Learned (for upstream absorption)

### Pattern 1: Local Method Constants (avoid cross-crate dependency)

ludoSpring mirrors `primalspring::ipc::methods` locally in `ipc/methods.rs` so
core IPC code can use compile-time constants without depending on the `primalspring`
crate at runtime (which is gated behind the `guidestone` feature).

**Recommendation for primalSpring**: Consider publishing `ipc::methods` as a
lightweight, zero-dependency micro-crate that any spring can import without pulling
in the full `primalspring` API. This would eliminate the need for each spring to
maintain a local mirror.

### Pattern 2: `is_skip_error` Enables Composition Degradation

The `is_skip_error()` method on `IpcError` enables a critical runtime pattern:
when an optional primal (petalTongue, rhizoCrypt, Squirrel) is absent, the
calling handler can report `degraded: true` and continue rather than failing.
This is what makes the game tick loop work in partial deployments.

**Recommendation for primalSpring**: Document this as a first-class composition
pattern in `SPRING_COMPOSITION_PATTERNS` §13 (graceful degradation). The semantics:
- `is_skip_error() == true` → primal absent, operation skippable
- `is_skip_error() == false` → real failure, propagate error

### Pattern 3: Composite Handlers (multi-primal in one RPC)

`game.tick` calls 3 primals in sequence (petalTongue scene push, petalTongue poll,
rhizoCrypt record) and computes local metrics — all in one handler. This is the
composition equivalent of a database stored procedure: the caller makes one call
and the spring orchestrates the multi-primal coordination.

**Recommendation for primalSpring**: Add a "composite handler" pattern to the
composition standard. The key properties:
- One external RPC call, multiple internal primal calls
- Each internal call uses `is_skip_error` independently
- Response includes per-subsystem status (`scene_pushed`, `scene_degraded`, etc.)
- Frame budget tracking (`frame_budget_ms`) for real-time composition

### Pattern 4: Cell Graph as Deployment Contract

`ludospring_cell.toml` declares all 14 nodes with capabilities, dependencies,
fallback behavior, and BTSP enforcement. This is the contract biomeOS uses to
deploy the full interactive experience.

**Recommendation for primalSpring**: Validate cell graphs at the proto-nucleate
level. A spring's cell graph should be machine-checkable against its niche
dependencies and capabilities. `lifecycle.composition` already provides the
runtime validation; the cell graph is the declarative counterpart.

### Pattern 5: Constants in Tests Are Literals

ludoSpring deliberately keeps string literals in test assertions even after
replacing them with constants in production code. This ensures tests validate
the constant values themselves rather than being tautological.

---

## Primal Use and Evolution Summary

### Primals ludoSpring Exercises

| Primal | How ludoSpring uses it | Maturity |
|--------|----------------------|----------|
| **BearDog** | BTSP handshake, Ed25519 signing, BLAKE3 hashing | Production-ready (exp064 validated) |
| **Songbird** | Socket discovery, capability announcement | Production-ready (gaming niche graph) |
| **barraCuda** | Tensor math, sigmoid, dot product, RNG | Production-ready (exp086/090/097 all PASS) |
| **coralReef** | Shader compilation (via toadStool chain) | Works (exp085: 7/8, one discovery gap) |
| **toadStool** | GPU compute dispatch (fog of war, lighting, pathfinding) | Works with CPU fallback (exp031/032) |
| **petalTongue** | Scene push, interaction poll, dashboard render | Wired (V52 handlers), needs live testing |
| **Squirrel** | NPC dialogue, narration, voice check | Wired (delegation handlers), needs live AI |
| **rhizoCrypt** | DAG session/event/dehydration | Blocked (GAP-06: no UDS transport) |
| **loamSpine** | Session commit, certificate minting | Blocked (GAP-07: startup panic) |
| **sweetGrass** | Attribution braids, dehydration records | Available via NeuralBridge |
| **NestGate** | Game state persistence (put/get) | Production-ready (exp094 validated) |

### What Worked Well

1. **Capability-based discovery** — zero hardcoded primal names means biomeOS can
   substitute implementations freely. This is the correct abstraction.
2. **Circuit breaker + exponential backoff** — `resilient_trio_call` in provenance
   module handles transient failures without cascading.
3. **`NeuralBridge` typed client** — capability-addressed IPC through biomeOS's
   Neural API is clean and testable.
4. **Three-tier dispatch** — lifecycle → infrastructure → science routing keeps
   the handler module maintainable at 30 methods.
5. **`ValidationHarness` + `BaselineProvenance`** — every experiment is reproducible
   with provenance tracking. This pattern should be ecosystem-wide.

### What Still Needs Evolution

1. **rhizoCrypt UDS** (GAP-06) — blocks 4+ experiments. TCP transport exists but
   ludoSpring is UDS-only per composition standard.
2. **loamSpine startup** (GAP-07) — runtime nesting panic blocks certificate operations.
3. **barraCuda formulation** (GAP-11) — Fitts/Hick formulas diverge slightly from
   Python baselines. Documented, adjusted in IPC golden values.
4. **Neural API capability registration** — biomeOS doesn't yet route `capability.call`
   to the correct primal dynamically. Static graph routing works.
5. **petalTongue live integration** — handlers are wired, but no E2E test with a
   running petalTongue instance yet. The `is_skip_error` degradation path is tested.

---

## For neuralAPI / biomeOS Team

### Deployment via Neural API

ludoSpring deploys as a single UniBin (`ludospring`) with 7 subcommands. The
`server` subcommand starts the JSON-RPC 2.0 UDS server. biomeOS should:

1. Deploy via `ludospring_cell.toml` (14 nodes)
2. Route `game.*` capabilities to the ludoSpring socket
3. Route `visualization.*` and `interaction.*` to petalTongue
4. BTSP handshake is automatic (ludoSpring relays to BearDog)

### Capability Registration

ludoSpring advertises 30 capabilities via `lifecycle.status` and `capability.list`.
The `config/capability_registry.toml` is the SSOT. biomeOS should parse this at
deploy time for routing table construction.

### Continuous Coordination

`graphs/composition/game_loop_continuous.toml` defines a 60 Hz tick loop:
evaluate_flow → dda_recommend → ai_narrate → render_scene → poll_interaction →
provenance_stamp → crypto_sign. Each node is `required = false` except the first
two, enabling graceful degradation at every step.

---

## For Upstream primalSpring Audit

### Code Health (V52)

| Metric | Value |
|--------|-------|
| Workspace tests | 817 |
| Clippy warnings | 0 |
| `unsafe` blocks | 0 |
| `#[allow()]` in prod | 0 |
| `Result<_, String>` in IPC | 0 |
| Hardcoded method strings | 0 |
| Hardcoded primal names | 0 |
| TODO/FIXME/HACK in `.rs` | 0 |
| External deps removable | 0 |
| Capabilities | 30 |
| MCP tools | 15/15 |
| Primal gaps tracked | 11 (GAP-01–GAP-11) |

### Evolution Requests for Upstream

1. **`primalspring::ipc::methods` micro-crate** — lightweight constants-only crate
   so springs don't maintain local mirrors
2. **`SPRING_COMPOSITION_PATTERNS` §12** — handler test extraction pattern
   (`#[path = "tests.rs"]`)
3. **`SPRING_COMPOSITION_PATTERNS` §13** — graceful degradation via `is_skip_error`
4. **Composite handler pattern** — multi-primal coordination in single RPC
5. **Cell graph validation tooling** — machine-check cell TOML against niche deps
6. **`IpcError` in all primalSpring examples** — some examples still use `String`

### Files Changed (V50–V52)

| Scope | Files | Key change |
|-------|-------|------------|
| IPC error types | `ipc/envelope.rs`, 9 client modules | `Result<_, String>` → `IpcError` |
| Method constants | `ipc/methods.rs` (new) | 19 constants, zero string literals |
| Dispatch routing | `handlers/mod.rs`, `handlers/neural.rs` | Constant-based matching |
| Push client | `visualization/push_client.rs` | 8 method strings replaced |
| Game tick | `handlers/delegation.rs` | 3 new handlers, `push_scene` evolved |
| Params | `ipc/params.rs` | `GameTickParams`, `SubscribeInteractionParams`, `PollInteractionParams` |
| Capabilities | `niche.rs`, `capability_domains.rs` | 27→30 capabilities |
| Deploy graphs | `graphs/ludospring_cell.toml` (new), `gaming_niche.toml`, `game_loop_continuous.toml` | Interaction loop wired |
| Docs | `CHANGELOG.md`, `README.md`, `CONTEXT.md`, `PRIMAL_GAPS.md`, niche YAML | V52 numbers |

---

## Archive Note

V49 handoff moves to archive. V46 already in archive (superseded by V47).

## License

AGPL-3.0-or-later
