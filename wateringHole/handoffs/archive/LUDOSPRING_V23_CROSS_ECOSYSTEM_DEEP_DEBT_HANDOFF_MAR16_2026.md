<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# ludoSpring V23 → barraCuda + toadStool Cross-Ecosystem Deep Debt Handoff

**Date:** March 16, 2026
**From:** ludoSpring V23 — 75 experiments, 1692 checks, 394 tests + 12 proptest + 6 IPC integration
**To:** barraCuda team (math primitives), toadStool team (GPU dispatch), All Springs
**Supersedes:** V22 Ecosystem Absorption
**License:** AGPL-3.0-or-later

---

## Executive Summary

V23 is a cross-ecosystem deep debt sprint absorbing patterns from 6 sibling springs
and 7 infrastructure primals reviewed on March 16:

- **Zero `#[allow()]` in entire codebase** — wetSpring V122 curated `#[expect(reason)]` pattern
- **Zero-panic validation binaries** — groundSpring V109 `let Ok else exit(1)` pattern across 14 experiments
- **Centralized `extract_rpc_result()`** — healthSpring V29 DRY IPC error extraction
- **`deny.toml` with `wildcards = "deny"`** — barraCuda Sprint 6 supply chain hardening
- **XDG-compliant socket resolution** — exp042 hardcoded paths evolved to env-based discovery
- **Unit conversion constants** — `MS_PER_SECOND`, `SECONDS_PER_MINUTE`, `DEFAULT_DT_S` centralized
- All files under 750 LOC, zero unsafe, zero mocks in production, `#![forbid(unsafe_code)]`

---

## Part 1: `#[expect(reason)]` Migration (wetSpring V122 pattern, all primals: P1)

Zero `#[allow()]` remaining in entire codebase. 13 files migrated using
wetSpring's curated reason dictionary:

| Reason | Lints |
|--------|-------|
| "test module: fail-fast on setup errors" | `clippy::expect_used`, `clippy::unwrap_used` |
| "validation harness: small-range numeric conversions" | `clippy::cast_possible_truncation`, `cast_sign_loss`, `cast_possible_wrap` |
| "validation harness: counter/timing values within f64 range" | `clippy::cast_precision_loss` |
| "validation harness: explicit cast for readability" | `clippy::cast_lossless` |
| "validation harness: domain-specific nomenclature" | `clippy::doc_markdown`, `clippy::similar_names` |
| "ecosystem convention: primal modules use domain-qualified names" | `clippy::module_name_repetitions` |
| "wire format types for IPC contract" | `dead_code` |

**All primals:** Adopt this dictionary for consistent lint justification across the ecosystem.
Parse `cargo clippy --message-format=json` for `unfulfilled_lint_expectations` to find stale entries.

---

## Part 2: Zero-Panic Validation Binaries (groundSpring V109 pattern, all springs: P1)

14 experiment binaries evolved from `.expect()`/`.unwrap()` to deterministic exit codes:

```rust
let Ok(parsed) = serde_json::from_str::<T>(&data) else {
    eprintln!("FATAL: deserialization failed");
    std::process::exit(1);
};
```

Affected experiments: exp045, exp046, exp048, exp049, exp050, exp051, exp052, exp054,
exp055, exp062, exp063, exp064, exp067, exp068.

**All springs:** Validation binaries should never panic — `exit(1)` gives the CI runner
a clean signal without stack traces or panic hooks.

---

## Part 3: Centralized `extract_rpc_result()` (healthSpring V29 pattern, all primals: P1)

New `ipc::envelope::extract_rpc_result()` replaces duplicated error-extraction
in `discovery.rs` and `neural_bridge.rs`:

```rust
pub fn extract_rpc_result(response: &serde_json::Value) -> Result<serde_json::Value, String>
```

Handles `error.code` + `error.message` extraction with safe defaults.
Two callers rewired, eliminating 12 lines of duplicated logic.

**All primals:** If you parse raw JSON-RPC responses, centralize error extraction.

---

## Part 4: `deny.toml` Supply Chain Hardening (barraCuda Sprint 6 pattern)

New `deny.toml` with `wildcards = "deny"`, license allowlist (AGPL-3.0-or-later,
Apache-2.0, MIT, BSD-2/3, ISC, MPL-2.0, Zlib, Unicode), vulnerability/unmaintained
advisory checks, and source registry restrictions.

---

## Part 5: Hardcoding Evolution

- **exp042**: `/run/user/{uid}/biomeos` → `XDG_RUNTIME_DIR` env with fallback
- **`rpc_call()`**: `&str` → `&Path` for socket paths (type-safe)
- **`engagement.rs`**: `60.0` → `tolerances::SECONDS_PER_MINUTE`
- **`state.rs`**: `1.0 / 60.0` → `tolerances::DEFAULT_DT_S`
- **`mapper.rs`**: `1000.0` → `tolerances::MS_PER_SECOND`

---

## Part 6: Ecosystem Review Findings (March 16)

Reviewed 7 springs + 7 primals. Absorbed patterns from:

| Source | Pattern | ludoSpring Implementation |
|--------|---------|--------------------------|
| wetSpring V122 | `#[expect(reason)]` curated dictionary | Zero `#[allow()]`, 13 files migrated |
| groundSpring V109 | Zero-panic validation binaries | 14 experiments converted |
| healthSpring V29 | `extract_rpc_error()` centralization | `extract_rpc_result()` in envelope.rs |
| barraCuda Sprint 6 | `deny.toml wildcards=deny` | New deny.toml |
| airSpring V084 | Primal binary 4-module pattern | Documented for future primal binary |
| neuralSpring S157 | Tower Atomic HTTP via Songbird | Noted for external HTTP needs |
| biomeOS v2.46 | `CapabilityClient` typed SDK | Noted for future IPC evolution |

### Not Yet Absorbed (Future)

| Source | Pattern | Reason |
|--------|---------|--------|
| biomeOS v2.46 | `CapabilityClient` typed SDK | Requires async runtime; ludoSpring IPC is sync |
| coralReef Iter 52 | Typed `IpcError` enum | Would benefit from ecosystem-wide error type |
| rhizoCrypt 0.13.0 | `checkout_slice` O(1) proofs | DAG experiments not perf-bottlenecked |

---

## Part 7: Code Quality Metrics

| Metric | V22 | V23 |
|--------|-----|-----|
| `#[allow()]` in codebase | 0 production, ~13 experiments | **0 anywhere** |
| `.unwrap()`/`.expect()` in validation | ~80 calls across 15 experiments | **0 in validation code** |
| IPC error extraction patterns | 2 (duplicated) | **1 (centralized)** |
| `deny.toml` | none | **wildcards=deny** |
| Hardcoded paths | 1 (exp042) | **0** |
| Magic numbers in library | 3 (60.0, 1000.0, 1/60) | **0** (named constants) |
| Files modified | — | 36 |
| Net lines | — | +289 |

---

## License

AGPL-3.0-or-later
