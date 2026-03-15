<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# ludoSpring V16 — Niche Deployment Handoff

**Date**: March 15, 2026
**From**: ludoSpring (game science spring)
**To**: biomeOS, toadStool, barraCuda, coralReef, petalTongue
**Status**: Complete — ludoSpring is now a deployable biomeOS niche citizen

---

## Executive Summary

ludoSpring V16 evolves from a validation library to a first-class biomeOS niche
citizen, following the Spring-as-Niche Deployment Standard with groundSpring V102
and airSpring V081 as reference implementations.

**What changed:**
- `lifecycle.status` handler added — ludoSpring is now discoverable by other primals
- Capability domain `game` with 12 methods and semantic mappings
- Domain registration/deregistration with Neural API on startup/shutdown
- `capability.list` now includes operation dependencies and cost estimates (Neural API Enhancements 2+3)
- Passive dispatch metrics emission for Pathway Learner
- Deploy graphs (`graphs/ludospring_deploy.toml`, `graphs/ludospring_gaming_niche.toml`)
- Niche YAML (`niches/ludospring-game.yaml`) with organisms, interactions, customization
- Provenance Trio wired at graph level (all nodes `fallback = "skip"`)

---

## Compliance Checklist

| # | Requirement | Status |
|---|------------|--------|
| 1 | UniBin binary with `server`, `status`, `version` | PASS |
| 2 | JSON-RPC 2.0 over Unix socket | PASS |
| 3 | Socket at `$XDG_RUNTIME_DIR/biomeos/ludospring-${FAMILY_ID}.sock` | PASS |
| 4 | `health.check` and `capability.list` methods | PASS |
| 5 | `lifecycle.status` for discovery probes | PASS (new in V16) |
| 6 | Capability domain `game` with semantic mappings | PASS (new in V16) |
| 7 | Deploy graph at `graphs/ludospring_deploy.toml` | PASS (new in V16) |
| 8 | Niche YAML at `niches/ludospring-game.yaml` | PASS (new in V16) |
| 9 | No hardcoded primal names in production code | PASS |
| 10 | `#![forbid(unsafe_code)]` | PASS |
| 11 | AGPL-3.0-or-later license | PASS |
| 12 | Neural API registration (`lifecycle.register`) | PASS |
| 13 | Clean SIGTERM shutdown with deregistration | PASS (new in V16) |
| 14 | Provenance Trio at graph level | PASS (new in V16) |

---

## Capability Domain: `game`

12 capabilities in the `game` domain:

| Capability | What It Does | Cost Estimate |
|------------|-------------|---------------|
| `game.evaluate_flow` | Csikszentmihalyi flow state evaluation | ~5us, low CPU |
| `game.fitts_cost` | Fitts's law movement time prediction | ~3us, low CPU |
| `game.engagement` | Action density, exploration, persistence metrics | ~10us, low CPU |
| `game.analyze_ui` | Tufte data-ink ratio, information density | ~50us, medium CPU |
| `game.accessibility` | Visual, auditory, motor, cognitive scoring | ~8us, low CPU |
| `game.wfc_step` | Wave function collapse constraint propagation | ~200us, medium CPU |
| `game.difficulty_adjustment` | DDA recommendation from performance window | ~15us, low CPU |
| `game.generate_noise` | Perlin/fBm noise field generation | ~100us, medium CPU |
| `game.begin_session` | Provenance trio session lifecycle (begin) | ~500us, low CPU |
| `game.record_action` | Provenance trio action recording | ~200us, low CPU |
| `game.complete_session` | Provenance trio dehydrate/commit/attribute | ~1ms, low CPU |
| `game.poll_telemetry` | Continuous coordination telemetry stream | ~10us, low CPU |

---

## Deploy Graphs

### `graphs/ludospring_deploy.toml`

5-phase deployment:

```
Phase 1: Tower Atomic (BearDog + Songbird) — security + discovery
Phase 2: ToadStool (optional, fallback=skip) — GPU compute
Phase 3: ludoSpring — game science (12 capabilities)
Phase 4: Validation — health check
Phase 5: Provenance Trio (optional, fallback=skip) — session lifecycle
```

### `graphs/ludospring_gaming_niche.toml`

Composes ludoSpring + petalTongue into a gaming niche:

```
Phase 1: Tower Atomic
Phase 2: ludoSpring (game) + petalTongue (visualization) — parallel
Phase 3: ToadStool (optional) — GPU
Phase 4: Validation
Phase 5: Provenance Trio (optional)
```

---

## Neural API Enhancements

### Enhancement 1: Passive Metrics

Every dispatch emits structured JSON to stderr:
```json
{"primal":"ludospring","op":"game.evaluate_flow","latency_us":4,"ok":true}
```
biomeOS Pathway Learner can scrape these logs for adaptive optimization.

### Enhancement 2: Operation Dependencies

`capability.list` response includes `operation_dependencies` map:
```json
{
  "game.evaluate_flow": { "requires": ["challenge", "skill"] },
  "game.complete_session": { "requires": ["session_id"], "depends_on": ["game.begin_session"] }
}
```

### Enhancement 3: Cost Estimates

`capability.list` response includes `cost_estimates` map:
```json
{
  "game.evaluate_flow": { "typical_latency_us": 5, "cpu_intensity": "low", "memory_bytes": 128 }
}
```

---

## Absorption Candidates

### biomeOS

- **Deploy graph execution**: `graphs/ludospring_deploy.toml` ready for `biomeos deploy`
- **Niche YAML**: `niches/ludospring-game.yaml` ready for BYOB templates
- **Capability domain**: Register `game` domain in `capability_registry.toml` and `capability_domains.rs`
- **Continuous coordination**: Graph includes commented `[graph.tick]` for future 60 Hz evolution

### toadStool

- **GPU dispatch**: `game.generate_noise` and `game.wfc_step` are GPU-promotion candidates
- **Frame budget**: `game.poll_telemetry` reports `frame_budget_ms: 16.67` for budget-aware scheduling
- **Cost estimates**: Pathway Learner can use cost data to optimize GPU dispatch decisions

### petalTongue

- **Gaming niche**: `graphs/ludospring_gaming_niche.toml` composes ludoSpring + petalTongue
- **Interaction feedback**: Fitts/Hick models inform petalTongue's interaction engine layout
- **Accessibility scoring**: `game.accessibility` evaluates petalTongue's rendered output

---

## Files Modified

| File | Change |
|------|--------|
| `barracuda/src/ipc/handlers.rs` | Added `lifecycle.status`, fixed health response, aligned caps, added metrics |
| `barracuda/src/ipc/provenance.rs` | Added `has_active_session()` for telemetry status |
| `barracuda/src/biomeos/mod.rs` | **NEW** — domain definition, registration, deregistration |
| `barracuda/src/lib.rs` | Added `pub mod biomeos` (gated on `ipc` feature) |
| `barracuda/src/bin/ludospring.rs` | Domain registration on startup, deregistration on shutdown |
| `graphs/ludospring_deploy.toml` | **NEW** — 5-phase deploy with provenance |
| `graphs/ludospring_gaming_niche.toml` | **NEW** — gaming niche composition |
| `niches/ludospring-game.yaml` | **NEW** — BYOB niche definition |
| `README.md` | Updated to V16, added niche deployment section |

---

## Continuous Coordination Readiness

The game engine is the first continuous niche (60 Hz tick). ludoSpring provides
two graph nodes: `game_logic` and `metrics`. Once biomeOS supports
`coordination = "Continuous"`, the deploy graph evolves to tick execution.
The `game.poll_telemetry` method already reports `frame_budget_ms` and
streaming/idle status for this transition.

---

*"The spring validates science. The niche deploys it."*
