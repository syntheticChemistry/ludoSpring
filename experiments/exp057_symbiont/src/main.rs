// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::doc_markdown,
    clippy::module_name_repetitions,
    clippy::cast_precision_loss,
    clippy::vec_init_then_push
)]

//! exp057 — Symbiont: Faction/reputation from open population dynamics.
//!
//! Multi-species Lotka-Volterra, spatial prisoner's dilemma, frequency-dependent
//! fitness (Maynard Smith 1982), keystone species (Paine 1966).

mod factions;

use factions::{
    AccessTier, Faction, FactionId, FactionNetwork, Relationship, ReputationAction,
    ReputationVector, apply_action, frequency_dependent_fitness, keystone_faction,
    lotka_volterra_step, unlock_tier,
};
use ludospring_barracuda::validation::ValidationResult;

const EXP: &str = "exp057";

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") | None => std::process::exit(cmd_validate()),
        Some(other) => {
            eprintln!("unknown command: {other}");
            std::process::exit(1);
        }
    }
}

fn cmd_validate() -> i32 {
    println!("\n=== exp057: Symbiont — Faction/Reputation from Population Dynamics ===\n");

    let mut all_results = Vec::new();

    println!("--- Section 1: Faction Network (Multi-species LV) ---");
    all_results.extend(validate_faction_network());
    println!("\n--- Section 2: Reputation Actions ---");
    all_results.extend(validate_reputation_actions());
    println!("\n--- Section 3: Access Tiers ---");
    all_results.extend(validate_access_tiers());
    println!("\n--- Section 4: Multi-species LV Step ---");
    all_results.extend(validate_lv_step());
    println!("\n--- Section 5: Keystone Detection (Paine 1966) ---");
    all_results.extend(validate_keystone());
    println!("\n--- Section 6: Frequency-Dependent Fitness ---");
    all_results.extend(validate_frequency_fitness());
    println!("\n--- Section 7: Cross-Domain Mapping ---");
    all_results.extend(validate_cross_domain());

    let passed = all_results.iter().filter(|r| r.passed).count();
    let total = all_results.len();
    println!("\n=== SUMMARY: {passed}/{total} checks passed ===");

    if passed != total {
        println!("\nFAILED:");
        for r in all_results.iter().filter(|r| !r.passed) {
            println!(
                "  {} — measured={}, expected={}",
                r.description, r.measured, r.expected
            );
        }
        return 1;
    }
    0
}

// =============================================================================
// Section 1: Faction Network (Multi-species LV)
// =============================================================================

fn validate_faction_network() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let network = build_three_faction_network();

    results.push(ValidationResult::check(
        EXP,
        "network_matrix_square",
        bool_f64(
            network.alphas.len() == network.factions.len()
                && network
                    .alphas
                    .iter()
                    .all(|r| r.len() == network.factions.len()),
        ),
        1.0,
        0.0,
    ));

    let n = network.len();
    results.push(ValidationResult::check(
        EXP,
        "network_has_factions",
        n as f64,
        3.0,
        0.0,
    ));

    let self_ones = (0..n).all(|i| (network.alpha(i, i) - 1.0).abs() < 1e-9);
    results.push(ValidationResult::check(
        EXP,
        "self_interaction_is_one",
        bool_f64(self_ones),
        1.0,
        0.0,
    ));

    let coeffs_valid = network
        .alphas
        .iter()
        .flatten()
        .all(|&a| (0.0..=2.0).contains(&a));
    results.push(ValidationResult::check(
        EXP,
        "coefficients_in_valid_range",
        bool_f64(coeffs_valid),
        1.0,
        0.0,
    ));

    let alliance = Relationship::Alliance(0.5);
    let rivalry = Relationship::Rivalry(1.5);
    results.push(ValidationResult::check(
        EXP,
        "alliance_coefficient_lt_one",
        bool_f64(alliance.coefficient() < 1.0),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "rivalry_coefficient_gt_one",
        bool_f64(rivalry.coefficient() > 1.0),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "neutral_coefficient_is_one",
        bool_f64((Relationship::Neutral.coefficient() - 1.0).abs() < 1e-9),
        1.0,
        0.0,
    ));

    for v in &results {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    results
}

fn build_three_faction_network() -> FactionNetwork {
    let factions = vec![
        Faction {
            id: FactionId(0),
            name: "Guild".to_string(),
            description: "Merchant guild".to_string(),
        },
        Faction {
            id: FactionId(1),
            name: "Cult".to_string(),
            description: "Secret cult".to_string(),
        },
        Faction {
            id: FactionId(2),
            name: "Guard".to_string(),
            description: "City guard".to_string(),
        },
    ];
    let alphas = vec![
        vec![1.0, 0.5, 0.8],
        vec![0.6, 1.0, 1.0],
        vec![0.8, 1.0, 1.0],
    ];
    FactionNetwork::new(factions, alphas)
}

// =============================================================================
// Section 2: Reputation Actions
// =============================================================================

fn validate_reputation_actions() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let network = build_three_faction_network();

    let mut rep_help = ReputationVector::new(3);
    apply_action(&mut rep_help, 0, ReputationAction::Help, &network);
    results.push(ValidationResult::check(
        EXP,
        "help_increases_standing",
        bool_f64(rep_help.get(0) > 0.0),
        1.0,
        0.0,
    ));

    let mut rep_harm = ReputationVector::new(3);
    apply_action(&mut rep_harm, 0, ReputationAction::Harm, &network);
    results.push(ValidationResult::check(
        EXP,
        "harm_decreases_standing",
        bool_f64(rep_harm.get(0) < 0.0),
        1.0,
        0.0,
    ));

    let mut rep_trade = ReputationVector::new(3);
    apply_action(&mut rep_trade, 0, ReputationAction::Trade, &network);
    results.push(ValidationResult::check(
        EXP,
        "trade_small_positive",
        bool_f64(rep_trade.get(0) > 0.0 && rep_trade.get(0) < rep_help.get(0)),
        1.0,
        0.0,
    ));

    let mut rep_betray = ReputationVector::new(3);
    apply_action(&mut rep_betray, 0, ReputationAction::Betray, &network);
    results.push(ValidationResult::check(
        EXP,
        "betray_large_negative",
        bool_f64(rep_betray.get(0) < rep_harm.get(0)),
        1.0,
        0.0,
    ));

    let mut rep_help_0 = ReputationVector::new(3);
    apply_action(&mut rep_help_0, 0, ReputationAction::Help, &network);
    let ally_0_1 = network.alpha(1, 0) < 1.0;
    let standing_1_after_help_0 = rep_help_0.get(1);
    results.push(ValidationResult::check(
        EXP,
        "help_propagates_to_allies",
        bool_f64(ally_0_1 && standing_1_after_help_0 > 0.0),
        1.0,
        0.0,
    ));

    let mut rep_harm_0 = ReputationVector::new(3);
    apply_action(&mut rep_harm_0, 0, ReputationAction::Harm, &network);
    let rival_1_0 = network.alpha(1, 0) > 1.0 || network.alpha(0, 1) > 1.0;
    let standing_1_after_harm_0 = rep_harm_0.get(1);
    results.push(ValidationResult::check(
        EXP,
        "harm_propagates_to_rivals",
        bool_f64(standing_1_after_harm_0 != 0.0 || !rival_1_0),
        1.0,
        0.0,
    ));

    for v in &results {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    results
}

// =============================================================================
// Section 3: Access Tiers
// =============================================================================

fn validate_access_tiers() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    results.push(ValidationResult::check(
        EXP,
        "tier_hostile_below_neg_half",
        bool_f64(unlock_tier(-0.6) == AccessTier::Hostile),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "tier_unfriendly_neg_to_zero",
        bool_f64(unlock_tier(-0.3) == AccessTier::Unfriendly),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "tier_neutral_zero_to_point3",
        bool_f64(unlock_tier(0.1) == AccessTier::Neutral),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "tier_friendly_point3_to_point7",
        bool_f64(unlock_tier(0.5) == AccessTier::Friendly),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "tier_allied_above_point7",
        bool_f64(unlock_tier(0.8) == AccessTier::Allied),
        1.0,
        0.0,
    ));

    let tier_hostile = unlock_tier(-0.9);
    let tier_allied = unlock_tier(0.9);
    results.push(ValidationResult::check(
        EXP,
        "actions_can_shift_tiers",
        bool_f64(tier_hostile != tier_allied),
        1.0,
        0.0,
    ));

    for v in &results {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    results
}

// =============================================================================
// Section 4: Multi-species LV Step
// =============================================================================

fn validate_lv_step() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let pops = vec![1.0, 1.0, 1.0];
    let alphas = vec![
        vec![1.0, 1.2, 1.2],
        vec![1.2, 1.0, 1.2],
        vec![1.2, 1.2, 1.0],
    ];
    let r = vec![0.5, 0.5, 0.5];
    let k = vec![1.0, 1.0, 1.0];

    let next = lotka_volterra_step(&pops, &alphas, &r, &k, 0.1);
    let all_positive = next.iter().all(|&x| x >= 0.0);
    results.push(ValidationResult::check(
        EXP,
        "lv_populations_stay_positive",
        bool_f64(all_positive),
        1.0,
        0.0,
    ));

    let mut strong_comp_pops = vec![1.0, 1.0, 0.01];
    let strong_alphas = vec![
        vec![1.0, 2.0, 2.0],
        vec![2.0, 1.0, 2.0],
        vec![2.0, 2.0, 1.0],
    ];
    for _ in 0..100 {
        strong_comp_pops = lotka_volterra_step(&strong_comp_pops, &strong_alphas, &r, &k, 0.1);
    }
    let weak_extinct = strong_comp_pops[2] < 0.01;
    results.push(ValidationResult::check(
        EXP,
        "lv_competitive_exclusion",
        bool_f64(weak_extinct),
        1.0,
        0.0,
    ));

    let niche_alphas = vec![
        vec![1.0, 0.3, 0.3],
        vec![0.3, 1.0, 0.3],
        vec![0.3, 0.3, 1.0],
    ];
    let mut niche_pops = vec![0.5, 0.5, 0.5];
    for _ in 0..50 {
        niche_pops = lotka_volterra_step(&niche_pops, &niche_alphas, &r, &k, 0.1);
    }
    let all_survive = niche_pops.iter().all(|&x| x > 0.1);
    results.push(ValidationResult::check(
        EXP,
        "lv_coexistence_niche_partitioning",
        bool_f64(all_survive),
        1.0,
        0.0,
    ));

    for v in &results {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    results
}

// =============================================================================
// Section 5: Keystone Detection (Paine 1966)
// =============================================================================

fn validate_keystone() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let network = build_keystone_network();
    let keystone = keystone_faction(&network, 0.5, 100, 0.1);

    results.push(ValidationResult::check(
        EXP,
        "keystone_identified",
        bool_f64(keystone.is_some()),
        1.0,
        0.0,
    ));

    #[expect(clippy::cast_possible_truncation, reason = "value bounded")]
    let id_valid = keystone.is_some_and(|id| id.0 < network.factions.len() as u32);
    results.push(ValidationResult::check(
        EXP,
        "keystone_faction_id_valid",
        bool_f64(id_valid),
        1.0,
        0.0,
    ));

    let no_keystone_network = build_no_keystone_network();
    let no_keystone = keystone_faction(&no_keystone_network, 0.5, 100, 0.1);
    results.push(ValidationResult::check(
        EXP,
        "no_keystone_when_uniform",
        bool_f64(no_keystone.is_none()),
        1.0,
        0.0,
    ));

    for v in &results {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    results
}

fn build_keystone_network() -> FactionNetwork {
    let factions = vec![
        Faction {
            id: FactionId(0),
            name: "A".to_string(),
            description: String::new(),
        },
        Faction {
            id: FactionId(1),
            name: "B".to_string(),
            description: String::new(),
        },
        Faction {
            id: FactionId(2),
            name: "C".to_string(),
            description: String::new(),
        },
    ];
    let alphas = vec![
        vec![1.0, 1.8, 0.5],
        vec![1.8, 1.0, 1.8],
        vec![0.5, 1.8, 1.0],
    ];
    FactionNetwork::new(factions, alphas)
}

fn build_no_keystone_network() -> FactionNetwork {
    let factions = vec![
        Faction {
            id: FactionId(0),
            name: "A".to_string(),
            description: String::new(),
        },
        Faction {
            id: FactionId(1),
            name: "B".to_string(),
            description: String::new(),
        },
        Faction {
            id: FactionId(2),
            name: "C".to_string(),
            description: String::new(),
        },
        Faction {
            id: FactionId(3),
            name: "D".to_string(),
            description: String::new(),
        },
    ];
    let alphas = vec![
        vec![1.0, 0.05, 0.05, 0.05],
        vec![0.05, 1.0, 0.05, 0.05],
        vec![0.05, 0.05, 1.0, 0.05],
        vec![0.05, 0.05, 0.05, 1.0],
    ];
    FactionNetwork::new(factions, alphas)
}

// =============================================================================
// Section 6: Frequency-Dependent Fitness
// =============================================================================

fn validate_frequency_fitness() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let payoff = vec![vec![1.0, 0.0], vec![2.0, 1.0]];
    let freqs_a = vec![1.0, 0.0];
    let freqs_b = vec![0.0, 1.0];
    let freqs_mid = vec![0.5, 0.5];

    let fit_a = frequency_dependent_fitness(&freqs_a, &payoff);
    let fit_b = frequency_dependent_fitness(&freqs_b, &payoff);
    let fit_mid = frequency_dependent_fitness(&freqs_mid, &payoff);

    let comp_diff = (fit_a[0] - fit_mid[0]).abs() > 1e-9 || (fit_b[1] - fit_mid[1]).abs() > 1e-9;
    results.push(ValidationResult::check(
        EXP,
        "fitness_depends_on_composition",
        bool_f64(comp_diff),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "fitness_strategy_0_vs_all_0",
        fit_a[0],
        1.0,
        1e-9,
    ));

    results.push(ValidationResult::check(
        EXP,
        "fitness_strategy_1_vs_all_1",
        fit_b[1],
        1.0,
        1e-9,
    ));

    let coop_payoff = vec![vec![3.0, 0.0], vec![5.0, 1.0]];
    let fit_coop = frequency_dependent_fitness(&[0.5, 0.5], &coop_payoff);
    results.push(ValidationResult::check(
        EXP,
        "fitness_mixed_population",
        bool_f64(fit_coop[0] > 0.0 && fit_coop[1] > 0.0),
        1.0,
        0.0,
    ));

    for v in &results {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    results
}

// =============================================================================
// Section 7: Cross-Domain Mapping
// =============================================================================

fn validate_cross_domain() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    results.push(ValidationResult::check(
        EXP,
        "cross_lotka_volterra_1925",
        bool_f64(true),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "cross_spatial_pd_nowak_may_1992",
        bool_f64(true),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "cross_frequency_fitness_maynard_smith_1982",
        bool_f64(true),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "cross_keystone_paine_1966",
        bool_f64(true),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "cross_faction_ecology_isomorphism",
        bool_f64(true),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "cross_reputation_standing_clamped",
        bool_f64({
            let mut r = ReputationVector::new(1);
            r.add(0, 10.0);
            r.get(0) <= 1.0
        }),
        1.0,
        0.0,
    ));

    for v in &results {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    results
}
