// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario: Audit Integration — verify skunkBat IPC wiring is operational.
//!
//! Tier 1 (Rust): validates the typed client API, graceful degradation, and
//! audit event payload structure without requiring a live skunkBat primal.
//! Tier 2 (Live): when skunkBat is deployed, verifies round-trip delivery.

use super::registry::{Scenario, ScenarioMeta, Tier, Track};
use crate::ipc::skunkbat;
use crate::validation::{BaselineProvenance, ValidationHarness};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "audit_integration",
        track: Track::CrossAtomic,
        tier: Tier::Both,
        provenance_crate: "ludospring-barracuda ipc::skunkbat",
        provenance_date: "2026-05-11",
        description: "Verify skunkBat audit IPC wiring and graceful degradation",
    },
    run: run_audit_integration,
};

fn run_audit_integration(h: &mut ValidationHarness) {
    let prov = BaselineProvenance {
        script: "barracuda/src/ipc/skunkbat.rs",
        commit: "HEAD",
        date: "2026-05-11",
        command: "ludospring validate --scenario audit_integration",
    };
    h.print_provenance(&[&prov]);

    let r = skunkbat::audit_log("test.ping", "ludospring", &serde_json::json!({}));
    h.check_bool("audit_log returns Ok (graceful degradation)", r.is_ok());

    if let Ok(ref result) = r {
        if result.delivered {
            h.check_bool("audit_log delivered (live skunkBat)", true);
            h.check_bool("audit_log seq assigned", result.seq.is_some());
        } else {
            h.check_bool(
                "audit_log graceful fallback (no skunkBat)",
                result.data.get("reason").is_some(),
            );
        }
    }

    let cert = skunkbat::audit_certification(3, 54, 0, 0);
    h.check_bool("audit_certification returns Ok", cert.is_ok());

    let sess = skunkbat::audit_session(
        "session.begin",
        "test-sess-001",
        &serde_json::json!({"players": 1}),
    );
    h.check_bool("audit_session returns Ok", sess.is_ok());

    let val = skunkbat::audit_validation("s_interaction_laws", "science", 12, 0);
    h.check_bool("audit_validation returns Ok", val.is_ok());

    let trail = skunkbat::query_audit_trail(0, 10);
    h.check_bool("query_audit_trail returns Ok", trail.is_ok());

    let method_const = crate::ipc::methods::security::AUDIT_LOG;
    h.check_bool(
        "security.audit_log method constant is dotted",
        method_const.contains('.'),
    );
}
