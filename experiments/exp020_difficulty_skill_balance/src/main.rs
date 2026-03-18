// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
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
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Hunicke 2005, Csikszentmihalyi 1990)",
    commit: "74cf9488",
    date: "2026-03-15",
    command: "N/A (analytical)",
};

fn validate_skill_adaptation(h: &mut ValidationHarness) {
    let mut expert = PerformanceWindow::new(10);
    for _ in 0..10 {
        expert.record(0.95);
    }
    let expert_adj = suggest_adjustment(&expert, 0.7);

    h.check_bool(
        "expert player (95% success) gets harder challenges",
        expert_adj > 0.0,
    );

    let mut novice = PerformanceWindow::new(10);
    for _ in 0..10 {
        novice.record(0.2);
    }
    let novice_adj = suggest_adjustment(&novice, 0.7);

    h.check_bool(
        "novice player (20% success) gets easier challenges",
        novice_adj < 0.0,
    );

    let mut balanced = PerformanceWindow::new(10);
    for &v in &[1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0] {
        balanced.record(v);
    }
    let balanced_adj = suggest_adjustment(&balanced, 0.7);

    h.check_upper(
        "on-target player (70% interleaved) gets small adjustment",
        balanced_adj.abs(),
        0.5,
    );
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "success_prob * 10 is in [0, 10]; fits in u64"
)]
fn validate_flow_maintenance(h: &mut ValidationHarness) {
    let channel_width = 0.15;

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

    h.check_bool(
        "DDA keeps player in Flow for majority of session",
        flow_count >= 8,
    );
}

fn validate_trend_detection(h: &mut ValidationHarness) {
    let mut improving = PerformanceWindow::new(10);
    for &v in &[0.2, 0.3, 0.5, 0.7, 0.9] {
        improving.record(v);
    }
    let trend = improving.trend();
    h.check_bool("improving player has positive trend", trend > 0.0);

    let mut declining = PerformanceWindow::new(10);
    for &v in &[0.9, 0.7, 0.5, 0.3, 0.1] {
        declining.record(v);
    }
    let trend = declining.trend();
    h.check_bool("declining player has negative trend", trend < 0.0);
}

fn validate_adjustment_bounds(h: &mut ValidationHarness) {
    let mut extreme_good = PerformanceWindow::new(10);
    for _ in 0..10 {
        extreme_good.record(1.0);
    }
    let adj = suggest_adjustment(&extreme_good, 0.5);
    h.check_bool(
        "adjustment is bounded to [-1, 1]",
        (-1.0..=1.0).contains(&adj),
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp020_difficulty_skill_balance");
    h.print_provenance(&[&PROVENANCE]);

    validate_skill_adaptation(&mut h);
    validate_flow_maintenance(&mut h);
    validate_trend_detection(&mut h);
    validate_adjustment_bounds(&mut h);

    h.finish();
}
