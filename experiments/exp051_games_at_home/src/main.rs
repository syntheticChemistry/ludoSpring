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

use ludospring_barracuda::validation::ValidationResult;

use models::{
    HumanComputeUnit, cross_domain_transfers, example_players, isomorphism_table,
    provenance_requirements, scale_comparison, simulate_feedback_loop,
};

const EXP: &str = "exp051_games_at_home";

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

// ===========================================================================
// Validation
// ===========================================================================

#[allow(clippy::cast_precision_loss)]
fn validate_isomorphism() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let table = isomorphism_table();

    // All concepts should structurally match
    let all_match = table.iter().all(|r| r.structural_match);
    results.push(ValidationResult::check(
        EXP,
        "all_12_isomorphism_pairs_match",
        bool_f64(all_match),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "isomorphism_has_12_concepts",
        table.len() as f64,
        12.0,
        0.0,
    ));

    // Games@Home has a strict superset of F@H's attribution
    // (sweetGrass provides full creative lineage vs F@H's team points)
    let attribution = table.iter().find(|r| r.concept == "Attribution").unwrap();
    results.push(ValidationResult::check(
        EXP,
        "games_attribution_exceeds_fah",
        bool_f64(attribution.games_at_home.contains("full creative lineage")),
        1.0,
        0.0,
    ));

    results
}

#[allow(clippy::cast_precision_loss)]
fn validate_human_compute() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let players = example_players();

    // All players produce novel decisions
    let all_positive = players.iter().all(|p| p.novel_decisions_per_week() > 0.0);
    results.push(ValidationResult::check(
        EXP,
        "all_players_produce_novel_data",
        bool_f64(all_positive),
        1.0,
        0.0,
    ));

    // RPGPT player has highest novelty rate (narrative branching = always novel)
    let rpgpt = players
        .iter()
        .find(|p| p.player_id == "rpgpt_campaign_player")
        .unwrap();
    let max_novelty = players
        .iter()
        .map(|p| p.novelty_rate)
        .fold(0.0_f64, f64::max);
    results.push(ValidationResult::check(
        EXP,
        "rpgpt_highest_novelty_rate",
        bool_f64((rpgpt.novelty_rate - max_novelty).abs() < f64::EPSILON),
        1.0,
        0.0,
    ));

    // RPGPT player has highest exploration value per session
    // (high creativity × high cross-domain × high novelty)
    let rpgpt_val = rpgpt.exploration_value_per_week() / rpgpt.sessions_per_week;
    let others_max_val = players
        .iter()
        .filter(|p| p.player_id != "rpgpt_campaign_player")
        .map(|p| p.exploration_value_per_week() / p.sessions_per_week)
        .fold(0.0_f64, f64::max);
    results.push(ValidationResult::check(
        EXP,
        "rpgpt_highest_exploration_value_per_session",
        bool_f64(rpgpt_val > others_max_val),
        1.0,
        0.0,
    ));

    // Commander player has high novelty despite being casual
    // (singleton + multiplayer naturally drives exploration)
    let commander = players
        .iter()
        .find(|p| p.player_id == "casual_commander")
        .unwrap();
    let competitive = players
        .iter()
        .find(|p| p.player_id == "competitive_standard")
        .unwrap();
    results.push(ValidationResult::check(
        EXP,
        "casual_commander_higher_novelty_than_competitive",
        bool_f64(commander.novelty_rate > competitive.novelty_rate),
        1.0,
        0.0,
    ));

    // Total weekly novel decisions across all players
    let total_novel: f64 = players
        .iter()
        .map(HumanComputeUnit::novel_decisions_per_week)
        .sum();
    results.push(ValidationResult::check(
        EXP,
        "total_novel_decisions_gt_1000_per_week",
        bool_f64(total_novel > 1000.0),
        1.0,
        0.0,
    ));

    results
}

#[allow(clippy::cast_precision_loss)]
fn validate_feedback_loop() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let players = example_players();
    let cycles = simulate_feedback_loop(&players, 20, 4); // 20 cycles, 4 weeks each

    // Loop should produce 20 cycles
    results.push(ValidationResult::check(
        EXP,
        "feedback_loop_runs_20_cycles",
        cycles.len() as f64,
        20.0,
        0.0,
    ));

    // Model accuracy should improve over time
    let first = &cycles[0];
    let last = &cycles[cycles.len() - 1];
    results.push(ValidationResult::check(
        EXP,
        "model_accuracy_improves",
        bool_f64(last.model_accuracy > first.model_accuracy),
        1.0,
        0.0,
    ));

    // Engagement should stabilize at a healthy level (not collapse)
    results.push(ValidationResult::check(
        EXP,
        "engagement_stabilizes_above_50pct",
        bool_f64(last.engagement > 0.5),
        1.0,
        0.0,
    ));

    // Trajectories should accumulate monotonically
    let monotonic = cycles
        .windows(2)
        .all(|w| w[1].trajectories >= w[0].trajectories);
    results.push(ValidationResult::check(
        EXP,
        "trajectories_accumulate_monotonically",
        bool_f64(monotonic),
        1.0,
        0.0,
    ));

    // The virtuous cycle: better targets → more engagement → more data → better model
    let mid = &cycles[cycles.len() / 2];
    results.push(ValidationResult::check(
        EXP,
        "mid_cycle_accuracy_gt_initial",
        bool_f64(mid.model_accuracy > first.model_accuracy),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "final_accuracy_gt_mid",
        bool_f64(last.model_accuracy > mid.model_accuracy),
        1.0,
        0.0,
    ));

    // Target quality correlates with model accuracy
    results.push(ValidationResult::check(
        EXP,
        "target_quality_tracks_model_accuracy",
        bool_f64(last.target_quality > first.target_quality),
        1.0,
        0.0,
    ));

    results
}

#[allow(clippy::cast_precision_loss)]
fn validate_cross_domain() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let transfers = cross_domain_transfers();

    // We should have 7 transfer domains
    results.push(ValidationResult::check(
        EXP,
        "seven_cross_domain_transfers_identified",
        transfers.len() as f64,
        7.0,
        0.0,
    ));

    // All should have meaningful structural similarity (> 0.5)
    let all_meaningful = transfers.iter().all(|t| t.structural_similarity > 0.5);
    results.push(ValidationResult::check(
        EXP,
        "all_transfers_have_meaningful_similarity",
        bool_f64(all_meaningful),
        1.0,
        0.0,
    ));

    // Average structural similarity should be high
    let avg_similarity: f64 = transfers
        .iter()
        .map(|t| t.structural_similarity)
        .sum::<f64>()
        / transfers.len() as f64;
    results.push(ValidationResult::check(
        EXP,
        "average_structural_similarity_gt_70pct",
        bool_f64(avg_similarity > 0.70),
        1.0,
        0.0,
    ));

    // The highest-transfer domain should be tree search heuristics
    // (human pruning intuition → MCTS heuristics is the most direct)
    let highest = transfers
        .iter()
        .max_by(|a, b| {
            a.structural_similarity
                .partial_cmp(&b.structural_similarity)
                .unwrap()
        })
        .unwrap();
    results.push(ValidationResult::check(
        EXP,
        "highest_transfer_is_tree_search_heuristics",
        bool_f64(highest.source_domain == "Game tree pruning (human intuition)"),
        1.0,
        0.0,
    ));

    results
}

fn validate_provenance() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let reqs = provenance_requirements();

    // Should cover all three primals
    let has_rhizo = reqs.iter().any(|r| r.primal.contains("rhizoCrypt"));
    let has_sweet = reqs.iter().any(|r| r.primal.contains("sweetGrass"));
    let has_loam = reqs.iter().any(|r| r.primal.contains("loamSpine"));

    results.push(ValidationResult::check(
        EXP,
        "provenance_covers_rhizocrypt",
        bool_f64(has_rhizo),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "provenance_covers_sweetgrass",
        bool_f64(has_sweet),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "provenance_covers_loamspine",
        bool_f64(has_loam),
        1.0,
        0.0,
    ));

    // Cross-domain attribution is captured (the key novel requirement)
    let cross_domain_req = reqs.iter().any(|r| r.data_element.contains("Cross-domain"));
    results.push(ValidationResult::check(
        EXP,
        "cross_domain_attribution_captured",
        bool_f64(cross_domain_req),
        1.0,
        0.0,
    ));

    results
}

fn validate_scale() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let scale = scale_comparison();

    // Games@Home has 200× more compute units than F@H
    let fah_units = scale
        .iter()
        .find(|s| s.metric == "Active compute units")
        .unwrap();
    let ratio = fah_units.games_at_home / fah_units.folding_at_home;
    results.push(ValidationResult::check(
        EXP,
        "games_200x_more_compute_units",
        bool_f64(ratio >= 200.0),
        1.0,
        0.0,
    ));

    // Games@Home costs nothing (or less — entertainment value)
    let cost = scale
        .iter()
        .find(|s| s.metric == "Compute cost per unit-hour (USD)")
        .unwrap();
    results.push(ValidationResult::check(
        EXP,
        "games_zero_compute_cost",
        bool_f64(cost.games_at_home < cost.folding_at_home),
        1.0,
        0.0,
    ));

    // Games@Home has higher creativity per trajectory
    let creativity = scale
        .iter()
        .find(|s| s.metric == "Creativity per trajectory")
        .unwrap();
    results.push(ValidationResult::check(
        EXP,
        "games_higher_creativity",
        bool_f64(creativity.games_at_home > creativity.folding_at_home),
        1.0,
        0.0,
    ));

    // Games@Home has higher cross-domain transfer
    let transfer = scale
        .iter()
        .find(|s| s.metric == "Cross-domain transfer potential")
        .unwrap();
    results.push(ValidationResult::check(
        EXP,
        "games_higher_transfer_potential",
        bool_f64(transfer.games_at_home > transfer.folding_at_home),
        1.0,
        0.0,
    ));

    // Games@Home search space is infinite (vs F@H's large but finite)
    let space = scale
        .iter()
        .find(|s| s.metric == "Search space size (log10)")
        .unwrap();
    results.push(ValidationResult::check(
        EXP,
        "games_search_space_infinite",
        bool_f64(space.games_at_home.is_infinite()),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// Main
// ===========================================================================

#[expect(
    clippy::too_many_lines,
    reason = "validation orchestrator — sequential check groups"
)]
fn cmd_validate() {
    println!("=== exp051: Games@Home — Distributed Human Computation ===\n");
    println!("Folding@Home uses CPUs to explore protein space.");
    println!("Games@Home uses human creativity to explore infinite decision space.");
    println!("Every game is an experiment. Every player is a researcher.\n");

    let mut all_results = Vec::new();

    println!("--- Folding@Home ↔ Games@Home Isomorphism ---");
    let table = isomorphism_table();
    for row in &table {
        println!(
            "  {:25} │ {:40} │ {}",
            row.concept, row.folding_at_home, row.games_at_home
        );
    }
    let r = validate_isomorphism();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- Human Compute Units ---");
    for p in example_players() {
        println!(
            "  {}: {:.0} novel decisions/wk, exploration value {:.0}/wk",
            p.player_id,
            p.novel_decisions_per_week(),
            p.exploration_value_per_week()
        );
    }
    let r = validate_human_compute();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- Feedback Loop (20 cycles × 4 weeks) ---");
    let cycles = simulate_feedback_loop(&example_players(), 20, 4);
    for c in &cycles {
        if c.cycle % 5 == 0 || c.cycle == 19 {
            println!(
                "  Cycle {:2}: trajectories={:8.0}, accuracy={:.2}, engagement={:.2}",
                c.cycle, c.trajectories, c.model_accuracy, c.engagement
            );
        }
    }
    let r = validate_feedback_loop();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- Cross-Domain Transfer ---");
    for t in cross_domain_transfers() {
        println!(
            "  {} → {} ({:.0}%)",
            t.source_domain,
            t.target_domain,
            t.structural_similarity * 100.0
        );
    }
    let r = validate_cross_domain();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- Provenance Requirements ---");
    for req in provenance_requirements() {
        println!("  [{}] {}", req.primal, req.data_element);
    }
    let r = validate_provenance();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- Scale: Folding@Home vs Games@Home ---");
    for s in scale_comparison() {
        let fah_str = if s.folding_at_home == 0.0 {
            "0".to_string()
        } else if s.folding_at_home.is_infinite() {
            "∞".to_string()
        } else {
            format!("{:.2e}", s.folding_at_home)
        };
        let gah_str = if s.games_at_home == 0.0 {
            "FREE".to_string()
        } else if s.games_at_home.is_infinite() {
            "∞".to_string()
        } else {
            format!("{:.2e}", s.games_at_home)
        };
        println!(
            "  {:40} │ {:>12} │ {:>12} {}",
            s.metric, fah_str, gah_str, s.unit
        );
    }
    let r = validate_scale();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    let passed = all_results.iter().filter(|r| r.passed).count();
    let total = all_results.len();
    println!("\n=== SUMMARY: {passed}/{total} checks passed ===");

    if passed == total {
        println!("\nHuman creativity is compute. Games are experiments.");
        println!("The provenance trio makes every discovery attributable.");
    } else {
        println!("\nFAILED:");
        for r in all_results.iter().filter(|r| !r.passed) {
            println!(
                "  {} — measured={}, expected={}",
                r.description, r.measured, r.expected
            );
        }
        std::process::exit(1);
    }
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
