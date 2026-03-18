// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp019: Composite interaction cost pipeline — validation binary.
//!
//! Chains Fitts + Hick + Steering + GOMS into a complete interaction
//! cost model for compound game tasks. Validates that the composite
//! pipeline produces correct totals and ordering.
//!
//! # Provenance
//!
//! Card, Moran & Newell (1983): composite task analysis.
//! Fitts (1954), Hick (1952), Accot & Zhai (1997): component laws.

use ludospring_barracuda::interaction::goms::{self, Operator, task_time};
use ludospring_barracuda::interaction::input_laws::{
    fitts_movement_time, hick_reaction_time, steering_time,
};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Card et al. 1983, Fitts, Hick, Accot & Zhai)",
    commit: "74cf9488",
    date: "2026-03-15",
    command: "N/A (analytical)",
};

/// Model a complete "open inventory, select item, drag to slot" task.
fn inventory_management_cost() -> f64 {
    let decide = hick_reaction_time(12, tolerances::HICK_A_MS, tolerances::HICK_B_MS);

    let acquire = fitts_movement_time(
        200.0,
        32.0,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
    );

    let drag = steering_time(
        150.0,
        40.0,
        tolerances::STEERING_A_MS,
        tolerances::STEERING_B_MS,
    );

    let goms_overhead = task_time(&[
        Operator::Mental,
        Operator::Point,
        Operator::Keystroke,
        Operator::Point,
        Operator::Keystroke,
    ]);

    decide + acquire + drag + goms_overhead
}

/// Model a "navigate menu, select option" task.
fn menu_navigation_cost(n_options: usize) -> f64 {
    let decide = hick_reaction_time(n_options, tolerances::HICK_A_MS, tolerances::HICK_B_MS);
    let acquire = fitts_movement_time(
        100.0,
        24.0,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
    );
    let goms = task_time(&[Operator::Mental, Operator::Point, Operator::Keystroke]);
    decide + acquire + goms
}

fn validate_composite_pipeline(h: &mut ValidationHarness) {
    let total = inventory_management_cost();

    h.check_bool("inventory management cost is positive", total > 0.0);

    let decide = hick_reaction_time(12, tolerances::HICK_A_MS, tolerances::HICK_B_MS);
    let acquire = fitts_movement_time(
        200.0,
        32.0,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
    );
    let drag = steering_time(
        150.0,
        40.0,
        tolerances::STEERING_A_MS,
        tolerances::STEERING_B_MS,
    );
    let component_sum = decide + acquire + drag;

    h.check_bool(
        "composite > sum of HCI components (GOMS adds overhead)",
        total > component_sum,
    );
}

fn validate_menu_depth_scaling(h: &mut ValidationHarness) {
    let cost_4 = menu_navigation_cost(4);
    let cost_8 = menu_navigation_cost(8);
    let cost_16 = menu_navigation_cost(16);
    let cost_32 = menu_navigation_cost(32);

    h.check_bool(
        "more options → higher cost (monotonic)",
        cost_4 < cost_8 && cost_8 < cost_16 && cost_16 < cost_32,
    );

    let growth_4_to_16 = cost_16 - cost_4;
    let growth_16_to_32 = cost_32 - cost_16;
    h.check_bool(
        "cost growth is sub-linear (Hick logarithmic)",
        growth_16_to_32 < growth_4_to_16,
    );
}

fn validate_analytical_decomposition(h: &mut ValidationHarness) {
    let goms_ops = [
        Operator::Mental,
        Operator::Point,
        Operator::Keystroke,
        Operator::Point,
        Operator::Keystroke,
    ];
    let goms_time = task_time(&goms_ops);
    let expected_goms = 2.0f64.mul_add(
        goms::times::KEYSTROKE_AVG,
        2.0f64.mul_add(goms::times::POINT, goms::times::MENTAL),
    );

    h.check_abs(
        "GOMS component exactly M + 2P + 2K",
        goms_time,
        expected_goms,
        tolerances::ANALYTICAL_TOL,
    );

    let hick_12 = hick_reaction_time(12, tolerances::HICK_A_MS, tolerances::HICK_B_MS);
    let expected_hick = tolerances::HICK_B_MS.mul_add((13.0_f64).log2(), tolerances::HICK_A_MS);

    h.check_abs(
        "Hick component exactly a + b·log₂(13)",
        hick_12,
        expected_hick,
        tolerances::ANALYTICAL_TOL,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp019_composite_interaction_cost");
    h.print_provenance(&[&PROVENANCE]);

    validate_composite_pipeline(&mut h);
    validate_menu_depth_scaling(&mut h);
    validate_analytical_decomposition(&mut h);

    h.finish();
}
