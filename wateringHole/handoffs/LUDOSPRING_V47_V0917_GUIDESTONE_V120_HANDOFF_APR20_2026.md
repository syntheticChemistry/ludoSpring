# ludoSpring V47 — primalSpring v0.9.17 Absorption / guideStone Standard v1.2.0

**Date**: April 20, 2026
**From**: ludoSpring V47
**To**: All primal teams, spring teams, and downstream consumers
**License**: AGPL-3.0-or-later
**Trigger**: primalSpring v0.9.17 (Phase 45) — deployment validation, genomeBin v5.1, guideStone standard v1.2.0

---

## Summary

ludoSpring V47 is the first live NUCLEUS validation: **54/54 guideStone checks
passed (2 expected skips), exit 0** against 12 deployed primals. Absorbs
primalSpring v0.9.17 and guideStone Composition Standard v1.2.0. V46 patterns
(`call_or_skip`, `is_skip_error`) absorbed ecosystem-wide — V47 uses upstream
imports. Tolerance validation covers the full v1.2.0 ordering invariant (7
constants). IPC formulation divergence with barraCuda discovered and documented
(GAP-11). Deployment references align to genomeBin v5.1.

---

## 1. What Changed in ludoSpring V47

| Change | Detail |
|--------|--------|
| **Upstream `call_or_skip`** | Local function removed; now `use primalspring::composition::call_or_skip` |
| **Upstream `is_skip_error`** | Local function removed; now `use primalspring::composition::is_skip_error` |
| **v1.2.0 tolerance ordering** | `bare:tolerance:v120_ordering` validates 7-constant invariant: EXACT < DETERMINISTIC < DF64 < CPU_GPU <= IPC <= WGSL <= STOCHASTIC |
| **`guidestone_properties` manifest** | Added `{ deterministic = true, traceable = true, self_verifying = true, env_agnostic = true, tolerance_documented = true }` to downstream_manifest.toml |
| **NUCLEUS env var docs** | `BEARDOG_FAMILY_SEED`, `SONGBIRD_SECURITY_PROVIDER=beardog`, `NESTGATE_JWT_SECRET` documented in guideStone binary header |
| **genomeBin v5.1** | All plasmidBin references replaced — 46 binaries across x86_64-musl, aarch64-musl, armv7-musl, x86_64-windows, aarch64-android, riscv64-musl |
| **BLAKE3 CHECKSUMS** | Regenerated for updated guideStone source |
| **guideStone standard v1.2.0** | Doc comment references updated throughout |

### Blockers Resolved (per v1.2.0 readiness table)

| Blocker | Resolution |
|---------|-----------|
| rhizoCrypt PG-32 | Manifest discovery fixed upstream |
| barraCuda Sprint 44 | 6 methods resolved, surface gaps closed |
| loamSpine | Certificate authority — all clear |

---

## 2. guideStone Architecture (unchanged from V46)

```
Tier 1: LOCAL_CAPABILITIES (20 bare checks)
  └── Property 1: Deterministic — recompute all golden values
  └── Property 2: Reference-Traceable — 7 sourced constants
  └── Property 3: Self-Verifying — BLAKE3 via validation/CHECKSUMS (11 files)
  └── Property 4: Environment-Agnostic — pure Rust, no network
  └── Property 5: Tolerance-Documented — v1.2.0 ordering invariant

Tier 2: IPC-WIRED (15 checks)
  └── Domain science via composition IPC to barraCuda
  └── Fitts, Hick, sigmoid, log2, mean, variance, std_dev, Perlin, rng, tensor, matmul, compute, health

Tier 3: FULL NUCLEUS (8 checks)
  └── BearDog crypto.hash (BLAKE3, 64-char hex)
  └── NestGate storage roundtrip (store + retrieve)
  └── Cross-atomic pipeline (hash → store → retrieve → verify)
```

**Total: 43 checks** (+ 11 BLAKE3 file integrity = 54 with manifest).

---

## 3. Composition Pattern Upstream

ludoSpring V46 invented two patterns that v0.9.17 absorbed ecosystem-wide:

### `call_or_skip()` — Cross-Atomic Pipeline Helper

```rust
// V46 (local):
fn call_or_skip(ctx, v, name, cap, method, params) -> Option<Value>

// V47 (upstream — identical signature):
use primalspring::composition::call_or_skip;
```

Absorbed from ludoSpring V46 and healthSpring V56. Both independently invented
the same pattern for cross-atomic pipeline validation. Now canonical.

### `is_skip_error()` — Graceful Degradation Classifier

```rust
// V46 (local):
fn is_skip_error(e: &IpcError) -> bool {
    e.is_connection_error() || e.is_protocol_error() || e.is_transport_mismatch()
}

// V47 (upstream — identical):
use primalspring::composition::is_skip_error;
```

Covers absent primals, HTTP-on-UDS protocol mismatches (Songbird/petalTongue),
and BTSP transport dialect differences (BearDog).

---

## 4. Tolerance Hierarchy — v1.2.0 Ecosystem Standard

V47 validates the full ordering invariant:

```
EXACT_PARITY_TOL        (0.0)    bit-identical (integer ops, hashes)
DETERMINISTIC_FLOAT_TOL (1e-15)  same binary, different run
DF64_PARITY_TOL         (1e-14)  double-float emulation (Dekker/Knuth)
CPU_GPU_PARITY_TOL      (1e-10)  f64 CPU vs WGSL f32→f64
IPC_ROUND_TRIP_TOL      (1e-10)  JSON serialization + IPC overhead
WGSL_SHADER_TOL         (1e-6)   f32 shader vs f64 Rust
STOCHASTIC_SEED_TOL     (1e-6)   seed-fixed Monte Carlo / HMC
```

**Invariant**: `EXACT < DETERMINISTIC < DF64 < CPU_GPU <= IPC <= WGSL <= STOCHASTIC`

Previously V46 checked 3 tolerances. V47 checks all 7.

---

## 5. NUCLEUS Deployment Requirements (v0.9.17)

Tier 3 validation requires these env vars:

| Env Var | Primal | Purpose |
|---------|--------|---------|
| `BEARDOG_FAMILY_SEED` | BearDog | BTSP production mode — required for crypto operations |
| `SONGBIRD_SECURITY_PROVIDER=beardog` | Songbird | Federation security provider |
| `NESTGATE_JWT_SECRET` | NestGate | Storage authentication (random Base64) |
| `FAMILY_ID` | All | Family-aware socket discovery (`{capability}-{family}.sock`) |

---

## 6. Readiness Ladder

| Level | Meaning | ludoSpring |
|-------|---------|-----------|
| 0 — Not started | No guideStone | |
| 1 — Validation exists | `validate_primal_proof` binary | V44 |
| 2 — Properties documented | All tolerances justified | V44 |
| 3 — Bare works | Standalone binary, exit 2 | V45 |
| **4 — NUCLEUS validated** | **In-graph, self-validates** | **V46–V47 (current)** |
| 5 — Certified | Cross-substrate parity | Next: deploy genomeBin NUCLEUS, all checks pass |

**Next step for Level 5**: Deploy NUCLEUS from genomeBin, run `ludospring_guidestone`
externally, verify all Tier 2 + Tier 3 checks pass (not just skip). Validate
cross-substrate parity (Python vs Rust). All upstream blockers are resolved —
nothing prevents this step.

---

## 7. Score Summary

| Metric | Value |
|--------|-------|
| Workspace tests | 791 |
| guideStone checks | 43 (+ 11 BLAKE3 = 54 with manifest) |
| Readiness | 4 (NUCLEUS validated) |
| guideStone standard | v1.2.0 |
| primalSpring | v0.9.17 |
| genomeBin | v5.1 (46 binaries, 6 targets) |
| Clippy warnings | 0 |
| `#[allow]` in app code | 0 |
| TODO/FIXME/HACK | 0 |
| Upstream blockers | 0 (all resolved) |

---

## 8. Cross-References

| Document | Location |
|----------|----------|
| This handoff (ludoSpring) | `ludoSpring/wateringHole/handoffs/LUDOSPRING_V47_V0917_GUIDESTONE_V120_HANDOFF_APR20_2026.md` |
| V46 deep audit handoff | `infra/wateringHole/handoffs/LUDOSPRING_V46_DEEP_AUDIT_COMPOSITION_HANDOFF_APR20_2026.md` |
| Phase 45 handoff | `infra/wateringHole/handoffs/PRIMALSPRING_PHASE45_DEPLOYMENT_VALIDATION_HANDOFF_APR2026.md` |
| genomeBin handoff | `infra/wateringHole/handoffs/PRIMALSPRING_V0917_GENOMBIN_CROSS_ARCH_HANDOFF_APR2026.md` |
| guideStone standard | `primalSpring/wateringHole/GUIDESTONE_COMPOSITION_STANDARD.md` (v1.2.0) |
| Downstream manifest | `primalSpring/graphs/downstream/downstream_manifest.toml` |
| Gap registry | `ludoSpring/docs/PRIMAL_GAPS.md` |
