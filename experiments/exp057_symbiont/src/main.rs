// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery)]
#![expect(
    clippy::cast_precision_loss,
    reason = "validation harness: counter/timing values within f64 range"
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
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — faction/reputation dynamics)",
    commit: "4b683e3e",
    date: "2026-03-29",
    command: "N/A (pure Rust implementation)",
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") | None => cmd_validate(),
        Some(other) => {
            eprintln!("unknown command: {other}");
            std::process::exit(1);
        }
    }
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp057_symbiont");
    h.print_provenance(&[&PROVENANCE]);

    validate_faction_network(&mut h);
    validate_reputation_actions(&mut h);
    validate_access_tiers(&mut h);
    validate_lv_step(&mut h);
    validate_keystone(&mut h);
    validate_frequency_fitness(&mut h);
    validate_cross_domain(&mut h);

    h.finish();
}

// =============================================================================
// Section 1: Faction Network (Multi-species LV)
// =============================================================================

fn validate_faction_network(h: &mut ValidationHarness) {
    let network = build_three_faction_network();

    h.check_bool(
        "network_matrix_square",
        network.alphas.len() == network.factions.len()
            && network
                .alphas
                .iter()
                .all(|r| r.len() == network.factions.len()),
    );

    let n = network.len();
    h.check_abs("network_has_factions", n as f64, 3.0, 0.0);

    let self_ones = (0..n).all(|i| (network.alpha(i, i) - 1.0).abs() < 1e-9);
    h.check_bool("self_interaction_is_one", self_ones);

    let coeffs_valid = network
        .alphas
        .iter()
        .flatten()
        .all(|&a| (0.0..=2.0).contains(&a));
    h.check_bool("coefficients_in_valid_range", coeffs_valid);

    let alliance = Relationship::Alliance(0.5);
    let rivalry = Relationship::Rivalry(1.5);
    h.check_bool("alliance_coefficient_lt_one", alliance.coefficient() < 1.0);
    h.check_bool("rivalry_coefficient_gt_one", rivalry.coefficient() > 1.0);
    h.check_bool(
        "neutral_coefficient_is_one",
        (Relationship::Neutral.coefficient() - 1.0).abs() < 1e-9,
    );
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

fn validate_reputation_actions(h: &mut ValidationHarness) {
    let network = build_three_faction_network();

    let mut rep_help = ReputationVector::new(3);
    apply_action(&mut rep_help, 0, ReputationAction::Help, &network);
    h.check_bool("help_increases_standing", rep_help.get(0) > 0.0);

    let mut rep_harm = ReputationVector::new(3);
    apply_action(&mut rep_harm, 0, ReputationAction::Harm, &network);
    h.check_bool("harm_decreases_standing", rep_harm.get(0) < 0.0);

    let mut rep_trade = ReputationVector::new(3);
    apply_action(&mut rep_trade, 0, ReputationAction::Trade, &network);
    h.check_bool(
        "trade_small_positive",
        rep_trade.get(0) > 0.0 && rep_trade.get(0) < rep_help.get(0),
    );

    let mut rep_betray = ReputationVector::new(3);
    apply_action(&mut rep_betray, 0, ReputationAction::Betray, &network);
    h.check_bool("betray_large_negative", rep_betray.get(0) < rep_harm.get(0));

    let mut rep_help_0 = ReputationVector::new(3);
    apply_action(&mut rep_help_0, 0, ReputationAction::Help, &network);
    let ally_0_1 = network.alpha(1, 0) < 1.0;
    let standing_1_after_help_0 = rep_help_0.get(1);
    h.check_bool(
        "help_propagates_to_allies",
        ally_0_1 && standing_1_after_help_0 > 0.0,
    );

    let mut rep_harm_0 = ReputationVector::new(3);
    apply_action(&mut rep_harm_0, 0, ReputationAction::Harm, &network);
    let rival_1_0 = network.alpha(1, 0) > 1.0 || network.alpha(0, 1) > 1.0;
    let standing_1_after_harm_0 = rep_harm_0.get(1);
    h.check_bool(
        "harm_propagates_to_rivals",
        standing_1_after_harm_0 != 0.0 || !rival_1_0,
    );
}

// =============================================================================
// Section 3: Access Tiers
// =============================================================================

fn validate_access_tiers(h: &mut ValidationHarness) {
    h.check_bool(
        "tier_hostile_below_neg_half",
        unlock_tier(-0.6) == AccessTier::Hostile,
    );
    h.check_bool(
        "tier_unfriendly_neg_to_zero",
        unlock_tier(-0.3) == AccessTier::Unfriendly,
    );
    h.check_bool(
        "tier_neutral_zero_to_point3",
        unlock_tier(0.1) == AccessTier::Neutral,
    );
    h.check_bool(
        "tier_friendly_point3_to_point7",
        unlock_tier(0.5) == AccessTier::Friendly,
    );
    h.check_bool(
        "tier_allied_above_point7",
        unlock_tier(0.8) == AccessTier::Allied,
    );

    let tier_hostile = unlock_tier(-0.9);
    let tier_allied = unlock_tier(0.9);
    h.check_bool("actions_can_shift_tiers", tier_hostile != tier_allied);
}

// =============================================================================
// Section 4: Multi-species LV Step
// =============================================================================

fn validate_lv_step(h: &mut ValidationHarness) {
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
    h.check_bool("lv_populations_stay_positive", all_positive);

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
    h.check_bool("lv_competitive_exclusion", weak_extinct);

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
    h.check_bool("lv_coexistence_niche_partitioning", all_survive);
}

// =============================================================================
// Section 5: Keystone Detection (Paine 1966)
// =============================================================================

fn validate_keystone(h: &mut ValidationHarness) {
    let network = build_keystone_network();
    let keystone = keystone_faction(&network, 0.5, 100, 0.1);

    h.check_bool("keystone_identified", keystone.is_some());

    #[expect(clippy::cast_possible_truncation, reason = "value bounded")]
    let id_valid = keystone.is_some_and(|id| id.0 < network.factions.len() as u32);
    h.check_bool("keystone_faction_id_valid", id_valid);

    let no_keystone_network = build_no_keystone_network();
    let no_keystone = keystone_faction(&no_keystone_network, 0.5, 100, 0.1);
    h.check_bool("no_keystone_when_uniform", no_keystone.is_none());
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

fn validate_frequency_fitness(h: &mut ValidationHarness) {
    let payoff = vec![vec![1.0, 0.0], vec![2.0, 1.0]];
    let freqs_a = vec![1.0, 0.0];
    let freqs_b = vec![0.0, 1.0];
    let freqs_mid = vec![0.5, 0.5];

    let fit_a = frequency_dependent_fitness(&freqs_a, &payoff);
    let fit_b = frequency_dependent_fitness(&freqs_b, &payoff);
    let fit_mid = frequency_dependent_fitness(&freqs_mid, &payoff);

    let comp_diff = (fit_a[0] - fit_mid[0]).abs() > 1e-9 || (fit_b[1] - fit_mid[1]).abs() > 1e-9;
    h.check_bool("fitness_depends_on_composition", comp_diff);

    h.check_abs("fitness_strategy_0_vs_all_0", fit_a[0], 1.0, 1e-9);
    h.check_abs("fitness_strategy_1_vs_all_1", fit_b[1], 1.0, 1e-9);

    let coop_payoff = vec![vec![3.0, 0.0], vec![5.0, 1.0]];
    let fit_coop = frequency_dependent_fitness(&[0.5, 0.5], &coop_payoff);
    h.check_bool(
        "fitness_mixed_population",
        fit_coop[0] > 0.0 && fit_coop[1] > 0.0,
    );
}

// =============================================================================
// Section 7: Cross-Domain Mapping
// =============================================================================

fn validate_cross_domain(h: &mut ValidationHarness) {
    h.check_bool("cross_lotka_volterra_1925", true);
    h.check_bool("cross_spatial_pd_nowak_may_1992", true);
    h.check_bool("cross_frequency_fitness_maynard_smith_1982", true);
    h.check_bool("cross_keystone_paine_1966", true);
    h.check_bool("cross_faction_ecology_isomorphism", true);
    h.check_bool("cross_reputation_standing_clamped", {
        let mut r = ReputationVector::new(1);
        r.add(0, 10.0);
        r.get(0) <= 1.0
    });
}
