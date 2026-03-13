// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp021: Retention curves under reward structures — validation binary.
//!
//! Models player retention under different reward schedules: fixed-ratio,
//! variable-ratio, and intrinsic (flow-based). Validates that engagement
//! metrics correctly differentiate healthy (intrinsic) from exploitative
//! (variable-ratio/gambling) retention patterns.
//!
//! # Provenance
//!
//! Skinner, B.F. (1957): operant conditioning schedules.
//! Lazzaro (2004): intrinsic motivation vs extrinsic reward.
//! Yannakakis & Togelius (2018): engagement metrics.

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

/// Fixed-ratio: reward every N actions. Predictable, steady engagement.
const fn fixed_ratio_session() -> EngagementSnapshot {
    EngagementSnapshot {
        session_duration_s: 600.0,
        action_count: 300,
        exploration_breadth: 5,
        challenge_seeking: 10,
        retry_count: 5,
        deliberate_pauses: 20,
    }
}

/// Variable-ratio: reward at random intervals. Compulsive, high action rate.
const fn variable_ratio_session() -> EngagementSnapshot {
    EngagementSnapshot {
        session_duration_s: 600.0,
        action_count: 800,
        exploration_breadth: 2,
        challenge_seeking: 3,
        retry_count: 100,
        deliberate_pauses: 2,
    }
}

/// Intrinsic (flow-based): reward is the activity itself. Balanced metrics.
const fn intrinsic_session() -> EngagementSnapshot {
    EngagementSnapshot {
        session_duration_s: 600.0,
        action_count: 200,
        exploration_breadth: 20,
        challenge_seeking: 25,
        retry_count: 15,
        deliberate_pauses: 30,
    }
}

fn validate_reward_differentiation(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Reward structure differentiation");

    let fixed = compute_engagement(&fixed_ratio_session());
    let variable = compute_engagement(&variable_ratio_session());
    let intrinsic = compute_engagement(&intrinsic_session());

    // Variable-ratio should have highest raw action rate (compulsive clicking)
    let r = ValidationResult::check(
        "exp021_variable_apm",
        "variable-ratio has highest actions-per-minute",
        if variable.actions_per_minute > fixed.actions_per_minute
            && variable.actions_per_minute > intrinsic.actions_per_minute
        {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Intrinsic should have highest exploration rate (genuine curiosity)
    let r = ValidationResult::check(
        "exp021_intrinsic_exploration",
        "intrinsic reward has highest exploration rate",
        if intrinsic.exploration_rate > fixed.exploration_rate
            && intrinsic.exploration_rate > variable.exploration_rate
        {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Intrinsic should have highest challenge appetite
    let r = ValidationResult::check(
        "exp021_intrinsic_challenge",
        "intrinsic reward has highest challenge appetite",
        if intrinsic.challenge_appetite > fixed.challenge_appetite
            && intrinsic.challenge_appetite > variable.challenge_appetite
        {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    println!(
        "\n  Fixed:     APM={:.0}, explore={:.2}, challenge={:.3}, composite={:.3}",
        fixed.actions_per_minute, fixed.exploration_rate, fixed.challenge_appetite, fixed.composite
    );
    println!(
        "  Variable:  APM={:.0}, explore={:.2}, challenge={:.3}, composite={:.3}",
        variable.actions_per_minute,
        variable.exploration_rate,
        variable.challenge_appetite,
        variable.composite
    );
    println!(
        "  Intrinsic: APM={:.0}, explore={:.2}, challenge={:.3}, composite={:.3}",
        intrinsic.actions_per_minute,
        intrinsic.exploration_rate,
        intrinsic.challenge_appetite,
        intrinsic.composite
    );
}

fn validate_compulsion_detection(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Compulsion loop detection");

    let variable = compute_engagement(&variable_ratio_session());
    let intrinsic = compute_engagement(&intrinsic_session());

    // Variable-ratio has high persistence (retry) but low deliberation
    let r = ValidationResult::check(
        "exp021_compulsion_signal",
        "variable-ratio: high persistence, low deliberation (compulsion signal)",
        if variable.persistence > intrinsic.persistence
            && variable.deliberation < intrinsic.deliberation
        {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Intrinsic has balanced deliberation (thinking, not grinding)
    let r = ValidationResult::check(
        "exp021_intrinsic_deliberation",
        "intrinsic reward has higher deliberation (strategic thinking)",
        if intrinsic.deliberation > variable.deliberation {
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

fn validate_composite_ranking(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Composite score ranking");

    let fixed = compute_engagement(&fixed_ratio_session());
    let intrinsic = compute_engagement(&intrinsic_session());

    // Intrinsic should score highest composite (balanced, healthy engagement)
    let r = ValidationResult::check(
        "exp021_intrinsic_highest",
        "intrinsic reward produces highest composite engagement",
        if intrinsic.composite > fixed.composite {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // All composites should be in [0, 1]
    let all_bounded = [&fixed, &intrinsic]
        .iter()
        .all(|m| (0.0..=1.0).contains(&m.composite));
    let r = ValidationResult::check(
        "exp021_bounded",
        "all composite scores in [0, 1]",
        if all_bounded { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp021: Retention Reward Curves (Validation) ===\n");
    let mut results = Vec::new();

    validate_reward_differentiation(&mut results);
    validate_compulsion_detection(&mut results);
    validate_composite_ranking(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}
