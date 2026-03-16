<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# ludoSpring V21 → barraCuda + toadStool Deep Debt Evolution Handoff

**Date:** March 16, 2026
**From:** ludoSpring V21 — 75 experiments, 1692 checks, 394 tests + 12 proptest + 6 IPC integration
**To:** barraCuda team (math primitives), toadStool team (GPU dispatch)
**Supersedes:** V20 Deep Primal Integration
**License:** AGPL-3.0-or-later

---

## Executive Summary

- V21 eliminates all `#[allow()]` suppressions through structural refactoring (not just annotation changes)
- Introduces `ValidationSink` trait — pluggable validation output for composable test infrastructure
- Typed toadStool IPC client (`ipc/toadstool.rs`) — the first typed client specifically for toadStool compute dispatch
- 6 IPC integration tests validate the JSON-RPC 2.0 server lifecycle end-to-end
- `#[expect(reason)]` adoption for edition 2024 — justified lint exceptions with documented rationale
- Platform-agnostic paths replace all hardcoded `/tmp` references
- `GAME_STATE_TOL` centralized — zero inline tolerance constants
- 75 .rs files, 19,302 lines (net +544 from V20)

---

## Part 1: Typed toadStool IPC Client (toadStool relevance: P0)

New `barracuda/src/ipc/toadstool.rs` provides a typed client for toadStool compute dispatch:

```rust
pub struct ComputeResult {
    pub workload_id: String,
    pub status: String,
    pub result: Option<serde_json::Value>,
}

pub struct SubstrateCapabilities {
    pub substrates: Vec<String>,
    pub capabilities: Vec<String>,
}

pub async fn submit_workload(bridge: &NeuralBridge, ...) -> Result<ComputeResult, ...>
pub async fn workload_status(bridge: &NeuralBridge, ...) -> Result<ComputeResult, ...>
pub async fn query_capabilities(bridge: &NeuralBridge) -> Result<SubstrateCapabilities, ...>
```

Methods map to toadStool JSON-RPC:
- `compute.submit` — submit GPU/CPU workload with shader path + parameters
- `compute.status` — poll workload completion
- `compute.capabilities` — query available substrates and features

Graceful degradation: all methods return structured errors when toadStool is unavailable (Neural API down, socket missing). The caller decides whether to fall back to CPU.

**toadStool action:** Verify these method names match your canonical JSON-RPC spec. If compute method signatures differ, this client is the contract to align to.

**barraCuda action:** If barraCuda dispatches work to toadStool for GPU execution, this client pattern is reusable. The `SubstrateCapabilities` type could move upstream.

---

## Part 2: ValidationSink Trait (barraCuda relevance: P1)

`barracuda/src/validation/mod.rs` now provides pluggable validation output:

```rust
pub trait ValidationSink {
    fn emit(&mut self, message: &str);
}

pub struct StderrSink;        // default — writes to stderr (hotSpring pattern)
pub struct BufferSink { ... } // captures output for testing
```

`ValidationHarness<S: ValidationSink = StderrSink>` is generic over sink. This means:
- Production experiments use `ValidationHarness::new()` (default stderr)
- Test code uses `ValidationHarness::with_sink(BufferSink::new())` for output capture
- Future sinks: JSON-RPC push to petalTongue, file logging, structured telemetry

**barraCuda action:** If barraCuda has validation infrastructure, this `ValidationSink` pattern decouples validation logic from output transport. Consider adopting or absorbing.

**toadStool action:** When toadStool validates GPU parity results, a `ValidationSink` that pushes to biomeOS monitoring would close the observability loop.

---

## Part 3: Session Decomposition Pattern (barraCuda relevance: P2)

`GameSession::resolve()` was a single 200+ line match statement covering all command variants. V21 extracts each arm into a focused method:

```rust
impl GameSession {
    pub fn resolve(&mut self, cmd: &Command) -> ActionOutcome {
        match cmd {
            Command::Wait => self.resolve_wait(),
            Command::EndTurn => self.resolve_end_turn(),
            Command::UseItem { .. } => self.resolve_use_item(cmd),
            Command::Custom { .. } => self.resolve_custom(cmd),
            // ...
        }
    }

    fn resolve_wait(&mut self) -> ActionOutcome { ... }
    fn resolve_end_turn(&mut self) -> ActionOutcome { ... }
    // ...
}
```

This eliminates `#[allow(clippy::too_many_lines)]` through structural decomposition, not annotation suppression.

**Pattern for barraCuda:** Any large dispatch function (e.g., shader selection, op routing) benefits from this extract-method pattern. Each branch becomes independently testable and documentable.

---

## Part 4: Typed Transition Verification (barraCuda relevance: P2)

`TransitionVerification` had five boolean fields (`inventory_preserved`, `disposition_unchanged`, etc.) that triggered `clippy::struct_excessive_bools`. V21 replaces them with a typed enum:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionIssue {
    InventoryLost,
    DispositionChanged,
    KnowledgeLost,
    ConditionMismatch,
    HpChanged,
}

pub struct TransitionVerification {
    pub issues: Vec<TransitionIssue>,
}

impl TransitionVerification {
    pub fn passed(&self) -> bool { self.issues.is_empty() }
    pub fn check_passed(&self, issue: &TransitionIssue) -> bool { !self.issues.contains(issue) }
    pub fn failure_descriptions(&self) -> Vec<&str> { ... }
}
```

**Pattern for barraCuda:** Any validation result with multiple independent checks benefits from this enum-of-issues pattern. It's more expressive than booleans and composes naturally (`issues.iter().filter()`).

---

## Part 5: IPC Integration Tests (toadStool relevance: P1)

New `barracuda/tests/ipc_integration.rs` with 6 tests:

| Test | What it validates |
|------|-------------------|
| `lifecycle_status` | Server starts, responds to `lifecycle.status`, shuts down cleanly |
| `capability_list` | `capability.list` returns all 24 registered capabilities |
| `game_evaluate_flow` | `game.evaluate_flow` returns valid flow state from parameters |
| `game_fitts_cost` | `game.fitts_cost` computes correct Fitts's law movement time |
| `unknown_method_returns_error` | Unknown methods return JSON-RPC -32601 Method not found |
| `health_check_responds` | `health.check` returns healthy status |

These are true integration tests — they spawn the server, connect via socket, send JSON-RPC requests, and verify responses.

**toadStool action:** If toadStool has a JSON-RPC server, this test pattern is directly reusable. The `start_server()` / `send_rpc()` / `cleanup()` helpers are generic.

---

## Part 6: `#[expect]` Evolution (barraCuda relevance: P2)

Rust edition 2024 provides `#[expect(lint, reason = "...")]` as a superior alternative to `#[allow()]`. The key difference: `#[expect]` warns when the expected lint no longer fires (the code was fixed but the annotation wasn't removed).

V21 converts the one remaining `#[allow(dead_code)]` in experiment code:

```rust
// Before (V20):
#[allow(dead_code)]
mod protocol;

// After (V21):
#[expect(dead_code, reason = "wire format types for IPC contract — constructed by remote callers, not locally")]
mod protocol;
```

**barraCuda action:** If barraCuda has any `#[allow()]` annotations, convert to `#[expect(reason)]`. The reason field documents *why* the exception exists, and the lint warns when the exception becomes stale.

---

## Part 7: Platform-Agnostic Paths (toadStool relevance: P2)

V21 eliminates hardcoded `/tmp` paths:

```rust
// Before (V20):
let biomeos_path = PathBuf::from("/tmp/biomeos/ludospring.sock");

// After (V21):
let biomeos_path = std::env::temp_dir().join("biomeos").join("ludospring.sock");
```

This matters for cross-platform builds and containerized environments where `/tmp` may not exist or may be mounted differently.

**toadStool action:** If toadStool uses hardcoded paths for socket files or temp directories, apply the same `temp_dir()` / XDG pattern.

---

## Part 8: Absorption Opportunities (updated from V20)

| ludoSpring module | Lines | What barraCuda gets | Priority | V21 notes |
|-------------------|-------|---------------------|----------|-----------|
| `procedural::noise` | ~200 | Perlin 2D/3D + fBm | P1 | GPU-ready, Tier A |
| `procedural::wfc` | ~265 | Wave Function Collapse | P2 | Needs barrier sync |
| `procedural::bsp` | ~220 | BSP spatial partitioning | P2 | Recursive → iterative for GPU |
| `game::engine::gpu` | ~360 | 5 GpuOp dispatch types | P1 | Session decomposition makes integration cleaner |
| `capability_domains.rs` | ~100 | Domain/Method introspection | P1 | Reusable for any primal |
| `tolerances/` (pattern) | ~300 | 6-submodule decomposition | P2 | Template for organized constants |
| `validation/` (pattern) | ~400 | `ValidationSink` trait + `ValidationHarness<S>` | P1 | Composable validation infrastructure |
| `ipc/toadstool.rs` | ~80 | Typed toadStool client | P0 | First typed contract for toadStool methods |

### WGSL Shaders for toadStool Absorption (preserved from V20)

| Shader | Path | Workgroup | Purpose |
|--------|------|-----------|---------|
| `fog_of_war.wgsl` | `barracuda/shaders/game/` | 64 | Per-tile visibility from viewer position |
| `tile_lighting.wgsl` | `barracuda/shaders/game/` | 64 | Point light propagation (1/d² falloff) |
| `pathfind_wavefront.wgsl` | `barracuda/shaders/game/` | 64 | BFS expansion (one ring per dispatch) |

Plus Tier A parity shaders from exp030: `perlin_2d.wgsl`, `engagement_batch.wgsl`, `dda_raycast.wgsl`, `sigmoid.wgsl`, `dot_product.wgsl`, `reduce_sum.wgsl`, `softmax.wgsl`, `lcg.wgsl`, `relu.wgsl`, `scale.wgsl`, `abs.wgsl`.

---

## Part 9: Code Quality Metrics

| Metric | V20 | V21 |
|--------|-----|-----|
| .rs files (barracuda) | 74 | 75 |
| Lines (barracuda) | 18,758 | 19,302 |
| Tests (workspace) | 394 | 394 + 6 IPC integration |
| Clippy warnings | 0 | 0 |
| `#[allow()]` in production | 0 | 0 |
| `#[allow()]` suppressions via refactoring | 2 remaining | 0 — all eliminated |
| Magic numbers in prod | 0 | 0 |
| Inline tolerance constants | 4 experiments | 0 — all centralized |
| Production panics | 0 | 0 |
| Hardcoded paths | 2 experiments | 0 — all `temp_dir()` |
| Typed IPC clients | NestGate, Squirrel, trio | + toadStool |
| Validation infrastructure | `eprintln!` only | `ValidationSink` trait (pluggable) |
| IPC integration tests | 0 | 6 |

---

## Part 10: Learnings for barraCuda Evolution

From building ludoSpring V21, patterns that would benefit barraCuda directly:

1. **`ValidationSink` trait** — validation output decoupled from transport. barraCuda's test infrastructure could use this for GPU parity validation that pushes results to monitoring.

2. **`#[expect(reason)]`** — edition 2024's superior lint annotation. Any `#[allow()]` in barraCuda should migrate. The `reason` field becomes documentation.

3. **Tolerance submodule pattern** — 6 domain-specific files with `mod.rs` re-exports. Scales well past 100+ constants without losing discoverability.

4. **Typed IPC clients** — `ipc/toadstool.rs` is the contract ludoSpring expects. If toadStool method signatures evolve, this client is the integration point.

5. **Session decomposition** — extract-method for large dispatch functions. Each branch becomes independently testable. Applied to `resolve()`, applicable to any op dispatch.

6. **TransitionIssue enum** — typed error enum replaces boolean fields. More expressive, composes naturally with iterators, and eliminates `struct_excessive_bools`.

---

## License

AGPL-3.0-or-later
