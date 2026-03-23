// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp074 — Dialogue Plane Flow Monitoring
//!
//! Validates ludoSpring Flow detection, Hick's law, and DDA in
//! conversation context:
//! 1. Flow state detected when challenge matches skill
//! 2. Stalling triggers Anxiety or Boredom
//! 3. Hick's law threshold (>6 options) triggers complexity reduction
//! 4. DDA adjusts NPC cooperativeness based on flow state

use ludospring_barracuda::game::rpgpt::dialogue::{DialogueExchange, DialogueFlowTracker};
use ludospring_barracuda::game::ruleset::DegreeOfSuccess;
use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::interaction::input_laws::hick_reaction_time;
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const EXP: &str = "exp074_dialogue_flow_monitoring";

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "specs/RPGPT_DIALOGUE_PLANE_EXPERIMENTS.md",
    commit: "74cf9488",
    date: "2026-03-15",
    command: "cargo run -p exp074_dialogue_flow_monitoring",
};

fn validate_flow_detection_balanced(h: &mut ValidationHarness) {
    // Scenario A: skilled player, cooperative NPC -> Flow
    let challenge = 0.5;
    let skill = 0.5;
    let state = evaluate_flow(challenge, skill, 0.15);
    h.check_bool("balanced_is_flow", state == FlowState::Flow);
}

fn validate_anxiety_detection(h: &mut ValidationHarness) {
    // Scenario B: skilled player, stonewalling NPC -> Anxiety
    let challenge = 0.9;
    let skill = 0.4;
    let state = evaluate_flow(challenge, skill, 0.15);
    h.check_bool("high_challenge_anxiety", state == FlowState::Anxiety);
}

fn validate_boredom_detection(h: &mut ValidationHarness) {
    // Scenario C: skilled player, talkative NPC -> Boredom
    let challenge = 0.1;
    let skill = 0.7;
    let state = evaluate_flow(challenge, skill, 0.15);
    h.check_bool("low_challenge_boredom", state == FlowState::Boredom);
}

fn validate_hick_threshold(h: &mut ValidationHarness) {
    // Hick's law: RT = a + b * log2(N+1)
    let rt_4 = hick_reaction_time(4, 200.0, 150.0);
    let rt_6 = hick_reaction_time(6, 200.0, 150.0);
    let rt_8 = hick_reaction_time(8, 200.0, 150.0);

    h.check_bool("hick_rt_4_reasonable", rt_4 > 200.0 && rt_4 < 600.0);
    h.check_bool("hick_rt_6_higher", rt_6 > rt_4);
    h.check_bool("hick_rt_8_higher", rt_8 > rt_6);

    // Threshold: >6 options is problematic
    h.check_bool("hick_threshold_exceeded_at_8", rt_8 > 600.0);

    // Reducing to 4 brings it down
    h.check_bool("hick_reduced_at_4", rt_4 < rt_8);
}

fn validate_dda_anxiety_response(h: &mut ValidationHarness) {
    // DDA: Anxiety -> should suggest making it easier (negative adjustment)
    let mut window = PerformanceWindow::new(10);
    // Simulate mostly failures (player struggling)
    for _ in 0..8 {
        window.record(0.2);
    }
    for _ in 0..2 {
        window.record(0.8);
    }

    let adjustment = suggest_adjustment(&window, 0.6);
    h.check_bool("dda_anxiety_suggests_easier", adjustment < 0.0);
}

fn validate_dda_boredom_response(h: &mut ValidationHarness) {
    // DDA: Boredom -> should suggest making it harder (positive adjustment)
    let mut window = PerformanceWindow::new(10);
    for _ in 0..9 {
        window.record(0.9);
    }
    window.record(0.8);

    let adjustment = suggest_adjustment(&window, 0.6);
    h.check_bool("dda_boredom_suggests_harder", adjustment > 0.0);
}

fn validate_flow_no_dda(h: &mut ValidationHarness) {
    // Flow: DDA should suggest minimal/no adjustment
    let mut window = PerformanceWindow::new(10);
    for _ in 0..6 {
        window.record(0.6);
    }
    for _ in 0..4 {
        window.record(0.5);
    }

    let adjustment = suggest_adjustment(&window, 0.6);
    h.check_bool("flow_minimal_dda", adjustment.abs() < 0.3);
}

fn validate_dialogue_tracker_flow_integration(h: &mut ValidationHarness) {
    let mut tracker = DialogueFlowTracker::default();

    // Balanced conversation: mix of successes and partial successes
    for degree in [
        DegreeOfSuccess::Success,
        DegreeOfSuccess::PartialSuccess,
        DegreeOfSuccess::Success,
        DegreeOfSuccess::Failure,
        DegreeOfSuccess::Success,
    ] {
        tracker.record(DialogueExchange {
            skill: "Persuasion".into(),
            degree: Some(degree),
            options_count: 4,
            advanced: true,
        });
    }

    let flow_state = evaluate_flow(tracker.challenge(), tracker.skill(), 0.15);
    h.check_bool(
        "tracker_balanced_near_flow",
        matches!(
            flow_state,
            FlowState::Flow | FlowState::Relaxation | FlowState::Arousal
        ),
    );
    h.check_abs(
        "tracker_exchange_count",
        tracker.exchange_count() as f64,
        5.0,
        0.0,
    );
    h.check_abs(
        "tracker_avg_options",
        tracker.avg_options(),
        4.0,
        tolerances::GAME_STATE_TOL,
    );
}

fn validate_stall_detection(h: &mut ValidationHarness) {
    let mut tracker = DialogueFlowTracker::default();

    // Stalled conversation: several exchanges with no advancement
    for _ in 0..5 {
        tracker.record(DialogueExchange {
            skill: "Persuasion".into(),
            degree: Some(DegreeOfSuccess::Failure),
            options_count: 3,
            advanced: false,
        });
    }
    // One success
    tracker.record(DialogueExchange {
        skill: "Persuasion".into(),
        degree: Some(DegreeOfSuccess::Success),
        options_count: 3,
        advanced: true,
    });

    h.check_abs("stall_count_five", tracker.stall_count() as f64, 5.0, 0.0);
    h.check_abs(
        "success_rate_one_sixth",
        tracker.success_rate(),
        1.0 / 6.0,
        0.01,
    );
}

fn validate_option_overload(h: &mut ValidationHarness) {
    let mut tracker = DialogueFlowTracker::default();

    tracker.record(DialogueExchange {
        skill: "freeform".into(),
        degree: None,
        options_count: 8,
        advanced: true,
    });
    tracker.record(DialogueExchange {
        skill: "freeform".into(),
        degree: None,
        options_count: 10,
        advanced: true,
    });

    h.check_bool("avg_options_above_6", tracker.avg_options() > 6.0);
    h.check_abs(
        "avg_options_value",
        tracker.avg_options(),
        9.0,
        tolerances::GAME_STATE_TOL,
    );

    // Hick's law shows this is too many
    let rt_avg = hick_reaction_time(tracker.avg_options() as usize, 200.0, 150.0);
    h.check_bool("hick_warns_overload", rt_avg > 550.0);
}

fn validate_skill_estimate_evolves(h: &mut ValidationHarness) {
    let mut tracker = DialogueFlowTracker::default();
    let initial_skill = tracker.skill();

    // Series of successes should raise skill estimate
    for _ in 0..5 {
        tracker.record(DialogueExchange {
            skill: "Diplomacy".into(),
            degree: Some(DegreeOfSuccess::CriticalSuccess),
            options_count: 4,
            advanced: true,
        });
    }

    h.check_bool(
        "skill_increased_after_successes",
        tracker.skill() > initial_skill,
    );

    // Series of failures should lower it
    let mid_skill = tracker.skill();
    for _ in 0..5 {
        tracker.record(DialogueExchange {
            skill: "Diplomacy".into(),
            degree: Some(DegreeOfSuccess::CriticalFailure),
            options_count: 4,
            advanced: false,
        });
    }

    h.check_bool(
        "skill_decreased_after_failures",
        tracker.skill() < mid_skill,
    );
}

fn validate_cross_flow_states(h: &mut ValidationHarness) {
    // All 5 flow states reachable
    h.check_bool(
        "boredom_reachable",
        evaluate_flow(0.0, 1.0, 0.15) == FlowState::Boredom,
    );
    h.check_bool(
        "relaxation_reachable",
        evaluate_flow(0.3, 0.5, 0.15) == FlowState::Relaxation,
    );
    h.check_bool(
        "flow_reachable",
        evaluate_flow(0.5, 0.5, 0.15) == FlowState::Flow,
    );
    h.check_bool(
        "arousal_reachable",
        evaluate_flow(0.7, 0.5, 0.15) == FlowState::Arousal,
    );
    h.check_bool(
        "anxiety_reachable",
        evaluate_flow(1.0, 0.0, 0.15) == FlowState::Anxiety,
    );
}

fn main() {
    let mut h = ValidationHarness::new(EXP);
    h.print_provenance(&[&PROVENANCE]);

    validate_flow_detection_balanced(&mut h);
    validate_anxiety_detection(&mut h);
    validate_boredom_detection(&mut h);
    validate_hick_threshold(&mut h);
    validate_dda_anxiety_response(&mut h);
    validate_dda_boredom_response(&mut h);
    validate_flow_no_dda(&mut h);
    validate_dialogue_tracker_flow_integration(&mut h);
    validate_stall_detection(&mut h);
    validate_option_overload(&mut h);
    validate_skill_estimate_evolves(&mut h);
    validate_cross_flow_states(&mut h);

    h.finish();
}
