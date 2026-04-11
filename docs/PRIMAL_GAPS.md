# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring — Primal Gaps

**Last updated:** April 11, 2026 (proto-nucleate + code review)
**Proto-nucleate:** `primalSpring/graphs/downstream/ludospring_proto_nucleate.toml`
**Composition model:** `pure` (no downstream binary — biomeOS deploys the graph)
**Fragments declared:** `tower_atomic`, `node_atomic`, `meta_tier`

---

## Gap Registry

### GAP-01: coralReef IPC Client Not Wired

**Primal:** coralReef
**Status:** PARTIAL — typed client exists; product engine path still not wired
**Proto-nucleate:** Required (`shader.compile`, `shader.list`)
**Impact:** `barracuda/src/ipc/coralreef.rs` exposes `compile_wgsl` / `list_shaders`
via NeuralBridge `capability_call` to the shader domain, and `experiments/exp085_shader_dispatch_chain`
exercises compile → dispatch. Production GPU paths in `game/engine/gpu.rs` still
load WGSL with `include_str!` and dispatch through toadStool — they do not invoke
the coralReef client.

**Proposed wire:**
```json
{"method": "shader.compile.wgsl", "params": {"source": "...", "entry_point": "main"}}
```

**Workaround:** Embedded WGSL + toadStool `compute.dispatch.submit`.
**Owner:** coralReef team / ludoSpring
**Tracking:** This file

---

### GAP-02: barraCuda Direct Rust Import (Not IPC)

**Primal:** barraCuda
**Status:** OPEN — direct path dependency (`default-features = false`);
compile-time `barracuda::` usage unchanged (April 2026 review)
**Proto-nucleate:** Required via IPC (`tensor.fitts`, `tensor.flow_sigmoid`, etc.)
**Impact:** For `composition_model = "pure"`, barraCuda should be called via
`tensor.*` capability IPC through biomeOS, not via compile-time Rust imports.

**Current usage:**
- `barracuda::activations::sigmoid` in `interaction/flow.rs`
- `barracuda::stats::dot` in `metrics/engagement.rs`
- `barracuda::rng::lcg_step` in `procedural/bsp.rs`
- `barracuda::device::WgpuDevice` + `barracuda::session::TensorSession` in `gpu_context.rs`

**Migration path:** Replace direct imports with `capability_call("tensor", op, args)`
as barraCuda IPC surface matures. Keep path dep for validation binaries.
**Owner:** barraCuda IPC surface / ludoSpring
**Tracking:** This file

---

### GAP-03: Fragment Metadata Missing `nest_atomic`

**Primal:** NestGate (partial Nest)
**Status:** OPEN — unchanged in `ludospring_proto_nucleate.toml` (see GAP-09)
**Proto-nucleate:** NestGate node is present, but `fragments` metadata lists
only `["tower_atomic", "node_atomic", "meta_tier"]`.
**Impact:** Structural audit tools that check fragment consistency will miss the
Nest dependency. NestGate without provenance trio is partial Nest.

**Proposed fix:** Either:
1. Add `nest_atomic` to fragments (if NestGate implies Nest participation), or
2. Document that NestGate-without-trio is a valid partial composition

**Owner:** primalSpring graph maintainers
**Tracking:** Hand back to `primalSpring/docs/PRIMAL_GAPS.md`

---

### GAP-04: TensorSession Not Exercised in Product Paths

**Primal:** barraCuda
**Status:** OPEN — `GpuContext::tensor_session()` exists but has no call sites
in production code (April 2026 review)
**Impact:** GPU promotion story (Tier A shader rewire to `TensorSession` fused
ops) is infrastructure-only; no validation that the composition actually works
end-to-end through `TensorSession`.

**Proposed validation:** Wire `TensorSession` for at least one Tier A op
(sigmoid or dot) in a game engine path and add a validation experiment.
**Owner:** ludoSpring
**Tracking:** This file

---

### GAP-05: Provenance Trio Not in Proto-Nucleate

**Primal:** rhizoCrypt, loamSpine, sweetGrass
**Status:** OPEN — typed IPC clients in `ipc/provenance/`; still no trio nodes
in `ludospring_proto_nucleate.toml` (see GAP-09)
**Impact:** biomeOS deploying the proto-nucleate graph won't spawn or discover
trio primals; provenance functionality depends on external graph composition.

**Proposed fix:** Add optional trio nodes to proto-nucleate or document that
provenance is composed via Nest Atomic overlay graph.
**Owner:** primalSpring graph maintainers
**Tracking:** Hand back to primalSpring

---

### GAP-06: rhizoCrypt TCP-Only Transport

**Primal:** rhizoCrypt
**Status:** OPEN (upstream) — no ludoSpring-side fix; still TCP-only vs
proto-nucleate `transport = "uds_only"`
**Impact:** 9 composition checks fail; all provenance pipeline operations blocked
in UDS-only deployments.
**Severity:** CRITICAL
**Owner:** rhizoCrypt team
**Tracking:** Documented in CONTEXT.md V37.1 gap matrix

---

### GAP-07: loamSpine Startup Panic

**Primal:** loamSpine
**Status:** OPEN (upstream) — treat as unresolved pending loamSpine release
**Detail:** Runtime nesting panic on startup
**Impact:** 6 composition checks fail
**Severity:** CRITICAL
**Owner:** loamSpine team
**Tracking:** Documented in CONTEXT.md V37.1 gap matrix

---

### GAP-08: barraCuda Fitts/Hick Formula Mismatch

**Primal:** barraCuda
**Status:** OPEN (upstream) — ludoSpring still uses in-crate validated math;
IPC parity not verified
**Detail:** IPC-exposed Fitts/Hick formulas produce different values than
ludoSpring's validated implementations
**Impact:** 4 composition checks fail
**Severity:** HIGH
**Owner:** barraCuda team
**Tracking:** Documented in CONTEXT.md V37.1 gap matrix

---

### GAP-09: `nest_atomic` Fragment Missing vs. Nest-Side IPC Stubs

**Primals:** NestGate, rhizoCrypt, loamSpine, sweetGrass (nest-side surface)
**Status:** OPEN — graph/code mismatch
**Proto-nucleate:** `fragments` declares only `tower_atomic`, `node_atomic`,
`meta_tier` — there is **no** `nest_atomic` fragment. The graph includes a
`nestgate` node for `storage.*`, but not the provenance trio.
**Code:** ludoSpring ships local IPC stubs for the full nest-adjacent surface:
`ipc/nestgate.rs`, `ipc/provenance/rhizocrypt.rs`, `ipc/provenance/loamspine.rs`,
`ipc/provenance/sweetgrass.rs` (storage plus provenance).
**Impact:** Audit and fragment-consistency tooling cannot classify this
deployment as `nest_atomic`, while the codebase already anticipates those
capabilities. Unclear whether nest-side stubs are first-class composition targets
or aspirational until the graph declares them.
**Reconciliation:** Either upgrade `ludospring_proto_nucleate.toml` to add
`nest_atomic` to `fragments` and (when ready) optional trio nodes, **or**
document the stubs explicitly as aspirational / validation-only until the
proto-nucleate (or an overlay graph) includes them.
**Related:** GAP-03, GAP-05
**Owner:** primalSpring graph maintainers / ludoSpring
**Tracking:** This file

---

### GAP-10: `game.*` Primal Identity — ludoSpring Not a Graph Node

**Primal / domain:** `game` capability namespace (ludoSpring IPC surface)
**Status:** OPEN — architecture gap
**Proto-nucleate:** Declares `barracuda` with `by_capability = "tensor"` and
`tensor.*` methods for game math; NestGate covers `storage.*`. There is **no**
graph node whose registration advertises the **`game`** domain for methods such
as `game.evaluate_flow`, `game.fitts_cost`, `game.engagement`, `game.wfc_step`,
etc. Those methods are implemented by the ludoSpring barracuda IPC server
(exposed via sockets such as `ludospring.sock` in composition experiments).
**Impact:** After biomeOS deploys the proto-nucleate graph, discovery can resolve
tensor and storage primals, but **`game.*`** routing to ludoSpring as the
capability provider is not described by the graph. ludoSpring must be
addressable as the **`game.*`** provider for pure-composition game science.
**Reconciliation:** Add a node (or deployment manifest) that registers
ludoSpring with `by_capability` / capability list for `game.*`, **or** document
the biomeOS rule that maps the deployed graph to the ludoSpring process for
`game.*` dispatch.
**Owner:** biomeOS + primalSpring (+ ludoSpring for method contracts)
**Tracking:** This file; consider `primalSpring/docs/PRIMAL_GAPS.md`

---

## Gaps Handed Back to primalSpring

- **GAP-03** (fragment metadata) → `primalSpring/docs/PRIMAL_GAPS.md`
- **GAP-05** (trio not in proto-nucleate) → `primalSpring/docs/PRIMAL_GAPS.md`
- **GAP-09** (`nest_atomic` vs. stubs) → `primalSpring/docs/PRIMAL_GAPS.md`
- **GAP-10** (`game.*` graph identity) → `primalSpring/docs/PRIMAL_GAPS.md`

## Gaps Handed to Primal Teams

- **GAP-06** (rhizoCrypt TCP-only) → rhizoCrypt team
- **GAP-07** (loamSpine panic) → loamSpine team
- **GAP-08** (barraCuda formula mismatch) → barraCuda team

---

**License:** AGPL-3.0-or-later
