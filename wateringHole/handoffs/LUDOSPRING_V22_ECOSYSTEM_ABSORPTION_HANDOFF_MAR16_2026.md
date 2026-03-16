<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# ludoSpring V22 → barraCuda + toadStool Ecosystem Absorption Handoff

**Date:** March 16, 2026
**From:** ludoSpring V22 — 75 experiments, 1692 checks, 394 tests + 12 proptest + 6 IPC integration
**To:** barraCuda team (math primitives), toadStool team (GPU dispatch)
**Supersedes:** V21 Deep Debt Evolution
**License:** AGPL-3.0-or-later

---

## Executive Summary

V22 is an ecosystem absorption sprint — pulling in patterns and fixes discovered
across sibling springs and infrastructure primals during the March 16 cross-ecosystem
review. Key changes:

- toadStool `compute.dispatch.*` direct dispatch methods wired into typed IPC client
- Dual-format capability discovery fixes neuralSpring S156 interop issue
- Python tolerance mirror (46 constants) following wetSpring V121 pattern
- Write→Absorb→Lean status documented for noise module
- Deploy graph updated with dispatch capabilities
- All files under 750 LOC, zero `#[allow()]`, zero magic numbers

---

## Part 1: toadStool Direct Dispatch Client (toadStool relevance: P0)

ludoSpring V21 had `compute.submit/status/capabilities` (job queue API). V22 adds
`compute.dispatch.*` methods for direct GPU dispatch — lower latency, no queue:

```rust
pub fn dispatch_submit(shader_source, entry_point, workgroup_size, dispatch_size, buffers)
    -> Result<ComputeResult, String>

pub fn dispatch_result(workload_id) -> Result<ComputeResult, String>

pub fn dispatch_capabilities() -> Result<SubstrateCapabilities, String>
```

These map to toadStool's `compute.dispatch.submit`, `compute.dispatch.result`, and
`compute.dispatch.capabilities` methods confirmed in S155b handler dispatch.

**Use case:** Real-time game compute (fog of war, pathfinding, tile lighting) where
the 5-second job queue timeout is too slow. Direct dispatch targets sub-frame latency.

**toadStool action:** Confirm `compute.dispatch.submit` accepts the parameter schema:
`{ shader_source, entry_point, workgroup_size, dispatch_size, buffers, requester }`.

---

## Part 2: Dual-Format Capability Discovery (all primals: P1)

neuralSpring S156 discovered that `probe_capabilities` responses vary across the
ecosystem. Some primals return:

```json
{"capabilities": ["cap1", "cap2"]}
```

Others return:

```json
{"capabilities": {"capabilities": ["cap1", "cap2"]}}
```

ludoSpring V22 `extract_capabilities()` handles both formats:

```rust
fn extract_capabilities(result: &serde_json::Value) -> Vec<String> {
    match result.get("capabilities") {
        Some(Value::Array(arr)) => /* standard format */,
        Some(Value::Object(obj)) => /* nested format */,
        _ => Vec::new(),
    }
}
```

**All primals:** If your `lifecycle.status` handler returns the nested format,
consider standardizing to the flat array. But consumers should handle both.

---

## Part 3: Python Tolerance Mirror (barraCuda relevance: P2)

Following wetSpring V121's `scripts/tolerances.py`, ludoSpring V22 adds
`baselines/python/tolerances.py` with 46 named constants mirroring all 6 Rust
tolerance submodules:

```python
from tolerances import ANALYTICAL_TOL, FITTS_A_MOUSE_MS, GAME_STATE_TOL
assert abs(computed - expected) < ANALYTICAL_TOL
```

**barraCuda action:** If barraCuda has Python test infrastructure, consider
adopting the same pattern. Constants with provenance citations in both languages
prevent magic-number divergence.

---

## Part 4: Write → Absorb → Lean Status

| ludoSpring module | barraCuda status | ludoSpring action |
|-------------------|-----------------|-------------------|
| `procedural::noise` Perlin 2D | Absorbed (`ops::procedural::perlin_noise::perlin_2d_cpu`) | Retain as validation reference |
| `procedural::noise` fBm 2D | Absorbed (`ops::procedural::perlin_noise::fbm_2d_cpu`) | Retain as validation reference |
| `procedural::noise` Perlin 3D | **Not absorbed** | Candidate for upstream |
| `procedural::noise` fBm 3D | **Not absorbed** | Candidate for upstream |
| `procedural::wfc` | **Not absorbed** | P2 handoff item |
| `procedural::bsp` | **Not absorbed** | P2 handoff item |
| `procedural::lsystem` | **Not absorbed** | P3 handoff item |
| `interaction::input_laws` | **Not absorbed** | GPU batch: P1 handoff |
| `interaction::flow` | **Not absorbed** | GPU batch: P1 handoff |
| `metrics::engagement` | **Not absorbed** | GPU batch: P1 handoff |
| `game::raycaster` | **Not absorbed** | GPU batch: P1 handoff |
| `game::engine::gpu` (5 ops) | **Not absorbed** | Shaders in `barracuda/shaders/game/` |

**barraCuda action:** The 3D Perlin/fBm paths are trivial to absorb — same
algorithm as the 2D paths already in `ops::procedural`. The HCI batch ops
(Fitts, Hick, Steering, Flow, engagement) are all embarrassingly parallel
log/exp/compare operations — ideal Tier A GPU candidates.

---

## Part 5: Ecosystem Review Findings

From reviewing all 17 repos (7 springs + 10 primals) on March 16:

### Patterns We Absorbed

| Source | Pattern | ludoSpring Implementation |
|--------|---------|--------------------------|
| wetSpring V121 | Python tolerance mirror | `baselines/python/tolerances.py` (46 constants) |
| neuralSpring S156 | Dual-format capability parsing | `extract_capabilities()` in `discovery.rs` |
| airSpring V083 | JSON-RPC -32601 compliance | Already correct (verified) |
| toadStool S155b | `compute.dispatch.*` methods | 3 new typed client methods + 3 tests |

### Patterns We Confirmed

| Pattern | Status |
|---------|--------|
| `-32601` for unknown methods | Correct (handlers.rs line 65-67) |
| Capability-based discovery (no hardcoded names) | Correct |
| `#![forbid(unsafe_code)]` | Correct |
| `#[expect(reason)]` for justified exceptions | Correct |
| XDG-compliant socket resolution | Correct |
| `temp_dir()` for platform-agnostic paths | Correct |

### Learnings Relevant to barraCuda/toadStool

1. **biomeOS v2.43 streaming pipeline** — `PipelineExecutor` + NDJSON client could
   consume ludoSpring telemetry directly. Our `game.poll_telemetry` RPC already
   emits the right format.

2. **biomeOS `ludospring_analysis_loop.toml`** — a 10 Hz continuous analysis graph
   already exists for ludoSpring in biomeOS. We should verify our `poll_telemetry`
   response matches what that graph expects.

3. **rhizoCrypt 0.13.0 `checkout_slice`** — O(1) proof verification could improve
   our DAG experiment performance (exp046, exp053).

4. **coralReef GlowPlug** — systemd-managed GPU lifecycle daemon means our WGSL
   shaders will eventually have persistent GPU access without per-dispatch cold start.

---

## Part 6: Code Quality Metrics

| Metric | V21 | V22 |
|--------|-----|-----|
| toadStool dispatch methods | 3 (job queue only) | 6 (+ 3 direct dispatch) |
| Discovery format support | flat array only | flat array + nested object |
| Python tolerance constants | 0 (magic numbers) | 46 (named, cited) |
| Files modified | — | 5 |
| New files | — | 1 (`tolerances.py`) |
| Tests added | — | 7 (3 dispatch + 4 discovery) |
| Max file LOC | 742 | 742 |

---

## License

AGPL-3.0-or-later
