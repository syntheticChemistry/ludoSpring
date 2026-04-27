use primalspring::composition::{self, CompositionContext, is_skip_error};
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
pub const FITTS_MT_D100_W10: f64 = 708.847_613_416_814;

/// Hick's Law: RT = 200 + 150 × log₂(7 + 1) = 650.0
/// Hick (1952)
pub const HICK_RT_N7: f64 = 650.0;

/// Sigmoid(0.5) = 1 / (1 + e⁻⁰·⁵)
pub const SIGMOID_HALF: f64 = 0.622_459_331_201_854_6;

/// log₂(8) = 3.0 (exact)
pub const LOG2_OF_8: f64 = 3.0;

/// np.mean([1, 2, 3, 4, 5]) = 3.0 (exact)
pub const STATS_MEAN_1_5: f64 = 3.0;

/// np.var([2, 4, 4, 4, 5, 5, 7, 9], ddof=0) = 4.0
/// Source: baselines/python/run_all_baselines.py
pub const STATS_VAR_8ELEM: f64 = 4.0;

// ── IPC-expected values (barraCuda formulations) ─────────────────────
// barraCuda uses different formulations for Fitts and Hick than the Python
// baselines. The IPC checks compare against what barraCuda actually computes.
// Python gaps tracked in docs/PRIMAL_GAPS.md (GAP-11).

/// barraCuda Fitts: a + b × log₂(D/W + 1) — differs from Python's log₂(2D/W + 1)
pub const IPC_FITTS_MT_D100_W10: f64 = 568.914_742_795_594_6;

/// barraCuda Hick: a + b × log₂(N) — differs from Python's log₂(N + 1)
pub const IPC_HICK_RT_N7: f64 = 621.103_238_308_640_6;

/// barraCuda sample variance (ddof=1, N-1): 32/7 ≈ 4.571428571428571
pub const IPC_STATS_VAR_8ELEM_SAMPLE: f64 = 32.0 / 7.0;

/// Perlin noise at origin = 0.0 by construction
pub const PERLIN_ORIGIN: f64 = 0.0;

/// ULP-scaled tolerance for local recomputation (no IPC, no serialization).
/// 1024 × machine epsilon ≈ 2.3e-13 — covers FMA reordering on any arch.
pub const BARE_RECOMPUTE_TOL: f64 = 1024.0 * f64::EPSILON;

/// Known test payload for cross-atomic pipeline validation (base64 of
/// "ludospring-guidestone-cross-atomic-test-v1").
pub const CROSS_ATOMIC_PAYLOAD_B64: &str =
    "bHVkb3NwcmluZy1ndWlkZXN0b25lLWNyb3NzLWF0b21pYy10ZXN0LXYx";

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

pub fn validate_domain_scalar(
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

pub fn check_method_exists(
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
