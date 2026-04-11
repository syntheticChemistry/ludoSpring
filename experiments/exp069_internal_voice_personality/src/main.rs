// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp069 — Internal Voice Personality Consistency
//!
//! Validates that the 10 internal voices have distinct personality parameters,
//! opposing voice relationships, and correct constraint boundaries. This tests
//! the voice SYSTEM, not LLM quality — it validates that personality certs are
//! structurally sound and the selection/routing logic is correct.

use ludospring_barracuda::game::rpgpt::plane::PassiveCheckPriority;
use ludospring_barracuda::game::rpgpt::voice::{
    VoiceCheckResult, VoiceId, VoiceOutput, select_voice_outputs,
};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const EXP: &str = "exp069_internal_voice_personality";

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "specs/RPGPT_INTERNAL_VOICES_SPEC.md",
    commit: "19e402c0",
    date: "2026-04-10",
    command: "cargo run -p exp069_internal_voice_personality",
};

#[expect(
    clippy::cast_precision_loss,
    reason = "small fixed voice counts; compared to literal constants"
)]
fn validate_voice_identity(h: &mut ValidationHarness) {
    let all = VoiceId::ALL;

    h.check_abs("ten_voices_exist", all.len() as f64, 10.0, 0.0);

    let mut names = std::collections::HashSet::new();
    for v in all {
        let name = v.name();
        h.check_bool(&format!("voice_{name}_has_name"), !name.is_empty());
        names.insert(name);
    }
    h.check_abs("all_names_unique", names.len() as f64, 10.0, 0.0);
}

fn validate_temperature_ranges(h: &mut ValidationHarness) {
    for v in VoiceId::ALL {
        let t = v.recommended_temperature();
        h.check_bool(
            &format!("temp_{}_in_range", v.name()),
            (0.0..=1.0).contains(&t),
        );
    }

    h.check_bool(
        "logic_lower_temp_than_inland_empire",
        VoiceId::Logic.recommended_temperature() < VoiceId::InlandEmpire.recommended_temperature(),
    );

    h.check_bool(
        "composure_lowest_temp",
        VoiceId::ALL.iter().all(|&v| {
            v == VoiceId::Composure
                || VoiceId::Composure.recommended_temperature() <= v.recommended_temperature()
        }),
    );

    h.check_bool(
        "inland_empire_highest_temp",
        VoiceId::ALL.iter().all(|&v| {
            v == VoiceId::InlandEmpire
                || VoiceId::InlandEmpire.recommended_temperature() >= v.recommended_temperature()
        }),
    );
}

fn validate_max_tokens(h: &mut ValidationHarness) {
    for v in VoiceId::ALL {
        h.check_bool(&format!("tokens_{}_positive", v.name()), v.max_tokens() > 0);
        h.check_bool(
            &format!("tokens_{}_under_limit", v.name()),
            v.max_tokens() <= 100,
        );
    }

    h.check_bool(
        "endurance_fewer_tokens_than_encyclopedia",
        VoiceId::Endurance.max_tokens() < VoiceId::Encyclopedia.max_tokens(),
    );
}

fn validate_opposing_voices(h: &mut ValidationHarness) {
    h.check_bool(
        "logic_opposes_inland_empire",
        VoiceId::Logic.opposite() == Some(VoiceId::InlandEmpire),
    );
    h.check_bool(
        "inland_empire_opposes_logic",
        VoiceId::InlandEmpire.opposite() == Some(VoiceId::Logic),
    );
    h.check_bool(
        "empathy_opposes_authority",
        VoiceId::Empathy.opposite() == Some(VoiceId::Authority),
    );
    h.check_bool(
        "authority_opposes_empathy",
        VoiceId::Authority.opposite() == Some(VoiceId::Empathy),
    );
    h.check_bool(
        "electrochemistry_opposes_composure",
        VoiceId::Electrochemistry.opposite() == Some(VoiceId::Composure),
    );
    h.check_bool(
        "composure_opposes_electrochemistry",
        VoiceId::Composure.opposite() == Some(VoiceId::Electrochemistry),
    );

    h.check_bool(
        "perception_no_natural_opposite",
        VoiceId::Perception.opposite().is_none(),
    );
    h.check_bool(
        "endurance_no_natural_opposite",
        VoiceId::Endurance.opposite().is_none(),
    );
    h.check_bool(
        "encyclopedia_no_natural_opposite",
        VoiceId::Encyclopedia.opposite().is_none(),
    );

    for v in VoiceId::ALL {
        if let Some(opp) = v.opposite() {
            h.check_bool(&format!("opposite_not_self_{}", v.name()), v != opp);
        }
    }
}

fn validate_voice_check_evaluation(h: &mut ValidationHarness) {
    let pass = VoiceCheckResult::evaluate(VoiceId::Logic, 5, 11, 15, PassiveCheckPriority::High);
    h.check_bool("check_16_passes_dc15", pass.success);
    h.check_abs("check_16_roll_value", f64::from(pass.roll), 16.0, 0.0);

    let fail = VoiceCheckResult::evaluate(VoiceId::Logic, 5, 9, 15, PassiveCheckPriority::High);
    h.check_bool("check_14_fails_dc15", !fail.success);

    let exact =
        VoiceCheckResult::evaluate(VoiceId::Empathy, 5, 10, 15, PassiveCheckPriority::Medium);
    h.check_bool("exact_dc_passes", exact.success);

    let different_voices = [
        VoiceCheckResult::evaluate(VoiceId::Logic, 10, 10, 15, PassiveCheckPriority::High),
        VoiceCheckResult::evaluate(VoiceId::Empathy, 10, 10, 15, PassiveCheckPriority::High),
        VoiceCheckResult::evaluate(VoiceId::Perception, 10, 10, 15, PassiveCheckPriority::High),
    ];
    for r in &different_voices {
        h.check_bool(
            &format!("voice_id_preserved_{}", r.voice.name()),
            r.voice != VoiceId::Authority,
        );
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "selection length checked against small literals"
)]
fn validate_selection_priority(h: &mut ValidationHarness) {
    let outputs = vec![
        VoiceOutput {
            voice: VoiceId::Perception,
            text: "Hidden enemy.".into(),
            priority: PassiveCheckPriority::Critical,
            roll: 12,
        },
        VoiceOutput {
            voice: VoiceId::Logic,
            text: "Contradiction.".into(),
            priority: PassiveCheckPriority::High,
            roll: 18,
        },
        VoiceOutput {
            voice: VoiceId::Empathy,
            text: "Fear.".into(),
            priority: PassiveCheckPriority::High,
            roll: 15,
        },
        VoiceOutput {
            voice: VoiceId::Encyclopedia,
            text: "History.".into(),
            priority: PassiveCheckPriority::Medium,
            roll: 20,
        },
        VoiceOutput {
            voice: VoiceId::Electrochemistry,
            text: "Temptation.".into(),
            priority: PassiveCheckPriority::Low,
            roll: 19,
        },
    ];

    let selected = select_voice_outputs(outputs, 3);
    h.check_abs("selection_returns_3", selected.len() as f64, 3.0, 0.0);
    h.check_bool("critical_first", selected[0].voice == VoiceId::Perception);
    h.check_bool("high_roll_18_second", selected[1].voice == VoiceId::Logic);
    h.check_bool("high_roll_15_third", selected[2].voice == VoiceId::Empathy);
}

#[expect(
    clippy::cast_precision_loss,
    reason = "selection length checked against small literals"
)]
fn validate_empty_and_single(h: &mut ValidationHarness) {
    let empty: Vec<VoiceOutput> = vec![];
    let selected = select_voice_outputs(empty, 3);
    h.check_abs("empty_returns_zero", selected.len() as f64, 0.0, 0.0);

    let single = vec![VoiceOutput {
        voice: VoiceId::Logic,
        text: "Only one.".into(),
        priority: PassiveCheckPriority::High,
        roll: 15,
    }];
    let selected = select_voice_outputs(single, 3);
    h.check_abs("single_returns_one", selected.len() as f64, 1.0, 0.0);
}

fn main() {
    let mut h = ValidationHarness::new(EXP);
    h.print_provenance(&[&PROVENANCE]);

    validate_voice_identity(&mut h);
    validate_temperature_ranges(&mut h);
    validate_max_tokens(&mut h);
    validate_opposing_voices(&mut h);
    validate_voice_check_evaluation(&mut h);
    validate_selection_priority(&mut h);
    validate_empty_and_single(&mut h);

    h.finish();
}
