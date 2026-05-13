# ludoSpring V71 — Niche Convergence Handoff for Primals & Springs

**Date:** May 13, 2026
**From:** ludoSpring (Tower Atomic Specialist)
**To:** Upstream primals, sibling springs, downstream products
**Version:** V71 (896 tests, 10 scenarios, zero clippy, zero unsafe)

---

## Executive Summary

ludoSpring is the first atomic specialist to achieve **live validation** of its assigned
NUCLEUS tier (Tower: bearDog + songbird + skunkBat, 6/6 capabilities PASS). The
codebase is at zero deep debt with the MDA Framework, BM-004 matchmaking, and BM-005
chat pipeline newly implemented. Foundation Threads 9+10 are seeded with expressions
and validation targets.

This handoff communicates:
1. Wire protocol corrections discovered during live validation
2. Composition patterns validated for ecosystem adoption
3. Remaining gaps blocked on upstream
4. Recommendations for sibling springs and downstream products

---

## Part 1: Wire Protocol Findings (For All Primal Consumers)

During Tower atomic live validation, ludoSpring discovered protocol discrepancies
between the abstract capability definitions and the actual live wire behavior:

### bearDog (Wave 102+)

| Finding | Impact | Action |
|---------|--------|--------|
| `crypto.sign` / `crypto.verify` / `crypto.hash` expect **base64-encoded** `message`/`data` params | All springs calling these methods with raw UTF-8 strings will get errors | Encode payloads as base64 before calling |
| `crypto.seed_fingerprint` requires `FAMILY_SEED` env var on the server | Server returns error if env not set | Document in deploy recipes |
| Signature output is base64 (len=88 for Ed25519) | Parse as string, not binary | No action — just awareness |

### skunkBat (H2+)

| Finding | Impact | Action |
|---------|--------|--------|
| Audit is routed via `security.audit_log` (NOT `defense.audit`) | The capability registry says `defense.audit` but live wire uses `security.audit_log` | Update your IPC calls to use `security.audit_log` |
| Event format: `{ "event": "...", "source": "...", "details": {...} }` | Standard JSON params | No action |

### songbird (Wave 204+)

| Finding | Impact | Action |
|---------|--------|--------|
| `mesh.peers` returns `{ "peers": [] }` (not `{ "count": N }`) | Parse the array, not a count field | Iterate `result.peers` |
| Solo deployment returns empty peers array (not error) | Valid response even with no peers | Handle gracefully |

**Recommendation:** All springs with Tower composition wiring should verify against
these findings. The corrections have been absorbed upstream in primalSpring's docs.

---

## Part 2: Composition Patterns Validated

ludoSpring validates the following patterns that sibling springs can adopt:

### CallResult Enum (Semantic IPC Error Handling)

The standard `IpcError` conflates "primal unreachable" with "primal returned RPC
error." ludoSpring evolved a `CallResult` enum that distinguishes:

```rust
enum CallResult {
    Success(serde_json::Value),
    RpcError { code: i64, message: String },
    Unreachable,
}
```

This enables validation binaries to report `FAIL` (primal reached, method errored)
vs `SKIP` (primal not deployed) — critical for CI accuracy.

**Adopt in:** All validation binaries that call live primals.

### Pure Composition Deployment

ludoSpring does NOT deploy a binary to plasmidBin. Instead:
- Game science is served by composing primals via `ludospring_cell.toml`
- The UniBin binary (`ludospring`) is for validation, not deployment
- Science runs through IPC to barraCuda, Squirrel, petalTongue, etc.

This pattern is valid for any spring where the domain science is entirely
expressible through primal composition rather than custom compute.

### MDA Framework for Domain Validation Design

The MDA model (Mechanics → Dynamics → Aesthetics) maps to validation design:
- **Mechanics** = rules your system implements
- **Dynamics** = runtime behavior emerging from those rules + user input
- **Aesthetics** = measurable outcomes (the validation targets)

Other springs can apply this lens: define what your system's "aesthetics" are
(correctness, performance, coverage), then validate backward to mechanics.

---

## Part 3: For Upstream Primals

### coralReef
- **GAP-01** remains: `shader.compile` wired in ludoSpring, graceful degrade active
- When SM rebuild ships, ludoSpring will automatically use sovereign compilation
- No action needed from coralReef — we're ready

### bearDog
- Wire corrections absorbed (base64 params documented upstream)
- `FAMILY_SEED` requirement should be in deploy recipe docs
- No code changes needed

### songbird
- `mesh.peers` working correctly
- Solo deployment behavior is well-handled
- No issues

### skunkBat
- `security.audit_log` routing confirmed and documented
- No issues

### toadStool / barraCuda
- Tier 2 wiring (`toadstool.validate`, `barracuda.precision.route`) is in place
- Awaiting Phase 2 compositions where live compute trio is exercised

---

## Part 4: For Sibling Springs

### healthSpring (Nest Atomic Specialist)
- Your `s_nest_atomic` scenario should benefit from GAP-36 resolution
- Wire name finding: `content.*` (CAS) vs `storage.*` (blob) are intentionally distinct per biomeOS v3.53
- The `CallResult` pattern from ludoSpring is directly adoptable for your Nest validation

### airSpring / groundSpring / wetSpring / neuralSpring (Cross-Atomic)
- **Hold on full NUCLEUS until Tower + Node + Nest prove live individually**
- Use ludoSpring's wire corrections (base64 bearDog params, skunkBat method name)
- Foundation thread expressions: Thread 4 (wetSpring), Thread 9 (ludoSpring, DONE), Thread 10 (ludoSpring, DONE)

---

## Part 5: For Downstream Products

### projectNUCLEUS
- ludoSpring's 2 workload TOMLs are ready
- Adaptive tick model (`TICK_MODE=adaptive`) reference available
- Science validation routes through primal composition

### foundation (sporeGarden)
- Thread 9 expression + 13 targets: `GAMING_CREATIVE_SCIENCE.md`
- Thread 10 expression + targets: `PROVENANCE_ECONOMICS.md`
- Both seeded with ludoSpring paper sets (Fitts/Hick/MDA/Lazzaro/Perlin/WFC)

### lithoSpore
- N/A for ludoSpring domain (no LTEE reproduction)
- Cross-domain patterns (GenericFraudDetector, provenance trio) available if needed

---

## Part 6: Remaining Gaps

| Gap | Owner | Status | Blocked On |
|-----|-------|--------|-----------|
| GAP-01: coralReef sovereign compile | coralReef | Wired, blocked | SM rebuild |
| GAP-04: provenance deterministic replay | rhizoCrypt | OPEN | Upstream |
| GAP-05: Trio in proto-nucleate graph | primalSpring | OPEN | Graph completeness |
| GAP-14: Single provenance commit hash | ludoSpring | OPEN | Low priority |

All other gaps (16 total) are RESOLVED.

---

## Metrics

| Metric | Value |
|--------|-------|
| Workspace tests | 896 |
| Validation scenarios | 10 |
| Clippy warnings | 0 |
| Unsafe code | 0 (forbid) |
| External C deps | 0 |
| TODO/FIXME markers | 0 |
| Large files (>800 LOC) | 0 |
| Foundation threads seeded | 2 (Thread 9, Thread 10) |
| Papers implemented | 14 HCI models + MDA |
| Tower capabilities validated live | 6/6 |

---

## biomeOS / neuralAPI Deployment Notes

ludoSpring's cell graph (`ludospring_cell.toml`) composes 12 NUCLEUS nodes:
- Tower Atomic (bearDog, songbird, skunkBat) — validated LIVE
- Compute (barraCuda, toadStool, coralReef) — Tier 2 wired, awaiting live
- Provenance (rhizoCrypt, loamSpine, sweetGrass) — wired, trio not yet live
- AI (Squirrel) — inference routing wired
- Viz (petalTongue) — dashboard + streaming wired

Deployment via biomeOS: `biomeos deploy --graph graphs/ludospring_cell.toml`
Atomic instantiation: fragments (`tower_atomic`, `node_atomic`, `nest_atomic`, `meta_tier`)

The neuralAPI bridge discovers Squirrel and routes `ai.query`/`ai.analyze`/`ai.suggest`
through JSON-RPC. No hardcoded endpoints — capability-based discovery at runtime.
