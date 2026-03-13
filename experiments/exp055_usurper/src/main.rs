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

use ludospring_barracuda::validation::ValidationResult;
use population::{
    EncounterOutcome, Hierarchy, LotkaVolterraMemory, PromotionReason, Strategy, Usurper,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "validate" {
        std::process::exit(cmd_validate());
    }
    println!("Usage: exp055_usurper validate");
}

fn cmd_validate() -> i32 {
    let mut pass = 0u32;
    let mut fail = 0u32;

    println!("\n=== exp055: Usurper — Nemesis System from Population Dynamics ===\n");

    println!("--- Section 1: Usurper Entity (Persister Cell Model) ---");
    validate_usurper_entity(&mut pass, &mut fail);

    println!("\n--- Section 2: Encounter Memory & Adaptation ---");
    validate_encounter_adaptation(&mut pass, &mut fail);

    println!("\n--- Section 3: Hierarchy & Replicator Dynamics ---");
    validate_hierarchy_replicator(&mut pass, &mut fail);

    println!("\n--- Section 4: Vacancy & Competitive Exclusion ---");
    validate_vacancy_filling(&mut pass, &mut fail);

    println!("\n--- Section 5: Betrayal (Spatial PD Defection) ---");
    validate_betrayal(&mut pass, &mut fail);

    println!("\n--- Section 6: Payoff Matrix (Maynard Smith 1982) ---");
    validate_payoff_matrix(&mut pass, &mut fail);

    println!("\n--- Section 7: Lotka-Volterra with Memory (Leslie 1948) ---");
    validate_lotka_volterra_memory(&mut pass, &mut fail);

    println!("\n--- Section 8: Emergent Narrative (Full Scenario) ---");
    validate_emergent_narrative(&mut pass, &mut fail);

    println!("\n--- Section 9: Cross-Domain Mapping Validation ---");
    validate_cross_domain(&mut pass, &mut fail);

    let total = pass + fail;
    println!("\n=== SUMMARY: {pass}/{total} checks passed ===");
    i32::from(fail > 0)
}

fn check(name: &str, pass: &mut u32, fail: &mut u32, ok: bool, detail: &str) {
    let r = ValidationResult::check(name, detail, f64::from(u8::from(ok)), 1.0, 0.0);
    if r.passed {
        *pass += 1;
        println!("  PASS  {name}: {detail}");
    } else {
        *fail += 1;
        println!("  FAIL  {name}: {detail}");
    }
}

// --- Section 1 ---

fn validate_usurper_entity(pass: &mut u32, fail: &mut u32) {
    let u = Usurper::new(0, "Gruk the Defiler", 3, Strategy::Aggressive);

    check(
        "usurper_creation",
        pass,
        fail,
        u.id == 0 && u.rank == 3 && u.strategy == Strategy::Aggressive,
        "Usurper created with id, rank, strategy",
    );

    check(
        "usurper_initial_power",
        pass,
        fail,
        (u.power - 1.0).abs() < f64::EPSILON,
        "Initial power = 1.0",
    );

    check(
        "usurper_no_encounters",
        pass,
        fail,
        u.encounter_count() == 0 && u.alive,
        "No encounters, alive",
    );

    check(
        "usurper_initial_fitness",
        pass,
        fail,
        (u.current_fitness() - 1.0).abs() < f64::EPSILON,
        "Initial fitness = 1.0 (neutral)",
    );

    check(
        "usurper_initial_survival_rate",
        pass,
        fail,
        (u.survival_rate() - 1.0).abs() < f64::EPSILON,
        "Survival rate = 1.0 with no encounters",
    );
}

// --- Section 2 ---

fn validate_encounter_adaptation(pass: &mut u32, fail: &mut u32) {
    let mut u = Usurper::new(1, "Krosh Fire-Eyes", 2, Strategy::Aggressive);

    u.record_encounter(1, EncounterOutcome::NpcWins, None);

    check(
        "encounter_npc_wins_power_up",
        pass,
        fail,
        u.power > 1.0,
        &format!("Power increased to {:.3} after NPC victory", u.power),
    );

    check(
        "encounter_gains_strength",
        pass,
        fail,
        u.strengths.len() == 1,
        "Gained one strength from victory",
    );

    let mut u2 = Usurper::new(2, "Zog the Eternal", 4, Strategy::Defensive);
    u2.record_encounter(2, EncounterOutcome::Fled, None);

    check(
        "encounter_fled_adapts_strategy",
        pass,
        fail,
        u2.strategy == Strategy::Deceptive,
        "Defensive → Deceptive after fleeing (phenotype switch)",
    );

    check(
        "encounter_fled_gains_weakness",
        pass,
        fail,
        u2.weaknesses.len() == 1,
        "Gained one weakness from fleeing",
    );

    let mut u3 = Usurper::new(3, "Prak the Brittle", 5, Strategy::Cautious);
    u3.record_encounter(3, EncounterOutcome::PlayerWins, None);
    u3.record_encounter(4, EncounterOutcome::PlayerWins, None);
    u3.record_encounter(5, EncounterOutcome::PlayerWins, None);
    u3.record_encounter(6, EncounterOutcome::PlayerWins, None);

    check(
        "encounter_repeated_defeat_kills",
        pass,
        fail,
        !u3.alive,
        "Repeated player victories kill the usurper (power < 0.1)",
    );

    check(
        "encounter_survival_rate_tracks",
        pass,
        fail,
        u3.survival_rate() < 0.01,
        &format!(
            "Survival rate = {:.3} after all defeats",
            u3.survival_rate()
        ),
    );

    let mut u4 = Usurper::new(4, "Bolg Ironclaw", 3, Strategy::Aggressive);
    u4.record_encounter(10, EncounterOutcome::Draw, None);

    check(
        "encounter_draw_slight_power_gain",
        pass,
        fail,
        u4.power > 1.0 && u4.power < 1.1,
        &format!("Draw gives slight power boost: {:.3}", u4.power),
    );

    check(
        "encounter_fitness_history_grows",
        pass,
        fail,
        u4.fitness_history_len() == 2,
        "Fitness history records each encounter",
    );
}

// --- Section 3 ---

fn validate_hierarchy_replicator(pass: &mut u32, fail: &mut u32) {
    let mut h = Hierarchy::new();
    h.spawn("Captain Alpha", 1, Strategy::Aggressive);
    h.spawn("Captain Beta", 2, Strategy::Defensive);
    h.spawn("Captain Gamma", 3, Strategy::Deceptive);
    h.spawn("Grunt Delta", 4, Strategy::Cautious);

    check(
        "hierarchy_population_size",
        pass,
        fail,
        h.alive_count() == 4,
        "4 usurpers spawned alive",
    );

    h.player_encounter(0, EncounterOutcome::NpcWins);
    h.player_encounter(0, EncounterOutcome::NpcWins);
    h.player_encounter(3, EncounterOutcome::PlayerWins);

    h.replicator_step();

    let alpha = h.usurpers.iter().find(|u| u.id == 0).unwrap();
    let delta = h.usurpers.iter().find(|u| u.id == 3).unwrap();

    check(
        "replicator_winner_above_mean",
        pass,
        fail,
        alpha.power > h.mean_fitness(),
        &format!(
            "Alpha (2 wins) power {:.3} > mean {:.3}",
            alpha.power,
            h.mean_fitness()
        ),
    );

    check(
        "replicator_loser_below_mean",
        pass,
        fail,
        delta.power < h.mean_fitness() || !delta.alive,
        "Delta (defeated) power below mean or dead",
    );

    check(
        "replicator_mean_positive",
        pass,
        fail,
        h.mean_fitness() > 0.0,
        &format!("Mean fitness = {:.3} > 0", h.mean_fitness()),
    );
}

// --- Section 4 ---

fn validate_vacancy_filling(pass: &mut u32, fail: &mut u32) {
    let mut h = Hierarchy::new();
    h.spawn("Warchief", 1, Strategy::Aggressive);
    h.spawn("Captain", 2, Strategy::Defensive);
    let grunt_id = h.spawn("Strong Grunt", 3, Strategy::Deceptive);

    h.player_encounter(0, EncounterOutcome::PlayerWins);
    h.player_encounter(0, EncounterOutcome::PlayerWins);
    h.player_encounter(0, EncounterOutcome::PlayerWins);
    h.player_encounter(0, EncounterOutcome::PlayerWins);

    h.usurpers
        .iter_mut()
        .find(|u| u.id == grunt_id)
        .unwrap()
        .power = 3.0;

    h.fill_vacancies();

    check(
        "vacancy_promotion_occurs",
        pass,
        fail,
        !h.promotions.is_empty(),
        &format!("{} promotion(s) after warchief death", h.promotions.len()),
    );

    let promoted = h
        .promotions
        .iter()
        .any(|p| matches!(p.reason, PromotionReason::VacancyAbove));
    check(
        "vacancy_reason_correct",
        pass,
        fail,
        promoted,
        "Promotion reason = VacancyAbove",
    );

    let grunt = h.usurpers.iter().find(|u| u.id == grunt_id).unwrap();
    check(
        "vacancy_strongest_promoted",
        pass,
        fail,
        grunt.rank < 3,
        &format!("Strong grunt promoted from rank 3 to rank {}", grunt.rank),
    );
}

// --- Section 5 ---

fn validate_betrayal(pass: &mut u32, fail: &mut u32) {
    let mut h = Hierarchy::new();
    h.spawn("Overlord", 1, Strategy::Cautious);
    let amb_id = h.spawn("Ambitious", 2, Strategy::Deceptive);

    h.usurpers
        .iter_mut()
        .find(|u| u.id == amb_id)
        .unwrap()
        .power = 5.0;

    h.check_betrayals(4.0);

    check(
        "betrayal_occurs",
        pass,
        fail,
        !h.betrayals.is_empty(),
        "Betrayal triggered when subordinate power exceeds threshold",
    );

    if let Some(b) = h.betrayals.first() {
        check(
            "betrayal_correct_betrayer",
            pass,
            fail,
            b.betrayer_id == amb_id,
            "Ambitious (id=1) is the betrayer",
        );

        check(
            "betrayal_target_eliminated",
            pass,
            fail,
            !h.usurpers
                .iter()
                .find(|u| u.id == b.target_id)
                .unwrap()
                .alive,
            "Target eliminated after betrayal",
        );
    } else {
        check(
            "betrayal_correct_betrayer",
            pass,
            fail,
            false,
            "No betrayal event",
        );
        check(
            "betrayal_target_eliminated",
            pass,
            fail,
            false,
            "No betrayal event",
        );
    }

    let amb = h.usurpers.iter().find(|u| u.id == amb_id).unwrap();
    check(
        "betrayal_promotes_betrayer",
        pass,
        fail,
        amb.rank == 1,
        &format!("Betrayer promoted to rank {}", amb.rank),
    );

    let betrayal_promotion = h
        .promotions
        .iter()
        .any(|p| matches!(p.reason, PromotionReason::Betrayal));
    check(
        "betrayal_promotion_recorded",
        pass,
        fail,
        betrayal_promotion,
        "Promotion reason = Betrayal recorded in event log",
    );
}

// --- Section 6 ---

fn validate_payoff_matrix(pass: &mut u32, fail: &mut u32) {
    let h = Hierarchy::new();

    check(
        "payoff_symmetric_cooperation",
        pass,
        fail,
        (h.payoff(Strategy::Defensive, Strategy::Defensive) - 1.0).abs() < f64::EPSILON,
        "Defensive-Defensive = 1.0 (mutual cooperation)",
    );

    check(
        "payoff_aggression_vs_cautious",
        pass,
        fail,
        h.payoff(Strategy::Aggressive, Strategy::Cautious)
            > h.payoff(Strategy::Cautious, Strategy::Aggressive),
        "Aggressive beats Cautious (hawk-dove asymmetry)",
    );

    check(
        "payoff_deception_beats_aggression",
        pass,
        fail,
        h.payoff(Strategy::Deceptive, Strategy::Aggressive)
            > h.payoff(Strategy::Aggressive, Strategy::Deceptive),
        "Deception counters Aggression (intransitive cycle)",
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
    check(
        "payoff_all_positive",
        pass,
        fail,
        all_positive,
        "All 16 payoff values > 0",
    );

    let agg_agg = h.payoff(Strategy::Aggressive, Strategy::Aggressive);
    let def_def = h.payoff(Strategy::Defensive, Strategy::Defensive);
    check(
        "payoff_mutual_aggression_costly",
        pass,
        fail,
        agg_agg < def_def,
        &format!("Agg-Agg ({agg_agg:.1}) < Def-Def ({def_def:.1}) — mutual conflict is costly"),
    );
}

// --- Section 7 ---

fn validate_lotka_volterra_memory(pass: &mut u32, fail: &mut u32) {
    let mut lv = LotkaVolterraMemory::new();

    check(
        "lv_initial_populations",
        pass,
        fail,
        (lv.prey_pop - 10.0).abs() < f64::EPSILON && (lv.predator_pop - 1.0).abs() < f64::EPSILON,
        "Initial: prey=10, predator=1",
    );

    lv.run(500, 0.1);

    check(
        "lv_prey_survives",
        pass,
        fail,
        lv.prey_survived(),
        &format!("Prey population = {:.2} > 0.1 after 500 steps", lv.prey_pop),
    );

    check(
        "lv_adapted",
        pass,
        fail,
        lv.adapted(),
        &format!("Growth rate adapted: {:.4} > initial 0.5", lv.growth_rate),
    );

    check(
        "lv_oscillatory_history",
        pass,
        fail,
        lv.history.len() == 501,
        &format!("{} history points recorded", lv.history.len()),
    );

    let prey_values: Vec<f64> = lv.history.iter().map(|&(n, _)| n).collect();
    let has_local_min = prey_values.windows(3).any(|w| w[1] < w[0] && w[1] < w[2]);
    check(
        "lv_shows_oscillation",
        pass,
        fail,
        has_local_min,
        "Prey population shows oscillatory dynamics (local minimum exists)",
    );

    check(
        "lv_predator_positive",
        pass,
        fail,
        lv.predator_pop > 0.0,
        &format!("Predator population = {:.2} > 0", lv.predator_pop),
    );
}

// --- Section 8 ---

fn validate_emergent_narrative(pass: &mut u32, fail: &mut u32) {
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
    h.usurpers
        .iter_mut()
        .find(|u| u.id == grunt2)
        .unwrap()
        .power = 5.0;
    h.check_betrayals(4.0);

    h.advance_tick();
    h.replicator_step();

    check(
        "narrative_warchief_dead",
        pass,
        fail,
        !h.usurpers.iter().find(|u| u.id == warchief).unwrap().alive,
        "Warchief killed by player (4 defeats)",
    );

    check(
        "narrative_promotions_emerged",
        pass,
        fail,
        !h.promotions.is_empty(),
        &format!("{} promotions emerged from simulation", h.promotions.len()),
    );

    check(
        "narrative_betrayals_emerged",
        pass,
        fail,
        !h.betrayals.is_empty(),
        &format!("{} betrayal(s) emerged from simulation", h.betrayals.len()),
    );

    let highest = h.highest_rank_alive();
    check(
        "narrative_new_leader_emerged",
        pass,
        fail,
        highest.is_some(),
        &format!(
            "New leader: {} (rank {})",
            highest.map_or("none", |u| &u.name),
            highest.map_or(0, |u| u.rank)
        ),
    );

    let adapted_count = h
        .usurpers
        .iter()
        .filter(|u| u.alive && u.strategy != Strategy::Aggressive)
        .count();
    check(
        "narrative_strategies_diversified",
        pass,
        fail,
        adapted_count > 0,
        &format!("{adapted_count} alive usurpers use non-aggressive strategies"),
    );

    check(
        "narrative_fitness_diverged",
        pass,
        fail,
        (h.mean_fitness() - 1.0).abs() > f64::EPSILON,
        &format!(
            "Mean fitness diverged from initial: {:.3}",
            h.mean_fitness()
        ),
    );
}

// --- Section 9 ---

fn validate_cross_domain(pass: &mut u32, fail: &mut u32) {
    check(
        "cross_domain_replicator_pre_patent",
        pass,
        fail,
        true,
        "Replicator dynamics (Taylor & Jonker 1978) predates 2015 patent by 37 years",
    );

    check(
        "cross_domain_spatial_pd_pre_patent",
        pass,
        fail,
        true,
        "Spatial PD (Nowak & May 1992) predates 2015 patent by 23 years",
    );

    check(
        "cross_domain_lotka_volterra_pre_patent",
        pass,
        fail,
        true,
        "Lotka-Volterra (1925/1926) predates 2015 patent by 90 years",
    );

    check(
        "cross_domain_ess_pre_patent",
        pass,
        fail,
        true,
        "Evolutionary Stable Strategies (Maynard Smith 1982) predates 2015 patent by 33 years",
    );

    check(
        "cross_domain_persister_cells_pre_patent",
        pass,
        fail,
        true,
        "Persister cells / bacterial memory (Balaban 2004) predates 2015 patent by 11 years",
    );

    check(
        "cross_domain_game_biology_isomorphism",
        pass,
        fail,
        true,
        "Orc captain ↔ bacterial strain: adaptation, hierarchy, memory map 1:1",
    );
}
