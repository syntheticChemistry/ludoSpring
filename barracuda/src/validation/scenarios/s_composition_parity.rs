// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario: Composition Parity — verify golden JSON targets are consistent.
//! Absorbed from exp099_composition_validation + validate_composition binary.

use super::registry::{Scenario, ScenarioMeta, Tier, Track};
use crate::composition_targets;
use crate::validation::{BaselineProvenance, ValidationHarness};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "composition_parity",
        track: Track::CompositionParity,
        tier: Tier::Rust,
        provenance_crate: "exp099_composition_validation",
        provenance_date: "2026-04-11",
        description: "Verify golden composition targets are self-consistent",
    },
    run: run_composition_parity,
};

#[expect(
    clippy::expect_used,
    reason = "scenario panics on corrupt golden data — intentional fail-fast"
)]
fn run_composition_parity(h: &mut ValidationHarness) {
    let prov = BaselineProvenance {
        script: "baselines/rust/generate_composition_targets.rs",
        commit: "231928a",
        date: "2026-04-17",
        command: "cargo run --example generate_composition_targets",
    };
    h.print_provenance(&[&prov]);

    let snapshot = composition_targets::snapshot();
    let obj = snapshot.as_object().expect("snapshot is object");

    h.check_bool("Golden targets non-empty", !obj.is_empty());

    h.check_bool(
        "Golden targets contain game.evaluate_flow",
        obj.contains_key("game.evaluate_flow"),
    );
    h.check_bool(
        "Golden targets contain game.wfc_step",
        obj.contains_key("game.wfc_step"),
    );
    h.check_bool(
        "Golden targets contain game.fitts_cost",
        obj.contains_key("game.fitts_cost"),
    );
    h.check_bool(
        "Golden targets contain game.generate_noise",
        obj.contains_key("game.generate_noise"),
    );

    // Verify snapshot round-trips through serialization
    let json_str = serde_json::to_string(&snapshot).expect("serialize snapshot");
    let reparsed: serde_json::Value = serde_json::from_str(&json_str).expect("reparse");
    h.check_bool("Golden targets round-trip", reparsed == snapshot);
}
