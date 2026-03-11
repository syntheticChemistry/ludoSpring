// SPDX-License-Identifier: AGPL-3.0-or-later
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
use ludospring_barracuda::validation::ValidationResult;

fn report(r: &ValidationResult) {
    if r.passed {
        println!("  PASS  {}: {}", r.experiment, r.description);
    } else {
        println!(
            "  FAIL  {}: {} (got={:.4}, want={:.4}, tol={:.4})",
            r.experiment, r.description, r.measured, r.expected, r.tolerance
        );
    }
}

/// Model a complete "open inventory, select item, drag to slot" task.
fn inventory_management_cost() -> f64 {
    // Phase 1: Decide which item (Hick's law: 12 inventory slots)
    let decide = hick_reaction_time(12, tolerances::HICK_A_MS, tolerances::HICK_B_MS);

    // Phase 2: Move to item (Fitts's law: D=200px, W=32px icon)
    let acquire = fitts_movement_time(
        200.0,
        32.0,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
    );

    // Phase 3: Drag through inventory (Steering law: D=150px, W=40px lane)
    let drag = steering_time(
        150.0,
        40.0,
        tolerances::STEERING_A_MS,
        tolerances::STEERING_B_MS,
    );

    // Phase 4: GOMS operators: M(decide) + P(click) + K(grab) + P(drag) + K(drop)
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

fn validate_composite_pipeline(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Composite interaction cost pipeline");

    let total = inventory_management_cost();

    // Total should be positive and reasonably bounded
    let r = ValidationResult::check(
        "exp019_inventory_positive",
        "inventory management cost is positive",
        if total > 0.0 { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    println!("  Inventory management total: {total:.1}ms");

    // Each component should contribute
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

    let r = ValidationResult::check(
        "exp019_components_add",
        "composite > sum of HCI components (GOMS adds overhead)",
        if total > component_sum { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_menu_depth_scaling(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Menu depth scaling");

    let cost_4 = menu_navigation_cost(4);
    let cost_8 = menu_navigation_cost(8);
    let cost_16 = menu_navigation_cost(16);
    let cost_32 = menu_navigation_cost(32);

    let r = ValidationResult::check(
        "exp019_monotonic",
        "more options → higher cost (monotonic)",
        if cost_4 < cost_8 && cost_8 < cost_16 && cost_16 < cost_32 {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Growth should be sub-linear (log₂ from Hick's law)
    let growth_4_to_16 = cost_16 - cost_4;
    let growth_16_to_32 = cost_32 - cost_16;
    let r = ValidationResult::check(
        "exp019_sublinear",
        "cost growth is sub-linear (Hick logarithmic)",
        if growth_16_to_32 < growth_4_to_16 {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    println!("  Menu costs: 4={cost_4:.0}, 8={cost_8:.0}, 16={cost_16:.0}, 32={cost_32:.0}");
}

fn validate_analytical_decomposition(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Analytical decomposition");

    // Verify GOMS portion exactly matches analytical value
    let goms_ops = [
        Operator::Mental,
        Operator::Point,
        Operator::Keystroke,
        Operator::Point,
        Operator::Keystroke,
    ];
    let goms_time = task_time(&goms_ops);
    let expected_goms =
        goms::times::MENTAL + 2.0 * goms::times::POINT + 2.0 * goms::times::KEYSTROKE_AVG;

    let r = ValidationResult::check(
        "exp019_goms_exact",
        "GOMS component exactly M + 2P + 2K",
        goms_time,
        expected_goms,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Verify Hick component is analytically correct
    let hick_12 = hick_reaction_time(12, tolerances::HICK_A_MS, tolerances::HICK_B_MS);
    let expected_hick = tolerances::HICK_A_MS + tolerances::HICK_B_MS * (13.0_f64).log2();

    let r = ValidationResult::check(
        "exp019_hick_exact",
        "Hick component exactly a + b·log₂(13)",
        hick_12,
        expected_hick,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp019: Composite Interaction Cost (Validation) ===\n");
    let mut results = Vec::new();

    validate_composite_pipeline(&mut results);
    validate_menu_depth_scaling(&mut results);
    validate_analytical_decomposition(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}
