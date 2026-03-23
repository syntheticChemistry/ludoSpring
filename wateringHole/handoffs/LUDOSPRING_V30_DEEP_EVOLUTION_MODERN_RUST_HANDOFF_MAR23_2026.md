# ludoSpring V30 — Deep Evolution: Modern Rust, 91% Coverage, MCP

**Date:** March 23, 2026
**From:** ludoSpring V30
**To:** barraCuda, toadStool, coralReef, petalTongue, biomeOS, primalSpring, all springs
**Previous:** `archive/LUDOSPRING_V29_DEEP_EVOLUTION_CROSS_ECOSYSTEM_ABSORPTION_HANDOFF_MAR23_2026.md`
**Status:** Released — no breaking changes to JSON-RPC surface

---

## Summary

V30 is a deep quality evolution: modern idiomatic Rust patterns, 91.27% test
coverage (was 80.2%), handler architecture refactored into domain submodules,
MCP AI integration, optional tarpc typed RPC, CI pipeline, and scyBorg triple
license formalized. Tests grew from 402 to 675. All quality gates green.

## What Changed

### Architecture

| Change | Before | After |
|--------|--------|-------|
| Handler dispatch | Single `handlers.rs` (1208 LOC) | `handlers/{mod,lifecycle,science,delegation,mcp,neural}.rs` (5 submodules, all <300 LOC) |
| UniBin CLI | 3 separate dashboard binaries | 7 subcommands: `server`, `status`, `version`, `dashboard`, `live-session`, `tufte-dashboard` |
| Error handling | Manual `Display`/`Error` impls | `thiserror` 2.x derive on all error types |
| Coverage | 80.2% (80% floor) | 91.27% (85% floor) |
| Tests | 402 barracuda | 675 barracuda |
| CI | None | `.github/workflows/ci.yml` (fmt, clippy, test, doc, cargo deny) |

### New Capabilities

- **MCP `tools.list`**: Returns 8 science tool descriptors with JSON Schema `inputSchema` — enables Squirrel AI and any MCP client to discover and invoke game science methods
- **MCP `tools.call`**: Dispatches `{name, arguments}` to existing science handlers
- **tarpc optional**: `tarpc-ipc` feature with `LudoSpringService` typed trait mirroring JSON-RPC surface (tarpc 0.37)
- **Neural handler**: `lifecycle.register`, `capability.deregister`, `capability.discover`, `capability.call` routed through dispatch

### Quality Improvements

- `thiserror` 2.x on all error types — zero manual `Display`/`Error` impls
- Clippy: `cast_possible_truncation = "deny"`, `cast_sign_loss = "deny"`, `cast_precision_loss = "warn"`
- All 14 rustdoc intra-doc link warnings fixed
- Provenance trio coverage: ~40% → ~84% (param builders, response mappers, serde round-trips)
- External client coverage: squirrel 49% → 84%, toadstool 47% → 90%, nestgate 52% → 81%
- Mock IPC harness (`IpcTestServer`) for integration tests exercising connected code paths
- `CONTEXT.md` per `PUBLIC_SURFACE_STANDARD`
- `deploy/ludospring.toml` primalSpring deploy graph fragment

## Patterns for Absorption

### For barraCuda / toadStool / coralReef

- **`thiserror` migration pattern**: Replace manual `impl Display + Error` with `#[derive(thiserror::Error)]` + `#[error("...")]` + `#[source]`. Eliminates ~30 LOC per error type.
- **Handler domain split**: When `handlers.rs` exceeds 800 LOC, split by domain into `handlers/` directory. Keep `dispatch()` in `mod.rs`, tests in `mod.rs` (they test the public API), domain handlers as `pub(super)`.
- **`const fn` on struct constructors**: Clippy nursery catches trivial constructors that can be `const`.

### For petalTongue

- ludoSpring `tools.list` returns 8 tool descriptors. If petalTongue implements MCP tool discovery, it can auto-discover game science capabilities.
- Dashboard commands now run as `ludospring dashboard` (UniBin subcommand), not separate binary.

### For biomeOS / primalSpring

- `deploy/ludospring.toml` is a primalSpring-compatible deploy graph fragment with 26 capabilities and optional dependencies.
- `health.liveness` now returns `{"status": "alive"}` per SEMANTIC_METHOD_NAMING_STANDARD v2.1.

### For Squirrel AI

- `tools.list` + `tools.call` are the MCP interface. 8 game science tools with full JSON Schema input specs. Squirrel can invoke `game.evaluate_flow`, `game.fitts_cost`, etc. directly.

### For all springs

- **CI template**: `.github/workflows/ci.yml` pattern — push/PR triggers, Rust 1.87, `CARGO_TARGET_DIR` override for noexec mounts, 5 gates (fmt, clippy, test, doc, deny).
- **Coverage strategy**: Unit tests on param builders + response mappers + serde round-trips cover 80%+ of IPC client modules without needing live sockets. Integration tests with `IpcTestServer` cover the connected paths.
- **Triple license**: `LICENSE` (AGPL), `LICENSE-ORC` (game mechanics), `LICENSE-CC-BY-SA` (docs/creative).

## Verification

```bash
cargo fmt -p ludospring-barracuda -- --check          # 0 diffs
cargo clippy -p ludospring-barracuda --all-features -- -D warnings  # 0 warnings
cargo test -p ludospring-barracuda --all-features     # 675 tests, 0 failures
RUSTDOCFLAGS="-D warnings" cargo doc -p ludospring-barracuda --all-features --no-deps  # 0 warnings
cargo llvm-cov -p ludospring-barracuda --features ipc --lib --tests \
    --ignore-filename-regex bin/ --fail-under-lines 85  # 91.27% lines
```

## Files Changed (from V29)

- 30 files changed, 3510 insertions, 2845 deletions
- 13 new files: `handlers/{lifecycle,science,delegation,mcp,neural}.rs`, `bin/commands/{mod,dashboard,live_session,tufte_dashboard}.rs`, `ipc/tarpc_service.rs`, `.github/workflows/ci.yml`, `deploy/ludospring.toml`, `LICENSE-ORC`, `LICENSE-CC-BY-SA`, `CONTEXT.md`
