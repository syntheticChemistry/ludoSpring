# ludoSpring V13 → barraCuda/toadStool Cross-Spring Provenance Handoff — March 13, 2026

**From:** ludoSpring V13
**To:** barraCuda team, toadStool team, coralReef team
**License:** AGPL-3.0-or-later
**Date:** March 13, 2026
**Supersedes:** V3 GPU Evolution Handoff (March 11, 2026)

---

## Executive Summary

ludoSpring V13: **66 experiments, 1349 checks, 138 tests** — all green.

Since V3: **22 new experiments** (exp045-066) adding 939 checks. Key additions:

- **Lysogeny catalog** (exp055-060): 6 proprietary game mechanic recreations from open math — 237 checks
- **Novel Ferment Transcript** (exp061): Full NFT lifecycle with provenance trio — 89 checks
- **Cross-spring provenance** (exp062-066): BearDog signing, field sample/medical scaffolds, cross-domain fraud unification, radiating attribution — 228 checks

The 5 cross-spring provenance experiments (exp062-066) are the primary handoff target.

---

## Part 1: barraCuda Primitive Consumption (Current)

### 1.1 Stable Primitives (unchanged from V3)

| Primitive | Consumer | Status |
|-----------|---------|--------|
| `activations::sigmoid` | `interaction::flow::DifficultyCurve` | Stable |
| `stats::dot` | `metrics::engagement::compute_engagement` | Stable |
| `rng::lcg_step` | `procedural::bsp::generate_bsp` | Stable |
| `rng::state_to_f64` | `procedural::bsp::generate_bsp` | Stable |

### 1.2 Validation Harness (consumed by all 66 experiments)

| Primitive | Consumer | Status |
|-----------|---------|--------|
| `validation::ValidationResult` | All 66 experiments | Stable — hotSpring-pattern check harness |
| `validation::ValidationResult::check()` | All 66 experiments | 1349 calls |

### 1.3 New Experiments (exp062-066) — Minimal barraCuda Surface

exp062-066 use **only** `ValidationResult`. They do NOT consume game, procedural, interaction, or metrics primitives. Their math is domain-specific (provenance, fraud detection, attribution calculation) and lives in the experiment crates.

**Implication for barraCuda**: No new primitive absorption needed from these experiments. The domain models are ludoSpring-local. However, the fraud detection and attribution math may evolve into barracuda primitives if they prove reusable across springs.

---

## Part 2: GPU Shader Promotion Map (Updated from V3)

### Tier A — Ready Now (8 modules, unchanged)

| Module | WGSL Shader | Cross-Spring? | New Consumers Since V3 |
|--------|-------------|---------------|----------------------|
| `procedural::noise::perlin_2d` | `perlin_2d.wgsl` | Yes (wetSpring Anderson QS) | — |
| `procedural::noise::fbm_2d` | `fbm_2d.wgsl` | Potential | — |
| `game::raycaster::cast_rays` | `dda_raycast.wgsl` | No | — |
| `metrics::engagement::compute_engagement` | `engagement_batch.wgsl` | Yes (science exploration) | — |
| `metrics::fun_keys::classify_fun` | `fun_classify.wgsl` | Yes (science exploration) | — |
| `interaction::flow::evaluate_flow` | `flow_eval.wgsl` | Yes (science exploration) | — |
| `interaction::input_laws::*` | `interaction_laws.wgsl` | Potential (healthSpring UI) | — |
| `interaction::goms::task_time` | `goms_batch.wgsl` | Potential | — |

### New Tier A Candidates from exp062-066

| Module | Why GPU? | Priority |
|--------|---------|----------|
| `GenericFraudDetector` (exp065) | Batch fraud analysis across N DAGs — graph adjacency checks are embarrassingly parallel | P3 |
| `compute_distribution` (exp066) | Batch attribution calculation across N value events — weighted sum with decay | P3 |

These are lower priority than the existing Tier A modules but become relevant at scale (100K+ DAGs, 1M+ value events).

---

## Part 3: Cross-Spring Provenance Architecture (New)

### 3.1 What Was Built

| Experiment | Checks | Domain | Key Types |
|-----------|--------|--------|-----------|
| exp064 | 39/39 | BearDog signing | `SignedVertex`, `SignedCertificate`, `SignedBraid`, `ProvenanceChainVerifier` |
| exp062 | 39/39 | Field genomics (wetSpring) | `SampleType`, `SampleCondition`, `CustodyTransfer`, `SampleDag`, `SampleFraudType` |
| exp063 | 35/35 | Medical access (healthSpring) | `RecordType`, `ConsentScope`, `AccessEvent`, `AccessProof`, `MedicalFraudType` |
| exp065 | 74/74 | Cross-domain unification | `GenericFraudDetector`, `DomainVocabulary`, `GenericOp`, `GenericFraudType` |
| exp066 | 41/41 | Radiating attribution | `AttributionChain`, `RadiatingDistribution`, `DecayModel`, `RoleWeighting` |

### 3.2 BearDog IPC Wire Format (exp064)

exp064 validates the JSON-RPC 2.0 wire format for BearDog signing:

```json
{
    "jsonrpc": "2.0",
    "method": "crypto.sign_ed25519",
    "params": {
        "message": "<vertex_content_hex>",
        "key_id": "<signer_key_id>"
    },
    "id": "<uuid>"
}
```

**toadStool action**: When BearDog's `crypto.sign_ed25519` is live, exp064 provides the integration test pattern. The model signatures can be replaced with live IPC calls.

### 3.3 Provenance Trio Dependencies

exp062-064 depend on the provenance trio core crates:

```toml
rhizo-crypt-core = { path = "../../../phase2/rhizoCrypt/crates/rhizo-crypt-core" }
loam-spine-core = { path = "../../../phase2/loamSpine/crates/loam-spine-core" }
sweet-grass-core = { path = "../../../phase2/sweetGrass/crates/sweet-grass-core" }
```

exp065-066 have **zero trio dependencies** — they are pure domain modeling using only `ludospring-barracuda` (validation), `serde`, and `serde_json`.

---

## Part 4: Key Finding — Fraud Detection Is Domain-Agnostic (exp065)

The most significant result for the ecosystem: fraud detection reduces to 5 graph patterns that are identical across gaming, science, and medicine.

| Generic | Gaming (exp053) | Science (exp062) | Medical (exp063) |
|---------|----------------|-------------------|-------------------|
| OrphanObject | OrphanItem | PhantomSample | PhantomAccess |
| DuplicateIdentity | DuplicateCert | DuplicateAccession | ConsentForgery |
| UnauthorizedAction | SpeedViolation | UnauthorizedAccess | UnauthorizedAccess |
| ScopeViolation | ImpossibleKill | MislabeledSpecimen | ScopeViolation |
| BrokenChain | UnattributedLoot | BrokenColdChain | ExpiredConsent |

**barraCuda implication**: If fraud detection becomes a shared primitive (used by wetSpring, healthSpring, ludoSpring), the `GenericFraudDetector` pattern should be absorbed into barraCuda as a domain-agnostic graph analysis module. GPU batch execution would enable real-time fraud scanning across large DAGs.

---

## Part 5: Action Items

### For barraCuda (absorption)

| # | Action | Priority | Experiment |
|---|--------|----------|-----------|
| 1 | Perlin noise as first-class `barracuda::procedural` primitive | P1 | exp044 cross-spring critical |
| 2 | Batch metric shaders (engagement, flow, fun, DDA) | P1 | exp044 proves dual-domain |
| 3 | Consider `GenericFraudDetector` as graph analysis primitive | P3 | exp065 unification |
| 4 | Consider `compute_distribution` as weighted-sum primitive | P3 | exp066 attribution |

### For toadStool (GPU dispatch)

| # | Action | Priority | Experiment |
|---|--------|----------|-----------|
| 1 | GPU dispatch for 8 Tier A modules (noise, raycaster, metrics, interaction) | P1 | exp030 parity proven |
| 2 | coralReef sovereign compile for ludoSpring WGSL shaders | P2 | exp030 GPU→native |
| 3 | BearDog live IPC integration for exp064 signing | P2 | exp064 wire format validated |
| 4 | Batch fraud analysis GPU shader (if absorbed) | P3 | exp065 at scale |

### For coralReef

| # | Action | Priority |
|---|--------|----------|
| 1 | Include ludoSpring Tier A shaders in sovereign compile pipeline | P2 |

---

## Part 6: Quality Gates

| Check | Result |
|-------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy --pedantic` | 0 warnings (barracuda library) |
| `cargo test` | 138 tests, 0 failures |
| 67 validation binaries | 1349 checks, 0 failures |
| `#![forbid(unsafe_code)]` | All crate roots |
| Files > 1000 LOC | 1 (exp061 main.rs — candidate for split) |
| TODO/FIXME/HACK | None in Rust source |

---

## Part 7: Files of Interest

| Path | What |
|------|------|
| `experiments/exp064_beardog_signed_chain/src/signed_chain.rs` | BearDog signing model + IPC wire format |
| `experiments/exp065_cross_domain_fraud/src/unified.rs` | Domain-agnostic fraud detector |
| `experiments/exp066_radiating_attribution/src/attribution.rs` | sunCloud radiating attribution calculator |
| `experiments/exp062_field_sample_provenance/src/sample.rs` | wetSpring sample lifecycle model |
| `experiments/exp063_consent_gated_medical/src/medical.rs` | healthSpring consent-gated access model |
| `whitePaper/gen3/baseCamp/21_sovereign_sample_provenance.md` | Paper 21 — field-to-publication chain-of-custody |
| `whitePaper/gen3/baseCamp/22_zero_knowledge_medical_provenance.md` | Paper 22 — patient-owned medical records |
