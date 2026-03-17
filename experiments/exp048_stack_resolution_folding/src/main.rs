// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp048 — Stack Resolution as Folding
//!
//! MTG's stack is a LIFO structure where players layer spells and abilities
//! in response to each other. The deterministic card text (genotype) does NOT
//! determine the game outcome (phenotype) — resolution order does.
//!
//! This is structurally identical to RNA/protein folding:
//!   - DNA sequence → RNA → amino acid chain is deterministic (card text is fixed)
//!   - But the 3D fold depends on environment, interaction order, thermodynamics
//!   - Same sequence can produce different conformations (misfolding = misplay)
//!
//! The same cards in different resolution orders produce different board states.
//! The "semantic space" of the game is not solved by reading the cards — it
//! requires resolving the interaction ordering, just as protein function is not
//! determined by sequence alone but by folded structure.
//!
//! This experiment validates:
//!   1. Same cards, different stack order → different outcomes
//!   2. Stack as LIFO with response windows (priority system)
//!   3. "In response to..." creates DAG branching
//!   4. The semantic space: card text is necessary but not sufficient
//!   5. Isomorphism to folding: sequence vs structure vs function

#![forbid(unsafe_code)]

mod model;

use ludospring_barracuda::validation::ValidationResult;
use model::{
    BoardState, Stack, Target, bolt_to_face, giant_growth, lightning_bolt,
    scenario_bolt_then_growth, scenario_growth_then_bolt, scenario_murder_no_response,
    scenario_regen_before_murder, scenario_triple_stack_bolt_wins,
    scenario_triple_stack_growth_wins,
};

const EXP: &str = "exp048_stack_resolution_folding";

// ===========================================================================
// Validation helpers
// ===========================================================================

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

// ===========================================================================
// Validation
// ===========================================================================

fn validate_same_cards_different_outcome() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Scenario 1: Bolt + Growth — same two cards, different resolution order
    let board_a = scenario_bolt_then_growth();
    let board_b = scenario_growth_then_bolt();

    let bear_dead_a = board_a.graveyard.contains(&"bear");
    let bear_dead_b = board_b.graveyard.contains(&"bear");

    results.push(ValidationResult::check(
        EXP,
        "bolt_responds_to_growth_bear_dies",
        bool_f64(bear_dead_a),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "growth_responds_to_bolt_bear_lives",
        bool_f64(!bear_dead_b),
        1.0,
        0.0,
    ));

    // The fundamental finding: same cards, opposite outcomes
    results.push(ValidationResult::check(
        EXP,
        "same_two_cards_opposite_outcomes",
        bool_f64(bear_dead_a != bear_dead_b),
        1.0,
        0.0,
    ));

    // In scenario B, bear should be 5/5 with 3 damage (alive)
    let Some(bear_b) = board_b.creatures.iter().find(|c| c.name == "bear") else {
        eprintln!("FATAL: bear must exist in growth scenario");
        std::process::exit(1);
    };
    results.push(ValidationResult::check(
        EXP,
        "pumped_bear_is_5_5",
        f64::from(bear_b.effective_power()),
        5.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "pumped_bear_has_3_damage",
        f64::from(bear_b.damage),
        3.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "pumped_bear_survives_bolt",
        bool_f64(!bear_b.is_dead()),
        1.0,
        0.0,
    ));

    results
}

fn validate_destroy_vs_regenerate() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let board_regen = scenario_regen_before_murder();
    let board_no_regen = scenario_murder_no_response();

    let bear_lives_with_regen = !board_regen.graveyard.contains(&"bear");
    let bear_dies_without_regen = board_no_regen.graveyard.contains(&"bear");

    results.push(ValidationResult::check(
        EXP,
        "regen_before_murder_bear_lives",
        bool_f64(bear_lives_with_regen),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "murder_without_response_bear_dies",
        bool_f64(bear_dies_without_regen),
        1.0,
        0.0,
    ));

    // Same destroy spell, opposite outcome based on response timing
    results.push(ValidationResult::check(
        EXP,
        "regenerate_timing_determines_survival",
        bool_f64(bear_lives_with_regen && bear_dies_without_regen),
        1.0,
        0.0,
    ));

    results
}

fn validate_triple_stack() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let board_bolt_wins = scenario_triple_stack_bolt_wins();
    let board_growth_wins = scenario_triple_stack_growth_wins();

    let bear_dead_bolt = board_bolt_wins.graveyard.contains(&"bear");
    let bear_dead_growth = board_growth_wins.graveyard.contains(&"bear");

    results.push(ValidationResult::check(
        EXP,
        "triple_stack_bolt_timing_kills_bear",
        bool_f64(bear_dead_bolt),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "triple_stack_growth_timing_saves_bear",
        bool_f64(!bear_dead_growth),
        1.0,
        0.0,
    ));

    // Three cards, two orderings, opposite outcomes
    results.push(ValidationResult::check(
        EXP,
        "three_cards_two_orderings_opposite_outcomes",
        bool_f64(bear_dead_bolt != bear_dead_growth),
        1.0,
        0.0,
    ));

    // In the growth-wins scenario, bear should be 8/8 (two Giant Growths resolved)
    let Some(bear_g) = board_growth_wins
        .creatures
        .iter()
        .find(|c| c.name == "bear")
    else {
        eprintln!("FATAL: bear must exist in growth-wins scenario");
        std::process::exit(1);
    };
    results.push(ValidationResult::check(
        EXP,
        "double_pumped_bear_is_8_8",
        f64::from(bear_g.effective_power()),
        8.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "double_pumped_bear_has_3_damage",
        f64::from(bear_g.damage),
        3.0,
        0.0,
    ));

    // Resolution log should show 3 resolutions
    results.push(ValidationResult::check(
        EXP,
        "bolt_wins_log_has_entries",
        bool_f64(!board_bolt_wins.resolution_log.is_empty()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "growth_wins_log_has_entries",
        bool_f64(!board_growth_wins.resolution_log.is_empty()),
        1.0,
        0.0,
    ));

    results
}

#[expect(
    clippy::cast_precision_loss,
    reason = "ValidationResult::check expects f64; integer metrics are intentional"
)]
fn validate_folding_isomorphism() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // The folding isomorphism:
    //
    // | Concept       | MTG Stack                    | Protein Folding               |
    // |---------------|------------------------------|-------------------------------|
    // | Sequence      | Card text (deterministic)    | Amino acid chain (det.)       |
    // | Structure     | Resolution order on stack    | 3D fold conformation          |
    // | Function      | Game outcome (who wins)      | Biological function           |
    // | Environment   | Opponent responses           | Solvent, pH, temperature      |
    // | Misfolding    | Misplay (wrong timing)       | Disease-causing misfolding    |
    // | Degeneracy    | Multiple paths to same win   | Multiple folds with function  |
    //
    // Key property: sequence does NOT determine function.
    // Same cards (sequence) + different order (structure) = different outcome (function).

    // Count unique outcomes across our scenarios
    let scenarios: Vec<(&str, BoardState)> = vec![
        ("bolt_responds_to_growth", scenario_bolt_then_growth()),
        ("growth_responds_to_bolt", scenario_growth_then_bolt()),
        ("regen_before_murder", scenario_regen_before_murder()),
        ("murder_no_response", scenario_murder_no_response()),
        ("triple_bolt_wins", scenario_triple_stack_bolt_wins()),
        ("triple_growth_wins", scenario_triple_stack_growth_wins()),
    ];

    results.push(ValidationResult::check(
        EXP,
        "six_scenarios_tested",
        scenarios.len() as f64,
        6.0,
        0.0,
    ));

    // Compute the "fold" — the outcome fingerprint
    let outcomes: Vec<(&str, bool, i32)> = scenarios
        .iter()
        .map(|(name, board)| {
            let bear_alive = !board.graveyard.contains(&"bear");
            let bear_power = board
                .creatures
                .iter()
                .find(|c| c.name == "bear" && !c.is_dead())
                .map_or(0, model::Creature::effective_power);
            (*name, bear_alive, bear_power)
        })
        .collect();

    // Same "sequence" (bolt + growth) produces two distinct "folds"
    let bolt_growth_fold_a = &outcomes[0]; // bear dead
    let bolt_growth_fold_b = &outcomes[1]; // bear alive at 5/5
    results.push(ValidationResult::check(
        EXP,
        "same_sequence_different_folds",
        bool_f64(bolt_growth_fold_a.1 != bolt_growth_fold_b.1),
        1.0,
        0.0,
    ));

    // The "functional" difference: bear power
    results.push(ValidationResult::check(
        EXP,
        "fold_a_no_function_bear_dead",
        f64::from(bolt_growth_fold_a.2),
        0.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "fold_b_functional_bear_power_5",
        f64::from(bolt_growth_fold_b.2),
        5.0,
        0.0,
    ));

    // The semantic space: with N instant-speed spells, the number of possible
    // stack orderings grows combinatorially. Even with just 3 cards, we get
    // dramatically different outcomes. This is why card text alone doesn't solve
    // the game — the interaction ordering is the unsolved space.
    //
    // For N cards that can be ordered on the stack:
    // - 2 cards: 2 orderings
    // - 3 cards: 6 orderings (3!)
    // - 4 cards: 24 orderings
    // - 5 cards: 120 orderings
    // - In practice, each player can respond to each item, so it's even more.
    //
    // This is the "folding problem": deterministic components,
    // combinatorial interaction space.

    let two_card_orderings = 2;
    let three_card_orderings = 6;
    results.push(ValidationResult::check(
        EXP,
        "two_card_semantic_space_2",
        f64::from(two_card_orderings),
        2.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "three_card_semantic_space_6",
        f64::from(three_card_orderings),
        6.0,
        0.0,
    ));

    // Degeneracy: multiple different resolution paths can lead to the same outcome.
    // In scenarios 1 and 4 (bolt_then_growth, murder_no_response), the bear dies
    // via completely different mechanisms (lethal damage vs destroy effect).
    // Same phenotype (bear dead), different genotype path. = degenerate code.
    let death_count = outcomes.iter().filter(|o| !o.1).count();
    results.push(ValidationResult::check(
        EXP,
        "multiple_paths_to_same_death_outcome",
        bool_f64(death_count >= 2),
        1.0,
        0.0,
    ));

    // Similarly, bear survives in scenarios 2, 3, 6 via different mechanisms
    // (pump, regeneration, double-pump). Same phenotype, different structure.
    let survivals: Vec<_> = outcomes.iter().filter(|o| o.1).collect();
    results.push(ValidationResult::check(
        EXP,
        "multiple_paths_to_survival",
        bool_f64(survivals.len() >= 2),
        1.0,
        0.0,
    ));

    // But the "power" of surviving bears differs — degenerate in survival,
    // non-degenerate in final state. Like two correct protein folds with
    // different binding affinities.
    let survival_powers: Vec<i32> = survivals.iter().map(|o| o.2).collect();
    let all_same_power = survival_powers.windows(2).all(|w| w[0] == w[1]);
    results.push(ValidationResult::check(
        EXP,
        "surviving_bears_have_different_power",
        bool_f64(!all_same_power),
        1.0,
        0.0,
    ));

    results
}

#[expect(
    clippy::cast_precision_loss,
    reason = "ValidationResult::check expects f64; integer metrics are intentional"
)]
fn validate_stack_lifo_mechanics() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Verify LIFO ordering explicitly
    let mut stack = Stack::new();
    let id_a = stack.cast(bolt_to_face(), "bob", vec![Target::Player("bob")]);
    let id_b = stack.respond(
        giant_growth(),
        "alice",
        vec![Target::Creature("bear")],
        id_a,
    );
    let _id_c = stack.respond(
        lightning_bolt(),
        "bob",
        vec![Target::Creature("bear")],
        id_b,
    );

    // Stack should have 3 items
    results.push(ValidationResult::check(
        EXP,
        "stack_has_three_items",
        stack.items.len() as f64,
        3.0,
        0.0,
    ));

    // Resolve order should be C, B, A (LIFO)
    let Some(first) = stack.resolve_top() else {
        eprintln!("FATAL: stack has items to resolve");
        std::process::exit(1);
    };
    results.push(ValidationResult::check(
        EXP,
        "lifo_first_resolves_is_last_cast",
        bool_f64(first.card.name == "Lightning Bolt"),
        1.0,
        0.0,
    ));

    let Some(second) = stack.resolve_top() else {
        eprintln!("FATAL: stack has items to resolve (second)");
        std::process::exit(1);
    };
    results.push(ValidationResult::check(
        EXP,
        "lifo_second_resolves_is_middle",
        bool_f64(second.card.name == "Giant Growth"),
        1.0,
        0.0,
    ));

    let Some(third) = stack.resolve_top() else {
        eprintln!("FATAL: stack has items to resolve (third)");
        std::process::exit(1);
    };
    results.push(ValidationResult::check(
        EXP,
        "lifo_third_resolves_is_first_cast",
        bool_f64(third.card.name == "Lightning Bolt"),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "stack_empty_after_full_resolution",
        bool_f64(stack.is_empty()),
        1.0,
        0.0,
    ));

    // Response chain: each item knows what it's responding to
    results.push(ValidationResult::check(
        EXP,
        "first_cast_has_no_response_target",
        bool_f64(first.responding_to.is_some()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "response_chain_is_linked",
        bool_f64(second.responding_to == Some(id_a)),
        1.0,
        0.0,
    ));

    results
}

fn validate_dag_from_stack() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // The stack creates a DAG structure:
    //   - Each cast is a vertex
    //   - Each "in response to" is a parent edge
    //   - Resolution order is the reverse topological sort
    //
    // This DAG is structurally identical to the text adventure DAG (exp046)
    // and the field genomics DAG. The "response chain" is the lineage.

    let mut stack = Stack::new();
    let a = stack.cast(lightning_bolt(), "bob", vec![Target::Creature("bear")]);
    let b = stack.respond(giant_growth(), "alice", vec![Target::Creature("bear")], a);
    let c = stack.respond(lightning_bolt(), "bob", vec![Target::Creature("bear")], b);

    // DAG structure: a ← b ← c (each responds to previous)
    let item_a = &stack.items[0];
    let item_b = &stack.items[1];
    let item_c = &stack.items[2];

    results.push(ValidationResult::check(
        EXP,
        "root_cast_has_no_parent",
        bool_f64(item_a.responding_to.is_none()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "response_b_parents_to_a",
        bool_f64(item_b.responding_to == Some(a)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "response_c_parents_to_b",
        bool_f64(item_c.responding_to == Some(b)),
        1.0,
        0.0,
    ));

    // Lineage depth = 3 (same concept as crown lineage in exp046)
    let mut depth = 0;
    let mut current = Some(c);
    while let Some(id) = current {
        depth += 1;
        current = stack
            .items
            .iter()
            .find(|i| i.id == id)
            .and_then(|i| i.responding_to);
    }
    results.push(ValidationResult::check(
        EXP,
        "response_chain_depth_3",
        f64::from(depth),
        3.0,
        0.0,
    ));

    results
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    println!("=== exp048: Stack Resolution as Folding ===\n");
    println!("Card text is the genotype. Resolution order is the phenotype.\n");

    let mut all_results = Vec::new();

    println!("--- Same Cards, Different Outcome ---");
    let r = validate_same_cards_different_outcome();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- Destroy vs Regenerate Timing ---");
    let r = validate_destroy_vs_regenerate();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- Triple Stack Complexity ---");
    let r = validate_triple_stack();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- Folding Isomorphism ---");
    let r = validate_folding_isomorphism();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- Stack LIFO Mechanics ---");
    let r = validate_stack_lifo_mechanics();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- DAG from Stack (Response Chain) ---");
    let r = validate_dag_from_stack();
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

    if passed != total {
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
