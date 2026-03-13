// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp013: L-system protein backbone growth — validation binary.
//!
//! Validates Lindenmayer system implementations: algae growth (Fibonacci
//! sequence), Koch curve (self-similar fractal), and protein backbone
//! generation (helix/sheet/linker/turn elements).
//!
//! # Provenance
//!
//! Lindenmayer (1968). "Mathematical models for cellular interactions."
//! Prusinkiewicz & Lindenmayer (1990). "The Algorithmic Beauty of Plants."

use ludospring_barracuda::procedural::lsystem::{presets, turtle_interpret};
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

#[expect(
    clippy::cast_precision_loss,
    reason = "L-system lengths < 10000; fits in f64"
)]
fn validate_algae_fibonacci(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Algae growth (Fibonacci sequence)");
    let sys = presets::algae();
    let fibonacci = [1, 2, 3, 5, 8, 13, 21, 34];

    for (generation, &expected) in fibonacci.iter().enumerate() {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "generation index < 8; fits in u32"
        )]
        let gen_u32 = generation as u32;
        let actual = sys.symbol_count(gen_u32);
        let r = ValidationResult::check(
            "exp013_fibonacci",
            &format!("gen {generation}: length = {expected} (Fibonacci)"),
            actual as f64,
            f64::from(expected),
            tolerances::ANALYTICAL_TOL,
        );
        report(&r);
        results.push(r);
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "symbol counts < 10000; fits in f64"
)]
fn validate_koch_growth(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Koch curve growth");
    let sys = presets::koch_curve();

    let g0 = sys.symbol_count(0);
    let g1 = sys.symbol_count(1);

    let r = ValidationResult::check(
        "exp013_koch_g0",
        "Koch gen 0: 1 symbol",
        g0 as f64,
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp013_koch_g1",
        "Koch gen 1: 9 symbols (F+F-F-F+F)",
        g1 as f64,
        9.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_protein_backbone(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Protein backbone structure");
    let sys = presets::protein_backbone();
    let gen3 = sys.generate(3);

    let has_helix = gen3.contains('H');
    let has_sheet = gen3.contains('S');
    let has_linker = gen3.contains('L');
    let has_turn = gen3.contains('T');

    let r = ValidationResult::check(
        "exp013_protein_elements",
        "gen 3 contains all structural elements (H, S, L, T)",
        if has_helix && has_sheet && has_linker && has_turn {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Protein backbone should grow each generation
    let g1 = sys.symbol_count(1);
    let g2 = sys.symbol_count(2);
    let g3 = sys.symbol_count(3);
    let r = ValidationResult::check(
        "exp013_protein_growth",
        "protein grows: gen1 < gen2 < gen3",
        if g1 < g2 && g2 < g3 { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_turtle_geometry(results: &mut Vec<ValidationResult>) {
    println!("\nPart 4: Turtle interpretation geometry");
    let points = turtle_interpret("FF", 1.0, 90.0);
    let r = ValidationResult::check(
        "exp013_turtle_forward",
        "two forward steps → x=2.0",
        points.last().map_or(0.0, |p| p.0),
        2.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Square: F+F+F+F with 90° turns
    let square = turtle_interpret("F+F+F+F", 1.0, 90.0);
    let end = square.last().copied().unwrap_or((0.0, 0.0));
    let r = ValidationResult::check(
        "exp013_square_closed",
        "F+F+F+F with 90° returns near origin",
        end.0.hypot(end.1),
        0.0,
        1e-8,
    );
    report(&r);
    results.push(r);
}

fn validate_determinism(results: &mut Vec<ValidationResult>) {
    println!("\nPart 5: Determinism");
    let sys = presets::dragon_curve();
    let a = sys.generate(6);
    let b = sys.generate(6);
    let r = ValidationResult::check(
        "exp013_deterministic",
        "dragon curve gen 6 is deterministic",
        if a == b { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp013: L-System Protein Backbone (Validation) ===\n");
    let mut results = Vec::new();

    validate_algae_fibonacci(&mut results);
    validate_koch_growth(&mut results);
    validate_protein_backbone(&mut results);
    validate_turtle_geometry(&mut results);
    validate_determinism(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}
