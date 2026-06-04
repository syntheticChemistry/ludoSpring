# Cross-Tier Parity — ludoSpring

**Last updated:** Jun 3, 2026 (V82 — Wave 76 parity. All tiers operational.)
**Pattern source:** `primalSpring/docs/VALIDATION_TIERS.md`, lithoSpore reference implementation
**Status:** Active — All tiers operational. Tier 3 live (12/12 NUCLEUS, trio validated via UDS)

---

## Three-Tier Proof

ludoSpring validates the same science across three implementation tiers:

```
Tier 1: Python baselines    (baselines/python/*.py → golden values)
    │
    │  check_drift.py: |python_result - stored_golden| ≤ ANALYTICAL_TOL
    │
Tier 2: Rust validation     (barracuda + metalForge → computed values)
    │
    │  metalForge/forge/src/parity.rs: |cpu_f64 - gpu_f32| ≤ rel_tol
    │
Tier 3: Primal composition  (NUCLEUS graph → IPC-dispatched values)
    │
    │  validate_composition: |ipc_result - golden| ≤ ANALYTICAL_TOL
    ▼
Cross-tier: Python == Rust == IPC (within documented tolerances)
```

## Tier 1: Python Baselines

| Script | What It Computes | Golden Values |
|--------|------------------|---------------|
| `perlin_noise.py` | Perlin 2D noise field, fBm octave stacking | 64×64 field, 6 octaves |
| `interaction_laws.py` | Fitts, Hick, Steering law movement times | 100 distance×width pairs |
| `flow_engagement.py` | Flow zone, engagement composite, DDA curves | 20 difficulty levels |
| `goms_model.py` | GOMS/KLM task time predictions | 12 task decompositions |
| `lsystem_growth.py` | L-system string expansion (4 iterations) | Koch, Sierpinski, Dragon |
| `bsp_partition.py` | BSP tree partitioning (32×32 grid) | 8 levels, 256 rooms |
| `fun_keys_model.py` | Four Keys to Fun classification | 16 game element profiles |

**Stored golden values:** `baselines/python/combined_baselines.json`
**Drift check:** `python3 baselines/python/check_drift.py` (CI-enforced)
**Tolerance:** `ANALYTICAL_TOL = 1e-10` (from `baselines/python/tolerances.py`)

## Tier 2: Rust Validation

| Module | What It Validates | Tolerance |
|--------|-------------------|-----------|
| `metalForge/forge/src/parity.rs` | Perlin 2D, fBm, sigmoid, raycaster: CPU f64 vs GPU f32 | 1e-5 relative |
| `barracuda/src/validation/scenarios.rs` | 10 scenarios: interaction, procedural, engagement, composition | ANALYTICAL_TOL |
| `benchmarks/` | Criterion: perlin, fBm, raycaster, Fitts/Hick vs Python timing | Performance ratio |

**Cross-tier link:** Rust test `python_parity` in `barracuda/src/validation/` loads
`combined_baselines.json` and compares against Rust computations within `1e-10`.

## Tier 3: Primal Composition (NUCLEUS)

| Validator | What It Proves | Tolerance |
|-----------|----------------|-----------|
| `validate_composition` | IPC call to live primals matches golden values | ANALYTICAL_TOL |
| `validate_primal_proof` | Raw IPC to barraCuda `activation.fitts` matches paper formula | ANALYTICAL_TOL |
| `validate_tower_atomic` | Tower Atomic 6/6 capabilities live-validated | Protocol correctness |

**Trio enrichment:** When provenance primals are reachable:
- rhizoCrypt: DAG session records each validation step
- loamSpine: Final result hash + timestamp committed
- sweetGrass: Attribution braid links result to computation

## Parity Reporting

The `ludospring certify` command exercises all three tiers:

```bash
ludospring certify --tier 1   # Python drift check
ludospring certify --tier 2   # Rust validation scenarios
ludospring certify --tier 3   # NUCLEUS IPC parity
ludospring certify --all      # Full cross-tier proof
```

Output (JSON):
```json
{
  "tier_1": {"status": "pass", "checks": 31, "drift": 0},
  "tier_2": {"status": "pass", "checks": 982, "max_error": 1.2e-7},
  "tier_3": {"status": "pass", "checks": 27, "primals_reached": ["barracuda", "beardog", "nestgate"]},
  "cross_tier_parity": true
}
```

## Degradation Rules (Wave 20 PM Compliance)

- Tier 1 failure → CI blocks merge (Python baselines are SSOT)
- Tier 2 failure → Local build fails (`cargo test` required)
- Tier 3 failure → Graceful skip (primals may be offline)
- Trio unreachable → Domain logic proceeds; provenance enrichment skipped
- Never gate science behind provenance — provenance is enrichment
