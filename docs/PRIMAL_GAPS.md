# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring — Primal Gaps

**Last updated:** April 25, 2026 (V52 — Game tick loop: `game.tick` composite handler, `game.subscribe_interaction`, `game.poll_interaction` with `is_skip_error` degradation; 30 capabilities; `ludospring_cell.toml` NUCLEUS cell graph. 817 tests, zero clippy.)
**Proto-nucleate:** `primalSpring/graphs/downstream/downstream_manifest.toml` (ludospring entry)
**Cell graph:** `primalSpring/graphs/cells/ludospring_cell.toml` (14 nodes, all `security_model = "btsp"`)
**Composition model:** `pure` (no downstream binary — biomeOS deploys the graph)
**Fragments declared:** `tower_atomic`, `node_atomic`, `nest_atomic`, `meta_tier`

### Composition Patterns Absorbed (April 11, 2026)

| Pattern | Source | Location |
|---|---|---|
| `IpcErrorPhase` + `PhasedIpcError` | primalSpring `ecoPrimal/src/ipc/error.rs` | `ipc/envelope.rs` |
| `classify_io_error` + upstream query methods | primalSpring v0.9.17 `ecoPrimal/src/ipc/error.rs` | `ipc/envelope.rs` (V50) |
| Method normalization (`normalize_method`) | `SPRING_COMPOSITION_PATTERNS` §1 | `ipc/envelope.rs` + `ipc/handlers/mod.rs` |
| Two-tier dispatch (lifecycle / infra / science) | `SPRING_COMPOSITION_PATTERNS` §4 | `ipc/handlers/mod.rs` |
| Tiered discovery (`DiscoveryTier`, `DiscoveryResult`) | `SPRING_COMPOSITION_PATTERNS` §3 | `ipc/discovery/mod.rs` |
| `NicheDependency` table | `SPRING_COMPOSITION_PATTERNS` §11 | `niche.rs` |
| Typed inference wire types | neuralSpring `inference.*` | `ipc/squirrel.rs` |
| `CompositionReport` + live validation | `SPRING_COMPOSITION_PATTERNS` §5 | `ipc/composition.rs` |
| `--rpc-bind` CLI flag | genomeBin startup contract (was `--port`) | `bin/ludospring.rs` |
| `is_retriable` / `is_recoverable` / `is_method_not_found` | primalSpring `PhasedIpcError` | `ipc/envelope.rs` |
| `lifecycle.composition` handler | `SPRING_COMPOSITION_PATTERNS` §5 | `ipc/handlers/lifecycle.rs` |
| Capability-first composition probing (`by_capability` → name fallback) | `SPRING_COMPOSITION_PATTERNS` §3 | `ipc/composition.rs` |
| `nest_atomic` in declared fragments | NUCLEUS alignment | `ipc/composition.rs` + `config/capability_registry.toml` |
| `ipc::methods` constants (viz, interaction, health, lifecycle, capability) | primalSpring v0.9.17 `ecoPrimal/src/ipc/methods.rs` | `ipc/methods.rs` (V51) |
| `is_skip_error` graceful degradation pattern | primalSpring v0.9.17 `ecoPrimal/src/composition/mod.rs` | `ipc/envelope.rs` (V51) |
| Constant-based dispatch routing (zero hardcoded method strings) | primalSpring v0.9.17 composition patterns | `handlers/mod.rs`, `handlers/neural.rs`, `push_client.rs` (V51) |
| `game.tick` composite handler (push→poll→record→metrics) | primalSpring `CompositionContext` loop pattern | `handlers/delegation.rs` (V52) |
| `game.subscribe_interaction` / `game.poll_interaction` | petalTongue interaction domain | `handlers/delegation.rs` (V52) |
| `handle_push_scene` semantic `is_skip_error` degradation | primalSpring `call_or_skip` pattern | `handlers/delegation.rs` (V52) |
| `ludospring_cell.toml` NUCLEUS cell graph (14 nodes) | primalSpring cell deployment pattern | `graphs/ludospring_cell.toml` (V52) |

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
**Status:** PARTIAL → guideStone readiness 4, live NUCLEUS validated (V47, April 20 2026).
`ludospring_guidestone` binary — three-tier architecture (54/54 checks, exit 0):
Tier 1 (bare, 31 checks): 20 structural + 11 BLAKE3 integrity.
Tier 2 (IPC, 15 checks): domain science via `primalspring::composition` with upstream
`call_or_skip`/`is_skip_error` (guideStone standard v1.2.0). 13 pass + 2 skip.
Tier 3 (NUCLEUS, 8 checks): BearDog `crypto.hash` (base64), NestGate
`storage.store`/`storage.retrieve` roundtrip, cross-atomic pipeline
(hash→store→retrieve→verify). See GAP-11 for formulation divergences.
`validate_primal_proof` (raw IPC) retained for comparison. Library path dep
retained for Level 2 tests.
**Proto-nucleate:** Required via IPC — 19 methods in downstream manifest.
Domain-level methods (`math.flow.evaluate`, `math.engagement.composite`)
remain gaps.
**Impact:** guideStone readiness 4 (NUCLEUS validated). All upstream blockers
resolved per guideStone standard v1.2.0 (rhizoCrypt PG-32, barraCuda Sprint 44,
loamSpine). Full certification (Level 5) requires all Tier 2 + Tier 3 checks
passing against live NUCLEUS with cross-substrate parity.

**Current usage (library dep — Level 2 validation):**
- `barracuda::activations::sigmoid` in `interaction/flow.rs`
- `barracuda::stats::dot` in `metrics/engagement.rs`
- `barracuda::rng::lcg_step` in `procedural/bsp.rs`
- `barracuda::device::WgpuDevice` + `barracuda::session::TensorSession` in `gpu_context.rs`

**IPC-validated methods (guideStone readiness 3 — `ludospring_guidestone`):**
- `activation.fitts`, `activation.hick` — interaction laws
- `math.sigmoid`, `math.log2` — math primitives
- `stats.mean`, `stats.variance`, `stats.std_dev` — statistics
- `noise.perlin2d` — procedural generation
- `rng.uniform` — stochastic operations
- `tensor.create`, `tensor.matmul` — GPU tensor surface
- `compute.capabilities` — compute probes
- `health.readiness` — ecosystem probes

**Remaining IPC gaps (domain compositions not in barraCuda):**
- `math.flow.evaluate` — composable from sigmoid + clamp
- `math.engagement.composite` — composable from stats.weighted_mean + tensor ops

**Migration path:** Domain-level methods should either be absorbed upstream
by barraCuda or composed from existing barraCuda primitives at the spring
binary level. Keep library path dep for Level 2 validation binaries.
**Owner:** barraCuda IPC surface / ludoSpring
**Tracking:** This file + `validate_primal_proof` exit codes

---

### GAP-03: Fragment Metadata Missing `nest_atomic`

**Primal:** NestGate (partial Nest)
**Status:** RESOLVED (V42) — `nest_atomic` added to `downstream_manifest.toml` (see GAP-09)
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
**Status:** PARTIAL — `game::engine::tensor_ops` (`sigmoid_batch_gpu`,
`validate_sigmoid_cpu_gpu_parity`) exercises `GpuContext::tensor_session()` for
Tier A sigmoid with CPU reference parity; broader engine migration and a
dedicated validation experiment remain open (April 2026).
**Impact:** Tier A sigmoid now has an in-tree `TensorSession` product hook;
GPU promotion for other ops (e.g. dot) and full gameplay integration are still
outstanding.

**Proposed validation:** Extend `TensorSession` coverage (e.g. dot) and add a
standalone validation experiment beyond the engine parity helpers.
**Owner:** ludoSpring
**Tracking:** This file

---

### GAP-05: Provenance Trio Not in Proto-Nucleate

**Primal:** rhizoCrypt, loamSpine, sweetGrass
**Status:** PARTIAL — typed IPC clients in `ipc/provenance/`; trio nodes
depend on Nest Atomic overlay (see GAP-09); `nest_atomic` now in fragments
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
**Status:** SUPERSEDED by GAP-11 — live NUCLEUS validation (V47) confirmed and
precisely measured the formulation divergences. guideStone Tier 2 now uses
barraCuda-expected values with the divergence documented in GAP-11.
**Detail:** See GAP-11 for exact formulations and numerical values.
**Impact:** Resolved via dual-value approach (bare=Python, IPC=barraCuda)
**Severity:** ~~HIGH~~ → DOCUMENTED
**Owner:** barraCuda team
**Tracking:** GAP-11 in this file

---

### GAP-09: `nest_atomic` Fragment — Aspirational Until Trio Upstream Resolves

**Primals:** NestGate, rhizoCrypt, loamSpine, sweetGrass (nest-side surface)
**Status:** RESOLVED (V42) — `nest_atomic` added to declared fragments.
Trio primals remain `required: false` until upstream blockers resolve.
**Proto-nucleate:** `fragments` now declares `tower_atomic`, `node_atomic`,
`nest_atomic`, `meta_tier`. The graph includes a `nestgate` node for
`storage.*` and typed IPC stubs for the full provenance trio.
**Code:** ludoSpring ships IPC stubs for the full nest-adjacent surface:
`ipc/nestgate.rs`, `ipc/provenance/rhizocrypt.rs`, `ipc/provenance/loamspine.rs`,
`ipc/provenance/sweetgrass.rs` (storage plus provenance). The `NicheDependency`
table in `niche.rs` marks trio primals as `required: false`.
**Decision (April 2026 V42):** ludoSpring **adds** `nest_atomic` to fragments
to accurately reflect that it wires the full Nest surface. Trio primals
gracefully degrade when absent and will activate when the Nest overlay graph
is composed by biomeOS. The `CompositionReport` (now exposed via
`lifecycle.composition`) tracks trio primals as "absent" until deployed.
When rhizoCrypt ships UDS transport (GAP-06) and loamSpine resolves its
startup panic (GAP-07), trio `required` flags can be reconsidered.
**Related:** GAP-03, GAP-05, GAP-06, GAP-07
**Owner:** primalSpring graph maintainers / ludoSpring
**Tracking:** This file + `ipc/composition.rs` runtime validation

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

## Composition Validation Evolution (April 17, 2026)

ludoSpring now implements the full three-layer validation stack:

| Layer | Artifact | Validates | Binary/Test |
|-------|----------|-----------|-------------|
| 1 | `combined_baselines.json` | Python → Rust | `python_parity.rs`, `check_drift.py` |
| 2 | `composition_targets.json` | Rust library → golden targets | `composition_parity.rs`, `check_composition_drift` |
| 3 | IPC parity | Golden targets → primal composition | `validate_composition` (requires running server) |

### Composition methods validated

| Method | Layer 1 (Python) | Layer 2 (Rust targets) | Layer 3 (IPC) |
|--------|-----------------|----------------------|---------------|
| `game.evaluate_flow` | ✓ | ✓ | ✓ (exp099, exp100, validate_composition) |
| `game.fitts_cost` | ✓ | ✓ | ✓ |
| `game.engagement` | ✓ | ✓ | ✓ |
| `game.generate_noise` | ✓ | ✓ | ✓ |
| `game.difficulty_adjustment` | ✗ (Python DDA uses different model) | ✓ | ✓ |
| `game.accessibility` | ✗ (no Python baseline) | ✓ | ✓ |
| `game.wfc_step` | ✗ (no Python WFC) | ✓ | ✓ (exp099) |
| `lifecycle.composition` | — | — | ✓ (validate_composition) |
| `health.liveness` | — | — | ✓ |
| `health.readiness` | — | — | ✓ |

---

## Per-Primal Learnings (V43 Audit)

Findings from the V43 three-layer validation buildout. These complement
the gaps above with operational learnings for primal teams.

### coralReef — required vs used

`niche::DEPENDENCIES` marks coralReef `required: true` but the runtime GPU
path does not call `shader.compile`. `lifecycle.composition` reports "missing
required" in environments without coralReef even though ludoSpring runs fine.

**Action:** Either wire `compile_wgsl` into a prewarm path, or set
`required: false` until integrated. Tracked as part of GAP-01.

### toadStool — naming inconsistency

Deploy graph `ludospring_gaming_niche.toml` uses `toadstool.health` but
code uses `compute.health` / `compute.dispatch.submit`. This causes
confusion during graph validation.

**Action:** Align all deploy graph capability names with wire protocol names.

### petalTongue — silent push failure

`game.push_scene` handler returns `pushed: true` even when
`VisualizationPushClient::push_scene()` fails (error swallowed with
`let _ = ...`). Operators can't detect visualization failures.

**Action:** Propagate push errors into JSON-RPC result for honest telemetry.

### Squirrel — incomplete inference surface

`InferenceCompleteRequest`, `InferenceEmbedRequest` etc. are defined in
`ipc/squirrel.rs` but no `inference.*` wrapper functions are wired. Context
helpers (`context.create`, `context.update`) exist without game IPC exposure.

**Action:** Either expose through `game.*` capabilities or document as
internal-only.

### NestGate — unused API surface

`exists`, `list`, `metadata`, `delete` are implemented in the NestGate client
but not exposed as `game.*` capabilities. Risk of API drift.

**Action:** Expose or trim to match actual usage.

### Neural API — error contract

Registration ack and `capability.call` error JSON shapes are undocumented.
Springs can't distinguish "routed but primal failed" vs "Neural API down".

**Action:** biomeOS team to version error shapes.

### GAP-11: barraCuda Formulation Divergence (Fitts, Hick, Variance)

**Primal:** barraCuda
**Status:** DOCUMENTED — discovered during live NUCLEUS validation (V47)
**Impact:** guideStone IPC checks use barraCuda-expected values, not Python golden values.

Three formulation differences between barraCuda IPC and Python baselines:
1. **Fitts**: barraCuda uses `log₂(D/W + 1)`, Python uses `log₂(2D/W + 1)` (Shannon).
   Same params (D=100, W=10, a=50, b=150): barraCuda=568.91, Python=708.85.
2. **Hick**: barraCuda uses `log₂(N)`, Python uses `log₂(N + 1)`.
   Same params (N=7, a=200, b=150): barraCuda=621.10, Python=650.00.
3. **Variance**: barraCuda always returns sample variance (ddof=1, N-1),
   ignoring `ddof` parameter. Python golden uses population variance (ddof=0).
   Same data [2,4,4,4,5,5,7,9]: barraCuda=4.5714, Python=4.0.

guideStone bare checks (Tier 1) use Python golden values (reference-traceable).
guideStone IPC checks (Tier 2) use barraCuda-expected values (IPC parity).
Both are documented and deterministic — the formulation difference is tracked here.

**Action:** barraCuda team to verify formulation choices. If Shannon `log₂(2D/W + 1)`
is preferred, update the `activation.fitts` implementation. If `log₂(D/W + 1)` is
intentional, document the rationale. Same for Hick and variance convention.

---

**License:** AGPL-3.0-or-later
