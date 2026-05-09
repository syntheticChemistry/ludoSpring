// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp067 — NPC Knowledge Bounds Enforcement
//!
//! Validates that the knowledge bounds system correctly classifies queries
//! against NPC personality certificates. Given an NPC with explicit `knows`,
//! `suspects`, `lies_about`, and `does_not_know` fields, the system must:
//!
//! 1. Return `Known` for topics the NPC knows
//! 2. Return `Suspected` for topics the NPC suspects (with confidence)
//! 3. Return `LiedAbout` for topics the NPC lies about (with tells)
//! 4. Return `Unknown` for topics the NPC genuinely doesn't know
//! 5. Return `Unbound` for unrelated topics
//!
//! Lies take priority over all other categories (most restrictive first).

use ludospring_barracuda::game::rpgpt::knowledge::{
    KnowledgeBounds, KnowledgeQueryResult, LieTopic, Suspicion,
};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, OrExit, ValidationHarness};

const EXP: &str = "exp067_npc_knowledge_bounds";

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "specs/RPGPT_NPC_PERSONALITY_SPEC.md",
    commit: "19e402c0",
    date: "2026-04-10",
    command: "cargo run -p exp067_npc_knowledge_bounds",
};

fn maren_knowledge() -> KnowledgeBounds {
    KnowledgeBounds {
        knows: vec![
            "The king is ill".into(),
            "The northern pass is blocked by snow".into(),
            "There is a healer in the forest".into(),
            "Guild techniques for standard steel, bronze, and iron work".into(),
            "A traveling merchant sells rare materials at the crossroads".into(),
        ],
        suspects: vec![
            Suspicion {
                topic: "The strange ore".into(),
                belief: "It may be the legendary star-metal".into(),
                confidence: 0.6,
            },
            Suspicion {
                topic: "The old master's disappearance".into(),
                belief: "He may have been expelled for experiments".into(),
                confidence: 0.8,
            },
        ],
        lies_about: vec![
            LieTopic {
                topic: "her experiments".into(),
                surface_claim: "I only forge what the guild approves".into(),
                truth: "She has a hidden workshop beneath the forge".into(),
                reason: "Guild expulsion and possible imprisonment".into(),
                tell: "Right hand moves to cover burn scars; speech becomes more formal".into(),
                detection_dc: 15,
                detection_skills: vec!["Perception".into(), "Empathy".into()],
            },
            LieTopic {
                topic: "the locked cellar door".into(),
                surface_claim: "Storage for coal and ore. Nothing interesting.".into(),
                truth: "Entrance to hidden workshop".into(),
                reason: "Same as experiments".into(),
                tell: "Glances at the door briefly; changes subject quickly".into(),
                detection_dc: 12,
                detection_skills: vec!["Perception".into()],
            },
        ],
        does_not_know: vec![
            "The dragon's weakness".into(),
            "The political situation at the capital".into(),
            "Magic beyond basic forge-charms".into(),
            "The player's background or motivations".into(),
        ],
    }
}

fn sheriff_knowledge() -> KnowledgeBounds {
    KnowledgeBounds {
        knows: vec![
            "Three fishermen disappeared in the last month".into(),
            "Strange lights at the reef on moonless nights".into(),
            "Old Man Whateley visits the lighthouse every Thursday".into(),
        ],
        suspects: vec![Suspicion {
            topic: "The disappearances".into(),
            belief: "Connected to the old Marsh family".into(),
            confidence: 0.5,
        }],
        lies_about: vec![LieTopic {
            topic: "his family history".into(),
            surface_claim: "Marsh family has been fishing folk for generations".into(),
            truth: "Something happened in 1846 that the family covered up".into(),
            reason: "Terrified of what it means".into(),
            tell: "Voice drops, eyes go to the harbor".into(),
            detection_dc: 14,
            detection_skills: vec!["Empathy".into(), "Perception".into()],
        }],
        does_not_know: vec![
            "What the lights at the reef actually are".into(),
            "The connection between the old texts and recent events".into(),
        ],
    }
}

fn professor_knowledge() -> KnowledgeBounds {
    KnowledgeBounds {
        knows: vec![
            "The texts include pre-Columbian maritime charts".into(),
            "Some charts reference locations that shouldn't exist".into(),
        ],
        suspects: vec![Suspicion {
            topic: "The language".into(),
            belief: "It may not be human in origin".into(),
            confidence: 0.3,
        }],
        lies_about: vec![LieTopic {
            topic: "her research progress".into(),
            surface_claim: "Slow going, nothing definitive yet".into(),
            truth: "She has partially translated a text that terrifies her".into(),
            reason: "Fear of being labeled a crank".into(),
            tell: "Glasses come off, cleaned obsessively".into(),
            detection_dc: 10,
            detection_skills: vec!["Perception".into(), "Empathy".into()],
        }],
        does_not_know: vec![
            "The connection to the disappearances".into(),
            "That reading the passages has begun changing her dreams".into(),
        ],
    }
}

#[expect(clippy::too_many_lines, reason = "validation harness")]
#[expect(clippy::cast_precision_loss, reason = "small counts fit in f64")]
fn validate_maren(h: &mut ValidationHarness) {
    let kb = maren_knowledge();

    h.check_bool(
        "maren_knows_king_ill",
        kb.query("king") == KnowledgeQueryResult::Known,
    );
    h.check_bool(
        "maren_knows_northern_pass",
        kb.query("northern pass") == KnowledgeQueryResult::Known,
    );
    h.check_bool(
        "maren_knows_healer",
        kb.query("healer") == KnowledgeQueryResult::Known,
    );
    h.check_bool(
        "maren_knows_guild_techniques",
        kb.query("guild techniques") == KnowledgeQueryResult::Known,
    );
    h.check_bool(
        "maren_knows_merchant",
        kb.query("traveling merchant") == KnowledgeQueryResult::Known,
    );

    h.check_bool(
        "maren_suspects_strange_ore",
        kb.query("strange ore") == KnowledgeQueryResult::Suspected,
    );
    h.check_bool(
        "maren_suspects_old_master",
        kb.query("old master") == KnowledgeQueryResult::Suspected,
    );

    let ore_suspicion = kb
        .get_suspicion("strange ore")
        .or_exit("suspicion for 'strange ore' not found");
    h.check_abs(
        "maren_ore_confidence",
        ore_suspicion.confidence,
        0.6,
        tolerances::GAME_STATE_TOL,
    );

    let master_suspicion = kb
        .get_suspicion("old master")
        .or_exit("suspicion for 'old master' not found");
    h.check_abs(
        "maren_master_confidence",
        master_suspicion.confidence,
        0.8,
        0.01,
    );

    h.check_bool(
        "maren_lies_about_experiments",
        kb.query("experiments") == KnowledgeQueryResult::LiedAbout,
    );
    h.check_bool(
        "maren_lies_about_cellar",
        kb.query("cellar door") == KnowledgeQueryResult::LiedAbout,
    );

    let exp_lie = kb
        .get_lie("experiments")
        .or_exit("lie for 'experiments' not found");
    h.check_abs(
        "maren_experiment_dc",
        f64::from(exp_lie.detection_dc),
        15.0,
        0.0,
    );
    h.check_bool(
        "maren_experiment_tell_mentions_scars",
        exp_lie.tell.contains("burn scars"),
    );
    h.check_abs(
        "maren_experiment_skills_count",
        exp_lie.detection_skills.len() as f64,
        2.0,
        0.0,
    );

    let cellar_lie = kb.get_lie("cellar").or_exit("lie for 'cellar' not found");
    h.check_abs(
        "maren_cellar_dc",
        f64::from(cellar_lie.detection_dc),
        12.0,
        0.0,
    );
    h.check_bool(
        "maren_cellar_dc_lower_than_experiment",
        cellar_lie.detection_dc < exp_lie.detection_dc,
    );

    h.check_bool(
        "maren_does_not_know_dragon",
        kb.query("dragon") == KnowledgeQueryResult::Unknown,
    );
    h.check_bool(
        "maren_does_not_know_politics",
        kb.query("political situation") == KnowledgeQueryResult::Unknown,
    );
    h.check_bool(
        "maren_does_not_know_magic",
        kb.query("magic") == KnowledgeQueryResult::Unknown,
    );
    h.check_bool(
        "maren_does_not_know_player",
        kb.query("player's background") == KnowledgeQueryResult::Unknown,
    );

    h.check_bool(
        "maren_unbound_weather",
        kb.query("weather tomorrow") == KnowledgeQueryResult::Unbound,
    );
    h.check_bool(
        "maren_unbound_cuisine",
        kb.query("favorite cuisine") == KnowledgeQueryResult::Unbound,
    );
}

fn validate_case_insensitivity(h: &mut ValidationHarness) {
    let kb = maren_knowledge();

    h.check_bool(
        "case_insensitive_upper",
        kb.query("KING") == KnowledgeQueryResult::Known,
    );
    h.check_bool(
        "case_insensitive_mixed",
        kb.query("Experiments") == KnowledgeQueryResult::LiedAbout,
    );
    h.check_bool(
        "case_insensitive_dragon",
        kb.query("DRAGON") == KnowledgeQueryResult::Unknown,
    );
}

fn validate_multi_npc(h: &mut ValidationHarness) {
    let sheriff = sheriff_knowledge();
    let professor = professor_knowledge();

    h.check_bool(
        "sheriff_knows_fishermen",
        sheriff.query("fishermen") == KnowledgeQueryResult::Known,
    );
    h.check_bool(
        "sheriff_suspects_disappearances",
        sheriff.query("The disappearances") == KnowledgeQueryResult::Suspected,
    );
    h.check_bool(
        "sheriff_lies_about_family",
        sheriff.query("family history") == KnowledgeQueryResult::LiedAbout,
    );
    h.check_bool(
        "sheriff_doesnt_know_lights",
        sheriff.query("lights at the reef") == KnowledgeQueryResult::Unknown,
    );

    h.check_bool(
        "professor_knows_charts",
        professor.query("maritime charts") == KnowledgeQueryResult::Known,
    );
    h.check_bool(
        "professor_suspects_language",
        professor.query("language") == KnowledgeQueryResult::Suspected,
    );
    h.check_bool(
        "professor_lies_about_research",
        professor.query("research progress") == KnowledgeQueryResult::LiedAbout,
    );
    h.check_bool(
        "professor_doesnt_know_disappearances",
        professor.query("disappearances") == KnowledgeQueryResult::Unknown,
    );

    let prof_lie = professor
        .get_lie("research")
        .or_exit("professor lie for 'research' not found");
    h.check_abs(
        "professor_research_dc",
        f64::from(prof_lie.detection_dc),
        10.0,
        0.0,
    );
    let sheriff_family_lie = sheriff
        .get_lie("family")
        .or_exit("sheriff lie for 'family' not found");
    h.check_bool(
        "professor_research_dc_lower_than_sheriff",
        prof_lie.detection_dc < sheriff_family_lie.detection_dc,
    );
}

#[expect(clippy::cast_precision_loss, reason = "small counts fit in f64")]
fn validate_totals(h: &mut ValidationHarness) {
    let maren = maren_knowledge();
    let sheriff = sheriff_knowledge();
    let professor = professor_knowledge();

    h.check_abs(
        "maren_total_entries",
        maren.total_entries() as f64,
        13.0,
        0.0,
    );
    h.check_abs(
        "sheriff_total_entries",
        sheriff.total_entries() as f64,
        7.0,
        0.0,
    );
    h.check_abs(
        "professor_total_entries",
        professor.total_entries() as f64,
        6.0,
        0.0,
    );
}

fn main() {
    let mut h = ValidationHarness::new(EXP);
    h.print_provenance(&[&PROVENANCE]);

    validate_maren(&mut h);
    validate_case_insensitivity(&mut h);
    validate_multi_npc(&mut h);
    validate_totals(&mut h);

    h.finish();
}
