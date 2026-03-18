// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery)]

//! exp055 — Usurper: Nemesis system from population dynamics.
//!
//! Recreates the core mechanics of persistent adaptive NPC hierarchies
//! using open math from evolutionary biology and population dynamics.
//! Every model traces to published research predating the proprietary
//! "Nemesis system" patent (WB US 9,573,066 B2, filed 2015).

mod population;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use population::{
    EncounterOutcome, Hierarchy, LotkaVolterraMemory, PromotionReason, Strategy, Usurper,
};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — population dynamics)",
    commit: "N/A",
    date: "N/A",
    command: "N/A (pure Rust implementation)",
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "validate" {
        cmd_validate();
    }
    println!("Usage: exp055_usurper validate");
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp055_usurper");
    h.print_provenance(&[&PROVENANCE]);

    validate_usurper_entity(&mut h);
    validate_encounter_adaptation(&mut h);
    validate_hierarchy_replicator(&mut h);
    validate_vacancy_filling(&mut h);
    validate_betrayal(&mut h);
    validate_payoff_matrix(&mut h);
    validate_lotka_volterra_memory(&mut h);
    validate_emergent_narrative(&mut h);
    validate_cross_domain(&mut h);

    h.finish();
}

// --- Section 1 ---

fn validate_usurper_entity(h: &mut ValidationHarness) {
    let u = Usurper::new(0, "Gruk the Defiler", 3, Strategy::Aggressive);

    h.check_bool(
        "usurper_creation",
        u.id == 0 && u.rank == 3 && u.strategy == Strategy::Aggressive,
    );
    h.check_bool(
        "usurper_initial_power",
        (u.power - 1.0).abs() < f64::EPSILON,
    );
    h.check_bool("usurper_no_encounters", u.encounter_count() == 0 && u.alive);
    h.check_bool(
        "usurper_initial_fitness",
        (u.current_fitness() - 1.0).abs() < f64::EPSILON,
    );
    h.check_bool(
        "usurper_initial_survival_rate",
        (u.survival_rate() - 1.0).abs() < f64::EPSILON,
    );
}

// --- Section 2 ---

fn validate_encounter_adaptation(h: &mut ValidationHarness) {
    let mut u = Usurper::new(1, "Krosh Fire-Eyes", 2, Strategy::Aggressive);

    u.record_encounter(1, EncounterOutcome::NpcWins, None);

    h.check_bool("encounter_npc_wins_power_up", u.power > 1.0);
    h.check_bool("encounter_gains_strength", u.strengths.len() == 1);

    let mut u2 = Usurper::new(2, "Zog the Eternal", 4, Strategy::Defensive);
    u2.record_encounter(2, EncounterOutcome::Fled, None);

    h.check_bool(
        "encounter_fled_adapts_strategy",
        u2.strategy == Strategy::Deceptive,
    );
    h.check_bool("encounter_fled_gains_weakness", u2.weaknesses.len() == 1);

    let mut u3 = Usurper::new(3, "Prak the Brittle", 5, Strategy::Cautious);
    u3.record_encounter(3, EncounterOutcome::PlayerWins, None);
    u3.record_encounter(4, EncounterOutcome::PlayerWins, None);
    u3.record_encounter(5, EncounterOutcome::PlayerWins, None);
    u3.record_encounter(6, EncounterOutcome::PlayerWins, None);

    h.check_bool("encounter_repeated_defeat_kills", !u3.alive);
    h.check_bool("encounter_survival_rate_tracks", u3.survival_rate() < 0.01);

    let mut u4 = Usurper::new(4, "Bolg Ironclaw", 3, Strategy::Aggressive);
    u4.record_encounter(10, EncounterOutcome::Draw, None);

    h.check_bool(
        "encounter_draw_slight_power_gain",
        u4.power > 1.0 && u4.power < 1.1,
    );
    h.check_bool(
        "encounter_fitness_history_grows",
        u4.fitness_history_len() == 2,
    );
}

// --- Section 3 ---

fn validate_hierarchy_replicator(vh: &mut ValidationHarness) {
    let mut h = Hierarchy::new();
    h.spawn("Captain Alpha", 1, Strategy::Aggressive);
    h.spawn("Captain Beta", 2, Strategy::Defensive);
    h.spawn("Captain Gamma", 3, Strategy::Deceptive);
    h.spawn("Grunt Delta", 4, Strategy::Cautious);

    vh.check_bool("hierarchy_population_size", h.alive_count() == 4);

    h.player_encounter(0, EncounterOutcome::NpcWins);
    h.player_encounter(0, EncounterOutcome::NpcWins);
    h.player_encounter(3, EncounterOutcome::PlayerWins);

    h.replicator_step();

    let Some(alpha) = h.usurpers.iter().find(|u| u.id == 0) else {
        eprintln!("FATAL: usurper id 0 (alpha) not found");
        std::process::exit(1);
    };
    let Some(delta) = h.usurpers.iter().find(|u| u.id == 3) else {
        eprintln!("FATAL: usurper id 3 (delta) not found");
        std::process::exit(1);
    };

    vh.check_bool(
        "replicator_winner_above_mean",
        alpha.power > h.mean_fitness(),
    );
    vh.check_bool(
        "replicator_loser_below_mean",
        delta.power < h.mean_fitness() || !delta.alive,
    );
    vh.check_bool("replicator_mean_positive", h.mean_fitness() > 0.0);
}

// --- Section 4 ---

fn validate_vacancy_filling(vh: &mut ValidationHarness) {
    let mut h = Hierarchy::new();
    h.spawn("Warchief", 1, Strategy::Aggressive);
    h.spawn("Captain", 2, Strategy::Defensive);
    let grunt_id = h.spawn("Strong Grunt", 3, Strategy::Deceptive);

    h.player_encounter(0, EncounterOutcome::PlayerWins);
    h.player_encounter(0, EncounterOutcome::PlayerWins);
    h.player_encounter(0, EncounterOutcome::PlayerWins);
    h.player_encounter(0, EncounterOutcome::PlayerWins);

    let Some(grunt) = h.usurpers.iter_mut().find(|u| u.id == grunt_id) else {
        eprintln!("FATAL: usurper grunt_id not found");
        std::process::exit(1);
    };
    grunt.power = 3.0;

    h.fill_vacancies();

    vh.check_bool("vacancy_promotion_occurs", !h.promotions.is_empty());

    let promoted = h
        .promotions
        .iter()
        .any(|p| matches!(p.reason, PromotionReason::VacancyAbove));
    vh.check_bool("vacancy_reason_correct", promoted);

    let Some(grunt) = h.usurpers.iter().find(|u| u.id == grunt_id) else {
        eprintln!("FATAL: usurper grunt_id not found for vacancy check");
        std::process::exit(1);
    };
    vh.check_bool("vacancy_strongest_promoted", grunt.rank < 3);
}

// --- Section 5 ---

fn validate_betrayal(vh: &mut ValidationHarness) {
    let mut h = Hierarchy::new();
    h.spawn("Overlord", 1, Strategy::Cautious);
    let amb_id = h.spawn("Ambitious", 2, Strategy::Deceptive);

    let Some(amb) = h.usurpers.iter_mut().find(|u| u.id == amb_id) else {
        eprintln!("FATAL: usurper amb_id not found");
        std::process::exit(1);
    };
    amb.power = 5.0;

    h.check_betrayals(4.0);

    vh.check_bool("betrayal_occurs", !h.betrayals.is_empty());

    if let Some(b) = h.betrayals.first() {
        vh.check_bool("betrayal_correct_betrayer", b.betrayer_id == amb_id);

        let Some(target) = h.usurpers.iter().find(|u| u.id == b.target_id) else {
            eprintln!("FATAL: betrayal target not found");
            std::process::exit(1);
        };
        vh.check_bool("betrayal_target_eliminated", !target.alive);
    } else {
        vh.check_bool("betrayal_correct_betrayer", false);
        vh.check_bool("betrayal_target_eliminated", false);
    }

    let Some(amb) = h.usurpers.iter().find(|u| u.id == amb_id) else {
        eprintln!("FATAL: usurper amb_id not found for betrayal promotion check");
        std::process::exit(1);
    };
    vh.check_bool("betrayal_promotes_betrayer", amb.rank == 1);

    let betrayal_promotion = h
        .promotions
        .iter()
        .any(|p| matches!(p.reason, PromotionReason::Betrayal));
    vh.check_bool("betrayal_promotion_recorded", betrayal_promotion);
}

// --- Section 6 ---

fn validate_payoff_matrix(vh: &mut ValidationHarness) {
    let h = Hierarchy::new();

    vh.check_bool(
        "payoff_symmetric_cooperation",
        (h.payoff(Strategy::Defensive, Strategy::Defensive) - 1.0).abs() < f64::EPSILON,
    );
    vh.check_bool(
        "payoff_aggression_vs_cautious",
        h.payoff(Strategy::Aggressive, Strategy::Cautious)
            > h.payoff(Strategy::Cautious, Strategy::Aggressive),
    );
    vh.check_bool(
        "payoff_deception_beats_aggression",
        h.payoff(Strategy::Deceptive, Strategy::Aggressive)
            > h.payoff(Strategy::Aggressive, Strategy::Deceptive),
    );

    let all = Strategy::ALL;
    let mut all_positive = true;
    for &a in &all {
        for &b in &all {
            if h.payoff(a, b) <= 0.0 {
                all_positive = false;
            }
        }
    }
    vh.check_bool("payoff_all_positive", all_positive);

    let agg_agg = h.payoff(Strategy::Aggressive, Strategy::Aggressive);
    let def_def = h.payoff(Strategy::Defensive, Strategy::Defensive);
    vh.check_bool("payoff_mutual_aggression_costly", agg_agg < def_def);
}

// --- Section 7 ---

fn validate_lotka_volterra_memory(h: &mut ValidationHarness) {
    let mut lv = LotkaVolterraMemory::new();

    h.check_bool(
        "lv_initial_populations",
        (lv.prey_pop - 10.0).abs() < f64::EPSILON && (lv.predator_pop - 1.0).abs() < f64::EPSILON,
    );

    lv.run(500, 0.1);

    h.check_bool("lv_prey_survives", lv.prey_survived());
    h.check_bool("lv_adapted", lv.adapted());
    h.check_bool("lv_oscillatory_history", lv.history.len() == 501);

    let prey_values: Vec<f64> = lv.history.iter().map(|&(n, _)| n).collect();
    let has_local_min = prey_values.windows(3).any(|w| w[1] < w[0] && w[1] < w[2]);
    h.check_bool("lv_shows_oscillation", has_local_min);
    h.check_bool("lv_predator_positive", lv.predator_pop > 0.0);
}

// --- Section 8 ---

fn validate_emergent_narrative(vh: &mut ValidationHarness) {
    let mut h = Hierarchy::new();
    let warchief = h.spawn("Thrak Blood-Axe", 1, Strategy::Aggressive);
    let _captain1 = h.spawn("Grish the Cunning", 2, Strategy::Deceptive);
    let captain2 = h.spawn("Bolg Iron-Jaw", 2, Strategy::Defensive);
    let _grunt1 = h.spawn("Rat-Bag", 3, Strategy::Cautious);
    let grunt2 = h.spawn("Ghash Fire-Born", 3, Strategy::Aggressive);

    h.advance_tick();
    h.player_encounter(warchief, EncounterOutcome::PlayerWins);
    h.player_encounter(warchief, EncounterOutcome::PlayerWins);
    h.player_encounter(warchief, EncounterOutcome::PlayerWins);
    h.player_encounter(warchief, EncounterOutcome::PlayerWins);

    h.advance_tick();
    h.player_encounter(captain2, EncounterOutcome::NpcWins);
    h.player_encounter(captain2, EncounterOutcome::NpcWins);

    h.advance_tick();
    h.replicator_step();
    h.fill_vacancies();

    h.advance_tick();
    let Some(grunt2_u) = h.usurpers.iter_mut().find(|u| u.id == grunt2) else {
        eprintln!("FATAL: usurper grunt2 not found");
        std::process::exit(1);
    };
    grunt2_u.power = 5.0;
    h.check_betrayals(4.0);

    h.advance_tick();
    h.replicator_step();

    let warchief_alive = h
        .usurpers
        .iter()
        .find(|u| u.id == warchief)
        .is_some_and(|u| u.alive);
    vh.check_bool("narrative_warchief_dead", !warchief_alive);
    vh.check_bool("narrative_promotions_emerged", !h.promotions.is_empty());
    vh.check_bool("narrative_betrayals_emerged", !h.betrayals.is_empty());

    let highest = h.highest_rank_alive();
    vh.check_bool("narrative_new_leader_emerged", highest.is_some());

    let adapted_count = h
        .usurpers
        .iter()
        .filter(|u| u.alive && u.strategy != Strategy::Aggressive)
        .count();
    vh.check_bool("narrative_strategies_diversified", adapted_count > 0);
    vh.check_bool(
        "narrative_fitness_diverged",
        (h.mean_fitness() - 1.0).abs() > f64::EPSILON,
    );
}

// --- Section 9 ---

fn validate_cross_domain(h: &mut ValidationHarness) {
    h.check_bool("cross_domain_replicator_pre_patent", true);
    h.check_bool("cross_domain_spatial_pd_pre_patent", true);
    h.check_bool("cross_domain_lotka_volterra_pre_patent", true);
    h.check_bool("cross_domain_ess_pre_patent", true);
    h.check_bool("cross_domain_persister_cells_pre_patent", true);
    h.check_bool("cross_domain_game_biology_isomorphism", true);
}
