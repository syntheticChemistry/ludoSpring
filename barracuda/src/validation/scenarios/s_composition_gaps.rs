// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario: Composition Gaps — close the remaining 11/141 checks.
//!
//! Tier 2 (Live): requires deployed primals from plasmidBin. Exercises the
//! IPC paths that blocked the final 11 composition checks:
//!
//! - PG-47: barraCuda `noise.perlin3d` (lattice zero) + `stats.entropy`
//! - PG-48: petalTongue scene push under threading
//! - Squirrel `ai.query` inference routing
//! - Provenance replay edge cases (content ownership attribution)
//!
//! When run against current primals (barraCuda with perlin3d + entropy,
//! petalTongue with threading fix), these checks should now PASS.

use super::registry::{Scenario, ScenarioMeta, Tier, Track};
use crate::ipc::NeuralBridge;
use crate::validation::{BaselineProvenance, ValidationHarness};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "composition_gaps",
        track: Track::CompositionParity,
        tier: Tier::Live,
        provenance_crate: "exp091/exp095/exp096 (fossil)",
        provenance_date: "2026-05-11",
        description: "Close remaining 11/141 composition gaps against live primals",
    },
    run: run_composition_gaps,
};

fn run_composition_gaps(h: &mut ValidationHarness) {
    let prov = BaselineProvenance {
        script: "validation/scenarios/s_composition_gaps.rs",
        commit: "HEAD",
        date: "2026-05-11",
        command: "ludospring validate --tier live --scenario composition_gaps",
    };
    h.print_provenance(&[&prov]);

    let Ok(bridge) = NeuralBridge::discover() else {
        h.check_bool("Neural API available (required for live tier)", false);
        return;
    };

    check_perlin3d(&bridge, h);
    check_entropy(&bridge, h);
    check_petaltongue(&bridge, h);
    check_squirrel(&bridge, h);
    check_provenance(&bridge, h);
}

/// PG-47: barraCuda perlin3d lattice zero at integer coordinates.
fn check_perlin3d(bridge: &NeuralBridge, h: &mut ValidationHarness) {
    let result = bridge.capability_call(
        "compute",
        "noise.perlin3d",
        &serde_json::json!({"x": 1.0, "y": 1.0, "z": 1.0}),
    );
    if let Ok(val) = result {
        let noise_val = extract_f64(&val);
        h.check_abs(
            "PG-47: perlin3d lattice zero at (1,1,1)",
            noise_val,
            0.0,
            1e-10,
        );
    } else {
        h.check_bool("PG-47: noise.perlin3d IPC reachable", false);
    }
}

/// PG-47: barraCuda stats.entropy — Shannon entropy via IPC.
fn check_entropy(bridge: &NeuralBridge, h: &mut ValidationHarness) {
    if let Ok(val) = bridge.capability_call(
        "compute",
        "stats.entropy",
        &serde_json::json!({"counts": [10, 10, 10, 10]}),
    ) {
        h.check_abs(
            "PG-47: stats.entropy uniform 4-bin = ln(4)",
            extract_f64(&val),
            4.0_f64.ln(),
            1e-6,
        );
    } else {
        h.check_bool("PG-47: stats.entropy IPC reachable", false);
    }

    if let Ok(val) = bridge.capability_call(
        "compute",
        "stats.entropy",
        &serde_json::json!({"counts": [100]}),
    ) {
        h.check_abs(
            "PG-47: stats.entropy single bin = 0",
            extract_f64(&val),
            0.0,
            1e-10,
        );
    } else {
        h.check_bool("PG-47: stats.entropy single-bin reachable", false);
    }

    if let Ok(val) = bridge.capability_call(
        "compute",
        "stats.entropy",
        &serde_json::json!({"counts": [99, 1]}),
    ) {
        let e = extract_f64(&val);
        h.check_bool("PG-47: stats.entropy skewed > 0", e > 0.0);
        h.check_bool("PG-47: stats.entropy skewed < uniform", e < 2.0_f64.ln());
    } else {
        h.check_bool("PG-47: stats.entropy skewed reachable", false);
    }
}

/// PG-48: petalTongue scene push + dismiss under threading.
fn check_petaltongue(bridge: &NeuralBridge, h: &mut ValidationHarness) {
    let scene = bridge.capability_call(
        "visualization",
        "visualization.render.scene",
        &serde_json::json!({
            "scene_id": "composition_gap_test",
            "type": "status_board",
            "data": {"message": "composition gap validation"},
        }),
    );
    h.check_bool("PG-48: petalTongue scene push accepted", scene.is_ok());

    let dismiss = bridge.capability_call(
        "visualization",
        "visualization.dismiss",
        &serde_json::json!({"scene_id": "composition_gap_test"}),
    );
    h.check_bool("PG-48: petalTongue dismiss accepted", dismiss.is_ok());
}

/// Squirrel inference routing via ai.query.
fn check_squirrel(bridge: &NeuralBridge, h: &mut ValidationHarness) {
    let result = bridge.capability_call(
        "ai",
        "ai.query",
        &serde_json::json!({
            "messages": [
                {"role": "system", "content": "You are a test NPC. Respond with exactly: OK"},
                {"role": "user", "content": "ping"}
            ],
            "model": "default",
            "max_tokens": 10,
        }),
    );
    h.check_bool("Squirrel: ai.query inference routed", result.is_ok());
}

/// Content ownership: provenance DAG session create.
fn check_provenance(bridge: &NeuralBridge, h: &mut ValidationHarness) {
    let dag_create = bridge.capability_call(
        "provenance",
        "dag.session.create",
        &serde_json::json!({"label": "composition_gap_test"}),
    );
    h.check_bool(
        "Provenance: DAG session create accepted",
        dag_create.is_ok(),
    );
}

fn extract_f64(val: &serde_json::Value) -> f64 {
    val.get("result")
        .or_else(|| val.get("value"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(f64::NAN)
}
