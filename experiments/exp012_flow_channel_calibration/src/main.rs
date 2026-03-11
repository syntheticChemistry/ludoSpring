// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp012: Flow channel calibration — validation binary.
//!
//! Sweeps the flow channel width parameter to validate boundary transitions
//! between all five states (Boredom, Relaxation, Flow, Arousal, Anxiety).
//!
//! # Provenance
//!
//! Csikszentmihalyi (1990). "Flow: The Psychology of Optimal Experience."
//! Chen (2007). "Flow in Games." M.S. Thesis, USC.
//! Python baseline: `baselines/python/flow_engagement.py` (2026-03-11).

use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
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

fn validate_boundary_transitions(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Boundary transitions at cw=0.15");
    let cw = 0.15;

    let transitions: &[(&str, f64, f64, FlowState)] = &[
        ("exact diagonal = Flow", 0.5, 0.5, FlowState::Flow),
        ("challenge slightly above = Flow", 0.6, 0.5, FlowState::Flow),
        ("challenge slightly below = Flow", 0.4, 0.5, FlowState::Flow),
        (
            "challenge well above = Arousal",
            0.75,
            0.5,
            FlowState::Arousal,
        ),
        (
            "challenge far above = Anxiety",
            0.95,
            0.5,
            FlowState::Anxiety,
        ),
        (
            "challenge well below = Relaxation",
            0.3,
            0.5,
            FlowState::Relaxation,
        ),
        (
            "challenge far below = Boredom",
            0.1,
            0.5,
            FlowState::Boredom,
        ),
    ];

    for (desc, challenge, skill, expected) in transitions {
        let actual = evaluate_flow(*challenge, *skill, cw);
        let r = ValidationResult::check(
            "exp012_boundary",
            desc,
            if actual == *expected { 1.0 } else { 0.0 },
            1.0,
            tolerances::ANALYTICAL_TOL,
        );
        report(&r);
        results.push(r);
    }
}

fn validate_width_sweep(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Channel width sweep");
    // Wider channel → more situations classified as Flow
    let challenge = 0.65;
    let skill = 0.5;

    let narrow = evaluate_flow(challenge, skill, 0.05);
    let medium = evaluate_flow(challenge, skill, 0.20);
    let wide = evaluate_flow(challenge, skill, 0.50);

    let r = ValidationResult::check(
        "exp012_narrow",
        "narrow channel (0.05): c=0.65, s=0.5 is NOT Flow",
        if narrow == FlowState::Flow { 0.0 } else { 1.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp012_medium",
        "medium channel (0.20): c=0.65, s=0.5 IS Flow",
        if medium == FlowState::Flow { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp012_wide",
        "wide channel (0.50): c=0.65, s=0.5 IS Flow",
        if wide == FlowState::Flow { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_symmetry(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Symmetry properties");
    let cw = 0.15;

    // Flow is symmetric: (c+d, s) and (c, s+d) should give matching
    // states when mirrored
    let above = evaluate_flow(0.75, 0.5, cw);
    let below = evaluate_flow(0.25, 0.5, cw);

    let r = ValidationResult::check(
        "exp012_above_arousal",
        "c=0.75, s=0.5 → Arousal (above channel)",
        if above == FlowState::Arousal {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp012_below_relaxation",
        "c=0.25, s=0.5 → Relaxation (below channel)",
        if below == FlowState::Relaxation {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // All five states should be reachable
    let states = [
        evaluate_flow(0.5, 0.5, cw),
        evaluate_flow(0.3, 0.5, cw),
        evaluate_flow(0.75, 0.5, cw),
        evaluate_flow(0.1, 0.9, cw),
        evaluate_flow(0.9, 0.1, cw),
    ];
    let unique: std::collections::HashSet<_> = states.iter().map(|s| s.as_str()).collect();
    #[expect(clippy::cast_precision_loss, reason = "set size ≤ 5; fits in f64")]
    let count = unique.len() as f64;
    let r = ValidationResult::check(
        "exp012_all_states",
        "all 5 flow states reachable with cw=0.15",
        count,
        5.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp012: Flow Channel Calibration (Validation) ===\n");
    let mut results = Vec::new();

    validate_boundary_transitions(&mut results);
    validate_width_sweep(&mut results);
    validate_symmetry(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}
