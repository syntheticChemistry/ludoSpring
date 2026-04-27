# ludoSpring V55 — Deep Debt Resolution

**Date:** April 27, 2026
**From:** ludoSpring V55
**For:** primalSpring, all spring teams, primal teams (barraCuda, petalTongue, BearDog, NestGate, Squirrel, coralReef, toadStool), biomeOS, neuralSpring
**Supersedes:** V49 (deep debt) for code quality topics

---

## Executive Summary

V55 completes a systematic deep debt resolution across the `barracuda` crate.
The codebase now has **zero `Result<_, String>`** anywhere — library modules use
domain-specific `thiserror` enums, binaries use `CliError`/`RunnerError`/`IpcError`.
All UDS JSON-RPC transport is deduplicated into a shared `RpcClient`. Discovery is
capability-first (no hardcoded primal names in any dispatch path). The two 800+
line files are modularized. 820 workspace tests, zero clippy warnings.

---

## What Changed

### 1. Method Constants Centralized (`ipc/methods.rs`)

Expanded from 5 modules to 15: added `activation`, `math`, `noise`, `compute`,
`storage`, `dag`, `braid`, `ai`, `spine`, `tensor`. Every raw method string
literal in production IPC code replaced with a constant. Compile-time test
verifies `health::LIVENESS` and `health::READINESS` exist in both
`capability_domains::all_methods()` and `niche::CAPABILITIES`.

**For upstream:** primalSpring could centralize its own `methods.rs` using the
same pattern — modules per domain, compile-time consistency tests.

### 2. IpcError Source Chaining

Added `From<serde_json::Error>` and `From<std::io::Error>` for `IpcError`.
The `classify_io_error` function maps `ConnectionRefused`/`NotFound` to
`IpcError::Connect`, `TimedOut`/`WouldBlock` to `IpcError::Timeout`, and
everything else to `IpcError::Io`. This eliminates all `.to_string()` error
conversion patterns in the IPC layer.

**For primal teams:** If your IPC code has `map_err(|e| format!("..."))`,
this pattern replaces it with a single `?` operator.

### 3. Shared RpcClient (`ipc/rpc_client.rs`)

New `RpcClient` struct encapsulates:
- `new(socket_path, timeout)` — stores path and timeout
- `call(method, params)` — builds envelope, sends, extracts `result` field
- `send_raw(request)` — sends pre-built envelope, returns full response

Refactored `neural_bridge.rs`, `push_client.rs`, `discovery/mod.rs`, and
`btsp.rs` to delegate transport to `RpcClient`. ~120 lines of duplicated
connect-timeout-clone-write-read-parse code removed.

**For upstream absorption:** Any spring or primal with UDS JSON-RPC can adopt
`RpcClient` directly. The pattern is: store `RpcClient` as a field, delegate
`call`/`send_raw` in your domain methods.

### 4. Capability-First NicheDependency

`NicheDependency` struct evolved:

```rust
// Before
pub struct NicheDependency {
    pub name: &'static str,       // hardcoded primal name
    pub capability: &'static str,
    pub role: &'static str,
    pub required: bool,
}

// After
pub struct NicheDependency {
    pub capability: &'static str,        // primary — what we need
    pub role: &'static str,
    pub required: bool,
    pub hint_name: Option<&'static str>, // fallback for socket filename only
}
```

Discovery resolves by capability first (`{capability}.sock`), falling back
to `hint_name` only for socket filename matching. No production code path
uses primal names for identity — only as a degradation hint.

**For all springs:** Recommended evolution. Aligns `niche.rs` with the
NUCLEUS composition model where primals are discovered by capability, not name.

### 5. Typed Library Errors

| Module | Error Type | Replaces |
|--------|-----------|----------|
| `game/voxel.rs` | `VoxelError` | `Result<_, String>` |
| `validation/mod.rs` | `BaselineError` | `Result<_, String>` |
| `composition_targets.rs` | `ComparisonError` | `Result<_, String>` |

All use `thiserror` with `#[error]` and `#[source]` for proper error chains.
Test assertions updated from `err.contains("...")` to `matches!(err, Variant { .. })`.

### 6. Typed Binary Errors

| Binary | Error Type | Replaces |
|--------|-----------|----------|
| `ludospring` + commands | `CliError` (Io, Serialize, Ipc) | `Result<_, String>` |
| `validate_all` | `RunnerError` (CurrentExe, NoParent, Spawn) | `Result<_, String>` |
| `validate_primal_proof` | `IpcError` via `RpcClient` | `Result<_, String>` + hand-rolled socket I/O |

**`validate_primal_proof`** now uses `RpcClient` directly — its hand-rolled
`rpc_call` function (28 lines of duplicated socket I/O) replaced by a 6-line
wrapper around `RpcClient::send_raw`.

### 7. File Refactors

| File | Before | After |
|------|--------|-------|
| `ipc/envelope.rs` | 824 lines (430 prod + 394 test) | 409 lines + `envelope_tests.rs` via `#[path]` |
| `bin/ludospring_guidestone.rs` | 812 lines | `guidestone/main.rs` (117), `constants.rs` (141), `tier1.rs` (164), `tier2.rs` (220), `tier3.rs` (184) |

### 8. Test Delta

817 → 820 workspace tests. New tests:
- `RpcClient::connect_to_missing_socket_returns_connect_error` — verifies `IpcError::Connect` classification
- Method constants consistency — verifies `health::LIVENESS` and `health::READINESS` in capability registries
- `BaselineError` variant matching — verifies typed error variant assertions

---

## Patterns for Upstream Absorption

### For primalSpring

1. **Method constants expansion:** ludoSpring's `ipc/methods.rs` now has 15 domain modules. primalSpring could centralize its own methods registry and recommend the pattern to all springs.
2. **`RpcClient` as shared library:** The `RpcClient` pattern could live in `ecoPrimal` as a shared crate, saving every spring from implementing their own UDS transport.
3. **Capability-first `NicheDependency`:** The `hint_name` pattern is a clean migration path — existing code that used `dep.name` moves to `dep.hint_name.unwrap_or("unknown")` with zero semantic change, while new code uses `dep.capability`.

### For primal teams

1. **`RpcClient`:** If your primal has internal UDS clients (e.g., BearDog calling rhizoCrypt, petalTongue calling loamSpine), `RpcClient` eliminates duplicated socket I/O.
2. **Typed errors:** If your primal still uses `Result<_, String>` anywhere, the `thiserror` enum pattern with `From` impls is the idiomatic replacement. One `?` replaces `map_err(|e| format!(...))`.
3. **`ipc::methods` constants:** If your primal dispatches by raw method string, centralizing to constants prevents typos and enables compile-time checking.

### For biomeOS / neuralAPI

The cell graph (`ludospring_cell.toml`) is **unchanged** — V55 is purely internal
code quality. However, the underlying IPC is now cleaner:
- Typed errors flow through composition (`is_skip_error` checks typed `IpcError` variants, not string matching)
- `RpcClient` means every primal interaction goes through a single, tested transport path
- Capability-first discovery means the composition model and the code model are now aligned

---

## NUCLEUS Composition Status

| Aspect | Status |
|--------|--------|
| Cell graph | `ludospring_cell.toml` — 12 nodes, pure composition, BTSP |
| Capabilities | 30 (27 game + 3 infrastructure) |
| MCP tools | 15/15 |
| Live composition | 18/20 capabilities verified, game.tick in 0.6ms |
| guideStone | Readiness 4 — three-tier (bare + IPC + NUCLEUS cross-atomic) |
| Tests | 820 workspace tests |
| Clippy | Zero warnings with `-D warnings --all-features` |
| `Result<_, String>` | Zero (entire codebase) |
| Hardcoded primal names | Zero (capability-first discovery) |
| Primal gaps | 7 remaining (GAP-01–06, GAP-09); GAP-07/08/10/11 resolved |

---

## Files Changed (35 modified, 3 new, 1 deleted)

### New Files
- `barracuda/src/ipc/rpc_client.rs` — shared `RpcClient`
- `barracuda/src/ipc/envelope_tests.rs` — extracted test module
- `barracuda/src/bin/guidestone/` — `main.rs`, `constants.rs`, `tier1.rs`, `tier2.rs`, `tier3.rs`

### Deleted
- `barracuda/src/bin/ludospring_guidestone.rs` — replaced by `guidestone/` directory

### Key Modifications
- `ipc/methods.rs` — expanded from 5 to 15 domain modules
- `ipc/envelope.rs` — `From` impls + test extraction (824 → 409 lines)
- `ipc/neural_bridge.rs`, `ipc/discovery/mod.rs`, `ipc/btsp.rs`, `visualization/push_client.rs` — delegated to `RpcClient`
- `niche.rs` — capability-first `NicheDependency`
- `game/voxel.rs`, `validation/mod.rs`, `composition_targets.rs` — typed library errors
- `bin/commands/mod.rs` — `CliError` enum
- `bin/validate_all.rs` — `RunnerError` enum
- `bin/validate_primal_proof.rs` — uses `RpcClient` + `IpcError`

---

## Action Items

| Team | Action | Priority |
|------|--------|----------|
| primalSpring | Audit V55 patterns for upstream absorption (especially `RpcClient` and capability-first niche) | High |
| primalSpring | Consider extracting `RpcClient` to shared `ecoPrimal` crate | Medium |
| All springs | Evolve `NicheDependency` to capability-first (`hint_name` pattern) | Medium |
| All springs | Replace `Result<_, String>` with domain `thiserror` enums where present | Medium |
| Primal teams | Review `RpcClient` pattern for internal UDS clients | Low |
| biomeOS | No action — cell graph unchanged, IPC layer improved internally | None |
