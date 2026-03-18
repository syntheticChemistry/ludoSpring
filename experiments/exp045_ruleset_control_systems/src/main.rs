// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp045 — Ruleset Control Systems
//!
//! Three structurally different open RPG rulesets implemented as control
//! systems, proving the type model handles d20, Fudge dice, and roll-under
//! through the same interface.
//!
//! Rulesets:
//!   1. Pathfinder 2e (ORC License) — d20 + modifier vs DC, 4 degrees, 3 actions
//!   2. FATE Core (CC-BY) — 4dF + skill vs difficulty, stress/consequences, Aspects
//!   3. Cairn (CC-BY-SA) — d20 roll-under, direct damage, 3 abilities, inventory slots
//!
//! Anti-cheat = chain-of-custody isomorphism: the same DAG vertex types that
//! track item lineage here track sample lineage in field genomics.

mod rulesets;

use ludospring_barracuda::game::ruleset::{
    Character, Condition, DegreeOfSuccess, DiceResult, DiceSystem, Ruleset,
};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

use rulesets::{Cairn, FateCore, Pathfinder2e};
use rulesets::{cairn_ability_target, fate_skill_modifier, pf2e_skill_modifier};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Pathfinder 2e ORC, FATE Core, Cairn)",
    commit: "74cf9488",
    date: "2026-03-15",
    command: "N/A (ruleset control systems)",
};

// ===========================================================================
// Validation
// ===========================================================================

fn validate_pf2e(h: &mut ValidationHarness) {
    let pf2e = Pathfinder2e;
    let summary = pf2e.summary();
    let character = pf2e.default_character("Valeros");

    h.check_bool(
        "pf2e_dice_system_is_d20",
        summary.dice_system == DiceSystem::D20,
    );
    h.check_abs(
        "pf2e_actions_per_turn_3",
        f64::from(summary.action_economy.actions),
        3.0,
        0.0,
    );
    h.check_abs(
        "pf2e_reactions_per_round_1",
        f64::from(summary.action_economy.reactions),
        1.0,
        0.0,
    );
    #[expect(
        clippy::cast_precision_loss,
        reason = "ability_count and degree_count are small"
    )]
    h.check_abs(
        "pf2e_six_ability_scores",
        summary.ability_count as f64,
        6.0,
        0.0,
    );
    h.check_bool("pf2e_has_proficiency", summary.has_proficiency);
    #[expect(clippy::cast_precision_loss, reason = "degree_count is small")]
    h.check_abs(
        "pf2e_four_degrees_of_success",
        summary.degree_count as f64,
        4.0,
        0.0,
    );

    // Character checks
    #[expect(clippy::cast_precision_loss, reason = "ability count is small")]
    h.check_abs(
        "pf2e_character_has_six_abilities",
        character.abilities.len() as f64,
        6.0,
        0.0,
    );
    let Some(str_ability) = character.abilities.iter().find(|a| a.name == "Strength") else {
        eprintln!("FATAL: Strength ability not found");
        std::process::exit(1);
    };
    let str_mod = str_ability.modifier;
    h.check_abs("pf2e_str_14_gives_mod_2", f64::from(str_mod), 2.0, 0.0);

    // Skill modifier: Perception (Trained, WIS 14, level 1) = 2 (WIS mod) + 1 (level) + 2 (trained) = 5
    let percep_mod = pf2e_skill_modifier(&character, "Perception");
    h.check_abs(
        "pf2e_perception_trained_mod_5",
        f64::from(percep_mod),
        5.0,
        0.0,
    );

    // Untrained skill: Thievery (Untrained, DEX 12) = 1 (DEX mod) + 0 = 1
    let thievery_mod = pf2e_skill_modifier(&character, "Thievery");
    h.check_abs(
        "pf2e_thievery_untrained_mod_1",
        f64::from(thievery_mod),
        1.0,
        0.0,
    );

    // Resolution: roll 15, modifier +5, DC 20 → 15+5=20 >= 20 → Success
    let degree = pf2e.resolve_check(5, 20, &DiceResult::single(15));
    h.check_abs(
        "pf2e_roll15_mod5_dc20_success",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::Success.as_i32()),
        0.0,
    );

    // Resolution: roll 15, mod +5, DC 10 → 20-10=10 → CritSuccess
    let degree = pf2e.resolve_check(5, 10, &DiceResult::single(15));
    h.check_abs(
        "pf2e_roll15_mod5_dc10_critsuccess",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::CriticalSuccess.as_i32()),
        0.0,
    );

    // Resolution: roll 5, mod +5, DC 20 → 10-20=-10 → Failure
    let degree = pf2e.resolve_check(5, 20, &DiceResult::single(5));
    h.check_abs(
        "pf2e_roll5_mod5_dc20_failure",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::Failure.as_i32()),
        0.0,
    );

    // Natural 20 promotes: roll 20, mod +0, DC 30 → 20-30=-10 → Failure, but nat20 → Success
    let degree = pf2e.resolve_check(0, 30, &DiceResult::single(20));
    h.check_abs(
        "pf2e_nat20_promotes_failure_to_success",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::Success.as_i32()),
        0.0,
    );

    // Natural 1 demotes: roll 1, mod +10, DC 5 → 11-5=6 → Success, but nat1 → Failure
    let degree = pf2e.resolve_check(10, 5, &DiceResult::single(1));
    h.check_abs(
        "pf2e_nat1_demotes_success_to_failure",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::Failure.as_i32()),
        0.0,
    );

    // Condition decay: Frightened 3 → 2 → 1 → 0 over 3 turns
    let mut frightened = Condition {
        name: "Frightened".into(),
        value: 3,
        decay_per_turn: 1,
        turns_remaining: None,
    };
    frightened.tick();
    frightened.tick();
    frightened.tick();
    h.check_abs(
        "pf2e_frightened3_decays_to_0_in_3_turns",
        f64::from(frightened.value),
        0.0,
        0.0,
    );
}

#[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
fn validate_fate(h: &mut ValidationHarness) {
    let fate = FateCore;
    let summary = fate.summary();
    let character = fate.default_character("Zird the Arcane");

    h.check_bool(
        "fate_dice_system_is_fudge",
        summary.dice_system == DiceSystem::FudgeDice,
    );
    h.check_bool("fate_has_aspects", summary.has_aspects);
    h.check_abs(
        "fate_no_ability_scores",
        summary.ability_count as f64,
        0.0,
        0.0,
    );
    h.check_abs(
        "fate_five_degrees_of_success",
        summary.degree_count as f64,
        5.0,
        0.0,
    );
    h.check_bool("fate_no_proficiency", !summary.has_proficiency);

    // Character checks
    h.check_abs(
        "fate_character_has_three_aspects",
        character.tags.len() as f64,
        3.0,
        0.0,
    );
    h.check_abs(
        "fate_character_has_stress_tracks",
        character.resource_tracks.len() as f64,
        3.0,
        0.0,
    );
    let fight_rating = fate_skill_modifier(&character, "Fight");
    h.check_abs("fate_fight_rating_3", f64::from(fight_rating), 3.0, 0.0);
    let Some(fate_points) = character.resource_tracks.get("Fate Points") else {
        eprintln!("FATAL: Fate Points track not found");
        std::process::exit(1);
    };
    h.check_abs(
        "fate_starts_with_3_fate_points",
        f64::from(fate_points.current),
        3.0,
        0.0,
    );

    // Resolution: 4dF=[+1,+1,0,-1]=+1, Fight +3, difficulty 2 → total 4, shifts = 4-2 = 2 → Success
    let degree = fate.resolve_check(3, 2, &DiceResult::multi(vec![1, 1, 0, -1]));
    h.check_abs(
        "fate_4df_plus1_fight3_diff2_success",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::Success.as_i32()),
        0.0,
    );

    // Resolution: 4dF=[+1,+1,+1,+1]=+4, Fight +3 = 7, difficulty 2 → shifts 5 → CritSuccess (succeed with style)
    let degree = fate.resolve_check(3, 2, &DiceResult::multi(vec![1, 1, 1, 1]));
    h.check_abs(
        "fate_perfect_roll_critsuccess",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::CriticalSuccess.as_i32()),
        0.0,
    );

    // Resolution: 4dF=[0,0,0,0]=0, Fight +3 = 3, difficulty 3 → shifts 0 → PartialSuccess (tie)
    let degree = fate.resolve_check(3, 3, &DiceResult::multi(vec![0, 0, 0, 0]));
    h.check_abs(
        "fate_tie_is_partial_success",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::PartialSuccess.as_i32()),
        0.0,
    );

    // Resolution: 4dF=[-1,-1,-1,-1]=-4, Will +1 = -3, difficulty 2 → shifts -5 → CritFailure
    let degree = fate.resolve_check(1, 2, &DiceResult::multi(vec![-1, -1, -1, -1]));
    h.check_abs(
        "fate_worst_roll_critfailure",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::CriticalFailure.as_i32()),
        0.0,
    );

    // Aspect invocation: Fate Point spend adds +2 to skill
    let boosted = fate_skill_modifier(&character, "Fight") + 2;
    let degree = fate.resolve_check(boosted, 4, &DiceResult::multi(vec![0, 0, 0, 0]));
    h.check_abs(
        "fate_aspect_invoke_adds_2_shifts",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::Success.as_i32()),
        0.0,
    );
}

/// Apply Cairn damage overflow (HP first, then STR) and return final Strength value.
fn cairn_damage_overflow_str_value(character: &mut Character, damage: i32) -> i32 {
    let overflow = damage - character.hp_current;
    character.hp_current = 0;
    if overflow > 0 {
        let Some(str_ability) = character
            .abilities
            .iter_mut()
            .find(|a| a.name == "Strength")
        else {
            eprintln!("FATAL: Strength ability not found");
            std::process::exit(1);
        };
        str_ability.value -= overflow;
        str_ability.modifier = str_ability.value;
    }
    let Some(str_ability) = character.abilities.iter().find(|a| a.name == "Strength") else {
        eprintln!("FATAL: Strength ability not found");
        std::process::exit(1);
    };
    str_ability.value
}

#[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
fn validate_cairn(h: &mut ValidationHarness) {
    let cairn = Cairn;
    let summary = cairn.summary();
    let character = cairn.default_character("Esme");

    h.check_bool(
        "cairn_dice_system_is_roll_under",
        summary.dice_system == DiceSystem::RollUnder,
    );
    h.check_abs(
        "cairn_three_ability_scores",
        summary.ability_count as f64,
        3.0,
        0.0,
    );
    h.check_abs(
        "cairn_one_action_per_turn",
        f64::from(summary.action_economy.actions),
        1.0,
        0.0,
    );
    h.check_bool("cairn_no_proficiency", !summary.has_proficiency);

    // Character checks
    h.check_abs(
        "cairn_character_has_three_abilities",
        character.abilities.len() as f64,
        3.0,
        0.0,
    );
    h.check_bool(
        "cairn_has_inventory_slots",
        character.resource_tracks.contains_key("Inventory Slots"),
    );
    let str_val = cairn_ability_target(&character, "Strength");
    h.check_abs("cairn_str_12", f64::from(str_val), 12.0, 0.0);

    // Resolution: roll 8, STR 12 → 8 ≤ 12 → Success
    let degree = cairn.resolve_check(12, 0, &DiceResult::single(8));
    h.check_abs(
        "cairn_roll8_str12_success",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::Success.as_i32()),
        0.0,
    );

    // Resolution: roll 15, STR 12 → 15 > 12 → Failure
    let degree = cairn.resolve_check(12, 0, &DiceResult::single(15));
    h.check_abs(
        "cairn_roll15_str12_failure",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::Failure.as_i32()),
        0.0,
    );

    // Resolution: roll 1 → CritSuccess (always)
    let degree = cairn.resolve_check(12, 0, &DiceResult::single(1));
    h.check_abs(
        "cairn_roll1_critsuccess",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::CriticalSuccess.as_i32()),
        0.0,
    );

    // Resolution: roll 20 → CritFailure (always)
    let degree = cairn.resolve_check(18, 0, &DiceResult::single(20));
    h.check_abs(
        "cairn_roll20_critfailure",
        f64::from(degree.as_i32()),
        f64::from(DegreeOfSuccess::CriticalFailure.as_i32()),
        0.0,
    );

    // Direct damage: Cairn damage goes to HP first, then STR. Sword (d6=6) vs 4 HP → HP=0, overflow to STR
    let mut esme = cairn.default_character("Esme");
    let str_after_damage = cairn_damage_overflow_str_value(&mut esme, 6);
    h.check_abs(
        "cairn_damage_overflows_hp_to_str",
        f64::from(str_after_damage),
        10.0, // 12 - (6-4) = 10
        0.0,
    );
}

fn validate_cross_system(h: &mut ValidationHarness) {
    let systems: Vec<Box<dyn Ruleset>> =
        vec![Box::new(Pathfinder2e), Box::new(FateCore), Box::new(Cairn)];

    #[expect(clippy::cast_precision_loss, reason = "system count is small")]
    h.check_abs(
        "three_control_rulesets_loaded",
        systems.len() as f64,
        3.0,
        0.0,
    );

    // All three produce different dice systems
    let dice_systems: Vec<_> = systems.iter().map(|s| s.dice_system().clone()).collect();
    let all_different = dice_systems[0] != dice_systems[1]
        && dice_systems[1] != dice_systems[2]
        && dice_systems[0] != dice_systems[2];
    h.check_bool("all_dice_systems_different", all_different);

    // All three produce valid characters with names
    let characters: Vec<_> = systems
        .iter()
        .map(|s| s.default_character("Test"))
        .collect();
    let all_named = characters.iter().all(|c| !c.name.is_empty());
    h.check_bool("all_characters_have_names", all_named);

    // Different ability count: PF2e=6, FATE=0, Cairn=3
    let ability_counts: Vec<usize> = characters.iter().map(|c| c.abilities.len()).collect();
    h.check_bool(
        "pf2e_6_fate_0_cairn_3_abilities",
        ability_counts[0] == 6 && ability_counts[1] == 0 && ability_counts[2] == 3,
    );

    // All three resolve the same DiceResult differently
    let roll = DiceResult::single(10);
    let degrees: Vec<DegreeOfSuccess> = systems
        .iter()
        .map(|s| s.resolve_check(2, 15, &roll))
        .collect();
    // PF2e: 10+2=12 vs DC15 → Failure. FATE: 10+2=12 vs 15 → CritFail. Cairn: 10 ≤ 2? → Failure.
    // The point: same input, different resolution — the trait dispatches correctly.
    let at_least_two_different =
        degrees[0] != degrees[1] || degrees[1] != degrees[2] || degrees[0] != degrees[2];
    h.check_bool(
        "same_roll_different_resolution_per_system",
        at_least_two_different,
    );

    // Structural diversity: action economies differ
    let actions: Vec<u8> = systems.iter().map(|s| s.action_economy().actions).collect();
    let actions_differ = actions[0] != actions[1] || actions[1] != actions[2];
    h.check_bool("action_economies_differ", actions_differ);

    // All summaries serialize to JSON (proxy for loamSpine certificate readiness)
    let summaries: Vec<_> = systems.iter().map(|s| s.summary()).collect();
    let all_serialize = summaries
        .iter()
        .all(|s| !s.name.is_empty() && !s.license.is_empty());
    h.check_bool("all_summaries_serialize_ready", all_serialize);
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp045_ruleset_control_systems");
    h.print_provenance(&[&PROVENANCE]);

    validate_pf2e(&mut h);
    validate_fate(&mut h);
    validate_cairn(&mut h);
    validate_cross_system(&mut h);

    h.finish();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") | None => cmd_validate(),
        Some(other) => {
            eprintln!("Unknown command: {other}");
            eprintln!("Usage: exp045_ruleset_control_systems [validate]");
            std::process::exit(1);
        }
    }
}
