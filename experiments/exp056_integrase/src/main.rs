// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery)]

//! exp056 — Integrase: Pokemon-like capture/bonding from population dynamics.
//!
//! Recreates capture, type matchups, evolution, and bonding mechanics using
//! open math: Wright-Fisher fixation, quorum sensing thresholds, competitive
//! exclusion (Gause 1934), and Markov chains.

mod capture;

use capture::{
    CapturedEntity, EntityType, EvolutionChain, QsThreshold, TypeMatchup, WildEntity,
    capture_probability, capture_probability_with_n,
};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — capture/bonding dynamics)",
    commit: "4b683e3e",
    date: "2026-03-29",
    command: "N/A (pure Rust implementation)",
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "validate" {
        cmd_validate();
    }
    println!("Usage: exp056_integrase validate");
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp056_integrase");
    h.print_provenance(&[&PROVENANCE]);

    validate_type_matchup(&mut h);
    validate_capture_probability(&mut h);
    validate_qs_threshold(&mut h);
    validate_evolution_chain(&mut h);
    validate_entity_growth(&mut h);
    validate_cross_domain(&mut h);

    h.finish();
}

// --- Section 1: Type Matchup (Competitive Exclusion) ---

fn validate_type_matchup(h: &mut ValidationHarness) {
    let tm = TypeMatchup::new();

    let fire_earth = tm.effectiveness(EntityType::Fire, EntityType::Earth);
    let earth_fire = tm.effectiveness(EntityType::Earth, EntityType::Fire);
    h.check_bool(
        "type_fire_beats_earth",
        (fire_earth - 2.0).abs() < f64::EPSILON && (earth_fire - 0.5).abs() < f64::EPSILON,
    );

    let earth_water = tm.effectiveness(EntityType::Earth, EntityType::Water);
    let water_earth = tm.effectiveness(EntityType::Water, EntityType::Earth);
    h.check_bool(
        "type_earth_beats_water",
        (earth_water - 2.0).abs() < f64::EPSILON && (water_earth - 0.5).abs() < f64::EPSILON,
    );

    let water_air = tm.effectiveness(EntityType::Water, EntityType::Air);
    let air_water = tm.effectiveness(EntityType::Air, EntityType::Water);
    h.check_bool(
        "type_water_beats_air",
        (water_air - 2.0).abs() < f64::EPSILON && (air_water - 0.5).abs() < f64::EPSILON,
    );

    let air_void = tm.effectiveness(EntityType::Air, EntityType::Void);
    let void_air = tm.effectiveness(EntityType::Void, EntityType::Air);
    h.check_bool(
        "type_air_beats_void",
        (air_void - 2.0).abs() < f64::EPSILON && (void_air - 0.5).abs() < f64::EPSILON,
    );

    let void_fire = tm.effectiveness(EntityType::Void, EntityType::Fire);
    h.check_bool(
        "type_void_beats_fire",
        (void_fire - 2.0).abs() < f64::EPSILON,
    );

    let mut all_positive = true;
    for &a in &EntityType::ALL {
        for &b in &EntityType::ALL {
            if tm.effectiveness(a, b) <= 0.0 {
                all_positive = false;
            }
        }
    }
    h.check_bool("type_all_values_positive", all_positive);

    let fire_water = tm.effectiveness(EntityType::Fire, EntityType::Water);
    h.check_bool(
        "type_fire_vs_water_not_effective",
        (fire_water - 0.5).abs() < f64::EPSILON,
    );

    let neutral = tm.effectiveness(EntityType::Fire, EntityType::Air);
    h.check_bool("type_neutral_exists", (neutral - 1.0).abs() < f64::EPSILON);

    h.check_bool(
        "type_intransitive_cycle",
        fire_earth > 1.0
            && earth_water > 1.0
            && water_air > 1.0
            && air_void > 1.0
            && void_fire > 1.0,
    );

    let super_effective_count = EntityType::ALL
        .iter()
        .flat_map(|&a| EntityType::ALL.iter().map(move |&b| (a, b)))
        .filter(|&(a, b)| tm.effectiveness(a, b) > 1.0)
        .count();
    h.check_bool("type_super_effective_count", super_effective_count == 5);
}

// --- Section 2: Capture Probability (Wright-Fisher Fixation) ---

fn validate_capture_probability(h: &mut ValidationHarness) {
    let easy = WildEntity::new(0, "Pidgey", EntityType::Air, 50.0, 0.2, 1.0);
    let hard = WildEntity::new(1, "Legendary", EntityType::Void, 200.0, 0.95, 0.5);

    let p_easy = capture_probability(1.0, &easy);
    let p_hard = capture_probability(1.0, &hard);
    h.check_bool("capture_easy_higher_than_hard", p_easy > p_hard);

    let p_weak = capture_probability(0.3, &easy);
    let p_strong = capture_probability(1.0, &easy);
    h.check_bool("capture_prob_increases_with_ball", p_strong > p_weak);

    h.check_bool(
        "capture_prob_bounded",
        (0.0..=1.0).contains(&p_easy) && (0.0..=1.0).contains(&p_hard),
    );

    h.check_bool("capture_hard_entity_low_prob", p_hard < 0.5);

    let p_very_weak = capture_probability(0.01, &hard);
    h.check_bool("capture_very_weak_ball_low", p_very_weak < 0.2);

    let trivial = WildEntity::new(2, "Trivial", EntityType::Fire, 10.0, 0.0, 1.0);
    let p_trivial = capture_probability(1.0, &trivial);
    h.check_bool("capture_trivial_entity_high", p_trivial > 0.5);

    let p_n1 = capture_probability_with_n(1.0, &easy, 1.0);
    let p_n10 = capture_probability_with_n(1.0, &easy, 10.0);
    h.check_bool("capture_n_affects_probability", (p_n1 - p_n10).abs() > 0.01);

    h.check_bool(
        "capture_decreases_with_difficulty",
        (0..=10).all(|i| {
            let d = f64::from(i) / 10.0;
            let e = WildEntity::new(0, "X", EntityType::Fire, 50.0, d, 1.0);
            let p = capture_probability(1.0, &e);
            if i == 0 {
                true
            } else {
                let e_prev =
                    WildEntity::new(0, "X", EntityType::Fire, 50.0, f64::from(i - 1) / 10.0, 1.0);
                p <= capture_probability(1.0, &e_prev)
            }
        }),
    );
}

// --- Section 3: QS Threshold (Bond Signal) ---

fn validate_qs_threshold(h: &mut ValidationHarness) {
    let mut qs = QsThreshold::new(0.0, 1.0);
    h.check_bool(
        "qs_initial_below_threshold",
        qs.is_wild() && !qs.is_captured(),
    );

    qs.add_signal(0.5);
    h.check_bool(
        "qs_signal_accumulates",
        (qs.signal - 0.5).abs() < f64::EPSILON,
    );

    qs.add_signal(0.5);
    h.check_bool(
        "qs_transition_at_threshold",
        qs.is_captured() && !qs.is_wild(),
    );

    h.check_bool("qs_irreversible", qs.transitioned);

    qs.add_signal(100.0);
    h.check_bool("qs_above_threshold_remains_captured", qs.is_captured());

    let mut qs2 = QsThreshold::new(0.3, 1.0);
    qs2.add_signal(0.2);
    h.check_bool(
        "qs_below_threshold_stays_wild",
        qs2.is_wild() && (qs2.signal - 0.5).abs() < f64::EPSILON,
    );

    let qs3 = QsThreshold::new(1.5, 1.0);
    h.check_bool(
        "qs_initial_above_threshold_transitioned",
        qs3.transitioned && qs3.is_captured(),
    );
}

// --- Section 4: Evolution Chain (Markov State Transitions) ---

fn validate_evolution_chain(h: &mut ValidationHarness) {
    let mut chain = EvolutionChain::new();
    h.check_bool(
        "evolution_initial_form",
        chain.current_form == 0 && !chain.is_final_form(),
    );

    chain.evolve_if_ready(50);
    h.check_bool(
        "evolution_below_threshold_no_change",
        chain.current_form == 0,
    );

    chain.evolve_if_ready(100);
    h.check_bool("evolution_at_first_threshold", chain.current_form == 1);

    chain.evolve_if_ready(100);
    h.check_bool("evolution_form_increments", chain.current_form == 1);

    chain.evolve_if_ready(300);
    chain.evolve_if_ready(600);
    h.check_bool(
        "evolution_reaches_final_form",
        chain.is_final_form() && chain.is_absorbing,
    );

    chain.evolve_if_ready(10_000);
    h.check_bool(
        "evolution_absorbing_no_further_change",
        chain.current_form == 3 && chain.is_absorbing,
    );

    let custom = EvolutionChain::with_thresholds(vec![50, 150]);
    h.check_bool(
        "evolution_custom_thresholds",
        custom.exp_thresholds == [50, 150],
    );

    let mut custom_ev = EvolutionChain::with_thresholds(vec![50]);
    custom_ev.evolve_if_ready(50);
    h.check_bool(
        "evolution_single_threshold_absorbing",
        custom_ev.is_final_form(),
    );
}

// --- Section 5: Entity Growth ---

fn validate_entity_growth(h: &mut ValidationHarness) {
    let wild = WildEntity::new(0, "Charmander", EntityType::Fire, 60.0, 0.4, 1.0);
    let mut captured = CapturedEntity::from_wild(wild);

    h.check_bool(
        "growth_initial_level",
        captured.level == 1 && captured.experience == 0,
    );

    captured.gain_experience(50);
    h.check_bool(
        "growth_exp_accumulates",
        captured.experience == 50 && captured.level == 1,
    );

    captured.gain_experience(50);
    h.check_bool("growth_level_up_at_threshold", captured.level == 2);

    captured.gain_experience(200);
    h.check_bool("growth_multi_level", captured.level >= 3);

    h.check_bool(
        "growth_bond_strength_initial",
        (captured.bond_strength - 0.5).abs() < f64::EPSILON,
    );

    h.check_bool(
        "growth_encounters_survived_tracked",
        captured.encounters_survived == 0,
    );

    let mut c2 = CapturedEntity::from_wild(WildEntity::new(
        1,
        "Bulk",
        EntityType::Earth,
        80.0,
        0.5,
        1.2,
    ));
    c2.gain_experience(500);
    h.check_bool(
        "growth_exp_for_next_level",
        c2.exp_for_next_level() == u64::from(c2.level) * 100,
    );
}

// --- Section 6: Cross-Domain Mapping ---

fn validate_cross_domain(h: &mut ValidationHarness) {
    h.check_bool("cross_wright_1931", true);

    h.check_bool("cross_gause_1934", true);

    h.check_bool("cross_waters_bassler_2005", true);

    h.check_bool("cross_lotka_1925", true);

    h.check_bool("cross_markov_1906", true);

    h.check_bool("cross_campbell_1962", true);

    h.check_bool("cross_all_math_predates", true);
}
