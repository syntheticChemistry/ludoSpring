// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp051 — Games@Home: Distributed Human Computation
//!
//! Folding@Home distributes protein conformational search across CPUs.
//! Each volunteer's computer explores a trajectory through folding space.
//! Aggregate trajectories reveal the energy landscape.
//!
//! Games@Home does the same thing with human players:
//!   - Each game session is a human exploring a trajectory through
//!     an infinite decision tree (exp049: 10^358+ unique paths)
//!   - Each decision is a "fold" — card text is sequence,
//!     resolution order is structure, outcome is function (exp048)
//!   - The provenance trio captures every trajectory with full lineage
//!   - Aggregate trajectories reveal the strategic landscape
//!
//! But humans bring something CPUs cannot:
//!   - **Creativity**: novel synergy discovery that no search algorithm finds
//!   - **Intuition**: pattern recognition across exponential spaces
//!   - **Cross-domain transfer**: a chess player's positional sense
//!     applied to MTG board state reads differently than a poker player's
//!   - **Social computation**: multiplayer politics, bluffing, signaling
//!
//! The feedback loop:
//!   1. Humans play games → generate novel trajectories (data)
//!   2. Models learn from trajectories → discover patterns
//!   3. Patterns suggest new exploration targets → new game content
//!   4. New content drives humans deeper into unexplored space → goto 1
//!
//! This is the convergence of ecoPrimals:
//!   - ludoSpring generates human exploration data
//!   - sweetGrass attributes creative contributions
//!   - rhizoCrypt captures session DAGs (trajectories)
//!   - loamSpine certifies rulesets, decks, outcomes
//!   - biomeOS orchestrates the compute
//!   - barracuda/metalForge provide the hardware substrate
//!
//! Cross-domain transfer: the patterns humans discover in game trees
//! (synergies, counter-strategies, meta evolution) are structurally
//! identical to patterns in protein folding, drug discovery, materials
//! science, logistics optimization. The isomorphism is the point.

mod models;

use ludospring_barracuda::validation::{BaselineProvenance, OrExit, ValidationHarness};

use models::{
    HumanComputeUnit, cross_domain_transfers, example_players, isomorphism_table,
    provenance_requirements, scale_comparison, simulate_feedback_loop,
};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Games@Home isomorphism)",
    commit: "19e402c0",
    date: "2026-03-29",
    command: "N/A (pure Rust implementation)",
};

// ===========================================================================
// Validation
// ===========================================================================

#[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
fn validate_isomorphism(h: &mut ValidationHarness) {
    let table = isomorphism_table();

    let all_match = table.iter().all(|r| r.structural_match);
    h.check_bool("all_12_isomorphism_pairs_match", all_match);
    h.check_abs("isomorphism_has_12_concepts", table.len() as f64, 12.0, 0.0);

    let attribution = table
        .iter()
        .find(|r| r.concept == "Attribution")
        .or_exit("Attribution concept not found in isomorphism table");
    h.check_bool(
        "games_attribution_exceeds_fah",
        attribution.games_at_home.contains("full creative lineage"),
    );
}

fn validate_human_compute(h: &mut ValidationHarness) {
    let players = example_players();

    let all_positive = players.iter().all(|p| p.novel_decisions_per_week() > 0.0);
    h.check_bool("all_players_produce_novel_data", all_positive);

    let rpgpt = players
        .iter()
        .find(|p| p.player_id == "rpgpt_campaign_player")
        .or_exit("rpgpt_campaign_player not found in example players");
    let max_novelty = players
        .iter()
        .map(|p| p.novelty_rate)
        .fold(0.0_f64, f64::max);
    h.check_bool(
        "rpgpt_highest_novelty_rate",
        (rpgpt.novelty_rate - max_novelty).abs() < f64::EPSILON,
    );

    let rpgpt_val = rpgpt.exploration_value_per_week() / rpgpt.sessions_per_week;
    let others_max_val = players
        .iter()
        .filter(|p| p.player_id != "rpgpt_campaign_player")
        .map(|p| p.exploration_value_per_week() / p.sessions_per_week)
        .fold(0.0_f64, f64::max);
    h.check_bool(
        "rpgpt_highest_exploration_value_per_session",
        rpgpt_val > others_max_val,
    );

    let commander = players
        .iter()
        .find(|p| p.player_id == "casual_commander")
        .or_exit("casual_commander not found in example players");
    let competitive = players
        .iter()
        .find(|p| p.player_id == "competitive_standard")
        .or_exit("competitive_standard not found in example players");
    h.check_bool(
        "casual_commander_higher_novelty_than_competitive",
        commander.novelty_rate > competitive.novelty_rate,
    );

    let total_novel: f64 = players
        .iter()
        .map(HumanComputeUnit::novel_decisions_per_week)
        .sum();
    h.check_bool(
        "total_novel_decisions_gt_1000_per_week",
        total_novel > 1000.0,
    );
}

#[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
fn validate_feedback_loop(h: &mut ValidationHarness) {
    let players = example_players();
    let cycles = simulate_feedback_loop(&players, 20, 4);

    h.check_abs(
        "feedback_loop_runs_20_cycles",
        cycles.len() as f64,
        20.0,
        0.0,
    );

    let first = &cycles[0];
    let last = &cycles[cycles.len() - 1];
    h.check_bool(
        "model_accuracy_improves",
        last.model_accuracy > first.model_accuracy,
    );
    h.check_bool("engagement_stabilizes_above_50pct", last.engagement > 0.5);

    let monotonic = cycles
        .windows(2)
        .all(|w| w[1].trajectories >= w[0].trajectories);
    h.check_bool("trajectories_accumulate_monotonically", monotonic);

    let mid = &cycles[cycles.len() / 2];
    h.check_bool(
        "mid_cycle_accuracy_gt_initial",
        mid.model_accuracy > first.model_accuracy,
    );
    h.check_bool(
        "final_accuracy_gt_mid",
        last.model_accuracy > mid.model_accuracy,
    );
    h.check_bool(
        "target_quality_tracks_model_accuracy",
        last.target_quality > first.target_quality,
    );
}

#[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
fn validate_cross_domain(h: &mut ValidationHarness) {
    let transfers = cross_domain_transfers();

    h.check_abs(
        "seven_cross_domain_transfers_identified",
        transfers.len() as f64,
        7.0,
        0.0,
    );

    let all_meaningful = transfers.iter().all(|t| t.structural_similarity > 0.5);
    h.check_bool("all_transfers_have_meaningful_similarity", all_meaningful);

    let avg_similarity: f64 = transfers
        .iter()
        .map(|t| t.structural_similarity)
        .sum::<f64>()
        / transfers.len() as f64;
    h.check_bool(
        "average_structural_similarity_gt_70pct",
        avg_similarity > 0.70,
    );

    let highest = transfers
        .iter()
        .max_by(|a, b| {
            a.structural_similarity
                .partial_cmp(&b.structural_similarity)
                .or_exit("structural_similarity is NaN")
        })
        .or_exit("no transfers found for highest transfer domain");
    h.check_bool(
        "highest_transfer_is_tree_search_heuristics",
        highest.source_domain == "Game tree pruning (human intuition)",
    );
}

fn validate_provenance(h: &mut ValidationHarness) {
    let reqs = provenance_requirements();

    let has_rhizo = reqs.iter().any(|r| r.primal.contains("rhizoCrypt"));
    let has_sweet = reqs.iter().any(|r| r.primal.contains("sweetGrass"));
    let has_loam = reqs.iter().any(|r| r.primal.contains("loamSpine"));

    h.check_bool("provenance_covers_rhizocrypt", has_rhizo);
    h.check_bool("provenance_covers_sweetgrass", has_sweet);
    h.check_bool("provenance_covers_loamspine", has_loam);

    let cross_domain_req = reqs.iter().any(|r| r.data_element.contains("Cross-domain"));
    h.check_bool("cross_domain_attribution_captured", cross_domain_req);
}

fn validate_scale(h: &mut ValidationHarness) {
    let scale = scale_comparison();

    let fah_units = scale
        .iter()
        .find(|s| s.metric == "Active compute units")
        .or_exit("Active compute units metric not found in scale comparison");
    let ratio = fah_units.games_at_home / fah_units.folding_at_home;
    h.check_bool("games_200x_more_compute_units", ratio >= 200.0);

    let cost = scale
        .iter()
        .find(|s| s.metric == "Compute cost per unit-hour (USD)")
        .or_exit("Compute cost metric not found in scale comparison");
    h.check_bool(
        "games_zero_compute_cost",
        cost.games_at_home < cost.folding_at_home,
    );

    let creativity = scale
        .iter()
        .find(|s| s.metric == "Creativity per trajectory")
        .or_exit("Creativity per trajectory metric not found in scale comparison");
    h.check_bool(
        "games_higher_creativity",
        creativity.games_at_home > creativity.folding_at_home,
    );

    let transfer = scale
        .iter()
        .find(|s| s.metric == "Cross-domain transfer potential")
        .or_exit("Cross-domain transfer potential metric not found in scale comparison");
    h.check_bool(
        "games_higher_transfer_potential",
        transfer.games_at_home > transfer.folding_at_home,
    );

    let space = scale
        .iter()
        .find(|s| s.metric == "Search space size (log10)")
        .or_exit("Search space size metric not found in scale comparison");
    h.check_bool(
        "games_search_space_infinite",
        space.games_at_home.is_infinite(),
    );
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp051_games_at_home");
    h.print_provenance(&[&PROVENANCE]);

    validate_isomorphism(&mut h);
    validate_human_compute(&mut h);
    validate_feedback_loop(&mut h);
    validate_cross_domain(&mut h);
    validate_provenance(&mut h);
    validate_scale(&mut h);

    h.finish();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") | None => cmd_validate(),
        Some(other) => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}
