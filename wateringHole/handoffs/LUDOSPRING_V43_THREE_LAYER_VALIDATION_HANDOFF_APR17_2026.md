# ludoSpring V43 — Three-Layer Validation & Primal Composition Handoff

**Date:** April 17, 2026
**From:** ludoSpring V43
**To:** barraCuda, toadStool, coralReef, biomeOS, petalTongue, Squirrel, NestGate, rhizoCrypt, loamSpine, sweetGrass, primalSpring, all spring teams
**License:** AGPL-3.0-or-later

---

## Executive Summary

ludoSpring now implements a complete three-layer validation stack proving that
peer-reviewed game science models work identically across Python baselines,
Rust library code, and NUCLEUS primal composition via IPC. This handoff
documents what we built, what we learned about each primal, and what we
recommend for ecosystem evolution.

### The Three Layers

| Layer | Artifact | Validates | Entry Point |
|-------|----------|-----------|-------------|
| 1 | `combined_baselines.json` | Python → Rust | `python_parity.rs`, `check_drift.py` |
| 2 | `composition_targets.json` | Rust library → golden targets | `composition_parity.rs`, `check_composition_drift` |
| 3 | IPC parity | Golden targets → primal composition | `validate_composition` binary |

### Key Numbers

- **790+** workspace tests (0 failed)
- **7** composition method groups with golden targets
- **6** Layer 2.5 integration tests
- **4** validation binaries (`validate_interaction`, `validate_procedural`, `validate_engagement`, `validate_composition`)
- **10** primal gaps tracked (GAP-01–GAP-10)
- **plasmidBin** v0.10.0 harvested, sha256-verified

---

## Part 1: What We Built

### validate_composition binary (Layer 3)

Loads `composition_targets.json`, discovers ludoSpring IPC socket, calls each
`game.*` method over JSON-RPC, compares to golden values within named
tolerances. Also validates `lifecycle.composition` report (model, fragments)
and `health.*` probes. Exit 0 (pass), 1 (fail), 2 (skip — server not running).

### composition_parity.rs (Layer 2.5)

Integration tests that parse `composition_targets.json` and call Rust library
functions directly. Catches drift between stored golden values and current
library output — the CI gate that prevents Layer 3 from testing stale targets.

### check_composition_drift (drift detector)

Rust example analogous to `check_drift.py`. Recomputes all targets from
library calls, compares to stored JSON. Runs in CI after tests.

### composition_targets.json (golden values)

7 method groups: `game.evaluate_flow`, `game.fitts_cost` (including Hick and
steering variants), `game.engagement`, `game.generate_noise`,
`game.difficulty_adjustment`, `game.accessibility`, `game.wfc_step`.
Provenance block with tolerance definitions and method inventory.

---

## Part 2: Per-Primal Learnings & Recommendations

### barraCuda (tensor math)

**Usage:** CPU math centralized through `crate::barcuda_math` re-exports
(sigmoid, dot, lcg). `TensorSession` sigmoid batch wired for GPU parity.

**What worked:** Clean delegation pattern — ludoSpring never reinvents math
that barraCuda provides. Tolerance constants sourced from barraCuda where
applicable.

**Gaps found:**
- `TensorSession` did not expose `sigmoid` — we implemented `SessionOp::Sigmoid`
  upstream. Other springs will benefit.
- `gpu_context.rs` comment says product code doesn't exercise `TensorSession` but
  `tensor_ops.rs` now does — stale comment (fixed locally).

**Recommendation for barraCuda team:**
- Absorb: Perlin 2D/3D + fBm (~200 LOC), WFC (~265 LOC), BSP (~220 LOC)
- Stabilize `TensorSession` unary op set with documented numerical expectations
- Consider absorbing `ValidationHarness` + `ValidationSink` pattern (~400 LOC)

### toadStool (compute dispatch)

**Usage:** `compute.dispatch.submit` for GPU workloads, `compute.capabilities`
for substrate detection. Product path: `game.gpu.*` handlers → toadStool.

**What worked:** Dispatch-first approach (not queued `submit`) meets 60Hz budget.

**Gaps found:**
- Deploy graph uses `toadstool.health` vs code's `compute.health` — naming mismatch
- `submit_workload` / `workload_status` unused by handlers (drift risk)

**Recommendation for toadStool team:**
- Stabilize `compute.dispatch.submit` buffer layout / response fields
- One canonical capability naming scheme across deploy TOMLs and wire protocol

### coralReef (shader compiler)

**Usage:** `shader.compile` / `shader.list` client ready but not integrated
into GPU path. Fallback: embedded WGSL + ToadStool/wgpu.

**Gaps found:**
- Niche marks coralReef `required: true` while runtime doesn't depend on it —
  `lifecycle.composition` reports "missing required" in environments without coralReef

**Recommendation for coralReef team:**
- Document `shader.compile` return contract (binary handle vs SPIR-V vs cache key)
- ludoSpring will integrate when contract stabilizes (or we'll set `required: false`)

### biomeOS / Neural API (orchestration)

**Usage:** `NeuralBridge` for capability routing, domain registration via
`biomeos::register_domain`. 30 capabilities registered.

**What worked:** Capability-first discovery with 6-tier fallback.

**Gaps found:**
- `provenance::has_active_session()` only checks socket path exists, not game
  session state — misleading for lifecycle callers
- Registration ack JSON shape undocumented — springs can't distinguish "routed but
  primal failed" vs "Neural API down"

**Recommendation for biomeOS team:**
- Versioned error shapes for `capability.call` failures
- Stable registration ack contract

### Squirrel (AI inference)

**Usage:** `ai.query`, `ai.suggest`, `ai.analyze` for NPC dialogue, narration,
voice check. `InferenceCompleteRequest` types defined but `inference.*`
wrappers not yet wired.

**Gaps found:**
- Chat-completion JSON varies across backends — `squirrel_chat_metadata` needs
  multiple fallback fields (content/text/message/summary)
- Context helpers (`context.create`, `context.update`) exist without `game.*` IPC

**Recommendation for Squirrel/neuralSpring team:**
- Standardize chat-completion response JSON (model, usage/tokens fields)
- Document context session lifecycle for springs that need memory windows

### petalTongue (visualization)

**Usage:** Direct UDS to peer advertising `visualization.render`. Scene push
for RPGPT channels, dashboards, streaming sessions.

**Gaps found:**
- `game.push_scene` returns `pushed: true` even if discovery fails (errors
  swallowed with `let _ =`)

**Recommendation for petalTongue team:**
- Keep `visualization.render` advertisement stable
- Document scene JSON schema for multi-channel rendering

### NestGate (storage)

**Usage:** `storage.store`, `storage.retrieve` via Neural API. Handlers:
`game.storage_put`, `game.storage_get`.

**Gaps found:**
- `exists`, `list`, `metadata`, `delete` implemented in client but not exposed
  as `game.*` capabilities in niche — unused API surface

**Recommendation for NestGate team:**
- Consistent key/content-hash semantics in responses

### Provenance Trio (rhizoCrypt + loamSpine + sweetGrass)

**Usage:** Session lifecycle: rhizoCrypt DAG → dehydration → sweetGrass
attribution → loamSpine certificate mint → braid creation.

**Gaps found:**
- **GAP-06:** rhizoCrypt has no UDS transport (blocks 4 experiments)
- **GAP-07:** loamSpine startup panic (runtime nesting, blocks 1 experiment)
- sweetGrass `record_dehydration` errors swallowed — partial attribution failures hidden

**Recommendation for trio teams:**
- Prioritize UDS transport for rhizoCrypt (unblocks nest_atomic composition)
- Fix loamSpine startup (tokio runtime nesting)
- Document dehydration summary wire types (`merkle_root`, `frontier`, `vertex_count`)

---

## Part 3: Composition Patterns for NUCLEUS Deployment

### Pattern: Three-Layer Golden Chain

Every spring should implement:
1. **Python baselines** with provenance (script, commit, date, command)
2. **Rust library** matching Python within named tolerances
3. **composition_targets.json** produced by Rust generator (golden values)
4. **Layer 2.5 tests** consuming JSON against library (drift guard)
5. **validate_composition** binary consuming JSON against IPC (composition proof)

### Pattern: Honest Skip (exit 2)

When primals aren't running, validators should exit 2 (skip) — never fake a
pass. `validate_all` treats exit 2 as skip, not failure. This allows CI to
run validation binaries without a full NUCLEUS stack.

### Pattern: Named Tolerances in Golden JSON

`composition_targets.json._provenance.tolerances` defines named tolerance
tiers (analytical, game_state, noise_mean). Validators and tests reference
these by name, never hardcode literals.

### Pattern: Capability-First Discovery

`discover_by_capability(dep.capability)` first, `discover_primal_tiered(dep.name)`
fallback. Deploy graphs should use `by_capability` not primal names where possible.

### Pattern: Fragment Resolution

Deploy graphs use `resolve = true` in `[graph.metadata]` to inherit from
NUCLEUS fragments. Custom profiles apply only delta nodes.

---

## Part 4: Actions for Receiving Teams

| Team | Action | Priority |
|------|--------|----------|
| **barraCuda** | Absorb Perlin/fBm/WFC/BSP; stabilize TensorSession ops | P1 |
| **barraCuda** | Consider absorbing ValidationHarness pattern | P2 |
| **toadStool** | Stabilize compute.dispatch.submit contract; fix naming | P1 |
| **coralReef** | Document shader.compile return contract | P2 |
| **biomeOS** | Versioned capability.call error shapes | P1 |
| **Squirrel** | Standardize chat-completion JSON | P2 |
| **petalTongue** | Propagate push errors; document scene schema | P2 |
| **NestGate** | Consistent key/hash semantics | P3 |
| **rhizoCrypt** | UDS transport (GAP-06) | P0 |
| **loamSpine** | Fix startup panic (GAP-07) | P0 |
| **sweetGrass** | Surface attribution failures | P2 |
| **primalSpring** | Adopt three-layer pattern; update composition guidance | P1 |
| **All springs** | Copy ludoSpring's validate_composition pattern | P2 |

---

## Part 5: Deploy Graph Alignment Items

1. `ludospring_deploy.toml` capabilities list is a subset of `niche::CAPABILITIES` (27 FQNs) — should enumerate all or document intentional subset
2. `ludospring_gaming_niche.toml` uses `toadstool.health` vs code's `compute.health` — naming mismatch
3. Both graphs now have `resolve = true` and fragment metadata — correct

---

**License:** AGPL-3.0-or-later
