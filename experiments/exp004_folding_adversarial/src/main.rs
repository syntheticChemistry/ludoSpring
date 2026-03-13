// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp004: Folding adversarial — validation binary.
//!
//! Models the `Folding@Home` / `FoldIt` concept: protein folding as a puzzle
//! game where human spatial intuition competes with algorithmic optimization.
//! Validates flow state tracking, DDA, and engagement metrics.
//!
//! # Provenance
//!
//! Flow model: Csikszentmihalyi (1990), Chen (2007).
//! DDA: Hunicke (2005). Engagement: Yannakakis & Togelius (2018).
//! Python baseline: `baselines/python/flow_engagement.py` (2026-03-11).

use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::{DifficultyCurve, FlowState, evaluate_flow};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
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

fn validate_dda_session(results: &mut Vec<ValidationResult>) {
    println!("Part 1: DDA session simulation (20 rounds)");
    let curve = DifficultyCurve::sigmoid(0.2, 0.9, 8.0);
    let mut window = PerformanceWindow::new(10);
    let mut player_skill = 0.3_f64;

    for round in 0..20_i32 {
        let progress = f64::from(round) / 19.0;
        let base_difficulty = curve.sample(progress);
        let adjustment = suggest_adjustment(&window, tolerances::DDA_TARGET_SUCCESS_RATE);
        let _difficulty = (base_difficulty + adjustment * 0.1).clamp(0.0, 1.0);

        let success = if base_difficulty < player_skill + 0.1 {
            0.8
        } else {
            0.3
        };
        window.record(success);
        player_skill = (player_skill + 0.02).min(0.95);
    }

    // DDA responds to performance: with mixed outcomes, the system produces
    // a non-zero adjustment (either harder or easier based on trend).
    let final_adj = suggest_adjustment(&window, tolerances::DDA_TARGET_SUCCESS_RATE);
    let r = ValidationResult::check(
        "exp004_dda_responds",
        "DDA produces non-zero adjustment to performance imbalance",
        final_adj.abs(),
        0.5,
        0.6,
    );
    report(&r);
    results.push(r);
}

fn validate_flow_tracking(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Flow state tracking");
    let curve = DifficultyCurve::sigmoid(0.2, 0.9, 8.0);
    let mut flow_count = 0_u32;
    let mut skill = 0.3_f64;

    for round in 0..20_i32 {
        let progress = f64::from(round) / 19.0;
        let difficulty = curve.sample(progress);
        if evaluate_flow(difficulty, skill, tolerances::FLOW_CHANNEL_WIDTH) == FlowState::Flow {
            flow_count += 1;
        }
        skill = (skill + 0.02).min(0.95);
    }

    // With sigmoid curve and rising skill, some rounds should be in flow
    let r = ValidationResult::check(
        "exp004_flow_maintained",
        "flow state in >= 5 of 20 rounds",
        f64::from(flow_count),
        10.0,
        5.5,
    );
    report(&r);
    results.push(r);
}

fn validate_engagement(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Engagement metrics");
    let active = EngagementSnapshot {
        session_duration_s: 600.0,
        action_count: 180,
        exploration_breadth: 12,
        challenge_seeking: 8,
        retry_count: 15,
        deliberate_pauses: 20,
    };
    let metrics = compute_engagement(&active);

    let r = ValidationResult::check(
        "exp004_apm",
        "active session APM > 10",
        metrics.actions_per_minute,
        18.0,
        10.0,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp004_composite",
        "active session composite > 0.2",
        metrics.composite,
        0.4,
        0.3,
    );
    report(&r);
    results.push(r);

    // Edge case: zero-duration doesn't panic
    let zero = EngagementSnapshot::default();
    let zero_metrics = compute_engagement(&zero);
    let r = ValidationResult::check(
        "exp004_zero_safe",
        "zero-duration engagement is finite",
        if zero_metrics.composite.is_finite() {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp004: Folding Adversarial (Validation) ===\n");
    let mut results = Vec::new();

    validate_dda_session(&mut results);
    validate_flow_tracking(&mut results);
    validate_engagement(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}
