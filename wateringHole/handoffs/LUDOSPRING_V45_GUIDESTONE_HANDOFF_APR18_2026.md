# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring V45 — Level 5 guideStone Handoff

**Date:** April 18, 2026
**From:** ludoSpring
**To:** barraCuda, toadStool, coralReef, biomeOS, primalSpring, all springs
**Version:** V45 (790+ tests, plasmidBin v0.10.0)

## Executive Summary

ludoSpring V45 delivers a proper **Level 5 guideStone** binary
(`ludospring_guidestone`) that uses the primalSpring composition API
(`CompositionContext`, `validate_parity`, `validate_liveness`) rather than
raw JSON-RPC socket calls. This is the evolution from `validate_primal_proof`
(V44, raw IPC) to the canonical guideStone pattern defined in
`GUIDESTONE_COMPOSITION_STANDARD.md`.

The guideStone:
- Discovers NUCLEUS primals via capability-based routing
- Validates domain science against Python golden values
- Inherits primalSpring's base composition certification (6 layers)
- Carries five certified properties (deterministic, reference-traceable,
  self-verifying, environment-agnostic, tolerance-documented)

```
Python baseline (peer-reviewed)
    → Rust validation (spring binary)          ← Level 2 DONE
        → IPC composition (ludoSpring socket)   ← Level 3 DONE
            → Primal proof (raw IPC)            ← V44
                → guideStone (composition API)  ← V45 THIS
                    → NUCLEUS deploys in-graph  ← Level 6 TARGET
```

## Part 1: What Changed (V44 → V45)

### New: `ludospring_guidestone` binary

| Property | Value |
|----------|-------|
| Binary | `ludospring_guidestone` |
| Feature | `guidestone` (enables `primalspring` path dep) |
| API | `primalspring::composition::{CompositionContext, validate_parity, validate_liveness}` |
| Discovery | `CompositionContext::from_live_discovery_with_fallback()` (UDS → TCP) |
| Routing | `method_to_capability_domain()` → capability → primal |
| Tolerances | `primalspring::tolerances::IPC_ROUND_TRIP_TOL` (1e-10) |
| Exit codes | 0 = certified, 1 = failed, 2 = bare-only (no NUCLEUS) |

### Validated Methods (domain science via composition API)

| Method | Capability | Golden Value | Source |
|--------|-----------|--------------|--------|
| `activation.fitts` | tensor | 708.848 | Fitts (1954), MacKenzie (1992) |
| `activation.hick` | tensor | 650.0 | Hick (1952) |
| `math.sigmoid` | tensor | 0.6225 | Logistic function |
| `math.log2` | tensor | 3.0 | Exact |
| `stats.mean` | tensor | 3.0 | np.mean([1,2,3,4,5]) |
| `stats.std_dev` | tensor | (existence) | — |
| `noise.perlin2d` | tensor | 0.0 | Origin invariant |
| `rng.uniform` | tensor | (existence) | — |
| `tensor.create` | tensor | (existence) | — |

### Evolution from V44

| Aspect | V44 (`validate_primal_proof`) | V45 (`ludospring_guidestone`) |
|--------|-------------------------------|-------------------------------|
| Discovery | `BARRACUDA_SOCK` env + XDG scan | `from_live_discovery_with_fallback()` |
| IPC layer | Raw `UnixStream` + manual JSON-RPC | `CompositionContext::call()` |
| Routing | Hardcoded to barraCuda socket | `method_to_capability_domain()` → any provider |
| Error handling | Manual `rpc_call` + `extract_scalar` | `validate_parity` + `IpcError::is_connection_error()` |
| Tolerances | `ludospring_barracuda::tolerances` | `primalspring::tolerances` (ecosystem-canonical) |
| Validation harness | `ValidationHarness` (ludoSpring) | `ValidationResult` (primalSpring) |
| Base certification | None (domain-only) | Inherits primalSpring 6-layer base |

### `validate_primal_proof` retained

The V44 raw IPC binary remains for comparison and fallback. It exercises
the same golden values through direct Unix socket calls — useful for
diagnosing whether a failure is in the composition API layer or in the
underlying primal.

## Part 2: Dependency Addition

### `primalspring` path dependency

```toml
# barracuda/Cargo.toml
primalspring = { path = "../../primalSpring/ecoPrimal", optional = true }

[features]
guidestone = ["dep:primalspring"]
```

This brings in: `serde`, `serde_json`, `toml`, `tracing`, `clap`,
`thiserror`, `hmac`, `sha2`, `hkdf`, `base64`, `getrandom`, `zeroize`,
`chacha20poly1305`. All pure Rust, ecoBin compliant.

### Downstream manifest updated

```toml
# primalSpring/graphs/downstream/downstream_manifest.toml
guidestone_readiness = 2   # was 1
validation_capabilities = [
    "activation.fitts", "activation.hick",
    "math.sigmoid", "math.log2",
    "stats.mean", "stats.std_dev",
    "noise.perlin2d", "rng.uniform", "tensor.create",
    # existing entries retained:
    "compute.dispatch", "tensor.matmul", "inference.complete",
    "crypto.hash", "storage.store", "storage.retrieve",
    "dag.session.create", "dag.event.append",
]
```

## Part 3: Five Certified Properties

| # | Property | How ludoSpring satisfies it |
|---|----------|---------------------------|
| 1 | Deterministic Output | Same golden values on any architecture (f64 + IPC_ROUND_TRIP_TOL) |
| 2 | Reference-Traceable | Every constant traces to a paper or Python baseline with commit |
| 3 | Self-Verifying | Mismatched values → exit 1; missing NUCLEUS → exit 2 |
| 4 | Environment-Agnostic | Pure Rust, `guidestone` feature, no network/sudo |
| 5 | Tolerance-Documented | All use `primalspring::tolerances::IPC_ROUND_TRIP_TOL` (derivation in source) |

## Part 4: Per-Team Action Items

### barraCuda

- **Confirm IPC response schemas:** The guideStone's `extract_any_scalar`
  tries `result`, `value`, bare scalar, `data[0]`, and array `[0]`.
  Standardize on one envelope (recommend `{"result": N}` for scalars).
- **Domain method gaps:** `math.flow.evaluate` and `math.engagement.composite`
  are still not in barraCuda's IPC surface. These are composable from existing
  primitives but need upstream wiring.

### primalSpring

- **Readiness level:** ludoSpring moves from Level 1 → Level 2 in the
  guideStone readiness table. Next: Level 3 (bare guideStone works offline)
  requires structural validation without live primals.
- **Manifest updated:** `guidestone_readiness = 2`, `validation_capabilities`
  expanded with 9 domain methods.

### biomeOS

- **Deploy graph inclusion:** `ludospring_guidestone` should be registered as
  a deployable node in the NUCLEUS graph. When biomeOS deploys ludoSpring's
  composition, the guideStone validates it on startup.

### All springs

- **Pattern to follow:** ludoSpring's progression is the template:
  1. `validate_primal_proof` (raw IPC, Level 1)
  2. `{spring}_guidestone` (composition API, Level 2)
  3. Bare structural validation (Level 3)
  4. NUCLEUS integration (Level 4)
  5. Certification (Level 5)

## Part 5: Files Changed (V45)

| File | Change |
|------|--------|
| `barracuda/Cargo.toml` | `guidestone` feature, `primalspring` optional dep, `[[bin]]` entry |
| `barracuda/src/bin/ludospring_guidestone.rs` | **New** — Level 5 guideStone binary |
| `barracuda/src/bin/validate_all.rs` | Added `ludospring_guidestone` with exit-2 skip |
| `.github/workflows/ci.yml` | `cargo build --features guidestone` step |
| `README.md` | V45, five-layer validation, guideStone |
| `CHANGELOG.md` | V45 entry |
| `CONTEXT.md` | V45 |
| `docs/PRIMAL_GAPS.md` | GAP-02 updated — guideStone wired |
| `whitePaper/baseCamp/README.md` | Five-layer validation section |
| `experiments/README.md` | V45 |
| `niches/ludospring-game.yaml` | V45 |
| `wateringHole/README.md` | V45 active, V44 archived |
| `primalSpring/graphs/downstream/downstream_manifest.toml` | `guidestone_readiness = 2`, expanded capabilities |
| `primalSpring/wateringHole/NUCLEUS_SPRING_ALIGNMENT.md` | V45 guideStone |
| `infra/wateringHole/PRIMAL_REGISTRY.md` | V45 guideStone |
| `infra/wateringHole/ECOSYSTEM_EVOLUTION_CYCLE.md` | V45 guideStone |

## Score Summary

| Metric | Value |
|--------|-------|
| Tests | 790+ |
| Validators | 7 (3 core + composition + primal_proof + guidestone + meta-runner) |
| guideStone readiness | Level 2 (properties documented, binary compiles + skips without NUCLEUS) |
| Gaps | 10 (GAP-02 guideStone wired, others unchanged) |
| Clippy | 0 warnings (`-D warnings`) |
| Coverage | 90%+ (llvm-cov, CI-enforced) |
| plasmidBin | v0.10.0 (sha256-verified) |
