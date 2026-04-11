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
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/flow_engagement.py",
    commit: "19e402c0",
    date: "2026-03-11",
    command: "python3 baselines/python/run_all_baselines.py",
};

fn validate_dda_session(h: &mut ValidationHarness) {
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

    let final_adj = suggest_adjustment(&window, tolerances::DDA_TARGET_SUCCESS_RATE);
    h.check_bool(
        "DDA produces non-zero adjustment to performance imbalance",
        final_adj.abs() > f64::EPSILON,
    );
}

fn validate_flow_tracking(h: &mut ValidationHarness) {
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

    h.check_lower(
        "flow state in >= 5 of 20 rounds",
        f64::from(flow_count),
        5.0,
    );
}

fn validate_engagement(h: &mut ValidationHarness) {
    let active = EngagementSnapshot {
        session_duration_s: 600.0,
        action_count: 300,
        exploration_breadth: 18,
        challenge_seeking: 12,
        retry_count: 20,
        deliberate_pauses: 25,
    };
    let metrics = compute_engagement(&active);

    h.check_lower("active session APM > 10", metrics.actions_per_minute, 10.0);
    h.check_lower("active session composite > 0.2", metrics.composite, 0.2);

    let zero = EngagementSnapshot::default();
    let zero_metrics = compute_engagement(&zero);
    h.check_bool(
        "zero-duration engagement is finite",
        zero_metrics.composite.is_finite(),
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp004_folding_adversarial");
    h.print_provenance(&[&PROVENANCE]);

    validate_dda_session(&mut h);
    validate_flow_tracking(&mut h);
    validate_engagement(&mut h);

    h.finish();
}
