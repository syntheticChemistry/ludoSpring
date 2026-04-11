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
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Skinner/Lazzaro/Yannakakis)",
    commit: "19e402c0",
    date: "2026-03-29",
    command: "N/A (engagement metrics from barracuda)",
};

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

fn validate_reward_differentiation(h: &mut ValidationHarness) {
    let fixed = compute_engagement(&fixed_ratio_session());
    let variable = compute_engagement(&variable_ratio_session());
    let intrinsic = compute_engagement(&intrinsic_session());

    // Variable-ratio should have highest raw action rate (compulsive clicking)
    h.check_bool(
        "variable-ratio has highest actions-per-minute",
        variable.actions_per_minute > fixed.actions_per_minute
            && variable.actions_per_minute > intrinsic.actions_per_minute,
    );

    // Intrinsic should have highest exploration rate (genuine curiosity)
    h.check_bool(
        "intrinsic reward has highest exploration rate",
        intrinsic.exploration_rate > fixed.exploration_rate
            && intrinsic.exploration_rate > variable.exploration_rate,
    );

    // Intrinsic should have highest challenge appetite
    h.check_bool(
        "intrinsic reward has highest challenge appetite",
        intrinsic.challenge_appetite > fixed.challenge_appetite
            && intrinsic.challenge_appetite > variable.challenge_appetite,
    );

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

fn validate_compulsion_detection(h: &mut ValidationHarness) {
    let variable = compute_engagement(&variable_ratio_session());
    let intrinsic = compute_engagement(&intrinsic_session());

    // Variable-ratio has high persistence (retry) but low deliberation
    h.check_bool(
        "variable-ratio: high persistence, low deliberation (compulsion signal)",
        variable.persistence > intrinsic.persistence
            && variable.deliberation < intrinsic.deliberation,
    );

    // Intrinsic has balanced deliberation (thinking, not grinding)
    h.check_bool(
        "intrinsic reward has higher deliberation (strategic thinking)",
        intrinsic.deliberation > variable.deliberation,
    );
}

fn validate_composite_ranking(h: &mut ValidationHarness) {
    let fixed = compute_engagement(&fixed_ratio_session());
    let intrinsic = compute_engagement(&intrinsic_session());

    // Intrinsic should score highest composite (balanced, healthy engagement)
    h.check_bool(
        "intrinsic reward produces highest composite engagement",
        intrinsic.composite > fixed.composite,
    );

    // All composites should be in [0, 1]
    let all_bounded = [&fixed, &intrinsic]
        .iter()
        .all(|m| (0.0..=1.0).contains(&m.composite));
    h.check_bool("all composite scores in [0, 1]", all_bounded);
}

fn main() {
    let mut h = ValidationHarness::new("exp021_retention_reward_curves");
    h.print_provenance(&[&PROVENANCE]);

    validate_reward_differentiation(&mut h);
    validate_compulsion_detection(&mut h);
    validate_composite_ranking(&mut h);

    h.finish();
}
