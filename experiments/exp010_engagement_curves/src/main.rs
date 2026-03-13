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

fn validate_flow_states(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Flow state evaluation");
    let cw = 0.15;

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
        let r = ValidationResult::check(
            "exp010_flow",
            desc,
            if actual == *expected { 1.0 } else { 0.0 },
            1.0,
            tolerances::ANALYTICAL_TOL,
        );
        report(&r);
        results.push(r);
    }
}

fn validate_difficulty_curves(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Difficulty curves");

    let linear = DifficultyCurve::linear(0.1, 0.9);
    let r = ValidationResult::check(
        "exp010_linear_start",
        "linear curve starts at 0.1",
        linear.sample(0.0),
        0.1,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp010_linear_end",
        "linear curve ends at 0.9",
        linear.sample(1.0),
        0.9,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp010_linear_mid",
        "linear curve midpoint is 0.5",
        linear.sample(0.5),
        0.5,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let sigmoid = DifficultyCurve::sigmoid(0.1, 0.9, 10.0);
    let mut monotonic = true;
    let mut prev = 0.0;
    for i in 0..=100 {
        let t = f64::from(i) / 100.0;
        let val = sigmoid.sample(t);
        if val < prev - 1e-10 {
            monotonic = false;
        }
        prev = val;
    }
    let r = ValidationResult::check(
        "exp010_sigmoid_mono",
        "sigmoid curve is monotonically increasing",
        if monotonic { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_engagement_archetypes(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Engagement archetypes");

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

    let r = ValidationResult::check(
        "exp010_hardcore_high",
        "hardcore scores higher than casual",
        if met_hc.composite > met_ca.composite {
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
        "exp010_idle_low",
        "idle player scores < 0.1",
        if met_id.composite < 0.1 { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp010_explorer_breadth",
        "explorer has highest exploration_rate",
        if met_ex.exploration_rate > met_hc.exploration_rate
            && met_ex.exploration_rate > met_ca.exploration_rate
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

    let r = ValidationResult::check(
        "exp010_hardcore_persistent",
        "hardcore player has highest persistence",
        if met_hc.persistence > met_ca.persistence && met_hc.persistence > met_ex.persistence {
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
        "\n  Composites: hardcore={:.3}, casual={:.3}, explorer={:.3}, idle={:.3}",
        met_hc.composite, met_ca.composite, met_ex.composite, met_id.composite
    );
}

fn main() {
    println!("=== Exp010: Engagement Curve Validation ===\n");
    let mut results = Vec::new();

    validate_flow_states(&mut results);
    validate_difficulty_curves(&mut results);
    validate_engagement_archetypes(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}
