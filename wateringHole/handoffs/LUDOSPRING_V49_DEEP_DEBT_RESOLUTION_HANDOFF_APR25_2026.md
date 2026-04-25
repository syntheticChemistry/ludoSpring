# ludoSpring V49 — Deep Debt Resolution Handoff

**From:** ludoSpring V49 (composition evolution partner)
**For:** primalSpring team, all spring teams, primal teams
**Date:** April 25, 2026
**Previous:** V47 (live NUCLEUS validated, 54/54), V48 (Phase 45c debt: BTSP relay, interaction.poll)

---

## What Changed (V48→V49)

V49 is a **deep debt resolution** pass — no new features, only structural quality
improvements that make the codebase ready for the next phase of live composition.

### 1. Handler Test Extraction (structural clarity)

`ipc/handlers/mod.rs` was 818 lines — 650+ were inline tests. Extracted to
`ipc/handlers/tests.rs`. Production dispatch module is now 169 lines. The test
file includes all 35+ handler integration tests. This is the pattern we recommend
for all springs: keep dispatch modules lean, move test bodies to `#[path]` files.

### 2. Capability-Based Discovery (self-knowledge principle)

**Before:** `validate_primal_proof.rs` contained hardcoded socket names:
`["barracuda-core.sock", "barracuda.sock", "compute.sock"]` and
`n.starts_with("barracuda")`.

**After:** Discovery queries `niche::DEPENDENCIES` for the `compute`/`tensor`
capability entry, then scans socket dirs for `{capability}.sock`,
`{dep_name}.sock`, or `{dep_name}-core.sock`. Falls back to pattern scanning
filtered by capability keyword.

`validate_composition.rs` derives fallback socket names from
`niche::NICHE_NAME` and `niche::NICHE_DOMAIN` constants instead of
hardcoded `"ludospring.sock"`.

**Pattern for primals team:** A primal should never search for another primal by
name. Search by capability. The `NicheDependency` table provides the mapping.

### 3. MCP Surface Complete (15/15 tools)

`tools.call` and `mcp_tools_descriptors` previously only exposed 13 of 15
`game.*` methods. Added:
- `game.record_action` — provenance DAG recording
- `game.voice_check` — AI voice personality consistency

This closes the MCP surface gap. Squirrel and other AI-capable primals can now
invoke the full ludoSpring capability set via the MCP protocol.

### 4. External Dependency Removal (base64)

Replaced the `base64 = "0.22"` crate (used only for BTSP `family_seed` encoding)
with a 20-line inline `base64_encode()` function. RFC 4648 test vectors validate
correctness. One fewer transitive dependency in the `ipc` feature graph.

**Pattern for primals team:** When the surface area of a dependency is small
(single function, single encoding), prefer an inline implementation over a crate.
This reduces supply chain surface and build times.

### 5. Typed Errors in BTSP (idiomatic Rust)

All `Result<_, String>` in `ipc/btsp.rs` evolved to `Result<_, IpcError>`:

| Function | Before | After |
|----------|--------|-------|
| `beardog_call` | `Result<Value, String>` | `Result<Value, IpcError>` |
| `write_json_line` | `Result<(), String>` | `Result<(), IpcError>` |
| `write_error_frame` | `Result<(), String>` | `Result<(), IpcError>` |
| `classify_first_line` | `Result<FirstLineResult, String>` | `Result<FirstLineResult, IpcError>` |
| `perform_handshake` | `Result<(), String>` | `Result<(), IpcError>` |

`IpcError` variants used: `Connect`, `Io`, `Serialization`, `RpcError`,
`NotFound`, `NoResult`. This enables smart retry logic in the caller — connect
failures are retriable, parse failures are not.

**Pattern for primals team:** The `IpcError` + `IpcErrorPhase` pattern
(originally from primalSpring, absorbed in V44) should be the standard for all
primal IPC code. `Result<_, String>` is a code smell in the IPC layer.

### 6. Named Constants

- `ACCEPT_POLL_MS = 50` replaces magic number in server accept loop
- `DEFAULT_FAMILY_ID = "default"` replaces inline string in `niche.rs`

---

## Quality Gates

| Metric | V48 | V49 |
|--------|-----|-----|
| Workspace tests | 798 | **799** |
| Clippy warnings | 0 | 0 |
| `unsafe` blocks | 0 | 0 |
| `#[allow()]` in prod | 0 | 0 |
| `Result<_, String>` in BTSP | 5 functions | **0** |
| MCP tools exposed | 13/15 | **15/15** |
| External deps (ipc feature) | 7 (incl. base64) | **6** |
| `handlers/mod.rs` lines | 818 | **169** |
| Hardcoded primal names in bins | 3 files | **0** |

---

## Composition Patterns Learned

### Pattern 1: Capability-Based Discovery Over Name-Based

```
// BAD: hardcoded primal name
for name in &["barracuda-core.sock", "barracuda.sock"] { ... }

// GOOD: query niche dependency table by capability
let dep = niche::DEPENDENCIES.iter().find(|d| d.capability == "compute");
let cap_sock = dir.join(format!("{}.sock", dep.capability));
```

This is critical for NUCLEUS deployment via neuralAPI: biomeOS may substitute
primals with different names but the same capability. The wire contract is the
capability, not the binary name.

### Pattern 2: Inline Small Encoders

The `base64` crate adds 15+ transitive crate compilations. For a single
`encode()` call, a 20-line inline function is simpler, faster to compile, and
has zero supply chain risk. The same applies to hex encoding, URL encoding,
and other small-surface utilities.

### Pattern 3: Test Extraction for Large Dispatch Modules

When a module exceeds ~200 lines and the bulk is tests, use:
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "tests.rs"]
mod tests;
```
This keeps the production module readable without losing test colocality.

### Pattern 4: Typed Errors at the IPC Boundary

Every function that crosses a socket boundary should return `IpcError`, not
`String`. The error variants encode retry semantics:
- `Connect` → retriable (primal may not be running yet)
- `Timeout` → retriable (primal may be under load)
- `Serialization` → not retriable (bad request)
- `RpcError` → inspect code (-32601 = method not found → wrong primal)
- `NotFound` → recoverable (try different discovery tier)

---

## NUCLEUS Deployment Implications

ludoSpring is cell-graph ready (`ludospring_cell.toml`, 14 nodes, all
`security_model = "btsp"`). V49 ensures the deploy path is clean:

1. **BTSP relay** (V48): `ludospring-barracuda` auto-detects and relays BTSP
   handshake to BearDog. Typed errors propagate correctly.
2. **Capability discovery** (V49): validation binaries find primals by capability,
   not by name. biomeOS can deploy any compute primal as long as it advertises
   the `compute` capability.
3. **MCP complete** (V49): Squirrel (or any MCP client) can invoke all 15
   `game.*` methods. No hidden capabilities.
4. **Interactive session** (V48): `interaction.poll` wired to petalTongue.
   The full loop: `game.push_scene → petalTongue → player → interaction.poll → game.record_action`.

### For neuralAPI / biomeOS team

- `register_capabilities_from_graphs()` can read ludoSpring's 30 capabilities
- `btsp.escalate` RPC for runtime cleartext→BTSP transition is supported
- All 14 cell graph nodes declare `security_model = "btsp"`
- The `ludospring-barracuda` binary is the single deploy artifact (UniBin with
  7 subcommands)

---

## For Upstream primalSpring Audit

### Code health
- Zero `TODO`/`FIXME`/`HACK`/`XXX` in all `.rs` files
- Zero `todo!`/`unimplemented!` in all `.rs` files
- Zero `#[allow(dead_code)]` in `barracuda/src/`
- Zero `.sh` scripts, zero `.py` files outside `baselines/python/`
- Zero temp/scratch/draft files
- `wateringHole/handoffs/archive/` contains historical handoffs (fossil record)

### Remaining gaps (GAP-01–GAP-11)
All documented in `docs/PRIMAL_GAPS.md`. No new gaps found in V49.
GAP-11 (barraCuda formulation divergence) is the only open semantic gap —
IPC expected values are adjusted to match barraCuda's mathematical formulation
rather than Python-aligned golden values.

### Evolution requests for upstream
1. **`primalspring::composition::base64_encode()`** — consider absorbing the
   inline encoder so springs don't each implement their own
2. **`IpcError` in all primalSpring examples** — some examples still use
   `Result<_, String>`, which contradicts the typed error standard
3. **Handler test extraction pattern** — document in `SPRING_COMPOSITION_PATTERNS`
   as §12 (test colocality pattern)

---

## Files Changed (V49)

| File | Change |
|------|--------|
| `barracuda/src/ipc/handlers/mod.rs` | 818→169L (tests extracted) |
| `barracuda/src/ipc/handlers/tests.rs` | **NEW** (extracted tests) |
| `barracuda/src/ipc/handlers/mcp.rs` | +2 tools (record_action, voice_check) |
| `barracuda/src/ipc/btsp.rs` | `Result<_, String>` → `IpcError`; inline base64 |
| `barracuda/src/ipc/server.rs` | `ACCEPT_POLL_MS` named constant |
| `barracuda/src/niche.rs` | `DEFAULT_FAMILY_ID` named constant |
| `barracuda/src/bin/validate_primal_proof.rs` | Capability-based discovery |
| `barracuda/src/bin/validate_composition.rs` | Niche-constant fallback names |
| `barracuda/Cargo.toml` | `base64` dep removed |
| All docs | V47→V49, 791→799 |
