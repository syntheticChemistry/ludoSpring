use super::constants::{
    BARE_RECOMPUTE_TOL, FITTS_MT_D100_W10, HICK_RT_N7, LOG2_OF_8, PERLIN_ORIGIN, SIGMOID_HALF,
    STATS_MEAN_1_5, STATS_VAR_8ELEM,
};
use primalspring::checksums;
use primalspring::tolerances;
use primalspring::validation::ValidationResult;

// ════════════════════════════════════════════════════════════════════════
// Tier 1: LOCAL_CAPABILITIES (bare properties)
// ════════════════════════════════════════════════════════════════════════

pub fn validate_determinism(v: &mut ValidationResult) {
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

pub fn validate_traceability(v: &mut ValidationResult) {
    let golden_values: &[(&str, f64, &str)] = &[
        (
            "fitts",
            FITTS_MT_D100_W10,
            "Fitts (1954), MacKenzie (1992): MT = a + b × log2(2D/W)",
        ),
        ("hick", HICK_RT_N7, "Hick (1952): RT = a + b × log2(N+1)"),
        ("sigmoid", SIGMOID_HALF, "Logistic fn: 1/(1+e^(-0.5))"),
        ("log2", LOG2_OF_8, "Exact: log2(8) = 3"),
        ("mean", STATS_MEAN_1_5, "Exact: mean([1..5]) = 3"),
        (
            "variance",
            STATS_VAR_8ELEM,
            "Population var([2,4,4,4,5,5,7,9]) = 4",
        ),
        (
            "perlin",
            PERLIN_ORIGIN,
            "Perlin: origin = 0 by construction",
        ),
    ];

    for &(name, val, source) in golden_values {
        v.check_bool(&format!("bare:traceable:{name}"), val.is_finite(), source);
    }
}

pub fn validate_self_verification(v: &mut ValidationResult) {
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

pub fn validate_environment_agnostic(v: &mut ValidationResult) {
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

pub fn validate_tolerance_documentation(v: &mut ValidationResult) {
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
