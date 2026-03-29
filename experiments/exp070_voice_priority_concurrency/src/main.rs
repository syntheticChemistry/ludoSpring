// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp070 — Voice Priority and Concurrency
//!
//! Validates the passive check priority system:
//! - Maximum 3 voices speak per action
//! - Priority ordering: critical > high > medium > low
//! - Higher rolls break ties within priority levels
//! - Edge cases: zero voices, all same priority, all fail

use ludospring_barracuda::game::rpgpt::plane::PassiveCheckPriority;
use ludospring_barracuda::game::rpgpt::voice::{
    VoiceCheckResult, VoiceId, VoiceOutput, select_voice_outputs,
};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const EXP: &str = "exp070_voice_priority_concurrency";

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "specs/RPGPT_INTERNAL_VOICES_SPEC.md",
    commit: "4b683e3e",
    date: "2026-03-15",
    command: "cargo run -p exp070_voice_priority_concurrency",
};

fn make_output(voice: VoiceId, priority: PassiveCheckPriority, roll: i32) -> VoiceOutput {
    VoiceOutput {
        voice,
        text: format!("{} speaks.", voice.name()),
        priority,
        roll,
    }
}

fn validate_max_three(h: &mut ValidationHarness) {
    let outputs = vec![
        make_output(VoiceId::Perception, PassiveCheckPriority::Critical, 12),
        make_output(VoiceId::Logic, PassiveCheckPriority::High, 18),
        make_output(VoiceId::Empathy, PassiveCheckPriority::High, 15),
        make_output(VoiceId::Encyclopedia, PassiveCheckPriority::Medium, 20),
        make_output(VoiceId::Electrochemistry, PassiveCheckPriority::Low, 19),
    ];
    let selected = select_voice_outputs(outputs, 3);
    h.check_abs("max_three_enforced", selected.len() as f64, 3.0, 0.0);
}

fn validate_priority_order(h: &mut ValidationHarness) {
    let outputs = vec![
        make_output(VoiceId::Electrochemistry, PassiveCheckPriority::Low, 20),
        make_output(VoiceId::Encyclopedia, PassiveCheckPriority::Medium, 19),
        make_output(VoiceId::Logic, PassiveCheckPriority::High, 18),
        make_output(VoiceId::Perception, PassiveCheckPriority::Critical, 10),
    ];
    let selected = select_voice_outputs(outputs, 3);
    h.check_bool(
        "critical_before_high",
        selected[0].voice == VoiceId::Perception,
    );
    h.check_bool("high_before_medium", selected[1].voice == VoiceId::Logic);
    h.check_bool(
        "medium_before_low",
        selected[2].voice == VoiceId::Encyclopedia,
    );
}

fn validate_tie_breaking(h: &mut ValidationHarness) {
    let outputs = vec![
        make_output(VoiceId::Logic, PassiveCheckPriority::High, 15),
        make_output(VoiceId::Empathy, PassiveCheckPriority::High, 18),
        make_output(VoiceId::Rhetoric, PassiveCheckPriority::High, 12),
    ];
    let selected = select_voice_outputs(outputs, 3);
    h.check_bool("highest_roll_first", selected[0].voice == VoiceId::Empathy);
    h.check_bool("second_highest_second", selected[1].voice == VoiceId::Logic);
    h.check_bool("lowest_roll_third", selected[2].voice == VoiceId::Rhetoric);
}

fn validate_fewer_than_max(h: &mut ValidationHarness) {
    let outputs = vec![make_output(VoiceId::Logic, PassiveCheckPriority::High, 15)];
    let selected = select_voice_outputs(outputs, 3);
    h.check_abs("single_voice_returns_one", selected.len() as f64, 1.0, 0.0);

    let outputs = vec![
        make_output(VoiceId::Logic, PassiveCheckPriority::High, 15),
        make_output(VoiceId::Empathy, PassiveCheckPriority::Medium, 12),
    ];
    let selected = select_voice_outputs(outputs, 3);
    h.check_abs("two_voices_returns_two", selected.len() as f64, 2.0, 0.0);
}

fn validate_zero_max(h: &mut ValidationHarness) {
    let outputs = vec![make_output(
        VoiceId::Logic,
        PassiveCheckPriority::Critical,
        20,
    )];
    let selected = select_voice_outputs(outputs, 0);
    h.check_abs("zero_max_returns_empty", selected.len() as f64, 0.0, 0.0);
}

fn validate_empty_input(h: &mut ValidationHarness) {
    let selected = select_voice_outputs(vec![], 3);
    h.check_abs("empty_input_returns_empty", selected.len() as f64, 0.0, 0.0);
}

fn validate_all_same_priority(h: &mut ValidationHarness) {
    let outputs = vec![
        make_output(VoiceId::Logic, PassiveCheckPriority::Medium, 10),
        make_output(VoiceId::Empathy, PassiveCheckPriority::Medium, 20),
        make_output(VoiceId::Perception, PassiveCheckPriority::Medium, 15),
        make_output(VoiceId::Rhetoric, PassiveCheckPriority::Medium, 18),
    ];
    let selected = select_voice_outputs(outputs, 3);
    h.check_abs("same_priority_still_three", selected.len() as f64, 3.0, 0.0);
    h.check_bool(
        "highest_roll_wins_tie",
        selected[0].voice == VoiceId::Empathy,
    );
    h.check_bool("second_roll_second", selected[1].voice == VoiceId::Rhetoric);
    h.check_bool("third_roll_third", selected[2].voice == VoiceId::Perception);
}

fn validate_check_result_gating(h: &mut ValidationHarness) {
    let checks = [
        VoiceCheckResult::evaluate(VoiceId::Logic, 10, 10, 15, PassiveCheckPriority::High),
        VoiceCheckResult::evaluate(VoiceId::Empathy, 3, 5, 15, PassiveCheckPriority::High),
        VoiceCheckResult::evaluate(
            VoiceId::Perception,
            15,
            5,
            15,
            PassiveCheckPriority::Critical,
        ),
        VoiceCheckResult::evaluate(VoiceId::Encyclopedia, 2, 3, 15, PassiveCheckPriority::Low),
    ];

    let passed: Vec<&VoiceCheckResult> = checks.iter().filter(|c| c.success).collect();
    let failed: Vec<&VoiceCheckResult> = checks.iter().filter(|c| !c.success).collect();

    h.check_abs("two_checks_pass", passed.len() as f64, 2.0, 0.0);
    h.check_abs("two_checks_fail", failed.len() as f64, 2.0, 0.0);

    h.check_bool("logic_passes_20_vs_15", checks[0].success);
    h.check_bool("empathy_fails_8_vs_15", !checks[1].success);
    h.check_bool("perception_passes_20_vs_15", checks[2].success);
    h.check_bool("encyclopedia_fails_5_vs_15", !checks[3].success);

    let voice_outputs: Vec<VoiceOutput> = passed
        .iter()
        .map(|c| VoiceOutput {
            voice: c.voice,
            text: format!("{} observes.", c.voice.name()),
            priority: c.priority,
            roll: c.roll,
        })
        .collect();

    let selected = select_voice_outputs(voice_outputs, 3);
    h.check_abs("filtered_outputs_count", selected.len() as f64, 2.0, 0.0);
}

fn validate_large_batch(h: &mut ValidationHarness) {
    let outputs: Vec<VoiceOutput> = VoiceId::ALL
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let priority = match i % 4 {
                0 => PassiveCheckPriority::Critical,
                1 => PassiveCheckPriority::High,
                2 => PassiveCheckPriority::Medium,
                _ => PassiveCheckPriority::Low,
            };
            make_output(v, priority, (20 - i as i32).max(1))
        })
        .collect();

    h.check_abs("ten_voices_generated", outputs.len() as f64, 10.0, 0.0);

    let selected = select_voice_outputs(outputs, 3);
    h.check_abs(
        "large_batch_capped_at_three",
        selected.len() as f64,
        3.0,
        0.0,
    );

    h.check_bool(
        "large_batch_first_is_critical",
        selected[0].priority == PassiveCheckPriority::Critical,
    );
}

fn main() {
    let mut h = ValidationHarness::new(EXP);
    h.print_provenance(&[&PROVENANCE]);

    validate_max_three(&mut h);
    validate_priority_order(&mut h);
    validate_tie_breaking(&mut h);
    validate_fewer_than_max(&mut h);
    validate_zero_max(&mut h);
    validate_empty_input(&mut h);
    validate_all_same_priority(&mut h);
    validate_check_result_gating(&mut h);
    validate_large_batch(&mut h);

    h.finish();
}
