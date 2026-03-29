// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp068 — Lie Detection via Passive Checks
//!
//! When an NPC lies (per knowledge bounds), the passive check system rolls
//! against the NPC's `detection_dc`. On success, a voice observation about the
//! `tell` is generated — revealing the TELL, not the TRUTH. On failure, the lie
//! passes without comment.
//!
//! Validates statistical properties of the passive check system across skill
//! levels and detection DCs.

use ludospring_barracuda::game::rpgpt::knowledge::{KnowledgeBounds, LieTopic};
use ludospring_barracuda::game::rpgpt::plane::PassiveCheckPriority;
use ludospring_barracuda::game::rpgpt::voice::{VoiceCheckResult, VoiceId};
use ludospring_barracuda::validation::{BaselineProvenance, OrExit, ValidationHarness};

const EXP: &str = "exp068_lie_detection_passive_checks";

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "specs/RPGPT_INTERNAL_VOICES_SPEC.md",
    commit: "4b683e3e",
    date: "2026-03-29",
    command: "cargo run -p exp068_lie_detection_passive_checks",
};

fn maren_knowledge() -> KnowledgeBounds {
    KnowledgeBounds {
        knows: vec!["The king is ill".into()],
        suspects: vec![],
        lies_about: vec![
            LieTopic {
                topic: "experiments".into(),
                surface_claim: "I only forge what the guild approves".into(),
                truth: "Hidden workshop beneath the forge".into(),
                reason: "Guild expulsion".into(),
                tell: "Right hand moves to cover burn scars".into(),
                detection_dc: 15,
                detection_skills: vec!["Perception".into(), "Empathy".into()],
            },
            LieTopic {
                topic: "cellar door".into(),
                surface_claim: "Storage for coal and ore".into(),
                truth: "Entrance to hidden workshop".into(),
                reason: "Same as experiments".into(),
                tell: "Glances at the door briefly; changes subject".into(),
                detection_dc: 12,
                detection_skills: vec!["Perception".into()],
            },
        ],
        does_not_know: vec!["The dragon's weakness".into()],
    }
}

fn simulate_checks(skill_modifier: i32, dc: u8, trials: usize) -> f64 {
    let mut successes = 0usize;
    for i in 0..trials {
        let die_roll = (i % 20) as i32 + 1; // deterministic 1-20 cycling
        let result = VoiceCheckResult::evaluate(
            VoiceId::Perception,
            skill_modifier,
            die_roll,
            dc,
            PassiveCheckPriority::High,
        );
        if result.success {
            successes += 1;
        }
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "trial counts fit in f64 mantissa"
    )]
    {
        successes as f64 / trials as f64
    }
}

fn validate_detection_rates(h: &mut ValidationHarness) {
    let trials = 1000;
    let dc = 15u8;

    let rate_skill_5 = simulate_checks(5, dc, trials);
    let rate_skill_10 = simulate_checks(10, dc, trials);
    let rate_skill_15 = simulate_checks(15, dc, trials);
    let rate_skill_20 = simulate_checks(20, dc, trials);

    h.check_bool("low_skill_lower_rate", rate_skill_5 < rate_skill_10);
    h.check_bool("mid_skill_lower_than_high", rate_skill_10 < rate_skill_15);
    h.check_bool("high_skill_lower_than_max", rate_skill_15 <= rate_skill_20);
    h.check_bool("max_skill_high_rate", rate_skill_20 > 0.9);
    h.check_bool("low_skill_low_rate", rate_skill_5 < 0.6);

    h.check_bool(
        "monotonic_skill_rate",
        rate_skill_5 <= rate_skill_10
            && rate_skill_10 <= rate_skill_15
            && rate_skill_15 <= rate_skill_20,
    );
}

fn validate_dc_affects_rate(h: &mut ValidationHarness) {
    let skill = 10;
    let trials = 1000;

    let rate_dc12 = simulate_checks(skill, 12, trials);
    let rate_dc15 = simulate_checks(skill, 15, trials);
    let rate_dc18 = simulate_checks(skill, 18, trials);

    h.check_bool("higher_dc_lower_rate_12_vs_15", rate_dc12 > rate_dc15);
    h.check_bool("higher_dc_lower_rate_15_vs_18", rate_dc15 > rate_dc18);
}

fn validate_lie_tell_association(h: &mut ValidationHarness) {
    let kb = maren_knowledge();

    let experiment_lie = kb.get_lie("experiments").or_exit("lie for 'experiments' not found");
    h.check_bool(
        "experiment_tell_mentions_scars",
        experiment_lie.tell.contains("burn scars"),
    );
    h.check_bool(
        "experiment_tell_not_truth",
        !experiment_lie.tell.contains("workshop"),
    );

    let cellar_lie = kb.get_lie("cellar").or_exit("lie for 'cellar' not found");
    h.check_bool(
        "cellar_tell_mentions_glance",
        cellar_lie.tell.contains("Glances"),
    );
    h.check_bool(
        "cellar_tell_not_truth",
        !cellar_lie.tell.contains("workshop"),
    );
}

fn validate_perception_vs_empathy(h: &mut ValidationHarness) {
    let kb = maren_knowledge();
    let experiment_lie = kb.get_lie("experiments").or_exit("lie for 'experiments' not found");

    h.check_bool(
        "perception_can_detect_experiments",
        experiment_lie
            .detection_skills
            .contains(&"Perception".into()),
    );
    h.check_bool(
        "empathy_can_detect_experiments",
        experiment_lie.detection_skills.contains(&"Empathy".into()),
    );

    let cellar_lie = kb.get_lie("cellar").or_exit("lie for 'cellar' not found");
    h.check_bool(
        "perception_can_detect_cellar",
        cellar_lie.detection_skills.contains(&"Perception".into()),
    );
    h.check_bool(
        "cellar_fewer_detection_skills",
        cellar_lie.detection_skills.len() < experiment_lie.detection_skills.len(),
    );
}

fn validate_voice_check_mechanics(h: &mut ValidationHarness) {
    let success =
        VoiceCheckResult::evaluate(VoiceId::Perception, 10, 10, 15, PassiveCheckPriority::High);
    h.check_bool("roll_20_meets_dc_15", success.success);

    let fail =
        VoiceCheckResult::evaluate(VoiceId::Perception, 5, 9, 15, PassiveCheckPriority::High);
    h.check_bool("roll_14_fails_dc_15", !fail.success);

    let exact =
        VoiceCheckResult::evaluate(VoiceId::Empathy, 5, 10, 15, PassiveCheckPriority::Medium);
    h.check_bool("exact_dc_succeeds", exact.success);

    h.check_bool("voice_id_preserved", success.voice == VoiceId::Perception);
    h.check_bool(
        "priority_preserved",
        success.priority == PassiveCheckPriority::High,
    );
}

fn main() {
    let mut h = ValidationHarness::new(EXP);
    h.print_provenance(&[&PROVENANCE]);

    validate_detection_rates(&mut h);
    validate_dc_affects_rate(&mut h);
    validate_lie_tell_association(&mut h);
    validate_perception_vs_empathy(&mut h);
    validate_voice_check_mechanics(&mut h);

    h.finish();
}
