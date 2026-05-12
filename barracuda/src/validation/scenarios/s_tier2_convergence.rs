// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario: Tier 2 Convergence — validate live IPC to toadStool and barraCuda
//! precision routing.
//!
//! Exercises the newly-unblocked Pass 14 APIs:
//! - `toadstool.validate` (workload pre-flight)
//! - `barracuda.precision.route` (precision advisory)
//!
//! Graceful degradation: when primals are not reachable, the scenario confirms
//! that the IPC layer returns well-formed unavailability rather than panicking.

use super::registry::{Scenario, ScenarioMeta, Tier, Track};
use crate::ipc::toadstool::{
    PrecisionAdvice, WorkloadValidation, precision_route, validate_workload,
};
use crate::validation::{BaselineProvenance, ValidationHarness};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "tier2_convergence",
        track: Track::CompositionParity,
        tier: Tier::Live,
        provenance_crate: "tier2_pass14",
        provenance_date: "2026-05-12",
        description: "Validate Tier 2 IPC wiring to toadstool.validate and barracuda.precision.route",
    },
    run: run_tier2_convergence,
};

fn run_tier2_convergence(h: &mut ValidationHarness) {
    let prov = BaselineProvenance {
        script: "toadStool S250 + barraCuda v0.4.0 Pass 14 APIs",
        commit: "tier2_v67",
        date: "2026-05-12",
        command: "live IPC pre-flight — degradation acceptance test",
    };
    h.print_provenance(&[&prov]);

    check_validate_workload_graceful(h);
    check_precision_route_graceful(h);
    check_workload_validation_struct(h);
    check_precision_advice_struct(h);
}

fn check_validate_workload_graceful(h: &mut ValidationHarness) {
    let result = validate_workload("ludospring-game-validation.toml");
    match result {
        Ok(v) => {
            if v.available {
                h.check_bool("toadstool.validate returns available pre-flight", true);
                h.check_bool(
                    "toadstool.validate precision_tier is non-empty",
                    !v.precision_tier.is_empty(),
                );
            } else {
                h.check_bool(
                    "toadstool.validate degrades gracefully (warnings present)",
                    !v.warnings.is_empty(),
                );
            }
        }
        Err(_) => {
            h.check_bool("toadstool.validate does not panic on IPC failure", false);
        }
    }
}

fn check_precision_route_graceful(h: &mut ValidationHarness) {
    let result = precision_route("math.sigmoid", "f64");
    match result {
        Ok(p) => {
            if p.available {
                h.check_bool("precision.route returns advisory with tier > 0", p.tier > 0);
                h.check_bool(
                    "precision.route hardware_hint is populated",
                    !p.hardware_hint.is_empty(),
                );
            } else {
                h.check_bool(
                    "precision.route degrades gracefully (tier == 0)",
                    p.tier == 0,
                );
            }
        }
        Err(_) => {
            h.check_bool("precision.route does not panic on IPC failure", false);
        }
    }
}

fn check_workload_validation_struct(h: &mut ValidationHarness) {
    let v = WorkloadValidation::from_response(&serde_json::json!({
        "valid": true,
        "gpu_available": false,
        "precision_tier": "f32",
        "estimated_dispatch_time_ms": 16,
        "warnings": [],
        "required_capabilities": ["compute.dispatch"]
    }));
    h.check_bool("WorkloadValidation parses valid=true", v.valid);
    h.check_bool(
        "WorkloadValidation parses precision_tier=f32",
        v.precision_tier == "f32",
    );
    h.check_bool(
        "WorkloadValidation parses estimated_dispatch_time_ms=16",
        v.estimated_dispatch_time_ms == Some(16),
    );
    h.check_bool(
        "WorkloadValidation parses required_capabilities",
        v.required_capabilities == vec!["compute.dispatch"],
    );
}

fn check_precision_advice_struct(h: &mut ValidationHarness) {
    let p = PrecisionAdvice::from_response(&serde_json::json!({
        "tier": 12,
        "hardware_hint": "tensor_core",
        "requires_compiler": true
    }));
    h.check_bool("PrecisionAdvice parses tier=12", p.tier == 12);
    h.check_bool(
        "PrecisionAdvice parses hardware_hint=tensor_core",
        p.hardware_hint == "tensor_core",
    );
    h.check_bool(
        "PrecisionAdvice parses requires_compiler=true",
        p.requires_compiler,
    );
}
