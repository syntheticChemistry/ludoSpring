# Degradation Behavior — ludoSpring

**Last updated:** May 17, 2026 (V76 — Wave 20 PM absorption)
**Pattern source:** `primalSpring CompositionContext` degradation table, lithoSpore R1
**Status:** Active

---

## Design Principle

ludoSpring's domain logic (game science, HCI models, procedural generation) NEVER
fails due to primal unavailability. Primals are enrichment — they accelerate, persist,
or visualize results, but the science itself computes locally in pure Rust.

Pattern: `has_capability()` before `call()`. Never gate science behind infrastructure.

---

## Per-Capability Degradation Table

| Capability | Primal | When Unreachable | Consumer Sees | Recovery |
|------------|--------|------------------|---------------|----------|
| `compute.submit` | toadStool | CPU fallback via `crate::math` | Identical result, slower | Auto-reconnect next frame |
| `compute.dispatch` | toadStool | CPU fallback for GPU shaders | Same output, higher latency | Auto-reconnect |
| `ai.query` / `ai.analyze` / `ai.suggest` | Squirrel | Deterministic NPC fallback | Scripted responses | Skip until available |
| `visualization.render.scene` | petalTongue | Scene payload buffered locally | JSON file in sandbox/ | Flush on reconnect |
| `dag.create_session` / `dag.add_vertex` | rhizoCrypt | Provenance skipped | `None` — domain proceeds | Enrichment when reconnected |
| `ledger.commit` | loamSpine | Spine entry skipped | `None` — domain proceeds | Deferred commit |
| `attribution.braid` | sweetGrass | Braid skipped | `None` — domain proceeds | Deferred braid |
| `storage.store` / `storage.retrieve` | NestGate | In-memory cache used | Cached data or `None` | Persist on reconnect |
| `crypto.hash` / `crypto.sign` | BearDog | Unsigned result | Result without signature | Sign retroactively |
| `discovery.*` | songBird | Static niche table fallback | `hint_name` resolution | Auto-reconnect |

---

## Implementation Patterns

### 1. IPC Call with Graceful Skip

```rust
// From barracuda/src/ipc/mod.rs — canonical pattern
match client.call(method, params).await {
    Ok(result) => Ok(Some(result)),
    Err(e) if is_skip_error(&e) => Ok(None),  // primal offline → skip
    Err(e) => Err(e),                          // real error → propagate
}
```

### 2. GPU Fallback (toadStool unreachable)

```rust
// From metalForge/forge/src/routing.rs — canonical pattern
let substrate = recommend_substrate(workload, gpu_available);
match substrate {
    Substrate::Gpu => dispatch_to_toadstool(workload),  // fast path
    Substrate::Cpu => compute_locally(workload),         // fallback — same result
    Substrate::Npu => dispatch_to_npu(workload),        // NPU path
}
```

### 3. Trio Enrichment (never blocks domain)

```rust
// From niche.rs — canonical pattern
let science_result = compute_game_science(input);  // ALWAYS runs first

// Enrichment — failures are non-fatal
if let Some(dag) = trio.rhizocrypt() {
    let _ = dag.add_vertex(&science_result);
}
if let Some(spine) = trio.loamspine() {
    let _ = spine.commit(&science_result.hash());
}
if let Some(braid) = trio.sweetgrass() {
    let _ = braid.attribute(&science_result, &roles);
}

science_result  // returned regardless of trio state
```

---

## Error Classification

| Error Type | Behavior | Example |
|------------|----------|---------|
| `ConnectionRefused` | Skip — primal not running | `is_skip_error` returns true |
| `ConnectionReset` | Skip — primal crashed | `is_skip_error` returns true |
| `Timeout` (>5s) | Skip — primal hung | `is_skip_error` returns true |
| `MethodNotFound` | Propagate — API mismatch | Needs version alignment |
| `InvalidParams` | Propagate — caller bug | Developer must fix |
| `InternalError` | Log + skip — primal bug | Report to primal team |

---

## What `health.readiness` Reports

When polled, ludoSpring reports its degradation state:

```json
{
  "status": "ready",
  "degraded_capabilities": [
    {"capability": "compute.submit", "reason": "toadstool_unreachable", "fallback": "cpu"},
    {"capability": "dag.create_session", "reason": "rhizocrypt_offline", "fallback": "skip"}
  ],
  "domain_logic": "fully_operational"
}
```

Key invariant: `domain_logic` is ALWAYS `fully_operational`. ludoSpring's science
never depends on external primal availability.

---

## Composition Report Integration

The `lifecycle.composition` handler reports which dependencies are live vs degraded:

```json
{
  "composition_status": "partial",
  "primals_reached": ["barracuda", "beardog", "songbird"],
  "primals_degraded": ["toadstool", "rhizocrypt"],
  "primals_absent": ["loamspine", "sweetgrass"],
  "domain_impact": "none"
}
```

`domain_impact: "none"` is the contract: science always works.
