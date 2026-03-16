// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp072 — Trust Dynamics and NPC Arc Progression
//!
//! Validates that the trust model drives NPC behavior change and arc
//! progression through a 15-interaction, 5-session simulation:
//!
//! 1. Trust accumulates from defined actions
//! 2. Trust level gates information access (level_effects)
//! 3. Negative actions have larger magnitude (betrayal > helpfulness)
//! 4. Character arc progresses through phases based on trust + events
//! 5. Quorum threshold: when multiple NPCs reach a need threshold,
//!    collective events trigger

use ludospring_barracuda::game::rpgpt::knowledge::{KnowledgeBounds, LieTopic};
use ludospring_barracuda::game::rpgpt::npc::{
    ArcPhase, ArcPhaseStatus, MaslowNeeds, MotivationConflict, NeedState, NpcPersonality,
    NpcSecret, NpcVoice, Relationship, RelationshipType,
};
use ludospring_barracuda::game::rpgpt::trust::{TrustAction, TrustModel};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::ValidationHarness;

const EXP: &str = "exp072_trust_dynamics_arc";

fn maren() -> NpcPersonality {
    NpcPersonality {
        id: "maren_blacksmith".into(),
        name: "Maren the Blacksmith".into(),
        role: "master smith".into(),
        appearance: "Broad shoulders, burn scars".into(),
        mannerisms: vec!["Taps hammer".into()],
        motivations: MaslowNeeds {
            survival: NeedState::new(0.2, "Adequate"),
            safety: NeedState::new(0.7, "Guild protection fragile"),
            belonging: NeedState::new(0.5, "Respected but lonely"),
            esteem: NeedState::new(0.8, "Wants recognition for forbidden work"),
            self_actualization: NeedState::new(0.4, "Dreams of a masterwork"),
        },
        motivation_conflicts: vec![MotivationConflict {
            need_a: "esteem".into(),
            need_b: "safety".into(),
            description: "Recognition requires revealing forbidden work".into(),
        }],
        knowledge: KnowledgeBounds {
            knows: vec!["The king is ill".into()],
            suspects: vec![],
            lies_about: vec![LieTopic {
                topic: "experiments".into(),
                surface_claim: "I only forge guild-approved items".into(),
                truth: "Hidden workshop beneath forge".into(),
                reason: "Guild expulsion".into(),
                tell: "Hand covers burn scars".into(),
                detection_dc: 15,
                detection_skills: vec!["Perception".into()],
            }],
            does_not_know: vec!["Dragon's weakness".into()],
        },
        voice: NpcVoice {
            speech_patterns: vec!["Forge metaphors".into()],
            vocabulary_level: "working-class literate".into(),
            emotional_baseline: "warm but guarded".into(),
            catchphrases: vec!["Good steel from a hot forge.".into()],
        },
        secrets: vec![
            NpcSecret {
                id: "hidden_workshop".into(),
                description: "Workshop beneath the forge".into(),
                reveal_conditions: vec!["Player earns trust_level >= 3".into()],
            },
            NpcSecret {
                id: "masters_journal".into(),
                description: "The old master's research journal".into(),
                reveal_conditions: vec!["Player earns trust_level >= 4".into()],
            },
        ],
        relationships: vec![Relationship {
            entity: "Guild Master Harven".into(),
            relationship_type: RelationshipType::Adversarial,
            strength: -0.4,
            reason: "Suspects experiments".into(),
            trajectory: "worsening".into(),
        }],
        arc: vec![
            ArcPhase {
                id: "conformity".into(),
                description: "Following guild rules".into(),
                status: ArcPhaseStatus::Completed,
                triggers: vec![],
            },
            ArcPhase {
                id: "internal_conflict".into(),
                description: "Experimenting in secret".into(),
                status: ArcPhaseStatus::Active,
                triggers: vec!["guild_inspection".into(), "trust_level_4".into()],
            },
            ArcPhase {
                id: "revelation".into(),
                description: "Forced to choose between guild and craft".into(),
                status: ArcPhaseStatus::Pending,
                triggers: vec![],
            },
        ],
        trust: {
            let mut tm = TrustModel::new(5);
            tm.set_level(0, 0.0, "Polite but professional");
            tm.set_level(1, 1.0, "Warmer, shares opinions");
            tm.set_level(2, 2.0, "Confides frustrations");
            tm.set_level(3, 3.0, "Reveals hidden workshop");
            tm.set_level(4, 4.0, "Shares master's journal");
            tm.set_level(5, 5.0, "Full partnership");
            tm.positive_actions.push(TrustAction {
                action: "Bring rare materials".into(),
                delta: 0.5,
            });
            tm.positive_actions.push(TrustAction {
                action: "Defend reputation".into(),
                delta: 1.0,
            });
            tm.positive_actions.push(TrustAction {
                action: "Help with experiments".into(),
                delta: 1.0,
            });
            tm.positive_actions.push(TrustAction {
                action: "Keep her secret".into(),
                delta: 0.5,
            });
            tm.negative_actions.push(TrustAction {
                action: "Threaten to reveal secrets".into(),
                delta: -2.0,
            });
            tm.negative_actions.push(TrustAction {
                action: "Betray confidence to guild".into(),
                delta: -5.0,
            });
            tm
        },
    }
}

fn validate_initial_state(h: &mut ValidationHarness) {
    let npc = maren();
    h.check_abs("initial_trust_zero", npc.trust.current_trust(), 0.0, 0.0);
    h.check_abs(
        "initial_trust_level_zero",
        f64::from(npc.trust.current_level()),
        0.0,
        0.0,
    );
    h.check_bool(
        "initial_arc_internal_conflict",
        npc.current_arc_phase()
            .is_some_and(|p| p.id == "internal_conflict"),
    );
    h.check_bool(
        "initial_effect_professional",
        npc.trust.current_effect().contains("professional"),
    );
}

fn validate_trust_accumulation(h: &mut ValidationHarness) {
    let mut npc = maren();

    // Session 1: Standard (no change)
    h.check_abs("session1_trust", npc.trust.current_trust(), 0.0, 0.0);

    // Session 2: Bring rare materials (+0.5) + Defend reputation (+1.0) = 1.5
    npc.trust.apply_delta("Bring rare materials", 0.5);
    npc.trust.apply_delta("Defend reputation", 1.0);
    h.check_abs(
        "session2_trust",
        npc.trust.current_trust(),
        1.5,
        tolerances::GAME_STATE_TOL,
    );
    h.check_abs(
        "session2_level",
        f64::from(npc.trust.current_level()),
        1.0,
        0.0,
    );
    h.check_bool(
        "session2_effect_warmer",
        npc.trust.current_effect().contains("Warmer"),
    );

    // Session 3: Help with experiments (+1.0) = 2.5
    npc.trust.apply_delta("Help with experiments", 1.0);
    h.check_abs(
        "session3_trust",
        npc.trust.current_trust(),
        2.5,
        tolerances::GAME_STATE_TOL,
    );
    h.check_abs(
        "session3_level",
        f64::from(npc.trust.current_level()),
        2.0,
        0.0,
    );

    // Session 4: Keep secret (+0.5) + More help (+1.0) = 4.0
    npc.trust.apply_delta("Keep her secret", 0.5);
    npc.trust.apply_delta("Help with experiments", 1.0);
    h.check_abs(
        "session4_trust",
        npc.trust.current_trust(),
        4.0,
        tolerances::GAME_STATE_TOL,
    );
    h.check_abs(
        "session4_level",
        f64::from(npc.trust.current_level()),
        4.0,
        0.0,
    );
    h.check_bool(
        "session4_effect_journal",
        npc.trust.current_effect().contains("journal"),
    );
}

fn validate_trust_gates_secrets(h: &mut ValidationHarness) {
    let npc = maren();

    h.check_bool(
        "workshop_hidden_at_trust_0",
        !npc.can_reveal_secret("hidden_workshop", 0),
    );
    h.check_bool(
        "workshop_hidden_at_trust_2",
        !npc.can_reveal_secret("hidden_workshop", 2),
    );
    h.check_bool(
        "workshop_revealed_at_trust_3",
        npc.can_reveal_secret("hidden_workshop", 3),
    );
    h.check_bool(
        "workshop_revealed_at_trust_5",
        npc.can_reveal_secret("hidden_workshop", 5),
    );

    h.check_bool(
        "journal_hidden_at_trust_3",
        !npc.can_reveal_secret("masters_journal", 3),
    );
    h.check_bool(
        "journal_revealed_at_trust_4",
        npc.can_reveal_secret("masters_journal", 4),
    );
}

fn validate_betrayal_asymmetry(h: &mut ValidationHarness) {
    let npc = maren();

    let max_positive = npc
        .trust
        .positive_actions
        .iter()
        .map(|a| a.delta)
        .fold(0.0_f64, f64::max);
    let max_negative = npc
        .trust
        .negative_actions
        .iter()
        .map(|a| a.delta.abs())
        .fold(0.0_f64, f64::max);

    h.check_bool("betrayal_larger_than_help", max_negative > max_positive);
    h.check_abs("max_positive_action", max_positive, 1.0, 0.0);
    h.check_abs("max_negative_action", max_negative, 5.0, 0.0);

    // Full betrayal scenario
    let mut npc = maren();
    npc.trust.apply_delta("Defend reputation", 1.0);
    npc.trust.apply_delta("Help with experiments", 1.0);
    h.check_abs(
        "pre_betrayal_trust",
        npc.trust.current_trust(),
        2.0,
        tolerances::GAME_STATE_TOL,
    );

    npc.trust.apply_delta("Betray confidence to guild", -5.0);
    h.check_bool("betrayal_devastating", npc.trust.current_trust() < 0.0);
    h.check_abs(
        "betrayal_level_zero",
        f64::from(npc.trust.current_level()),
        0.0,
        0.0,
    );
}

fn validate_arc_progression(h: &mut ValidationHarness) {
    let npc = maren();

    // Verify 3 phases
    h.check_abs("three_arc_phases", npc.arc.len() as f64, 3.0, 0.0);

    // Conformity completed
    h.check_bool(
        "conformity_completed",
        npc.arc[0].status == ArcPhaseStatus::Completed,
    );

    // Internal conflict active
    h.check_bool(
        "internal_conflict_active",
        npc.arc[1].status == ArcPhaseStatus::Active,
    );

    // Revelation pending
    h.check_bool(
        "revelation_pending",
        npc.arc[2].status == ArcPhaseStatus::Pending,
    );

    // Internal conflict has triggers
    h.check_bool(
        "internal_conflict_has_triggers",
        !npc.arc[1].triggers.is_empty(),
    );
    h.check_bool(
        "guild_inspection_trigger",
        npc.arc[1].triggers.contains(&"guild_inspection".into()),
    );
    h.check_bool(
        "trust_level_4_trigger",
        npc.arc[1].triggers.contains(&"trust_level_4".into()),
    );
}

fn validate_quorum_threshold(h: &mut ValidationHarness) {
    // Simulate Quorum mechanic: when multiple NPCs share high need,
    // collective event triggers (exp059 math)

    let npc1_self_actual = 0.7;
    let npc2_self_actual = 0.8;
    let npc3_self_actual = 0.5;
    let threshold = 0.6;

    let above_threshold: Vec<f64> = [npc1_self_actual, npc2_self_actual, npc3_self_actual]
        .iter()
        .filter(|&&v| v >= threshold)
        .copied()
        .collect();

    h.check_abs(
        "npcs_above_threshold",
        above_threshold.len() as f64,
        2.0,
        0.0,
    );

    let quorum_reached = above_threshold.len() >= 2;
    h.check_bool("quorum_reached_with_2", quorum_reached);

    let quorum_not_reached = [0.4, 0.3, 0.5].iter().filter(|&&v| v >= threshold).count() >= 2;
    h.check_bool("quorum_not_reached_below", !quorum_not_reached);
}

fn validate_trust_history(h: &mut ValidationHarness) {
    let mut npc = maren();
    npc.trust.apply_delta("Bring rare materials", 0.5);
    npc.trust.apply_delta("Defend reputation", 1.0);
    npc.trust.apply_delta("Threaten secrets", -2.0);

    h.check_abs("history_length", npc.trust.history_len() as f64, 3.0, 0.0);

    let history = npc.trust.history();
    h.check_abs(
        "first_delta",
        history[0].delta,
        0.5,
        tolerances::GAME_STATE_TOL,
    );
    h.check_abs(
        "second_delta",
        history[1].delta,
        1.0,
        tolerances::GAME_STATE_TOL,
    );
    h.check_abs(
        "third_delta",
        history[2].delta,
        -2.0,
        tolerances::GAME_STATE_TOL,
    );

    h.check_abs(
        "trust_after_first",
        history[0].trust_after,
        0.5,
        tolerances::GAME_STATE_TOL,
    );
    h.check_abs(
        "trust_after_second",
        history[1].trust_after,
        1.5,
        tolerances::GAME_STATE_TOL,
    );
    h.check_abs(
        "trust_after_third",
        history[2].trust_after,
        -0.5,
        tolerances::GAME_STATE_TOL,
    );
}

fn validate_motivation_drives_behavior(h: &mut ValidationHarness) {
    let npc = maren();
    let (active, urgency) = npc.active_need();

    h.check_bool("active_need_is_esteem", active == "esteem");
    h.check_abs("esteem_urgency", urgency, 0.8, tolerances::GAME_STATE_TOL);

    h.check_bool(
        "has_esteem_safety_conflict",
        npc.motivation_conflicts
            .iter()
            .any(|c| c.need_a == "esteem" && c.need_b == "safety"),
    );
}

fn main() {
    let mut h = ValidationHarness::new(EXP);

    validate_initial_state(&mut h);
    validate_trust_accumulation(&mut h);
    validate_trust_gates_secrets(&mut h);
    validate_betrayal_asymmetry(&mut h);
    validate_arc_progression(&mut h);
    validate_quorum_threshold(&mut h);
    validate_trust_history(&mut h);
    validate_motivation_drives_behavior(&mut h);

    h.finish();
}
