// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
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
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/flow_engagement.py",
    commit: "74cf9488",
    date: "2026-03-11",
    command: "python3 baselines/python/run_all_baselines.py",
};

fn validate_boundary_transitions(h: &mut ValidationHarness) {
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
        h.check_bool(desc, actual == *expected);
    }
}

fn validate_width_sweep(h: &mut ValidationHarness) {
    let challenge = 0.65;
    let skill = 0.5;

    let narrow = evaluate_flow(challenge, skill, 0.05);
    let medium = evaluate_flow(challenge, skill, 0.20);
    let wide = evaluate_flow(challenge, skill, 0.50);

    h.check_bool(
        "narrow channel (0.05): c=0.65, s=0.5 is NOT Flow",
        narrow != FlowState::Flow,
    );

    h.check_bool(
        "medium channel (0.20): c=0.65, s=0.5 IS Flow",
        medium == FlowState::Flow,
    );

    h.check_bool(
        "wide channel (0.50): c=0.65, s=0.5 IS Flow",
        wide == FlowState::Flow,
    );
}

fn validate_symmetry(h: &mut ValidationHarness) {
    let cw = 0.15;

    let above = evaluate_flow(0.75, 0.5, cw);
    let below = evaluate_flow(0.25, 0.5, cw);

    h.check_bool(
        "c=0.75, s=0.5 → Arousal (above channel)",
        above == FlowState::Arousal,
    );

    h.check_bool(
        "c=0.25, s=0.5 → Relaxation (below channel)",
        below == FlowState::Relaxation,
    );

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
    h.check_abs(
        "all 5 flow states reachable with cw=0.15",
        count,
        5.0,
        tolerances::ANALYTICAL_TOL,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp012_flow_channel_calibration");
    h.print_provenance(&[&PROVENANCE]);

    validate_boundary_transitions(&mut h);
    validate_width_sweep(&mut h);
    validate_symmetry(&mut h);

    h.finish();
}
