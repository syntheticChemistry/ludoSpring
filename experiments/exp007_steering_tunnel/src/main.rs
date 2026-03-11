// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp007: Steering law tunnel sweep — validation binary.
//!
//! Validates the steering law (Accot & Zhai 1997) across varying tunnel
//! widths and lengths, modeling corridor navigation in FPS games and
//! menu ribbon traversal in RTS interfaces.
//!
//! # Provenance
//!
//! Accot, J. & Zhai, S. (1997). "Beyond Fitts' law: models for trajectory-
//! based HCI tasks." CHI '97. T = a + b * (D/W).
//! Python baseline: `baselines/python/interaction_laws.py` (2026-03-11).

use ludospring_barracuda::interaction::input_laws::steering_time;
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

fn validate_known_values(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Analytical known values");
    // T = 10 + 5*(100/20) = 35
    let t = steering_time(100.0, 20.0, 10.0, 5.0);
    let r = ValidationResult::check(
        "exp007_exact",
        "steering T = a + b*(D/W) = 10 + 5*5 = 35",
        t,
        35.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_width_sweep(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Width sweep (narrower = slower)");
    let a = 10.0;
    let b = 5.0;
    let d = 200.0;

    let widths = [5.0, 10.0, 20.0, 40.0, 80.0];
    let mut times: Vec<f64> = Vec::new();

    for &w in &widths {
        times.push(steering_time(d, w, a, b));
    }

    // Each wider tunnel should be faster
    let all_decreasing = times.windows(2).all(|pair| pair[0] > pair[1]);
    let r = ValidationResult::check(
        "exp007_width_mono",
        "wider tunnels are faster (monotonically decreasing)",
        if all_decreasing { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Halving width should double the D/W contribution
    let narrow = steering_time(d, 10.0, a, b);
    let wide = steering_time(d, 20.0, a, b);
    let ratio = (narrow - a) / (wide - a);
    let r = ValidationResult::check(
        "exp007_half_width_double",
        "halving width doubles D/W term (ratio ~2.0)",
        ratio,
        2.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_game_scenarios(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Game corridor scenarios");
    let a = 50.0;
    let b = 8.0;

    // FPS corridor: long, medium width
    let corridor = steering_time(500.0, 30.0, a, b);
    // RTS menu ribbon: short, narrow
    let ribbon = steering_time(200.0, 8.0, a, b);
    // Doom doorway: very short, narrow
    let doorway = steering_time(20.0, 6.0, a, b);

    // Ribbon should be slower than corridor (higher D/W ratio)
    let r = ValidationResult::check(
        "exp007_ribbon_hard",
        "narrow menu ribbon harder than wide corridor",
        if ribbon > corridor { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Zero-width should return infinity
    let inf = steering_time(100.0, 0.0, a, b);
    let r = ValidationResult::check(
        "exp007_zero_width",
        "zero-width tunnel returns infinity",
        if inf.is_infinite() { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    println!(
        "\n  Scenarios: corridor={corridor:.0}ms, ribbon={ribbon:.0}ms, doorway={doorway:.0}ms"
    );
}

fn main() {
    println!("=== Exp007: Steering Law Tunnel Sweep (Validation) ===\n");
    let mut results = Vec::new();

    validate_known_values(&mut results);
    validate_width_sweep(&mut results);
    validate_game_scenarios(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}
