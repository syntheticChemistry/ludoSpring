# ludoSpring V28 — toadStool/barraCuda Deep Evolution Handoff

**Date:** 2026-03-18
**From:** ludoSpring V28
**To:** toadStool (compute orchestration), barraCuda (math primitives)
**License:** AGPL-3.0-or-later
**Covers:** V27–V28 (code quality sprint + capability-based discovery evolution)

---

## Executive Summary

1. **Capability-based discovery proven at scale** — exp042 fully wired to `discover_primals()`, discovering crypto and IPC primals by capability (`crypto.hash`, `system.ping`) with zero hardcoded names
2. **IPC integration tests hardened** — 6 tests now pass: lifecycle, capability list, flow eval, Fitts cost, health check, error handling. Test isolation fixed (unique socket paths per test)
3. **Zero hardcoded primal names** across entire codebase — all inter-primal references resolved by capability at runtime
4. **Zero `#[allow()]`, zero `unsafe`, zero TODO/FIXME** — complete idiomatic Rust with `#[expect(reason)]` curated dictionary
5. **150+ barraCuda primitives consumed** — 56 experiments use validation, 22 use tolerances, zero duplicate math

## Part 1: Capability-Based Discovery Patterns for Upstream

### Discovery Module (`ipc/discovery.rs`)

The `discover_primals()` → `PrimalRegistry` → `find("capability")` pattern is now battle-tested:

```
PrimalRegistry::new()
  → discovery_dirs() (XDG-compliant: BIOMEOS_SOCKET_DIR > $XDG_RUNTIME_DIR/biomeos/ > /tmp)
  → scan all *.sock files
  → probe_socket() sends lifecycle.status JSON-RPC
  → extract_capabilities() handles 4 response formats (flat, object, nested, double-nested)
  → register by capability
```

**toadStool action**: Adopt `discover_primals()` pattern for compute substrate discovery. The `extract_capabilities()` 4-format parser handles all observed ecosystem responses (airSpring, rhizoCrypt, biomeOS, groundSpring).

### IPC Integration Test Pattern

```rust
fn start_server() -> (PathBuf, Arc<AtomicBool>, JoinHandle<()>) {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = temp_dir().join(format!("test_{}_{id}", process::id()));
    // unique socket per test — prevents parallel test interference
}
```

**toadStool action**: Use atomic counter pattern for IPC test isolation. PID-only paths collide when tests run in parallel within the same binary.

## Part 2: barraCuda Primitive Consumption (V28 State)

| Category | Primitives Used | Experiments |
|----------|----------------|-------------|
| validation | ValidationHarness, BaselineProvenance, OrExit, BufferSink | 56 experiments |
| tolerances | 46+ named constants across 6 submodules | 22 experiments |
| interaction | flow, difficulty, goms, input_laws | 14 experiments |
| metrics | engagement, fun_keys, tufte_gaming | 12 experiments |
| procedural | noise, wfc, bsp, lsystem | 11 experiments |
| game | raycaster, ruleset, voxel, rpgpt | 8 experiments |
| telemetry | event protocol, mapper | 4 experiments |
| ipc/discovery | discover_primals, PrimalRegistry | 1 experiment (exp042) |

### Zero Duplicate Math

No experiment reimplements barraCuda primitives. Verified:
- exp034 has `python_sigmoid`, `python_dot`, etc. — intentional Python-equivalent implementations for parity testing
- exp032 has transfer-cost modeling — domain-specific, explicitly marked for future absorption
- exp055 has `mean_fitness()` — population ecology metric, not general stats

## Part 3: Patterns Worth Absorbing

### 1. Structured `# Errors` Documentation

All public `Result`-returning functions now have `# Errors` doc sections per `missing_errors_doc` warn lint:

```rust
/// # Errors
///
/// Returns [`IpcError::RpcError`] if response contains `"error"` field,
/// or [`IpcError::MissingField`] if neither `"result"` nor `"error"` present.
pub fn extract_rpc_result(...) -> Result<Value, IpcError>
```

**barraCuda action**: Enable `missing_errors_doc = "warn"` workspace-wide and document error conditions on all public `Result`-returning functions.

### 2. Environment-Driven Output Paths

```rust
let base = std::env::var("LUDOSPRING_OUTPUT_DIR").unwrap_or_else(|_| "sandbox".into());
let out_dir = Path::new(&base).join("scenarios");
```

**toadStool action**: Adopt environment variable pattern for all configurable paths. Convention: `{PRIMAL}_OUTPUT_DIR` with sensible default.

### 3. Workspace Lint Centralization

16 experiment `Cargo.toml` files reduced from 5-line local `[lints.clippy]` sections to:

```toml
[lints]
workspace = true
```

Root `Cargo.toml` owns all lint policy. Experiments inherit.

**barraCuda action**: If not already centralized, move all lint config to workspace root.

## Part 4: Quality Metrics (V28)

| Metric | Value |
|--------|-------|
| Experiments | 75 |
| Validation checks | 1692 |
| Unit/integration tests | 450+ |
| Proptest invariants | 19 |
| IPC integration tests | 6 (all passing) |
| `#[allow()]` | 0 |
| `unsafe` | 0 (workspace-level `forbid`) |
| TODO/FIXME/HACK | 0 |
| Hardcoded primal names | 0 |
| Hardcoded paths | 0 |
| clippy warnings | 0 (pedantic + nursery) |
| Files > 1000 LOC | 0 |
| C dependencies | 1 (`renderdoc-sys` via `wgpu-hal`, GPU feature only) |
| Python baseline drift | 0 |

## Part 5: Recommended Upstream Actions

### For barraCuda

| Priority | Action | Why |
|----------|--------|-----|
| P0 | Absorb `ValidationHarness` pattern | 56 experiments validate it works; generic over `ValidationSink` |
| P1 | Absorb `extract_capabilities()` 4-format parser | Handles all observed ecosystem response formats |
| P1 | Absorb `discover_primals()` → `PrimalRegistry` | Battle-tested capability discovery with XDG socket resolution |
| P1 | Enable `missing_errors_doc = "warn"` | ludoSpring proven at scale |
| P2 | Absorb Perlin 2D/3D + fBm | ~200 lines, GPU-ready, validated against Python + fastnoise-lite |
| P2 | Absorb `capability_domains` pattern | Structured introspection for any primal |
| P3 | Absorb `GenericFraudDetector` | Graph-based fraud detection works across gaming/science/medical |

### For toadStool

| Priority | Action | Why |
|----------|--------|-----|
| P0 | Adopt atomic-counter IPC test isolation | PID-only paths cause parallel test failures |
| P1 | Wire `compute.dispatch` to ludoSpring's 8 Tier A GPU modules | All pure math, embarrassingly parallel |
| P1 | Adopt environment-driven configurable paths | `{PRIMAL}_OUTPUT_DIR` convention |
| P2 | Expose `lifecycle.status` with `capabilities` array | Required for `discover_primals()` compatibility |

## License

AGPL-3.0-or-later
