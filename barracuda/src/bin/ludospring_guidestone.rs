// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)] // guideStone binary — no public API

//! ludoSpring guideStone — Level 5 self-validating NUCLEUS node.
//!
//! Inherits primalSpring base composition certification (6 layers).
//! Validates game science (interaction laws, procedural generation,
//! engagement metrics) through primal IPC against Python golden values.
//!
//! # Five Certified Properties
//!
//! 1. **Deterministic Output** — same binary, same results, any architecture
//! 2. **Reference-Traceable** — every number traces to a paper or proof
//! 3. **Self-Verifying** — tampered inputs detected, non-zero exit
//! 4. **Environment-Agnostic** — pure Rust, ecoBin, no network, no sudo
//! 5. **Tolerance-Documented** — every tolerance has a derivation
//!
//! # Exit Codes
//!
//! - `0` — all checks passed (NUCLEUS certified)
//! - `1` — one or more checks failed
//! - `2` — no NUCLEUS deployed (bare guideStone only)

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

/// Fitts' Law: MT = 50 + 150 × log₂(2 × 100 / 10) ≈ 708.848
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

/// Perlin noise at origin = 0.0 by construction
const PERLIN_ORIGIN: f64 = 0.0;

fn main() {
    let mut v = ValidationResult::new("ludoSpring guideStone — Game Science Certification");
    ValidationResult::print_banner("ludoSpring guideStone — Domain Science (Level 5)");

    // ── Discovery ──────────────────────────────────────────────────────
    v.section("Discovery");
    let mut ctx = CompositionContext::from_live_discovery_with_fallback();

    let required = &["tensor", "compute"];
    let alive = validate_liveness(&mut ctx, &mut v, required);

    if alive == 0 {
        eprintln!("[guideStone] No NUCLEUS primals discovered — bare only.");
        v.finish();
        std::process::exit(v.exit_code_skip_aware());
    }

    // ── Interaction Laws ───────────────────────────────────────────────
    v.section("Domain: Interaction Laws");
    validate_interaction_laws(&mut ctx, &mut v);

    // ── Math Primitives ────────────────────────────────────────────────
    v.section("Domain: Math Primitives");
    validate_math_primitives(&mut ctx, &mut v);

    // ── Statistics ─────────────────────────────────────────────────────
    v.section("Domain: Statistics");
    validate_statistics(&mut ctx, &mut v);

    // ── Procedural Generation ──────────────────────────────────────────
    v.section("Domain: Procedural Generation");
    validate_procedural(&mut ctx, &mut v);

    // ── Tensor Surface ─────────────────────────────────────────────────
    v.section("Domain: Tensor Surface");
    validate_tensor_surface(&mut ctx, &mut v);

    v.finish();
    std::process::exit(v.exit_code());
}

// ── Helpers ────────────────────────────────────────────────────────────

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

// ── Domain Validation Sections ─────────────────────────────────────────

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

fn validate_tensor_surface(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    check_method_exists(
        ctx,
        v,
        "domain:tensor_create",
        "tensor.create",
        serde_json::json!({"shape": [2, 2], "data": [1.0, 0.0, 0.0, 1.0]}),
    );
}
