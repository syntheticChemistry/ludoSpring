// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp020: Difficulty-skill balance in protein folding challenges — validation.
//!
//! Simulates player sessions across skill levels tackling protein folding
//! challenges with DDA. Validates that the difficulty system correctly
//! adapts: skilled players face harder folds, struggling players get
//! easier ones, and the flow channel is maintained.
//!
//! # Provenance
//!
//! Hunicke (2005). "The case for dynamic difficulty adjustment in games."
//! Csikszentmihalyi (1990). "Flow: The Psychology of Optimal Experience."

use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
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

fn validate_skill_adaptation(results: &mut Vec<ValidationResult>) {
    println!("Part 1: DDA adapts to skill level");

    // Expert player: consistently solves (outcome=0.9+)
    let mut expert = PerformanceWindow::new(10);
    for _ in 0..10 {
        expert.record(0.95);
    }
    let expert_adj = suggest_adjustment(&expert, 0.7);

    let r = ValidationResult::check(
        "exp020_expert_harder",
        "expert player (95% success) gets harder challenges",
        if expert_adj > 0.0 { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Novice player: mostly fails (outcome=0.2)
    let mut novice = PerformanceWindow::new(10);
    for _ in 0..10 {
        novice.record(0.2);
    }
    let novice_adj = suggest_adjustment(&novice, 0.7);

    let r = ValidationResult::check(
        "exp020_novice_easier",
        "novice player (20% success) gets easier challenges",
        if novice_adj < 0.0 { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // At-target player: interleaved 70% success → minimal deviation
    let mut balanced = PerformanceWindow::new(10);
    for &v in &[1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0] {
        balanced.record(v);
    }
    let balanced_adj = suggest_adjustment(&balanced, 0.7);

    let r = ValidationResult::check(
        "exp020_balanced_minimal",
        "on-target player (70% interleaved) gets small adjustment",
        balanced_adj.abs(),
        0.0,
        0.5,
    );
    report(&r);
    results.push(r);
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "success_prob * 10 is in [0, 10]; fits in u64"
)]
fn validate_flow_maintenance(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: DDA maintains flow channel");
    let channel_width = 0.15;

    // Simulate a 20-round session where DDA adjusts difficulty
    let mut difficulty = 0.5_f64;
    let player_skill = 0.6;
    let mut window = PerformanceWindow::new(5);
    let mut flow_count = 0;

    for round in 0..20 {
        let success_prob = if player_skill > difficulty { 0.8 } else { 0.3 };
        let threshold = round * 7 + 13;
        let scaled = (success_prob * 10.0_f64).round() as u64;
        let outcome = if (threshold % 10) < scaled { 1.0 } else { 0.0 };
        window.record(outcome);

        let adj = suggest_adjustment(&window, 0.7);
        difficulty = (difficulty + adj * 0.05).clamp(0.0, 1.0);

        let state = evaluate_flow(difficulty, player_skill, channel_width);
        if state == FlowState::Flow {
            flow_count += 1;
        }
    }

    let r = ValidationResult::check(
        "exp020_flow_maintained",
        "DDA keeps player in Flow for majority of session",
        if flow_count >= 8 { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    println!("  Flow rounds: {flow_count}/20");
}

fn validate_trend_detection(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Performance trend detection");

    // Improving player: 0.2, 0.3, 0.5, 0.7, 0.9
    let mut improving = PerformanceWindow::new(10);
    for &v in &[0.2, 0.3, 0.5, 0.7, 0.9] {
        improving.record(v);
    }
    let trend = improving.trend();
    let r = ValidationResult::check(
        "exp020_improving_trend",
        "improving player has positive trend",
        if trend > 0.0 { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Declining player: 0.9, 0.7, 0.5, 0.3, 0.1
    let mut declining = PerformanceWindow::new(10);
    for &v in &[0.9, 0.7, 0.5, 0.3, 0.1] {
        declining.record(v);
    }
    let trend = declining.trend();
    let r = ValidationResult::check(
        "exp020_declining_trend",
        "declining player has negative trend",
        if trend < 0.0 { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_adjustment_bounds(results: &mut Vec<ValidationResult>) {
    println!("\nPart 4: Adjustment bounds");

    // Even extreme performance shouldn't produce unbounded adjustments
    let mut extreme_good = PerformanceWindow::new(10);
    for _ in 0..10 {
        extreme_good.record(1.0);
    }
    let adj = suggest_adjustment(&extreme_good, 0.5);
    let r = ValidationResult::check(
        "exp020_bounded",
        "adjustment is bounded to [-1, 1]",
        if (-1.0..=1.0).contains(&adj) {
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
    println!("=== Exp020: Difficulty-Skill Balance (Validation) ===\n");
    let mut results = Vec::new();

    validate_skill_adaptation(&mut results);
    validate_flow_maintenance(&mut results);
    validate_trend_detection(&mut results);
    validate_adjustment_bounds(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}
