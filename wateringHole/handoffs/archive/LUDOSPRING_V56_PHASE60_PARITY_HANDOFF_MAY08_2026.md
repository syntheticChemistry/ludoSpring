# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring V56 — Phase 60 Cross-Spring Parity Handoff

**From:** ludoSpring V56
**Date:** May 8, 2026
**To:** primalSpring (upstream audit), primals teams (gap handbacks), downstream (sporeGarden)

---

## Summary

ludoSpring responds to primalSpring's Phase 60 cross-spring parity audit with
structural evolution: registry cross-sync tests, circuit breaker extraction,
GPU protocol centralization, Python CPU benchmarks, paper queue documentation,
and deploy graph alignment (Squirrel node + toadStool naming fix).

**Score (self-assessed):** GOOD → STRONG trajectory. All three primalSpring
evolution targets addressed (registry sync test: done; paper queue: documented;
git tracking: confirmed clean).

---

## What Changed (V55 → V56)

| Area | Change |
|------|--------|
| **Registry sync** | Two new tests in `niche.rs`: bidirectional local TOML sync + cross-sync against primalSpring canonical |
| **Circuit breaker** | Extracted from `provenance/mod.rs` (747→644L) to reusable `ipc/circuit_breaker.rs` (156L) |
| **GPU handlers** | 5 hardcoded `"ludospring_gpu_v1"` → centralized `GPU_PROTOCOL_TAG` constant |
| **Composition validation** | `game.wfc_step` added to exercised methods (was golden JSON gap) |
| **Deploy graph** | `ludospring_gaming_niche.toml`: Squirrel AI node added, toadStool `compute.*` wire naming aligned |
| **Python benchmark** | `baselines/python/bench_cpu_parity.py` — performance baseline for Criterion speedup ratio |
| **Paper queue** | `docs/PAPER_QUEUE.md` — foundation threads, papers 17-22/24a/26, datasets, benchmark gaps |
| **Provenance** | `composition_targets.json` gains `generated_date` field |
| **CHECKSUMS** | Regenerated with correct guidestone module paths (was stale from V47) |
| **sporeprint** | `validation-summary.md` refreshed (metrics, binary names, date) |
| **Gaps** | 4 new gaps documented (GAP-12–GAP-15) |

---

## Handbacks to Upstream Primals

### → primalSpring: GAP-12 — Register 15 Missing Methods

ludoSpring serves 30 capabilities. primalSpring's canonical `[game]` section
lists only 15. The following need registration:

```toml
# Provenance lifecycle
"game.record_action"
"game.poll_telemetry"

# Interaction (petalTongue delegation)
"game.subscribe_interaction"
"game.poll_interaction"

# Storage (NestGate delegation)
"game.storage_put"
"game.storage_get"

# DAG / Certificates / Attribution (provenance trio)
"game.query_vertices"
"game.mint_certificate"

# GPU compute (toadStool dispatch)
"game.gpu.fog_of_war"
"game.gpu.tile_lighting"
"game.gpu.pathfind"
"game.gpu.perlin_terrain"
"game.gpu.batch_raycast"
```

`health.liveness` and `health.readiness` are likely already in `[health]`.

### → barraCuda: GAP-13 — Build Regression

`crates/barracuda/src/tolerances/precision.rs:138` references
`crate::device::precision_tier::PrecisionTier` without `#[cfg(feature = "gpu")]`.
This breaks all consumers using `default-features = false` after the 0.3.12→0.3.13 bump.

**Fix:** Either gate `for_precision_tier()` behind `#[cfg(feature = "gpu")]`
or extract `PrecisionTier` enum to a feature-independent module.

### → sweetGrass: Capability Hint Mismatch

ludoSpring's `niche::DEPENDENCIES` probes `by_capability = "braid"` for
sweetGrass. If sweetGrass's `capability.list` response only advertises
`"commit"` (not `"braid"`), composition reports show sweetGrass as absent
even when running. Ensure `"braid"` is in advertised capabilities.

### → toadStool: Capability Naming

Wire protocol uses `compute.submit`, `compute.status`, `compute.dispatch.*`.
Older toadStool versions advertised `toadstool.health`, `toadstool.execute`.
ludoSpring's gaming niche graph has been updated to expect `compute.*` — ensure
toadStool advertises these method names via `capability.list`.

---

## Patterns for Downstream Absorption

### Pure Composition Model

ludoSpring is the first "pure composition" spring — no spring binary deploys
in production. The `ludospring_cell.toml` (12-node graph) and biomeOS
orchestrate primals directly. Pattern:

1. Spring validates science in Rust (local `cargo test`)
2. Spring's IPC server (`ludospring` binary) is for development/testing only
3. Production: biomeOS deploys the cell graph; primals serve capabilities
4. Spring validates composition via `validate_composition` + `ludospring_guidestone`

### Circuit Breaker (Reusable)

`ipc/circuit_breaker.rs` implements the healthSpring V32 pattern with env-var
configuration. Any spring/primal needing resilient IPC can absorb this directly:
- `LUDOSPRING_CIRCUIT_COOLDOWN_MS` (default 5000)
- `LUDOSPRING_CIRCUIT_MAX_RETRIES` (default 2)
- `LUDOSPRING_CIRCUIT_RETRY_DELAY_MS` (default 50)

Pattern: `resilient_call(|bridge| bridge.capability_call(...))` → `Option<Value>`.

### Registry Cross-Sync Test

Two-layer verification:
1. **Local**: `capabilities_match_local_registry_toml` — bidirectional sync
   between `niche.rs CAPABILITIES` and `config/capability_registry.toml`
2. **Canonical**: `capabilities_subset_of_primalspring_canonical` — ensures all
   primalSpring-registered `game.*` methods are served locally

Pattern is intentionally one-directional for canonical (we must serve what
upstream expects; we may serve more that hasn't been registered yet).

### CPU Performance Parity Benchmark

`baselines/python/bench_cpu_parity.py` provides Python timing for the same
algorithms Criterion benches measure in Rust. Enables automated speedup ratio
reporting without third-party benchmarking frameworks.

---

## For projectNUCLEUS / foundation

- **Thread 9 (Gaming)** and **Thread 10 (Provenance)** are `"mapped"` but not
  instrumented in foundation. ludoSpring's paper queue documents the science
  threads and datasets. Ready for `data/sources/thread09_gaming.toml` creation.
- **Workload TOMLs** for projectNUCLEUS `workloads/ludospring/` not yet created.
  Pattern from groundSpring's `gs-validate-all.toml` is the template.

---

## State After This Handoff

- **Tests:** 820+ (workspace `cargo test`)
- **Gaps:** 11 tracked (GAP-01–06, 09, 12–15; 07/08/10/11 resolved)
- **GuideStone:** Level 4 (NUCLEUS validated, 54/54 checks)
- **primalSpring dep:** v0.9.25 (feature-gated `guidestone`)
- **barraCuda dep:** v0.3.11 path (0.3.13 blocked by GAP-13)
- **Deploy graphs:** 12 (+1 composition subgraph)
- **Experiments:** 100 crates (all validate, none stub/placeholder)
- **Registry:** 30 capabilities, bidirectional sync-tested

---

**License:** AGPL-3.0-or-later
