// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp006: Hick's law menu depth — validation binary.
//!
//! Validates Hick's law across different menu architectures:
//! flat menus, hierarchical menus, and combined Fitts+Hick interaction cost.
//!
//! # Provenance
//!
//! Hick (1952), Hyman (1953): RT = a + b * log2(N+1).
//! Hierarchical menus: Cockburn et al. (2007) "A Predictive Model of
//! Menu Performance." CHI '07 — total time = sum of Hick RT per level.
//! Python baseline: `baselines/python/interaction_laws.py` (2026-03-11).

use ludospring_barracuda::interaction::input_laws::{hick_reaction_time, interaction_cost};
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

fn validate_flat_menu(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Flat menu Hick's law sweep");
    let a = tolerances::HICK_A_MS;
    let b = tolerances::HICK_B_MS;

    // Hick RT should increase logarithmically with choices
    let rt_2 = hick_reaction_time(2, a, b);
    let rt_4 = hick_reaction_time(4, a, b);
    let rt_8 = hick_reaction_time(8, a, b);
    let rt_16 = hick_reaction_time(16, a, b);

    let r = ValidationResult::check(
        "exp006_hick_log",
        "Hick RT grows logarithmically: ratio RT(16)/RT(2) < 2",
        rt_16 / rt_2,
        1.5,
        0.5,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp006_hick_mono",
        "RT(2) < RT(4) < RT(8) < RT(16)",
        if rt_2 < rt_4 && rt_4 < rt_8 && rt_8 < rt_16 {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Known value: N=7 → RT = 200 + 150*log2(8) = 650ms
    let r = ValidationResult::check(
        "exp006_hick_n7",
        "Hick RT for N=7 = 650ms (Python baseline)",
        hick_reaction_time(7, a, b),
        650.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_hierarchical_menu(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Hierarchical menu (Cockburn model)");
    let a = tolerances::HICK_A_MS;
    let b = tolerances::HICK_B_MS;

    // Flat: 64 items in one level
    let flat_64 = hick_reaction_time(64, a, b);

    // Hierarchical: 8 items × 8 items (two levels)
    let hier_8x8 = hick_reaction_time(8, a, b) + hick_reaction_time(8, a, b);

    // Hierarchical: 4 × 4 × 4 (three levels)
    let hier_4x4x4 =
        hick_reaction_time(4, a, b) + hick_reaction_time(4, a, b) + hick_reaction_time(4, a, b);

    // With a=200ms, hierarchy adds a per-level overhead.
    // Cockburn et al. (2007): hierarchy trades fewer choices per level
    // against the cost of additional decisions. With high `a`, hierarchy
    // can be slower — this validates the tradeoff is computed correctly.
    let r = ValidationResult::check(
        "exp006_hier_tradeoff",
        "hierarchy trades breadth for depth (8x8 > flat-64 when a is high)",
        if hier_8x8 > flat_64 { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Deep hierarchy adds overhead: 4x4x4 slower than 8x8
    let r = ValidationResult::check(
        "exp006_depth_tradeoff",
        "4x4x4 hierarchy slower than 8x8 (depth overhead)",
        hier_4x4x4 - hier_8x8,
        100.0,
        200.0,
    );
    report(&r);
    results.push(r);
}

fn validate_combined_cost(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Combined Fitts+Hick interaction cost");
    let cost = interaction_cost(
        100.0,
        20.0,
        5,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
        tolerances::HICK_A_MS,
        tolerances::HICK_B_MS,
    );

    // Cost should equal Fitts(100,20) + Hick(5)
    let fitts = tolerances::FITTS_B_MOUSE_MS.mul_add(
        (2.0 * 100.0 / 20.0 + 1.0_f64).log2(),
        tolerances::FITTS_A_MOUSE_MS,
    );
    let hick = tolerances::HICK_B_MS.mul_add(6.0_f64.log2(), tolerances::HICK_A_MS);
    let expected = fitts + hick;

    let r = ValidationResult::check(
        "exp006_combined",
        "interaction_cost = Fitts + Hick",
        cost,
        expected,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp006: Hick's Law Menu Depth (Validation) ===\n");
    let mut results = Vec::new();

    validate_flat_menu(&mut results);
    validate_hierarchical_menu(&mut results);
    validate_combined_cost(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}
