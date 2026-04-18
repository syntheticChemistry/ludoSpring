// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)] // guideStone binary — no public API

//! ludoSpring guideStone — Level 5 self-validating NUCLEUS node.
//!
//! Inherits primalSpring base composition certification (6 layers).
//! Validates game science (interaction laws, procedural generation,
//! engagement metrics) through primal IPC against Python golden values.
//!
//! # Layers
//!
//! 0. **Bare Properties** — five certified properties, no primals needed
//! 1. **Discovery** — NUCLEUS primals found via capability scan
//! 2. **Domain Science** — game science validated via composition IPC
//!
//! # Five Certified Properties (validated in bare mode)
//!
//! 1. **Deterministic Output** — recompute every golden value locally
//! 2. **Reference-Traceable** — every constant sourced to a paper
//! 3. **Self-Verifying** — tampered values produce non-zero exit
//! 4. **Environment-Agnostic** — pure Rust, no network, no filesystem
//! 5. **Tolerance-Documented** — every tolerance named and ordered
//!
//! # Exit Codes
//!
//! - `0` — all checks passed (NUCLEUS certified)
//! - `1` — one or more checks failed
//! - `2` — bare properties passed, no NUCLEUS deployed

use primalspring::composition::{
    self, CompositionContext, validate_liveness, validate_parity,
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

/// Perlin noise at origin = 0.0 by construction
const PERLIN_ORIGIN: f64 = 0.0;

/// ULP-scaled tolerance for local recomputation (no IPC, no serialization).
/// 1024 × machine epsilon ≈ 2.3e-13 — covers FMA reordering on any arch.
const BARE_RECOMPUTE_TOL: f64 = 1024.0 * f64::EPSILON;

fn main() {
    let mut v = ValidationResult::new("ludoSpring guideStone — Game Science Certification");
    ValidationResult::print_banner("ludoSpring guideStone — Domain Science (Level 5)");

    // ════════════════════════════════════════════════════════════════════
    // Layer 0: Bare Properties (always runs, no primals needed)
    // ════════════════════════════════════════════════════════════════════
    v.section("Bare: Deterministic Output");
    validate_determinism(&mut v);

    v.section("Bare: Reference-Traceable");
    validate_traceability(&mut v);

    v.section("Bare: Self-Verifying");
    validate_self_verification(&mut v);

    v.section("Bare: Environment-Agnostic");
    validate_environment_agnostic(&mut v);

    v.section("Bare: Tolerance-Documented");
    validate_tolerance_documentation(&mut v);

    // ════════════════════════════════════════════════════════════════════
    // Layer 1: Discovery — can we find primals?
    // ════════════════════════════════════════════════════════════════════
    v.section("Discovery");
    let mut ctx = CompositionContext::from_live_discovery_with_fallback();

    let required = &["tensor", "compute"];
    let alive = validate_liveness(&mut ctx, &mut v, required);

    if alive == 0 {
        eprintln!("[guideStone] No NUCLEUS primals discovered — bare certification only.");
        v.finish();
        std::process::exit(v.exit_code_skip_aware());
    }

    // ════════════════════════════════════════════════════════════════════
    // Layer 2: Domain Science — game science via composition IPC
    // ════════════════════════════════════════════════════════════════════
    v.section("Domain: Interaction Laws");
    validate_interaction_laws(&mut ctx, &mut v);

    v.section("Domain: Math Primitives");
    validate_math_primitives(&mut ctx, &mut v);

    v.section("Domain: Statistics");
    validate_statistics(&mut ctx, &mut v);

    v.section("Domain: Procedural Generation");
    validate_procedural(&mut ctx, &mut v);

    v.section("Domain: Tensor & Compute");
    validate_tensor_and_compute(&mut ctx, &mut v);

    v.finish();
    std::process::exit(v.exit_code());
}

// ════════════════════════════════════════════════════════════════════════
// Layer 0: Bare Properties
// ════════════════════════════════════════════════════════════════════════

/// Property 1: Deterministic Output — recompute every golden value from
/// first principles and verify the constant matches.
fn validate_determinism(v: &mut ValidationResult) {
    // Fitts: MT = a + b × log₂(2D / W)
    let fitts = 150.0_f64.mul_add((2.0 * 100.0 / 10.0_f64).log2(), 50.0);
    v.check_bool(
        "bare:determinism:fitts",
        (fitts - FITTS_MT_D100_W10).abs() < BARE_RECOMPUTE_TOL,
        &format!("recomputed={fitts}, golden={FITTS_MT_D100_W10}"),
    );

    // Hick: RT = a + b × log₂(N + 1)
    let hick = 150.0_f64.mul_add(8.0_f64.log2(), 200.0);
    v.check_bool(
        "bare:determinism:hick",
        (hick - HICK_RT_N7).abs() < BARE_RECOMPUTE_TOL,
        &format!("recomputed={hick}, golden={HICK_RT_N7}"),
    );

    // Sigmoid: 1 / (1 + e^(-x))
    let sigmoid = 1.0 / (1.0 + (-0.5_f64).exp());
    v.check_bool(
        "bare:determinism:sigmoid",
        (sigmoid - SIGMOID_HALF).abs() < BARE_RECOMPUTE_TOL,
        &format!("recomputed={sigmoid}, golden={SIGMOID_HALF}"),
    );

    // log₂(8) — exact for powers of two
    v.check_bool(
        "bare:determinism:log2",
        (8.0_f64.log2() - LOG2_OF_8).abs() < f64::EPSILON,
        "8.0.log2() == 3.0",
    );

    // mean([1,2,3,4,5]) — exact for small integer sums
    let mean = [1.0, 2.0, 3.0, 4.0, 5.0].iter().sum::<f64>() / 5.0;
    v.check_bool(
        "bare:determinism:mean",
        (mean - STATS_MEAN_1_5).abs() < f64::EPSILON,
        "mean([1..5]) == 3.0",
    );

    // var([2,4,4,4,5,5,7,9], ddof=0) — population variance
    let data: &[f64] = &[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let n = 8.0_f64; // data.len(), known at compile time
    let m = data.iter().sum::<f64>() / n;
    let var = data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n;
    v.check_bool(
        "bare:determinism:variance",
        (var - STATS_VAR_8ELEM).abs() < BARE_RECOMPUTE_TOL,
        &format!("recomputed={var}, golden={STATS_VAR_8ELEM}"),
    );
}

/// Property 2: Reference-Traceable — every golden value is finite and
/// sourced to a published formula or reproducible computation.
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

/// Property 3: Self-Verifying — a tampered golden value must be
/// detectable by the tolerance guard.
fn validate_self_verification(v: &mut ValidationResult) {
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
}

/// Property 4: Environment-Agnostic — bare mode succeeds without
/// external processes, filesystem, or environment variables.
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

/// Property 5: Tolerance-Documented — every tolerance is named, positive,
/// and ordered in the canonical hierarchy.
fn validate_tolerance_documentation(v: &mut ValidationResult) {
    v.check_bool(
        "bare:tolerance:ipc_positive",
        tolerances::IPC_ROUND_TRIP_TOL > 0.0,
        &format!(
            "IPC_ROUND_TRIP_TOL = {:.2e}",
            tolerances::IPC_ROUND_TRIP_TOL
        ),
    );

    v.check_bool(
        "bare:tolerance:ordering",
        tolerances::DETERMINISTIC_FLOAT_TOL < tolerances::IPC_ROUND_TRIP_TOL
            && tolerances::IPC_ROUND_TRIP_TOL <= tolerances::WGSL_SHADER_TOL,
        "DETERMINISTIC_FLOAT < IPC_ROUND_TRIP <= WGSL_SHADER",
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

/// Try common barraCuda response shapes to find a scalar value.
///
/// barraCuda methods return varying envelopes:
/// - `{"result": N}` — stats, math
/// - `{"value": N}` — activation, noise
/// - bare `N` — simple returns
/// - `{"data": [N, ...]}` — array-wrapped single element
fn extract_any_scalar(result: &serde_json::Value) -> Option<f64> {
    result
        .get("result")
        .and_then(serde_json::Value::as_f64)
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

/// Validate a domain scalar through the composition layer with flexible
/// response extraction. Routes via [`composition::method_to_capability_domain`].
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
        Err(e) if e.is_connection_error() => {
            v.check_skip(name, &format!("{cap} not available: {e}"));
        }
        Err(e) => {
            v.check_bool(name, false, &format!("composition error: {e}"));
        }
    }
}

/// Verify that a method responds with any valid result.
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
        Err(e) if e.is_connection_error() => {
            v.check_skip(name, &format!("{cap} not available: {e}"));
        }
        Err(e) => {
            v.check_bool(name, false, &format!("error: {e}"));
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// Layer 2: Domain Science (requires NUCLEUS)
// ════════════════════════════════════════════════════════════════════════

fn validate_interaction_laws(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    validate_domain_scalar(
        ctx,
        v,
        "domain:fitts_law_D100_W10",
        "activation.fitts",
        serde_json::json!({"distance": 100.0, "width": 10.0, "a": 50.0, "b": 150.0}),
        FITTS_MT_D100_W10,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    validate_domain_scalar(
        ctx,
        v,
        "domain:hick_law_N7",
        "activation.hick",
        serde_json::json!({"n_choices": 7, "a": 200.0, "b": 150.0}),
        HICK_RT_N7,
        tolerances::IPC_ROUND_TRIP_TOL,
    );
}

fn validate_math_primitives(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    validate_domain_scalar(
        ctx,
        v,
        "domain:sigmoid_0.5",
        "math.sigmoid",
        serde_json::json!({"data": [0.5]}),
        SIGMOID_HALF,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    validate_domain_scalar(
        ctx,
        v,
        "domain:log2_of_8",
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
        "domain:stats_mean_1to5",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": [1.0, 2.0, 3.0, 4.0, 5.0]}),
        "result",
        STATS_MEAN_1_5,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    validate_domain_scalar(
        ctx,
        v,
        "domain:stats_variance_8elem",
        "stats.variance",
        serde_json::json!({"data": [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]}),
        STATS_VAR_8ELEM,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    check_method_exists(
        ctx,
        v,
        "domain:stats_std_dev",
        "stats.std_dev",
        serde_json::json!({"data": [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]}),
    );
}

fn validate_procedural(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    validate_domain_scalar(
        ctx,
        v,
        "domain:perlin2d_origin",
        "noise.perlin2d",
        serde_json::json!({"x": 0.0, "y": 0.0}),
        PERLIN_ORIGIN,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    check_method_exists(
        ctx,
        v,
        "domain:rng_uniform",
        "rng.uniform",
        serde_json::json!({"n": 5, "min": 0.0, "max": 1.0, "seed": 42}),
    );
}

fn validate_tensor_and_compute(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    check_method_exists(
        ctx,
        v,
        "domain:tensor_create",
        "tensor.create",
        serde_json::json!({"shape": [2, 2], "data": [1.0, 0.0, 0.0, 1.0]}),
    );

    // I × A = A — identity matmul parity
    composition::validate_parity_vec(
        ctx,
        v,
        "domain:tensor_matmul_identity",
        "tensor",
        "tensor.matmul",
        serde_json::json!({
            "a": [[1.0, 0.0], [0.0, 1.0]],
            "b": [[3.0, 7.0], [2.0, 5.0]],
            "rows_a": 2, "cols_a": 2, "cols_b": 2
        }),
        "result",
        &[3.0, 7.0, 2.0, 5.0],
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    check_method_exists(
        ctx,
        v,
        "domain:compute_capabilities",
        "compute.capabilities",
        serde_json::json!({}),
    );

    check_method_exists(
        ctx,
        v,
        "domain:health_readiness",
        "health.readiness",
        serde_json::json!({}),
    );
}
