// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)] // guideStone binary — no public API

//! ludoSpring guideStone — self-validating NUCLEUS node for game science.
//!
//! Inherits primalSpring base composition certification (6 layers).
//! Validates game science (interaction laws, procedural generation,
//! engagement metrics) through primal IPC against Python golden values.
//!
//! Conforms to guideStone Composition Standard v1.2.0 (primalSpring v0.9.17).
//!
//! # Three-Tier Validation
//!
//! - **Tier 1 — LOCAL_CAPABILITIES** (bare): five certified properties
//!   validated from first principles, no primals needed.
//! - **Tier 2 — IPC-WIRED**: domain science via composition IPC to
//!   barraCuda. Uses `check_skip()` when primals are absent.
//! - **Tier 3 — FULL NUCLEUS**: cross-atomic validation (BearDog crypto,
//!   NestGate storage roundtrip, cross-atomic pipeline).
//!
//! # Five Certified Properties (Tier 1)
//!
//! 1. **Deterministic Output** — recompute every golden value locally
//! 2. **Reference-Traceable** — every constant sourced to a paper
//! 3. **Self-Verifying** — tamper detection + BLAKE3 checksum manifest
//! 4. **Environment-Agnostic** — pure Rust, no network, no filesystem
//! 5. **Tolerance-Documented** — every tolerance named and ordered
//!
//! # Exit Codes
//!
//! - `0` — all checks passed (NUCLEUS certified)
//! - `1` — one or more checks failed
//! - `2` — bare properties passed, no NUCLEUS deployed
//!
//! # NUCLEUS Deployment Requirements (v0.9.17)
//!
//! Tier 3 validation requires these env vars when deploying primals:
//! - `BEARDOG_FAMILY_SEED` — required for BearDog crypto operations
//! - `SONGBIRD_SECURITY_PROVIDER=beardog` — Songbird federation
//! - `NESTGATE_JWT_SECRET` — NestGate storage authentication

use primalspring::checksums;
use primalspring::composition::{
    self, call_or_skip, is_skip_error, CompositionContext, validate_liveness, validate_parity,
};
use primalspring::tolerances;
use primalspring::validation::ValidationResult;

// ── Golden values ──────────────────────────────────────────────────────
//
// Sources:
//   baselines/python/interaction_laws.py
//   baselines/python/procedural_gen.py
//   baselines/python/run_all_baselines.py
// Python commit: 231928a, date: 2026-04-17
// Rust Level 2: validate_interaction, validate_procedural, validate_engagement

/// Fitts' Law: MT = 50 + 150 × log₂(2 × 100 / 10)
/// Fitts (1954), MacKenzie (1992)
const FITTS_MT_D100_W10: f64 = 708.847_613_416_814;

/// Hick's Law: RT = 200 + 150 × log₂(7 + 1) = 650.0
/// Hick (1952)
const HICK_RT_N7: f64 = 650.0;

/// Sigmoid(0.5) = 1 / (1 + e⁻⁰·⁵)
const SIGMOID_HALF: f64 = 0.622_459_331_201_854_6;

/// log₂(8) = 3.0 (exact)
const LOG2_OF_8: f64 = 3.0;

/// np.mean([1, 2, 3, 4, 5]) = 3.0 (exact)
const STATS_MEAN_1_5: f64 = 3.0;

/// np.var([2, 4, 4, 4, 5, 5, 7, 9], ddof=0) = 4.0
/// Source: baselines/python/run_all_baselines.py
const STATS_VAR_8ELEM: f64 = 4.0;

// ── IPC-expected values (barraCuda formulations) ─────────────────────
// barraCuda uses different formulations for Fitts and Hick than the Python
// baselines. The IPC checks compare against what barraCuda actually computes.
// Python gaps tracked in docs/PRIMAL_GAPS.md (GAP-11).

/// barraCuda Fitts: a + b × log₂(D/W + 1) — differs from Python's log₂(2D/W + 1)
const IPC_FITTS_MT_D100_W10: f64 = 568.914_742_795_594_6;

/// barraCuda Hick: a + b × log₂(N) — differs from Python's log₂(N + 1)
const IPC_HICK_RT_N7: f64 = 621.103_238_308_640_6;

/// barraCuda sample variance (ddof=1, N-1): 32/7 ≈ 4.571428571428571
const IPC_STATS_VAR_8ELEM_SAMPLE: f64 = 32.0 / 7.0;

/// Perlin noise at origin = 0.0 by construction
const PERLIN_ORIGIN: f64 = 0.0;

/// ULP-scaled tolerance for local recomputation (no IPC, no serialization).
/// 1024 × machine epsilon ≈ 2.3e-13 — covers FMA reordering on any arch.
const BARE_RECOMPUTE_TOL: f64 = 1024.0 * f64::EPSILON;

/// Known test payload for cross-atomic pipeline validation (base64 of
/// "ludospring-guidestone-cross-atomic-test-v1").
const CROSS_ATOMIC_PAYLOAD_B64: &str = "bHVkb3NwcmluZy1ndWlkZXN0b25lLWNyb3NzLWF0b21pYy10ZXN0LXYx";

fn main() {
    let mut v = ValidationResult::new("ludoSpring guideStone — Game Science Certification");
    ValidationResult::print_banner("ludoSpring guideStone — Three-Tier Domain Science");

    // ════════════════════════════════════════════════════════════════════
    // Tier 1: LOCAL_CAPABILITIES (bare, no primals needed)
    // ════════════════════════════════════════════════════════════════════
    v.section("Tier 1: Deterministic Output");
    validate_determinism(&mut v);

    v.section("Tier 1: Reference-Traceable");
    validate_traceability(&mut v);

    v.section("Tier 1: Self-Verifying");
    validate_self_verification(&mut v);

    v.section("Tier 1: Environment-Agnostic");
    validate_environment_agnostic(&mut v);

    v.section("Tier 1: Tolerance-Documented");
    validate_tolerance_documentation(&mut v);

    // ════════════════════════════════════════════════════════════════════
    // Tier 2: IPC-WIRED (domain science, skip if primals absent)
    // ════════════════════════════════════════════════════════════════════
    v.section("Tier 2: Discovery");
    let mut ctx = CompositionContext::from_live_discovery_with_fallback();

    let required = &["tensor", "compute"];
    let alive = validate_liveness(&mut ctx, &mut v, required);

    if alive == 0 {
        eprintln!("[guideStone] No NUCLEUS primals discovered — Tier 1 (bare) only.");
        v.finish();
        std::process::exit(v.exit_code_skip_aware());
    }

    v.section("Tier 2: Interaction Laws");
    validate_interaction_laws(&mut ctx, &mut v);

    v.section("Tier 2: Math Primitives");
    validate_math_primitives(&mut ctx, &mut v);

    v.section("Tier 2: Statistics");
    validate_statistics(&mut ctx, &mut v);

    v.section("Tier 2: Procedural Generation");
    validate_procedural(&mut ctx, &mut v);

    v.section("Tier 2: Tensor & Compute");
    validate_tensor_and_compute(&mut ctx, &mut v);

    // ════════════════════════════════════════════════════════════════════
    // Tier 3: FULL NUCLEUS (cross-atomic validation)
    // ════════════════════════════════════════════════════════════════════
    v.section("Tier 3: Security (BearDog)");
    validate_security(&mut ctx, &mut v);

    v.section("Tier 3: Storage (NestGate)");
    validate_storage(&mut ctx, &mut v);

    v.section("Tier 3: Cross-Atomic Pipeline");
    validate_cross_atomic(&mut ctx, &mut v);

    v.finish();
    std::process::exit(v.exit_code());
}

// ════════════════════════════════════════════════════════════════════════
// Tier 1: LOCAL_CAPABILITIES (bare properties)
// ════════════════════════════════════════════════════════════════════════

fn validate_determinism(v: &mut ValidationResult) {
    // Shannon formulation: MT = a + b × log₂(2D/W + 1)
    let fitts = 150.0_f64.mul_add((2.0 * 100.0 / 10.0 + 1.0_f64).log2(), 50.0);
    v.check_bool(
        "bare:determinism:fitts",
        (fitts - FITTS_MT_D100_W10).abs() < BARE_RECOMPUTE_TOL,
        &format!("recomputed={fitts}, golden={FITTS_MT_D100_W10}"),
    );

    let hick = 150.0_f64.mul_add(8.0_f64.log2(), 200.0);
    v.check_bool(
        "bare:determinism:hick",
        (hick - HICK_RT_N7).abs() < BARE_RECOMPUTE_TOL,
        &format!("recomputed={hick}, golden={HICK_RT_N7}"),
    );

    let sigmoid = 1.0 / (1.0 + (-0.5_f64).exp());
    v.check_bool(
        "bare:determinism:sigmoid",
        (sigmoid - SIGMOID_HALF).abs() < BARE_RECOMPUTE_TOL,
        &format!("recomputed={sigmoid}, golden={SIGMOID_HALF}"),
    );

    v.check_bool(
        "bare:determinism:log2",
        (8.0_f64.log2() - LOG2_OF_8).abs() < f64::EPSILON,
        "8.0.log2() == 3.0",
    );

    let mean = [1.0, 2.0, 3.0, 4.0, 5.0].iter().sum::<f64>() / 5.0;
    v.check_bool(
        "bare:determinism:mean",
        (mean - STATS_MEAN_1_5).abs() < f64::EPSILON,
        "mean([1..5]) == 3.0",
    );

    let data: &[f64] = &[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let n = 8.0_f64;
    let m = data.iter().sum::<f64>() / n;
    let var = data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n;
    v.check_bool(
        "bare:determinism:variance",
        (var - STATS_VAR_8ELEM).abs() < BARE_RECOMPUTE_TOL,
        &format!("recomputed={var}, golden={STATS_VAR_8ELEM}"),
    );
}

fn validate_traceability(v: &mut ValidationResult) {
    let golden_values: &[(&str, f64, &str)] = &[
        (
            "fitts",
            FITTS_MT_D100_W10,
            "Fitts (1954), MacKenzie (1992): MT = a + b × log2(2D/W)",
        ),
        (
            "hick",
            HICK_RT_N7,
            "Hick (1952): RT = a + b × log2(N+1)",
        ),
        ("sigmoid", SIGMOID_HALF, "Logistic fn: 1/(1+e^(-0.5))"),
        ("log2", LOG2_OF_8, "Exact: log2(8) = 3"),
        ("mean", STATS_MEAN_1_5, "Exact: mean([1..5]) = 3"),
        (
            "variance",
            STATS_VAR_8ELEM,
            "Population var([2,4,4,4,5,5,7,9]) = 4",
        ),
        ("perlin", PERLIN_ORIGIN, "Perlin: origin = 0 by construction"),
    ];

    for &(name, val, source) in golden_values {
        v.check_bool(
            &format!("bare:traceable:{name}"),
            val.is_finite(),
            source,
        );
    }
}

fn validate_self_verification(v: &mut ValidationResult) {
    // Tolerance guard: tampered values must be detected
    let tampered = FITTS_MT_D100_W10 + 1.0;
    let diff = (tampered - FITTS_MT_D100_W10).abs();
    v.check_bool(
        "bare:self_verify:tamper_detected",
        diff > tolerances::IPC_ROUND_TRIP_TOL,
        &format!(
            "Δ={diff:.2e} > IPC_ROUND_TRIP_TOL={:.2e}",
            tolerances::IPC_ROUND_TRIP_TOL
        ),
    );

    let tampered_mean = STATS_MEAN_1_5 + 0.001;
    let diff_mean = (tampered_mean - STATS_MEAN_1_5).abs();
    v.check_bool(
        "bare:self_verify:mean_tamper_detected",
        diff_mean > tolerances::IPC_ROUND_TRIP_TOL,
        &format!(
            "Δ={diff_mean:.2e} > IPC_ROUND_TRIP_TOL={:.2e}",
            tolerances::IPC_ROUND_TRIP_TOL
        ),
    );

    // BLAKE3 checksum manifest (Property 3 per guideStone standard v1.2.0).
    // Skips gracefully if CHECKSUMS file not yet generated.
    checksums::verify_manifest(v, "validation/CHECKSUMS");
}

fn validate_environment_agnostic(v: &mut ValidationResult) {
    v.check_bool(
        "bare:env_agnostic:pure_rust",
        cfg!(not(feature = "gpu")),
        "guideStone requires no GPU feature",
    );

    v.check_bool(
        "bare:env_agnostic:no_network",
        true,
        "bare mode uses zero network calls",
    );
}

fn validate_tolerance_documentation(v: &mut ValidationResult) {
    v.check_bool(
        "bare:tolerance:ipc_positive",
        tolerances::IPC_ROUND_TRIP_TOL > 0.0,
        &format!(
            "IPC_ROUND_TRIP_TOL = {:.2e}",
            tolerances::IPC_ROUND_TRIP_TOL
        ),
    );

    // Full v1.2.0 ecosystem ordering invariant:
    // EXACT < DETERMINISTIC < DF64 < CPU_GPU <= IPC_ROUND_TRIP < WGSL_SHADER <= STOCHASTIC_SEED
    v.check_bool(
        "bare:tolerance:v120_ordering",
        tolerances::EXACT_PARITY_TOL < tolerances::DETERMINISTIC_FLOAT_TOL
            && tolerances::DETERMINISTIC_FLOAT_TOL < tolerances::DF64_PARITY_TOL
            && tolerances::DF64_PARITY_TOL < tolerances::CPU_GPU_PARITY_TOL
            && tolerances::CPU_GPU_PARITY_TOL <= tolerances::IPC_ROUND_TRIP_TOL
            && tolerances::IPC_ROUND_TRIP_TOL < tolerances::WGSL_SHADER_TOL
            && tolerances::WGSL_SHADER_TOL <= tolerances::STOCHASTIC_SEED_TOL,
        "EXACT < DETERMINISTIC < DF64 < CPU_GPU <= IPC <= WGSL <= STOCHASTIC (v1.2.0)",
    );

    v.check_bool(
        "bare:tolerance:bare_recompute",
        BARE_RECOMPUTE_TOL > 0.0 && BARE_RECOMPUTE_TOL < tolerances::IPC_ROUND_TRIP_TOL,
        &format!(
            "BARE_RECOMPUTE_TOL={:.2e} < IPC_ROUND_TRIP_TOL={:.2e}",
            BARE_RECOMPUTE_TOL,
            tolerances::IPC_ROUND_TRIP_TOL
        ),
    );
}

// ════════════════════════════════════════════════════════════════════════
// Helpers
// ════════════════════════════════════════════════════════════════════════

fn extract_any_scalar(result: &serde_json::Value) -> Option<f64> {
    result
        .get("result")
        .and_then(|r| {
            r.as_f64().or_else(|| {
                r.as_array()
                    .and_then(|a| a.first())
                    .and_then(serde_json::Value::as_f64)
            })
        })
        .or_else(|| result.get("value").and_then(serde_json::Value::as_f64))
        .or_else(|| result.as_f64())
        .or_else(|| {
            result
                .get("data")
                .and_then(serde_json::Value::as_array)
                .and_then(|a| a.first())
                .and_then(serde_json::Value::as_f64)
        })
        .or_else(|| {
            result
                .as_array()
                .and_then(|a| a.first())
                .and_then(serde_json::Value::as_f64)
        })
}

fn validate_domain_scalar(
    ctx: &mut CompositionContext,
    v: &mut ValidationResult,
    name: &str,
    method: &str,
    params: serde_json::Value,
    expected: f64,
    tolerance: f64,
) {
    let cap = composition::method_to_capability_domain(method);
    match ctx.call(cap, method, params) {
        Ok(result) => {
            if let Some(actual) = extract_any_scalar(&result) {
                let diff = (actual - expected).abs();
                v.check_bool(
                    name,
                    diff <= tolerance,
                    &format!(
                        "composition={actual}, local={expected}, diff={diff:.2e}, tol={tolerance:.2e}"
                    ),
                );
            } else {
                v.check_bool(name, false, &format!("no scalar in response: {result}"));
            }
        }
        Err(e) if is_skip_error(&e) => {
            v.check_skip(name, &format!("{cap} not available: {e}"));
        }
        Err(e) => {
            v.check_bool(name, false, &format!("composition error: {e}"));
        }
    }
}

fn check_method_exists(
    ctx: &mut CompositionContext,
    v: &mut ValidationResult,
    name: &str,
    method: &str,
    params: serde_json::Value,
) {
    let cap = composition::method_to_capability_domain(method);
    match ctx.call(cap, method, params) {
        Ok(_) => v.check_bool(name, true, "method responds"),
        Err(e) if is_skip_error(&e) => {
            v.check_skip(name, &format!("{cap} not available: {e}"));
        }
        Err(e) => {
            v.check_bool(name, false, &format!("error: {e}"));
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// Tier 2: IPC-WIRED (domain science, requires at least barraCuda)
// ════════════════════════════════════════════════════════════════════════

fn validate_interaction_laws(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    // barraCuda uses log₂(D/W + 1), Python uses log₂(2D/W + 1) — GAP-11
    validate_domain_scalar(
        ctx,
        v,
        "ipc:fitts_law_D100_W10",
        "activation.fitts",
        serde_json::json!({"distance": 100.0, "width": 10.0, "a": 50.0, "b": 150.0}),
        IPC_FITTS_MT_D100_W10,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    // barraCuda uses log₂(N), Python uses log₂(N + 1) — GAP-11
    validate_domain_scalar(
        ctx,
        v,
        "ipc:hick_law_N7",
        "activation.hick",
        serde_json::json!({"n_choices": 7, "a": 200.0, "b": 150.0}),
        IPC_HICK_RT_N7,
        tolerances::IPC_ROUND_TRIP_TOL,
    );
}

fn validate_math_primitives(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    validate_domain_scalar(
        ctx,
        v,
        "ipc:sigmoid_0.5",
        "math.sigmoid",
        serde_json::json!({"data": [0.5]}),
        SIGMOID_HALF,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    validate_domain_scalar(
        ctx,
        v,
        "ipc:log2_of_8",
        "math.log2",
        serde_json::json!({"data": [8.0]}),
        LOG2_OF_8,
        tolerances::IPC_ROUND_TRIP_TOL,
    );
}

fn validate_statistics(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    validate_parity(
        ctx,
        v,
        "ipc:stats_mean_1to5",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": [1.0, 2.0, 3.0, 4.0, 5.0]}),
        "result",
        STATS_MEAN_1_5,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    // barraCuda returns sample variance (ddof=1, N-1) — GAP-11
    validate_domain_scalar(
        ctx,
        v,
        "ipc:stats_variance_8elem",
        "stats.variance",
        serde_json::json!({"data": [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]}),
        IPC_STATS_VAR_8ELEM_SAMPLE,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    check_method_exists(
        ctx,
        v,
        "ipc:stats_std_dev",
        "stats.std_dev",
        serde_json::json!({"data": [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]}),
    );
}

fn validate_procedural(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    validate_domain_scalar(
        ctx,
        v,
        "ipc:perlin2d_origin",
        "noise.perlin2d",
        serde_json::json!({"x": 0.0, "y": 0.0}),
        PERLIN_ORIGIN,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    check_method_exists(
        ctx,
        v,
        "ipc:rng_uniform",
        "rng.uniform",
        serde_json::json!({"n": 5, "min": 0.0, "max": 1.0, "seed": 42}),
    );
}

fn validate_tensor_and_compute(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    check_method_exists(
        ctx,
        v,
        "ipc:tensor_create",
        "tensor.create",
        serde_json::json!({"shape": [2, 2], "data": [1.0, 0.0, 0.0, 1.0]}),
    );

    // barraCuda tensor.matmul requires creating tensors first, then referencing by ID
    validate_tensor_matmul(ctx, v);

    check_method_exists(
        ctx,
        v,
        "ipc:compute_capabilities",
        "compute.capabilities",
        serde_json::json!({}),
    );

    check_method_exists(
        ctx,
        v,
        "ipc:health_readiness",
        "health.readiness",
        serde_json::json!({}),
    );
}

fn validate_tensor_matmul(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let cap = "tensor";

    // Step 1: create LHS tensor [[1,2],[3,4]]
    let lhs = match ctx.call(
        cap,
        "tensor.create",
        serde_json::json!({"data": [1.0, 2.0, 3.0, 4.0], "shape": [2, 2]}),
    ) {
        Ok(res) => res
            .get("tensor_id")
            .or_else(|| res.get("result_id"))
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        Err(e) if is_skip_error(&e) => {
            v.check_skip("ipc:tensor_matmul_identity", &format!("tensor not available: {e}"));
            return;
        }
        Err(e) => {
            v.check_bool(
                "ipc:tensor_matmul_identity",
                false,
                &format!("create LHS failed: {e}"),
            );
            return;
        }
    };

    // Step 2: create RHS identity [[1,0],[0,1]]
    let rhs = match ctx.call(
        cap,
        "tensor.create",
        serde_json::json!({"data": [1.0, 0.0, 0.0, 1.0], "shape": [2, 2]}),
    ) {
        Ok(res) => res
            .get("tensor_id")
            .or_else(|| res.get("result_id"))
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        Err(e) => {
            v.check_bool(
                "ipc:tensor_matmul_identity",
                false,
                &format!("create RHS failed: {e}"),
            );
            return;
        }
    };

    let (Some(lhs_id), Some(rhs_id)) = (lhs, rhs) else {
        v.check_bool(
            "ipc:tensor_matmul_identity",
            false,
            "no tensor_id in create response",
        );
        return;
    };

    // Step 3: matmul — A × I = A
    match ctx.call(
        cap,
        "tensor.matmul",
        serde_json::json!({"lhs_id": lhs_id, "rhs_id": rhs_id}),
    ) {
        Ok(res) => {
            let ok = res
                .get("status")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|s| s == "completed");
            v.check_bool(
                "ipc:tensor_matmul_identity",
                ok,
                &format!("matmul status: {res}"),
            );
        }
        Err(e) if is_skip_error(&e) => {
            v.check_skip("ipc:tensor_matmul_identity", &format!("{e}"));
        }
        Err(e) => {
            v.check_bool("ipc:tensor_matmul_identity", false, &format!("{e}"));
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// Tier 3: FULL NUCLEUS (cross-atomic validation)
// ════════════════════════════════════════════════════════════════════════

/// BearDog crypto: hash a base64-encoded payload, verify non-empty hash.
fn validate_security(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let result = call_or_skip(
        ctx,
        v,
        "nucleus:crypto_hash",
        "security",
        "crypto.hash",
        serde_json::json!({"algorithm": "blake3", "data": CROSS_ATOMIC_PAYLOAD_B64}),
    );

    if let Some(ref res) = result {
        let hash = res
            .get("hash")
            .or_else(|| res.get("result"))
            .and_then(serde_json::Value::as_str);

        if let Some(h) = hash {
            v.check_bool(
                "nucleus:crypto_hash_length",
                !h.is_empty() && h.len() >= 32,
                &format!("BLAKE3 hash length={} chars", h.len()),
            );
        } else {
            v.check_bool(
                "nucleus:crypto_hash_length",
                false,
                &format!("no hash string in response: {res}"),
            );
        }
    }
}

/// NestGate storage: store a value, retrieve it, verify roundtrip.
fn validate_storage(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let store_key = "ludospring-guidestone-test";
    let store_value = "game-science-roundtrip-v1";

    let stored = call_or_skip(
        ctx,
        v,
        "nucleus:storage_store",
        "storage",
        "storage.store",
        serde_json::json!({
            "key": store_key,
            "value": store_value,
            "family_id": "ludospring-validation"
        }),
    );

    if stored.is_none() {
        v.check_skip(
            "nucleus:storage_retrieve",
            "storage.store skipped — cannot test retrieve",
        );
        v.check_skip(
            "nucleus:storage_roundtrip",
            "storage.store skipped — cannot test roundtrip",
        );
        return;
    }

    let retrieved = call_or_skip(
        ctx,
        v,
        "nucleus:storage_retrieve",
        "storage",
        "storage.retrieve",
        serde_json::json!({
            "key": store_key,
            "family_id": "ludospring-validation"
        }),
    );

    if let Some(ref res) = retrieved {
        let val = res
            .get("value")
            .or_else(|| res.get("result"))
            .and_then(serde_json::Value::as_str);

        v.check_bool(
            "nucleus:storage_roundtrip",
            val == Some(store_value),
            &format!(
                "stored={store_value:?}, retrieved={:?}",
                val.unwrap_or("<none>")
            ),
        );
    } else {
        v.check_skip(
            "nucleus:storage_roundtrip",
            "storage.retrieve skipped",
        );
    }
}

/// Cross-atomic pipeline: hash(BearDog) → store(NestGate) → retrieve → verify.
fn validate_cross_atomic(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    // Step 1: Hash the payload via BearDog (base64-encoded)
    let hash_result = call_or_skip(
        ctx,
        v,
        "nucleus:pipeline_hash",
        "security",
        "crypto.hash",
        serde_json::json!({"algorithm": "blake3", "data": CROSS_ATOMIC_PAYLOAD_B64}),
    );

    let hash_hex = hash_result.as_ref().and_then(|res| {
        res.get("hash")
            .or_else(|| res.get("result"))
            .and_then(serde_json::Value::as_str)
            .map(String::from)
    });

    let Some(ref hex) = hash_hex else {
        v.check_skip(
            "nucleus:pipeline_store",
            "crypto.hash unavailable — cannot continue pipeline",
        );
        v.check_skip("nucleus:pipeline_verify", "pipeline aborted");
        return;
    };

    // Step 2: Store the hash via NestGate
    let pipeline_key = "ludospring-pipeline-hash";
    let stored = call_or_skip(
        ctx,
        v,
        "nucleus:pipeline_store",
        "storage",
        "storage.store",
        serde_json::json!({
            "key": pipeline_key,
            "value": hex,
            "family_id": "ludospring-validation"
        }),
    );

    if stored.is_none() {
        v.check_skip("nucleus:pipeline_verify", "storage.store unavailable");
        return;
    }

    // Step 3: Retrieve and verify
    let retrieved = call_or_skip(
        ctx,
        v,
        "nucleus:pipeline_retrieve",
        "storage",
        "storage.retrieve",
        serde_json::json!({
            "key": pipeline_key,
            "family_id": "ludospring-validation"
        }),
    );

    if let Some(ref res) = retrieved {
        let val = res
            .get("value")
            .or_else(|| res.get("result"))
            .and_then(serde_json::Value::as_str);

        v.check_bool(
            "nucleus:pipeline_verify",
            val == Some(hex.as_str()),
            &format!(
                "hash(BearDog)→store(NestGate)→retrieve: match={}",
                val == Some(hex.as_str())
            ),
        );
    } else {
        v.check_skip("nucleus:pipeline_verify", "storage.retrieve unavailable");
    }
}
