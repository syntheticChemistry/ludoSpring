# ludoSpring V33 — Neural API E2E Pipeline Handoff

**Date:** March 29, 2026
**From:** ludoSpring
**To:** biomeOS, BearDog, Songbird, ToadStool, esotericWebb, all springs
**Version:** V33

---

## Summary

ludoSpring V33 achieves the first **live Neural API e2e pipeline** — real cryptographic
operations (Blake3, SHA3-256, ChaCha20-Poly1305), network discovery, and GPU compute
dispatch all routed through biomeOS `capability.call` to live primal processes.

**Key milestone:** This is no longer structural validation. Real BearDog crypto,
real Songbird networking, and real ToadStool compute dispatch are running on Unix
sockets and responding to JSON-RPC through the Neural API orchestration layer.

---

## What Was Validated

### Phase 1: Tower Atomic (BearDog + Songbird)

| Check | Status | Details |
|-------|--------|---------|
| BearDog server startup | PASS | Unix socket, FAMILY_ID=dev0, NODE_ID=tower1 |
| Songbird server startup | PASS | Unix socket, BEARDOG_SOCKET discovery |
| Discovery module probes real primals | PASS | Multi-format capability extraction |
| exp042 9/9 live checks | PASS | First real-primal validation run |

### Phase 2: Neural API Routing

| Check | Status | Details |
|-------|--------|---------|
| biomeOS neural-api startup | PASS | Bootstrap mode → manual capability registration |
| capability.register (11 domains) | PASS | crypto, security, network, compute, etc. |
| capability.call crypto.blake3_hash | PASS | Real hash through routing layer |
| capability.call crypto.encrypt/decrypt | PASS | Full ChaCha20-Poly1305 roundtrip |
| capability.call songbird reachability | PASS | Network primal routed through neural-api |
| capability.list includes all domains | PASS | 16 capabilities registered |
| exp083 10/10 Neural API e2e | PASS | Full pipeline validation experiment |

### Phase 3: Node (Tower + ToadStool)

| Check | Status | Details |
|-------|--------|---------|
| ToadStool server startup | PASS | JSON-RPC + tarpc dual-socket |
| GPU detection (RTX 4060) | PASS | Vulkan + DRM, sovereign pipeline |
| compute.dispatch.capabilities via Neural | PASS | DRM mode, nvidia driver detected |

---

## Code Changes

### barracuda/src/ipc/discovery.rs

- **Multi-probe fallback**: `lifecycle.status` → `health.check` + `capabilities.list`
- **Format E (BearDog)**: `provided_capabilities` with `type` + `methods` objects
- **Format F (Songbird)**: Top-level flat capability arrays
- **Semantic aliases**: `crypto` → `crypto.hash`, `crypto.encrypt`, `crypto.sign`, `crypto.verify`
- **Base capabilities**: Auto-inject `system.ping`, `health.check`, `health.liveness`
- Tests: 15 → 19

### experiments/exp083_neural_api_e2e/

New 10-check experiment validating full Neural API pipeline. Requires live primals.

### esotericWebb/webb/src/ipc/bridge/mod.rs

- Added `neural_api: Option<PrimalClient>` field to `PrimalBridge`
- `discover()` now resolves Neural API socket via `niche::resolve_neural_api_socket()`
- `neural_api_call()` translates domain+method to `capability.call`
- `resilient_call()` falls back to Neural API when direct client absent
- `has()` returns true if Neural API available even without direct connection
- 322 tests pass, zero regressions

---

## For biomeOS Team

1. **Bootstrap gap**: Neural API starts in bootstrap mode and doesn't auto-discover
   primals already on the socket directory. Lifecycle monitor runs every 10s but
   only checks `{primal}-{family_id}.sock` naming, not `{primal}.sock`. Consider
   scanning for both patterns.

2. **Registration protocol**: `capability.register` requires `{"capability": "X", "primal": "name", "socket": "/path"}`.
   This is manual. For auto-discovery, the bootstrap graph (`tower_atomic_bootstrap.toml`)
   needs to exist in the graphs directory, or biomeOS needs socket-dir scanning at startup.

3. **Socket naming**: BearDog defaults to `beardog.sock`, biomeOS expects `beardog-{family_id}.sock`.
   Symlinks work as a bridge but the convention should converge.

---

## For BearDog Team

1. **`lifecycle.status`**: Not implemented. ludoSpring's discovery now falls back to
   `health.check` + `capabilities.list`, but biomeOS may also probe with `lifecycle.status`.
   Consider adding it as an alias for `health.check` + capabilities.

2. **`crypto.hash`**: BearDog responds to this method but doesn't list it in
   `capabilities.list` / `provided_capabilities`. The `provided_capabilities` format
   lists individual algorithm methods (`blake3_hash`, etc.) but not the generic dispatcher.

3. **Key requirement for encrypt**: `chacha20_poly1305_encrypt` requires a `key` parameter.
   Neural API callers need to know this — consider documenting the key lifecycle
   (generate → encrypt → decrypt) in the capability schema.

---

## For ToadStool Team

1. **JSON-RPC socket naming**: Currently `toadstool.jsonrpc.sock` (separate from tarpc).
   Discovery expects `toadstool.sock` → gets the tarpc socket. Consider making the
   JSON-RPC socket the primary `.sock` or adding env override `TOADSTOOL_JSONRPC_SOCKET`.

2. **`health.check` / `capabilities.list`**: Not implemented on JSON-RPC socket.
   Discovery probe returns empty results. These should be added for ecosystem compatibility.

---

## For esotericWebb Team

1. **Neural API path is wired but unused by default**: `PrimalBridge::discover()` now
   tries `resolve_neural_api_socket()` after domain discovery. When the neural-api
   socket exists and is reachable, `resilient_call()` falls back to it for any domain
   without a direct client.

2. **Testing needed**: Launch esotericWebb with a running Tower + Neural API and verify
   that game/AI/compute calls route correctly through the orchestration layer.

---

## Next Steps

- **Phase 3**: Full NUCLEUS (NestGate + Squirrel + provenance trio)
- **Phase 4**: esotericWebb CRPG session consuming ludoSpring via Neural API
- **Phase 5**: benchScale multi-node deployment with agentReagents
- **Bootstrap automation**: biomeOS `nucleus` command with auto-discovery
- **Socket convention convergence**: `{primal}-{family_id}.sock` everywhere
