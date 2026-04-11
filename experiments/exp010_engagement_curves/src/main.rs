// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp010: Engagement curve validation — validation binary.
//!
//! Validates engagement scoring, flow state evaluation, and difficulty
//! curves across session archetypes: hardcore, casual, explorer, and idle.
//!
//! # Provenance
//!
//! - Csikszentmihalyi (1990): flow channel model.
//! - Lazzaro (2004): "Four Keys to More Emotion" — engagement components.
//! - Yannakakis & Togelius (2018): engagement measurement framework.
//! - Python baseline: `baselines/python/flow_engagement.py` (2026-03-11).

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

fn validate_flow_states(h: &mut ValidationHarness) {
    let cw = tolerances::FLOW_CHANNEL_WIDTH;

    let cases: &[(&str, f64, f64, FlowState)] = &[
        ("equal challenge/skill", 0.5, 0.5, FlowState::Flow),
        ("slight above", 0.6, 0.5, FlowState::Flow),
        ("arousal zone", 0.75, 0.5, FlowState::Arousal),
        ("anxiety zone", 0.95, 0.1, FlowState::Anxiety),
        ("relaxation zone", 0.35, 0.5, FlowState::Relaxation),
        ("boredom zone", 0.1, 0.9, FlowState::Boredom),
    ];

    for (desc, challenge, skill, expected) in cases {
        let actual = evaluate_flow(*challenge, *skill, cw);
        h.check_bool(desc, actual == *expected);
    }
}

fn validate_difficulty_curves(h: &mut ValidationHarness) {
    let linear = DifficultyCurve::linear(0.1, 0.9);
    h.check_abs(
        "linear curve starts at 0.1",
        linear.sample(0.0),
        0.1,
        tolerances::ANALYTICAL_TOL,
    );
    h.check_abs(
        "linear curve ends at 0.9",
        linear.sample(1.0),
        0.9,
        tolerances::ANALYTICAL_TOL,
    );
    h.check_abs(
        "linear curve midpoint is 0.5",
        linear.sample(0.5),
        0.5,
        tolerances::ANALYTICAL_TOL,
    );

    let sigmoid = DifficultyCurve::sigmoid(0.1, 0.9, 10.0);
    let mut prev = 0.0;
    let monotonic = (0..=100).all(|i| {
        let val = sigmoid.sample(f64::from(i) / 100.0);
        let ok = val >= prev - tolerances::ANALYTICAL_TOL;
        prev = val;
        ok
    });
    h.check_bool("sigmoid curve is monotonically increasing", monotonic);
}

fn validate_engagement_archetypes(h: &mut ValidationHarness) {
    let hardcore = EngagementSnapshot {
        session_duration_s: 7200.0,
        action_count: 5000,
        exploration_breadth: 50,
        challenge_seeking: 200,
        retry_count: 300,
        deliberate_pauses: 100,
    };
    let casual = EngagementSnapshot {
        session_duration_s: 1800.0,
        action_count: 300,
        exploration_breadth: 10,
        challenge_seeking: 5,
        retry_count: 10,
        deliberate_pauses: 20,
    };
    let explorer = EngagementSnapshot {
        session_duration_s: 3600.0,
        action_count: 800,
        exploration_breadth: 100,
        challenge_seeking: 30,
        retry_count: 5,
        deliberate_pauses: 80,
    };
    let idle = EngagementSnapshot {
        session_duration_s: 600.0,
        action_count: 10,
        exploration_breadth: 2,
        challenge_seeking: 0,
        retry_count: 0,
        deliberate_pauses: 0,
    };

    let met_hc = compute_engagement(&hardcore);
    let met_ca = compute_engagement(&casual);
    let met_ex = compute_engagement(&explorer);
    let met_id = compute_engagement(&idle);

    h.check_bool(
        "hardcore scores higher than casual",
        met_hc.composite > met_ca.composite,
    );
    h.check_upper("idle player scores < 0.1", met_id.composite, 0.1);
    h.check_bool(
        "explorer has highest exploration_rate",
        met_ex.exploration_rate > met_hc.exploration_rate
            && met_ex.exploration_rate > met_ca.exploration_rate,
    );
    h.check_bool(
        "hardcore player has highest persistence",
        met_hc.persistence > met_ca.persistence && met_hc.persistence > met_ex.persistence,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp010_engagement_curves");
    h.print_provenance(&[&PROVENANCE]);

    validate_flow_states(&mut h);
    validate_difficulty_curves(&mut h);
    validate_engagement_archetypes(&mut h);

    h.finish();
}
