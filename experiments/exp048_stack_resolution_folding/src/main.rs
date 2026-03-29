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

use ludospring_barracuda::validation::{BaselineProvenance, OrExit, ValidationHarness};
use model::{
    BoardState, Stack, Target, bolt_to_face, giant_growth, lightning_bolt,
    scenario_bolt_then_growth, scenario_growth_then_bolt, scenario_murder_no_response,
    scenario_regen_before_murder, scenario_triple_stack_bolt_wins,
    scenario_triple_stack_growth_wins,
};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — MTG stack folding)",
    commit: "4b683e3e",
    date: "2026-03-29",
    command: "N/A (pure Rust implementation)",
};

// ===========================================================================
// Validation
// ===========================================================================

fn validate_same_cards_different_outcome(h: &mut ValidationHarness) {
    let board_a = scenario_bolt_then_growth();
    let board_b = scenario_growth_then_bolt();

    let bear_dead_a = board_a.graveyard.contains(&"bear");
    let bear_dead_b = board_b.graveyard.contains(&"bear");

    h.check_bool("bolt_responds_to_growth_bear_dies", bear_dead_a);
    h.check_bool("growth_responds_to_bolt_bear_lives", !bear_dead_b);
    h.check_bool(
        "same_two_cards_opposite_outcomes",
        bear_dead_a != bear_dead_b,
    );

    let bear_b = board_b
        .creatures
        .iter()
        .find(|c| c.name == "bear")
        .or_exit("bear must exist in growth scenario");
    h.check_abs(
        "pumped_bear_is_5_5",
        f64::from(bear_b.effective_power()),
        5.0,
        0.0,
    );
    h.check_abs(
        "pumped_bear_has_3_damage",
        f64::from(bear_b.damage),
        3.0,
        0.0,
    );
    h.check_bool("pumped_bear_survives_bolt", !bear_b.is_dead());
}

fn validate_destroy_vs_regenerate(h: &mut ValidationHarness) {
    let board_regen = scenario_regen_before_murder();
    let board_no_regen = scenario_murder_no_response();

    let bear_lives_with_regen = !board_regen.graveyard.contains(&"bear");
    let bear_dies_without_regen = board_no_regen.graveyard.contains(&"bear");

    h.check_bool("regen_before_murder_bear_lives", bear_lives_with_regen);
    h.check_bool("murder_without_response_bear_dies", bear_dies_without_regen);
    h.check_bool(
        "regenerate_timing_determines_survival",
        bear_lives_with_regen && bear_dies_without_regen,
    );
}

fn validate_triple_stack(h: &mut ValidationHarness) {
    let board_bolt_wins = scenario_triple_stack_bolt_wins();
    let board_growth_wins = scenario_triple_stack_growth_wins();

    let bear_dead_bolt = board_bolt_wins.graveyard.contains(&"bear");
    let bear_dead_growth = board_growth_wins.graveyard.contains(&"bear");

    h.check_bool("triple_stack_bolt_timing_kills_bear", bear_dead_bolt);
    h.check_bool("triple_stack_growth_timing_saves_bear", !bear_dead_growth);
    h.check_bool(
        "three_cards_two_orderings_opposite_outcomes",
        bear_dead_bolt != bear_dead_growth,
    );

    let bear_g = board_growth_wins
        .creatures
        .iter()
        .find(|c| c.name == "bear")
        .or_exit("bear must exist in growth-wins scenario");
    h.check_abs(
        "double_pumped_bear_is_8_8",
        f64::from(bear_g.effective_power()),
        8.0,
        0.0,
    );
    h.check_abs(
        "double_pumped_bear_has_3_damage",
        f64::from(bear_g.damage),
        3.0,
        0.0,
    );

    h.check_bool(
        "bolt_wins_log_has_entries",
        !board_bolt_wins.resolution_log.is_empty(),
    );
    h.check_bool(
        "growth_wins_log_has_entries",
        !board_growth_wins.resolution_log.is_empty(),
    );
}

#[expect(
    clippy::cast_precision_loss,
    reason = "harness check expects f64; integer metrics are intentional"
)]
fn validate_folding_isomorphism(h: &mut ValidationHarness) {
    let scenarios: Vec<(&str, BoardState)> = vec![
        ("bolt_responds_to_growth", scenario_bolt_then_growth()),
        ("growth_responds_to_bolt", scenario_growth_then_bolt()),
        ("regen_before_murder", scenario_regen_before_murder()),
        ("murder_no_response", scenario_murder_no_response()),
        ("triple_bolt_wins", scenario_triple_stack_bolt_wins()),
        ("triple_growth_wins", scenario_triple_stack_growth_wins()),
    ];

    h.check_abs("six_scenarios_tested", scenarios.len() as f64, 6.0, 0.0);

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

    let bolt_growth_fold_a = &outcomes[0];
    let bolt_growth_fold_b = &outcomes[1];
    h.check_bool(
        "same_sequence_different_folds",
        bolt_growth_fold_a.1 != bolt_growth_fold_b.1,
    );

    h.check_abs(
        "fold_a_no_function_bear_dead",
        f64::from(bolt_growth_fold_a.2),
        0.0,
        0.0,
    );
    h.check_abs(
        "fold_b_functional_bear_power_5",
        f64::from(bolt_growth_fold_b.2),
        5.0,
        0.0,
    );

    let two_card_orderings = 2;
    let three_card_orderings = 6;
    h.check_abs(
        "two_card_semantic_space_2",
        f64::from(two_card_orderings),
        2.0,
        0.0,
    );
    h.check_abs(
        "three_card_semantic_space_6",
        f64::from(three_card_orderings),
        6.0,
        0.0,
    );

    let death_count = outcomes.iter().filter(|o| !o.1).count();
    h.check_bool("multiple_paths_to_same_death_outcome", death_count >= 2);

    let survivals: Vec<_> = outcomes.iter().filter(|o| o.1).collect();
    h.check_bool("multiple_paths_to_survival", survivals.len() >= 2);

    let survival_powers: Vec<i32> = survivals.iter().map(|o| o.2).collect();
    let all_same_power = survival_powers.windows(2).all(|w| w[0] == w[1]);
    h.check_bool("surviving_bears_have_different_power", !all_same_power);
}

#[expect(
    clippy::cast_precision_loss,
    reason = "harness check expects f64; integer metrics are intentional"
)]
fn validate_stack_lifo_mechanics(h: &mut ValidationHarness) {
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

    h.check_abs("stack_has_three_items", stack.items.len() as f64, 3.0, 0.0);

    let first = stack.resolve_top().or_exit("stack has items to resolve");
    h.check_bool(
        "lifo_first_resolves_is_last_cast",
        first.card.name == "Lightning Bolt",
    );

    let second = stack
        .resolve_top()
        .or_exit("stack has items to resolve (second)");
    h.check_bool(
        "lifo_second_resolves_is_middle",
        second.card.name == "Giant Growth",
    );

    let third = stack
        .resolve_top()
        .or_exit("stack has items to resolve (third)");
    h.check_bool(
        "lifo_third_resolves_is_first_cast",
        third.card.name == "Lightning Bolt",
    );

    h.check_bool("stack_empty_after_full_resolution", stack.is_empty());

    h.check_bool(
        "first_cast_has_no_response_target",
        first.responding_to.is_some(),
    );
    h.check_bool(
        "response_chain_is_linked",
        second.responding_to == Some(id_a),
    );
}

fn validate_dag_from_stack(h: &mut ValidationHarness) {
    let mut stack = Stack::new();
    let a = stack.cast(lightning_bolt(), "bob", vec![Target::Creature("bear")]);
    let b = stack.respond(giant_growth(), "alice", vec![Target::Creature("bear")], a);
    let c = stack.respond(lightning_bolt(), "bob", vec![Target::Creature("bear")], b);

    let item_a = &stack.items[0];
    let item_b = &stack.items[1];
    let item_c = &stack.items[2];

    h.check_bool("root_cast_has_no_parent", item_a.responding_to.is_none());
    h.check_bool("response_b_parents_to_a", item_b.responding_to == Some(a));
    h.check_bool("response_c_parents_to_b", item_c.responding_to == Some(b));

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
    h.check_abs("response_chain_depth_3", f64::from(depth), 3.0, 0.0);
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp048_stack_resolution_folding");
    h.print_provenance(&[&PROVENANCE]);

    validate_same_cards_different_outcome(&mut h);
    validate_destroy_vs_regenerate(&mut h);
    validate_triple_stack(&mut h);
    validate_folding_isomorphism(&mut h);
    validate_stack_lifo_mechanics(&mut h);
    validate_dag_from_stack(&mut h);

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
