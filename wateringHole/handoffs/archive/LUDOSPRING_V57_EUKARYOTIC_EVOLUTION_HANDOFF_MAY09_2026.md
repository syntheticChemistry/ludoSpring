# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring V57 — Interstadial Eukaryotic Evolution

**From:** ludoSpring V57
**Date:** May 9, 2026
**To:** primalSpring (audit response), downstream consumers

---

## Summary

ludoSpring completes the interstadial eukaryotic evolution:
- Single **UniBin** binary (`ludospring`) with `certify`/`validate`/`serve`/`status`/`version`
- **Certification organelle** (`barracuda/src/certification/`) absorbs guidestone three-tier
- **Validation scenarios** (`barracuda/src/validation/scenarios/`) with `ScenarioMeta` registry
- **100 experiment crates fossilized** to `fossilRecord/experiments_prokaryotic_may2026/`
- `ipc` is now the **default feature** (IPC-first for sovereign NUCLEUS deployment)
- All 35 bare `#[allow()]` → `#[expect(, reason)]`
- primalSpring **v0.9.25 pinned** with version constraint
- barraCuda **GAP-13 fixed** (upstream `#[cfg(feature = "gpu")]` gate)
- Workspace slimmed from 103 members → 3 (barracuda, benchmarks, metalForge/forge)

**Tier:** 3 → targeting 4 (IPC-first, certification organelle, scenario registry)

---

## UniBin Subcommands

| Command | Purpose |
|---------|---------|
| `ludospring certify --tier 3` | Run three-tier certification (bare → IPC → NUCLEUS) |
| `ludospring validate --tier rust` | Run Tier 1 scenarios (CI-safe, no IPC) |
| `ludospring validate --list` | List all registered scenarios |
| `ludospring serve` | Start IPC server (germination mode) |
| `ludospring status` | Health + capability discovery |
| `ludospring version` | Print version info |

---

## Architecture (Post-Evolution)

```
barracuda/src/
├── certification/          ← organelle (absorbed guidestone)
│   ├── mod.rs             certify(max_tier) → ValidationResult
│   ├── constants.rs       golden values + helpers
│   ├── tier1.rs           bare (deterministic, traceable, self-verifying)
│   ├── tier2.rs           IPC (domain science via CompositionContext)
│   └── tier3.rs           NUCLEUS (cross-atomic pipeline)
├── validation/
│   ├── mod.rs             ValidationHarness + BaselineProvenance
│   └── scenarios/         ← absorbed experiments
│       ├── mod.rs         build_registry() → ScenarioRegistry
│       ├── registry.rs    ScenarioMeta, Tier, Track, ScenarioRegistry
│       ├── s_interaction_laws.rs
│       ├── s_procedural_gen.rs
│       ├── s_engagement_metrics.rs
│       ├── s_composition_parity.rs
│       └── s_raycaster_budget.rs
├── bin/
│   ├── ludospring.rs      UniBin (certify + validate + serve + status + version)
│   └── guidestone/main.rs legacy compat (delegates to certification::certify)
└── ...
```

---

## Fossil Record

`fossilRecord/experiments_prokaryotic_may2026/` preserves all 100 original
experiment crate sources with full git history. The science is NOT lost —
it lives in:
1. Library modules (`game/`, `interaction/`, `procedural/`, `metrics/`)
2. Absorbed validation scenarios (5 representative)
3. Git history (all commit evolution preserved)

---

## What Remains for Tier 4

1. Close remaining `barracuda::` library calls not behind `ipc` feature
   (GPU types in `game/engine/tensor_ops.rs`, `game/engine/gpu_context.rs`)
2. Verify 60Hz tick budget still holds with IPC-only path
3. Expand scenario registry (absorb more prokaryotic experiments as needed)
4. Replace `discover_primal_tiered()` internals with `CompositionContext`
   (current implementation IS the correct pattern but uses custom code)

---

**License:** AGPL-3.0-or-later
