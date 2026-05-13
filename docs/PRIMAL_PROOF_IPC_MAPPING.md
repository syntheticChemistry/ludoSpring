# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring — Primal Proof IPC Mapping

**Last updated:** May 13, 2026 (V68 — Tier 2 contract alignment)
**Wire contract:** `primalSpring/docs/LIVE_SCIENCE_API.md`
**Feature flag:** `primal-proof` = `ipc` feature (library calls gated behind `local`)

---

## Overview

This document maps every ludoSpring domain operation to its JSON-RPC IPC
equivalent. When the `local` feature is disabled (`default = []` since V64),
all math routes through IPC. When enabled, library calls provide identical
results with lower latency.

The dual-path is validated by the `s_tier4_math_parity` scenario (identical
outputs regardless of path).

---

## Compute Dispatch (toadStool)

| Domain Operation | IPC Method | Module | Notes |
|-----------------|------------|--------|-------|
| Noise field generation (Perlin/fBm) | `compute.dispatch.submit` | `ipc/toadstool.rs` | WGSL shader via `barracuda/shaders/perlin_2d.wgsl` |
| Raycaster batch | `compute.dispatch.submit` | `ipc/toadstool.rs` | DDA columns in parallel |
| Engagement batch evaluation | `compute.dispatch.submit` | `ipc/toadstool.rs` | N-player metrics parallel |
| Workload pre-flight | `toadstool.validate` | `ipc/toadstool.rs` | Tier 2 pre-dispatch capability check |
| Workload enumeration | `toadstool.list_workloads` | `ipc/toadstool.rs` | Query available workload TOMLs |
| Substrate capabilities | `compute.capabilities` | `ipc/toadstool.rs` | GPU availability, f64 support |

## Precision Routing (barraCuda)

| Domain | IPC Method | Params | Expected Response |
|--------|-----------|--------|-------------------|
| `game_science` | `barracuda.precision.route` | `{ "domain": "game_science", "hardware_hint": "compute" }` | `F64` tier, `fma_safe: true` |
| `procedural_noise` | `barracuda.precision.route` | `{ "domain": "procedural_noise", "hardware_hint": "compute" }` | `F32` tier (noise is perceptual) |
| `interaction_laws` | `barracuda.precision.route` | `{ "domain": "interaction_laws", "hardware_hint": "compute" }` | `F64` tier (Fitts/Hick regression) |
| `engagement_metrics` | `barracuda.precision.route` | `{ "domain": "engagement_metrics", "hardware_hint": "compute" }` | `F64` tier (statistical correlations) |

## Math Operations (barraCuda library → IPC)

| Library Call (feature = "local") | IPC Equivalent | Method Constant |
|----------------------------------|----------------|-----------------|
| `barracuda::activations::sigmoid(x)` | `capability_call("math", "sigmoid", ...)` | `methods::math::SIGMOID` |
| `barracuda::stats::mean(xs)` | `capability_call("math", "mean", ...)` | `methods::math::MEAN` |
| `barracuda::stats::correlation::variance(xs)` | `capability_call("math", "variance", ...)` | `methods::math::VARIANCE` |
| `barracuda::rng::lcg_step(state)` | N/A (inline fallback) | — |
| `barracuda::rng::state_to_f64(state)` | N/A (inline fallback) | — |
| `barracuda::stats::dot(a, b)` | `capability_call("math", "dot", ...)` | — |

## Shader Compilation (coralReef)

| Operation | IPC Method | Status |
|-----------|-----------|--------|
| Compile Perlin 2D WGSL | `shader.compile.wgsl` | GAP-01 — blocked on FECS stability |
| Compile fBm WGSL | `shader.compile.wgsl` | GAP-01 — blocked on FECS stability |
| Compile raycaster WGSL | `shader.compile.wgsl` | GAP-01 — blocked on FECS stability |

## Visualization (petalTongue)

| Operation | IPC Method | Method Constant |
|-----------|-----------|-----------------|
| Push game scene | `visualization.render.scene` | `methods::visualization::RENDER_SCENE` |
| Push dashboard | `visualization.render.dashboard` | `methods::visualization::RENDER_DASHBOARD` |
| Stream metrics | `visualization.render.stream` | `methods::visualization::RENDER_STREAM` |
| Tufte pre-flight | `visualization.validate` | `methods::visualization::VALIDATE` |

## AI/Inference (Squirrel)

| Operation | IPC Method | Method Constant |
|-----------|-----------|-----------------|
| NPC dialogue generation | `ai.query` | `methods::ai::QUERY` |
| Play pattern analysis | `ai.analyze` | `methods::ai::ANALYZE` |
| Design suggestions | `ai.suggest` | `methods::ai::SUGGEST` |

## Provenance Trio

| Operation | IPC Method | Method Constant |
|-----------|-----------|-----------------|
| Create DAG session | `dag.session.create` | `methods::dag::SESSION_CREATE` |
| Append provenance event | `dag.event.append` | `methods::dag::EVENT_APPEND` |
| Merkle root query | `dag.merkle.root` | `methods::dag::MERKLE_ROOT` |
| Create braid entry | `braid.create` | `methods::braid::CREATE` |
| Seal ledger spine | `spine.seal` | `methods::spine::SEAL` |

## Security (skunkBat)

| Operation | IPC Method | Method Constant |
|-----------|-----------|-----------------|
| Emit audit event | `security.audit_log` | `methods::security::AUDIT_LOG` |
| Security scan | `security.scan` | `methods::security::SCAN` |

## Storage (NestGate)

| Operation | IPC Method | Method Constant |
|-----------|-----------|-----------------|
| Store content | `storage.put` | `methods::storage::PUT` |
| Retrieve content | `storage.get` | `methods::storage::GET` |

---

## Precision Strategy by Domain

ludoSpring's game science operates across precision tiers:

| Domain | Required Precision | Why |
|--------|-------------------|-----|
| Interaction laws (Fitts, Hick, Steering) | F64 | Regression coefficients from published papers |
| Flow/engagement metrics | F64 | Statistical correlations, Cohen's d |
| Procedural noise (Perlin, fBm) | F32 | Perceptual output — visual artifacts only at extreme error |
| Raycaster DDA | F32 | Integer grid traversal, float only for distance |
| Game state / ECS | F32 | Position/velocity — frame-rate bounded |
| Composition parity checks | F64 | Must match Python baselines exactly |

When `barracuda.precision.route` is live, these hardcoded decisions become
runtime queries — the precision tier adapts to available hardware.

---

## Graceful Degradation

Every IPC call in ludoSpring follows the degradation pattern:

```rust
let Ok(bridge) = NeuralBridge::discover() else {
    return Ok(TypedResult::unavailable());
};
bridge.capability_call(domain, method, &args)
    .map_or_else(
        |_| Ok(TypedResult::unavailable()),
        |result| Ok(TypedResult::from_response(&result)),
    )
```

When primals are not reachable:
- Functions return well-formed "unavailable" structs (not errors)
- Callers check `.available` before using results
- The `local` feature provides a compile-time fallback for math operations
