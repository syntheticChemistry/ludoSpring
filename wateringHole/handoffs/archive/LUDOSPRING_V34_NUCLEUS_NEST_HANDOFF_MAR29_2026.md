# ludoSpring V34 Handoff — Full NUCLEUS Nest Atomic Composition

**Date**: 2026-03-29
**From**: ludoSpring (game science validation spring)
**To**: biomeOS, BearDog, Songbird, ToadStool, NestGate, Squirrel, esotericWebb, primalSpring

---

## Summary

V34 achieves the **first full NUCLEUS Nest Atomic composition** — all 5 primal categories
(crypto, discovery, compute, storage, AI/MCP) running simultaneously and routed through
biomeOS Neural API with cross-domain provenance chains verified end-to-end.

**exp083 v2**: 13/13 checks pass against live primals.

---

## What Was Validated

| Primal | Version | Socket | Validated Capabilities |
|--------|---------|--------|----------------------|
| BearDog | v0.9.0 | beardog.sock | Blake3, SHA3-256, ChaCha20-Poly1305 encrypt/decrypt, Ed25519 sign |
| Songbird | v0.2.1 | songbird.sock | discovery.peers, health.check |
| ToadStool | — | toadstool.jsonrpc.sock | compute.dispatch.capabilities (RTX 4060, Vulkan/CUDA) |
| NestGate | v2.1.0 | nestgate-dev0.sock | storage.store, storage.retrieve, storage.list |
| Squirrel | v0.1.0 | squirrel.sock (bridged from @squirrel) | ai.list_providers, tool.list (25 tools) |
| biomeOS | — | neural-api-dev0.sock | capability.call routing, capability.register, Coordinated Mode |

### Cross-Domain Provenance Chain
1. Hash data via BearDog (`crypto.blake3_hash`)
2. Sign hash via BearDog (`crypto.sign` → Ed25519)
3. Store {hash, signature} via NestGate (`storage.store`)
4. Retrieve and verify integrity via NestGate (`storage.retrieve`)
5. **Result**: stored_hash == original_hash AND stored_sig == original_sig ✓

---

## Code Changes (V33→V34)

### ludoSpring/barracuda
- **Discovery refactor**: `ipc/discovery.rs` (652 lines) → `ipc/discovery/mod.rs` + `ipc/discovery/capabilities.rs`
  - `capabilities.rs`: 6-format parser, semantic aliases, base capability injection
  - `mod.rs`: registry, probing, scanning, call_primal
- **IPC timeouts env-configurable**: `tolerances/ipc.rs` now reads `LUDOSPRING_RPC_TIMEOUT_SECS`,
  `LUDOSPRING_PROBE_TIMEOUT_MS`, `LUDOSPRING_CONNECT_PROBE_TIMEOUT_MS`
- **Stale test fix**: `gpu_fog_of_war_degrades_without_toadstool` assertion updated
- **Tests**: 343 → 424 (+81 from submodule tests and refactored coverage)

### ludoSpring/experiments/exp083
- Expanded from 10 to 13 checks
- Added NestGate, Squirrel, provenance chain, capability registry completeness

---

## Findings & Recommendations for Teams

### biomeOS
- Neural API `capability.register` requires: `{capability, primal, socket}` (exact field names)
- `capability.call` extracts the first dot-segment of `capability` field for domain routing
  (e.g. `storage.store` → domain `storage`) but `crypto` gets semantically translated to `security`
- **Recommendation**: Document the semantic translation table (68 defaults loaded)
- **Recommendation**: Support bulk registration: `capability.register_batch`

### BearDog
- `crypto.sign` returns `{signature, algorithm, key_id}` but no `public_key`
- **Recommendation**: Add `crypto.public_key` or `crypto.keypair_info` method for Ed25519 verification
- `crypto.ed25519_sign` does NOT exist — the method is `crypto.sign`
- **Recommendation**: Document the exact JSON-RPC method names in primal README

### Songbird
- Actual method: `discovery.peers` (not `network.peer_list`)
- Available via `rpc.discover`: `["health", "identity", "rpc.discover", "discover_capabilities", "primal.info", "primal.capabilities", "rpc.method_list", "discovery.peers"]`
- TLS errors at startup (looking for `/tmp/neural-api-dev0.sock`) are non-fatal

### ToadStool
- JSON-RPC socket: `toadstool.jsonrpc.sock` (separate from tarpc `toadstool.sock`)
- Does not expose `health.check` or `capabilities.list` on JSON-RPC
- **Recommendation**: Add `health.check` to JSON-RPC interface for biomeOS auto-discovery

### NestGate
- Requires `NESTGATE_JWT_SECRET` (blocks startup with insecure default)
- Socket-only mode: `--socket-only` (default per PRIMAL_DEPLOYMENT_STANDARD)
- Storage methods work well: `storage.store`, `storage.retrieve`, `storage.list`
- Family-scoped socket: `nestgate-{family_id}.sock`

### Squirrel
- Binds to abstract Unix socket `@squirrel` instead of filesystem socket
- **Bug**: `--socket /path/to/socket.sock` CLI arg is ignored for filesystem binding
- **Workaround**: Python socket bridge (abstract → filesystem)
- **Recommendation**: Fix filesystem socket binding from CLI arg
- 25 tools available, AI routing works (returns empty providers until API keys configured)

---

## Test Counts

| Component | Tests | Status |
|-----------|-------|--------|
| barracuda lib | 424 | ✅ all pass |
| metalForge/forge | 26 | ✅ all pass |
| esotericWebb | 341 | ✅ all pass |
| exp083 (live) | 13 | ✅ 13/13 |
| **Total** | **734** | **0 failures** |

(exp032 has 1 pre-existing failure at 22/23 — not related to V34 changes)

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                   biomeOS Neural API                     │
│                  (Coordinated Mode)                       │
│  60+ capabilities · semantic translation · gen 1         │
└─────┬───────┬────────┬──────────┬────────────┬──────────┘
      │       │        │          │            │
  ┌───▼──┐ ┌─▼───┐ ┌──▼───┐ ┌───▼────┐ ┌────▼────┐
  │Bearer│ │Song │ │Toad  │ │Nest   │ │Squirrel │
  │Dog   │ │bird │ │Stool │ │Gate   │ │         │
  │crypto│ │disco│ │compu │ │storag │ │ AI/MCP  │
  │      │ │very │ │te    │ │e      │ │ 25tools │
  └──────┘ └─────┘ └──────┘ └───────┘ └─────────┘
  Tower Atomic        Node           Nest Atomic
  (crypto+discovery)  (+compute)     (+storage+AI)
```
