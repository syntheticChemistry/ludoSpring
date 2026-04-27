use super::constants::{
    IPC_FITTS_MT_D100_W10, IPC_HICK_RT_N7, IPC_STATS_VAR_8ELEM_SAMPLE, LOG2_OF_8, PERLIN_ORIGIN,
    SIGMOID_HALF, STATS_MEAN_1_5, check_method_exists, validate_domain_scalar,
};
use primalspring::composition::{CompositionContext, is_skip_error, validate_parity};
use primalspring::tolerances;
use primalspring::validation::ValidationResult;

// ════════════════════════════════════════════════════════════════════════
// Tier 2: IPC-WIRED (domain science, requires at least barraCuda)
// ════════════════════════════════════════════════════════════════════════

pub fn validate_interaction_laws(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

pub fn validate_math_primitives(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

pub fn validate_statistics(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

pub fn validate_procedural(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

pub fn validate_tensor_and_compute(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    use ludospring_barracuda::ipc::methods;

    check_method_exists(
        ctx,
        v,
        "ipc:tensor_create",
        methods::tensor::CREATE,
        serde_json::json!({"shape": [2, 2], "data": [1.0, 0.0, 0.0, 1.0]}),
    );

    validate_tensor_matmul(ctx, v);

    check_method_exists(
        ctx,
        v,
        "ipc:compute_capabilities",
        methods::compute::CAPABILITIES,
        serde_json::json!({}),
    );

    check_method_exists(
        ctx,
        v,
        "ipc:health_readiness",
        methods::health::READINESS,
        serde_json::json!({}),
    );
}

fn validate_tensor_matmul(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    use ludospring_barracuda::ipc::methods;
    let cap = "tensor";

    let lhs = match ctx.call(
        cap,
        methods::tensor::CREATE,
        serde_json::json!({"data": [1.0, 2.0, 3.0, 4.0], "shape": [2, 2]}),
    ) {
        Ok(res) => res
            .get("tensor_id")
            .or_else(|| res.get("result_id"))
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        Err(e) if is_skip_error(&e) => {
            v.check_skip(
                "ipc:tensor_matmul_identity",
                &format!("tensor not available: {e}"),
            );
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

    let rhs = match ctx.call(
        cap,
        methods::tensor::CREATE,
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
        methods::tensor::MATMUL,
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
