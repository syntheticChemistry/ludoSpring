<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# ludoSpring V18 — Niche Self-Knowledge + NeuralBridge Evolution Handoff

**Date**: March 15, 2026
**From**: ludoSpring (game science spring)
**To**: barraCuda team (math primitives), toadStool team (GPU dispatch), biomeOS team (orchestration)
**Covers**: V17 → V18 (niche.rs, NeuralBridge, platform-agnostic paths, socket centralization, capability deduplication)
**Status**: Complete — 66 experiments, 1371 checks, 244 tests + 12 proptest, 0 clippy warnings
**Supersedes**: V17 Deep Evolution Handoff (archived)

---

## Executive Summary

V18 evolves ludoSpring's primal architecture from scattered self-knowledge to a
centralized niche model, following the airSpring `niche.rs` pattern and groundSpring's
typed IPC client pattern. Key deliverables:

1. **`niche.rs` — single source of truth** for primal identity, capabilities, semantic
   mappings, operation dependencies, cost estimates, family ID resolution, and socket
   path resolution. All other modules delegate to niche.rs.

2. **`NeuralBridge` typed IPC client** — `discover()`, `capability_call()`,
   `discover_capability()`, `register()`, `deregister()` with configurable timeout
   (`BIOMEOS_RPC_TIMEOUT_SECS`). Replaces 3 independent RPC helper implementations.

3. **Platform-agnostic paths** — all `/tmp` hardcoding replaced with `std::env::temp_dir()`
   through `niche::socket_dirs()`. XDG-compliant chain: `BIOMEOS_SOCKET_DIR` →
   `$XDG_RUNTIME_DIR/biomeos/` → `temp_dir()/biomeos-$USER` → `temp_dir()`.

4. **Zero duplication** — `CAPABILITIES`, `SEMANTIC_MAPPINGS`, `operation_dependencies()`,
   and `cost_estimates()` defined once in `niche.rs`, consumed by handlers, biomeos, server,
   provenance, and all binaries.

5. **All deps pure Rust** — external dependency audit confirmed ecoBin compliance.
   Only `-sys` crate is `renderdoc-sys` (runtime dlopen in wgpu-hal, not build dep).

---

## Part 1: niche.rs Architecture

### What it contains

| Item | Type | Purpose |
|------|------|---------|
| `NICHE_NAME` | `&str` | `"ludospring"` — socket naming, registration, logging |
| `NICHE_DOMAIN` | `&str` | `"game"` — capability domain prefix |
| `CAPABILITIES` | `&[&str]` | 12 `game.*` methods |
| `SEMANTIC_MAPPINGS` | `&[(&str, &str)]` | Short name → fully qualified capability |
| `operation_dependencies()` | `fn → Value` | Input requirements per capability |
| `cost_estimates()` | `fn → Value` | Latency/CPU/memory per capability |
| `family_id()` | `fn → String` | `FAMILY_ID` → `BIOMEOS_FAMILY_ID` → `"default"` |
| `socket_dirs()` | `fn → Vec<PathBuf>` | XDG-compliant directory chain |
| `resolve_server_socket()` | `fn → PathBuf` | Where to bind IPC server |
| `resolve_neural_api_socket()` | `fn → Option<PathBuf>` | Where to find Neural API |

### Who consumes it

| Module | What it uses |
|--------|-------------|
| `ipc/handlers.rs` | `CAPABILITIES`, `NICHE_DOMAIN`, `operation_dependencies()`, `cost_estimates()` |
| `ipc/server.rs` | `resolve_server_socket()` |
| `ipc/discovery.rs` | `socket_dirs()` |
| `ipc/provenance.rs` | `resolve_neural_api_socket()` via NeuralBridge |
| `ipc/neural_bridge.rs` | `resolve_neural_api_socket()`, `NICHE_NAME`, `NICHE_DOMAIN`, `CAPABILITIES`, `SEMANTIC_MAPPINGS` |
| `biomeos/mod.rs` | Re-exports `GAME_CAPABILITIES`, `GAME_DOMAIN`, `GAME_SEMANTIC_MAPPINGS` from niche |
| `bin/ludospring.rs` | `family_id()`, `resolve_server_socket()` |
| `visualization/push_client.rs` | `NICHE_DOMAIN` |

### biomeOS team action

The `niche.rs` pattern (identity + capabilities + dependencies + costs + socket resolution)
should become a recommended pattern for all springs. airSpring, groundSpring, and now
ludoSpring all use it. Consider documenting it in `wateringHole/SPRING_NICHE_PATTERN.md`.

---

## Part 2: NeuralBridge Typed IPC Client

### API

```rust
let bridge = NeuralBridge::discover()?;          // XDG socket chain
bridge.capability_call("crypto", "sign", &args)?; // Route through biomeOS
bridge.discover_capability("visualization")?;      // Query providers
bridge.register(&our_socket)?;                     // Full registration
bridge.deregister()?;                              // Clean shutdown
```

### What it replaced

| Before (V17) | After (V18) | LOC saved |
|--------------|-------------|-----------|
| `biomeos/mod.rs::rpc_call()` | `NeuralBridge::register()` | ~50 |
| `provenance.rs::capability_call()` | `NeuralBridge::capability_call()` | ~60 |
| `bin/ludospring.rs::json_rpc_call()` + `register_with_neural_api()` | `biomeos::register_domain()` via NeuralBridge | ~50 |

Total: ~160 lines of duplicated socket I/O eliminated.

### toadStool team action

The `NeuralBridge` pattern (discover + capability_call + register + deregister) is the
recommended way for any primal to communicate with the Neural API. Consider absorbing
this pattern into a shared `biomeos-client` crate if multiple springs converge on it.

---

## Part 3: Platform-Agnostic Paths

### Before (V17)

```rust
PathBuf::from("/tmp")  // Hardcoded in 6 files
```

### After (V18)

```rust
std::env::temp_dir()   // Platform-agnostic via niche::socket_dirs()
```

Files changed: `discovery.rs`, `server.rs`, `biomeos/mod.rs`, `provenance.rs`,
`visualization/push_client.rs`, `bin/ludospring.rs`.

---

## Part 4: Quality at Handoff

| Check | Result |
|-------|--------|
| `cargo clippy --features ipc -p ludospring-barracuda` | 0 warnings |
| `cargo fmt --check` | 0 diffs |
| `cargo test --features ipc -p ludospring-barracuda` | 244 tests, 0 failures |
| `cargo check --workspace` | Clean (only external `loam-spine-core` unfulfilled lint) |
| TODO/FIXME/HACK in source | 0 |
| `#[allow()]` in production | 0 |
| `unsafe` blocks | 0 (workspace forbids) |
| External C dependencies | 0 |
| Hardcoded `/tmp` in production | 0 |
| Hardcoded primal names in production | 0 |

---

## Files Changed (V17 → V18)

| File | Change |
|------|--------|
| `barracuda/src/niche.rs` | **NEW** — centralized self-knowledge module |
| `barracuda/src/ipc/neural_bridge.rs` | **NEW** — typed Neural API client |
| `barracuda/src/lib.rs` | Added `niche` module, `PRIMAL_NAME` delegates to `niche::NICHE_NAME` |
| `barracuda/src/ipc/mod.rs` | Wired `NeuralBridge`, added public re-export |
| `barracuda/src/ipc/handlers.rs` | Delegates to `niche::CAPABILITIES`, `niche::NICHE_DOMAIN`, `niche::operation_dependencies()`, `niche::cost_estimates()` |
| `barracuda/src/ipc/server.rs` | Delegates to `niche::resolve_server_socket()` |
| `barracuda/src/ipc/discovery.rs` | `discovery_dirs()` delegates to `niche::socket_dirs()` |
| `barracuda/src/ipc/provenance.rs` | Refactored to use `NeuralBridge` instead of raw socket I/O |
| `barracuda/src/biomeos/mod.rs` | Re-exports from niche, uses `NeuralBridge` for registration |
| `barracuda/src/bin/ludospring.rs` | Uses `niche::*` for all resolution, removed duplicate RPC helpers |
| `barracuda/src/visualization/push_client.rs` | `temp_dir()` instead of `/tmp`, `NICHE_DOMAIN` from niche |

---

*We know ourselves. We discover others at runtime. We validate the math. You accelerate it.*
