# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring — Primal Gaps

**Last updated:** April 10, 2026 (V37.1 audit)
**Proto-nucleate:** `primalSpring/graphs/downstream/ludospring_proto_nucleate.toml`
**Composition model:** `pure` (no downstream binary — biomeOS deploys the graph)
**Fragments declared:** `tower_atomic`, `node_atomic`, `meta_tier`

---

## Gap Registry

### GAP-01: coralReef IPC Client Not Wired

**Primal:** coralReef
**Status:** NOT WIRED
**Proto-nucleate:** Required (`shader.compile`, `shader.list`)
**Impact:** Shader compilation path described in `game/engine/gpu.rs` docs but
no IPC client exists. All WGSL is currently embedded via `include_str!` and
dispatched through toadStool.

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
**Status:** Direct path dependency (`default-features = false`)
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
**Status:** Proto-nucleate includes NestGate node but `fragments` metadata lists
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
**Status:** `GpuContext::tensor_session()` imported but never called in production
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
**Status:** Typed IPC clients exist in `ipc/provenance/` but trio primals are
not listed as nodes in `ludospring_proto_nucleate.toml`.
**Impact:** biomeOS deploying the proto-nucleate graph won't spawn or discover
trio primals; provenance functionality depends on external graph composition.

**Proposed fix:** Add optional trio nodes to proto-nucleate or document that
provenance is composed via Nest Atomic overlay graph.
**Owner:** primalSpring graph maintainers
**Tracking:** Hand back to primalSpring

---

### GAP-06: rhizoCrypt TCP-Only Transport

**Primal:** rhizoCrypt
**Status:** rhizoCrypt only supports TCP transport, not UDS
**Impact:** 9 composition checks fail; all provenance pipeline operations blocked
in UDS-only deployments.
**Severity:** CRITICAL
**Owner:** rhizoCrypt team
**Tracking:** Documented in CONTEXT.md V37.1 gap matrix

---

### GAP-07: loamSpine Startup Panic

**Primal:** loamSpine
**Status:** Runtime nesting panic on startup
**Impact:** 6 composition checks fail
**Severity:** CRITICAL
**Owner:** loamSpine team
**Tracking:** Documented in CONTEXT.md V37.1 gap matrix

---

### GAP-08: barraCuda Fitts/Hick Formula Mismatch

**Primal:** barraCuda
**Status:** IPC-exposed Fitts/Hick formulas produce different values than
ludoSpring's validated implementations
**Impact:** 4 composition checks fail
**Severity:** HIGH
**Owner:** barraCuda team
**Tracking:** Documented in CONTEXT.md V37.1 gap matrix

---

## Gaps Handed Back to primalSpring

- **GAP-03** (fragment metadata) → `primalSpring/docs/PRIMAL_GAPS.md`
- **GAP-05** (trio not in proto-nucleate) → `primalSpring/docs/PRIMAL_GAPS.md`

## Gaps Handed to Primal Teams

- **GAP-06** (rhizoCrypt TCP-only) → rhizoCrypt team
- **GAP-07** (loamSpine panic) → loamSpine team
- **GAP-08** (barraCuda formula mismatch) → barraCuda team

---

**License:** AGPL-3.0-or-later
