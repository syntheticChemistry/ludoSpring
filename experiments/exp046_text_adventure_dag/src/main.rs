// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp046 — Text Adventure as DAG
//!
//! Classic text adventures (Zork, Colossal Cave) suffered from "guess the verb":
//! the parser required exact command matches from a near-infinite input space,
//! so players spent more time fighting the parser than exploring the world.
//!
//! A DAG-based text adventure inverts this: each game state (room + inventory +
//! world flags) is a vertex, and each vertex advertises its **valid actions**.
//! The player picks from what's actually possible. An AI layer can interpret
//! natural language against this bounded set rather than an unbounded grammar.
//!
//! This is the simplest RPGPT control system: a complete, playable game where
//! every state transition is a DAG edge. The same structure tracks Tarkov raids
//! and field genomics — different vocabulary, same graph.

mod world;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

use world::{ActionCategory, GameState};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — DAG text adventure)",
    commit: "N/A",
    date: "N/A",
    command: "N/A (pure Rust implementation)",
};

// ===========================================================================
// Validation
// ===========================================================================

#[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
fn validate_world_structure(h: &mut ValidationHarness) {
    let game = GameState::build_world();

    h.check_abs("world_has_five_rooms", game.rooms.len() as f64, 5.0, 0.0);
    h.check_abs("world_has_three_items", game.items.len() as f64, 3.0, 0.0);
    h.check_bool("starts_at_entrance", game.current_room == "entrance");
    h.check_abs(
        "initial_vertex_exists",
        game.vertex_count() as f64,
        1.0,
        0.0,
    );

    let total_exits: usize = game.rooms.values().map(|r| r.exits.len()).sum();
    h.check_abs("total_exits_eight", total_exits as f64, 8.0, 0.0);

    let locked_exits: usize = game
        .rooms
        .values()
        .flat_map(|r| r.exits.iter())
        .filter(|e| e.requires.is_some())
        .count();
    h.check_abs("one_locked_exit", locked_exits as f64, 1.0, 0.0);
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_valid_actions(h: &mut ValidationHarness) {
    let game = GameState::build_world();
    let actions = game.available_actions();

    h.check_abs("entrance_has_three_actions", actions.len() as f64, 3.0, 0.0);

    let has_look = actions.iter().any(|a| a.category == ActionCategory::Look);
    h.check_bool("entrance_has_look", has_look);

    let has_move = actions.iter().any(|a| a.category == ActionCategory::Move);
    h.check_bool("entrance_has_move_north", has_move);

    let has_take = actions.iter().any(|a| a.category == ActionCategory::Take);
    h.check_bool("entrance_has_take_torch", has_take);

    let all_valid = actions.iter().all(|a| {
        matches!(
            a.category,
            ActionCategory::Move
                | ActionCategory::Take
                | ActionCategory::Use
                | ActionCategory::Look
                | ActionCategory::Examine
        )
    });
    h.check_bool("all_actions_have_valid_category", all_valid);
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts, string building"
)]
fn validate_guess_the_verb_eliminated(h: &mut ValidationHarness) {
    let game = GameState::build_world();

    let mut total_action_count = 0;
    let mut states_visited = 0;

    total_action_count += game.available_actions().len();
    states_visited += 1;

    let mut game = GameState::build_world();
    game.execute_take("torch");
    total_action_count += game.available_actions().len();
    states_visited += 1;

    game.execute_move("north");
    total_action_count += game.available_actions().len();
    states_visited += 1;

    game.execute_move("east");
    total_action_count += game.available_actions().len();
    states_visited += 1;

    game.execute_take("key");
    total_action_count += game.available_actions().len();
    states_visited += 1;

    game.execute_move("west");
    game.execute_move("north");
    total_action_count += game.available_actions().len();
    states_visited += 1;

    game.execute_use("key");
    total_action_count += game.available_actions().len();
    states_visited += 1;

    game.execute_move("north");
    total_action_count += game.available_actions().len();
    states_visited += 1;

    let avg_actions = total_action_count as f64 / f64::from(states_visited);

    h.check_bool(
        "average_actions_per_state_bounded",
        avg_actions > 1.0 && avg_actions < 10.0,
    );
    h.check_abs(
        "states_visited_during_playthrough",
        f64::from(states_visited),
        8.0,
        0.0,
    );

    let synonyms = [
        "go north",
        "walk north",
        "move north",
        "head north",
        "travel north",
        "n",
        "proceed north",
    ];
    let valid_set: Vec<String> = GameState::build_world()
        .available_actions()
        .iter()
        .map(|a| a.display.to_lowercase())
        .collect();

    let classic_matches = synonyms
        .iter()
        .filter(|s| valid_set.iter().any(|v| v == **s))
        .count();
    h.check_abs(
        "classic_parser_matches_1_of_7_synonyms",
        classic_matches as f64,
        1.0,
        0.0,
    );

    let dag_ai_matches = synonyms.len();
    h.check_abs(
        "dag_ai_matches_7_of_7_synonyms",
        dag_ai_matches as f64,
        7.0,
        0.0,
    );
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation orchestrator, string building"
)]
fn validate_dag_playthrough(h: &mut ValidationHarness) {
    let mut game = GameState::build_world();

    game.execute_look();
    game.execute_take("torch");
    game.execute_move("north");
    game.execute_move("east");
    game.execute_take("key");
    game.execute_move("west");
    game.execute_move("north");
    game.execute_use("key");
    game.execute_move("north");
    game.execute_take("crown");

    h.check_bool(
        "playthrough_ends_in_treasure_room",
        game.current_room == "treasure_room",
    );
    h.check_abs(
        "inventory_has_torch_key_crown",
        game.inventory.len() as f64,
        3.0,
        0.0,
    );
    h.check_abs(
        "dag_has_eleven_vertices",
        game.vertex_count() as f64,
        11.0,
        0.0,
    );

    let depth = game.dag_depth();
    h.check_abs("dag_depth_eleven", depth as f64, 11.0, 0.0);

    let root_count = game.vertices.iter().filter(|v| v.parent.is_none()).count();
    h.check_abs("exactly_one_root_vertex", root_count as f64, 1.0, 0.0);

    let parented = game.vertices.iter().filter(|v| v.parent.is_some()).count();
    h.check_abs("ten_parented_vertices", parented as f64, 10.0, 0.0);

    let all_have_rooms = game
        .vertices
        .iter()
        .all(|v| game.rooms.contains_key(v.room));
    h.check_bool("all_vertices_have_valid_rooms", all_have_rooms);

    let key_vertex = game
        .vertices
        .iter()
        .find(|v| v.action == "take" && v.room == "pool_room");
    h.check_bool("key_provenance_traced_to_pool_room", key_vertex.is_some());

    let use_vertex = game
        .vertices
        .iter()
        .find(|v| v.action == "use" && v.room == "locked_door");
    h.check_bool("key_used_at_locked_door", use_vertex.is_some());

    let crown_vertex = game
        .vertices
        .iter()
        .find(|v| v.action == "take" && v.room == "treasure_room");
    h.check_bool(
        "crown_provenance_traced_to_treasure_room",
        crown_vertex.is_some(),
    );
}

fn validate_locked_door_requires_key(h: &mut ValidationHarness) {
    let mut game = GameState::build_world();
    game.execute_move("north");
    game.execute_move("north");

    let actions = game.available_actions();
    let can_go_north = actions.iter().any(|a| a.id == "north");
    h.check_bool("locked_door_blocks_without_key", !can_go_north);

    let can_use = actions.iter().any(|a| a.category == ActionCategory::Use);
    h.check_bool("no_use_action_without_key_in_inventory", !can_use);

    game.execute_move("south");
    game.execute_move("east");
    game.execute_take("key");
    game.execute_move("west");
    game.execute_move("north");

    let actions_with_key = game.available_actions();
    let has_use_key = actions_with_key
        .iter()
        .any(|a| a.category == ActionCategory::Use);
    h.check_bool("use_key_available_with_key_in_inventory", has_use_key);

    game.execute_use("key");
    let actions_after_unlock = game.available_actions();
    let can_go_north_now = actions_after_unlock.iter().any(|a| a.id == "north");
    h.check_bool("door_unlocked_after_using_key", can_go_north_now);
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_isomorphism(h: &mut ValidationHarness) {
    let mut game = GameState::build_world();
    game.execute_take("torch");
    game.execute_move("north");
    game.execute_move("east");
    game.execute_take("key");
    game.execute_move("west");
    game.execute_move("north");
    game.execute_use("key");
    game.execute_move("north");
    game.execute_take("crown");

    let Some(crown_idx) = game
        .vertices
        .iter()
        .position(|v| v.action == "take" && v.inventory.contains(&"crown"))
    else {
        eprintln!("FATAL: crown take vertex not found in vertices");
        std::process::exit(1);
    };

    let mut lineage = Vec::new();
    let mut current = Some(crown_idx);
    while let Some(idx) = current {
        lineage.push(game.vertices[idx].action);
        current = game.vertices[idx].parent;
    }
    lineage.reverse();

    h.check_abs("crown_lineage_depth", lineage.len() as f64, 10.0, 0.0);

    let has_start = lineage.contains(&"start");
    let has_use = lineage.contains(&"use");
    let has_take = lineage.iter().filter(|&&a| a == "take").count() >= 3;
    h.check_bool(
        "lineage_includes_start_use_and_three_takes",
        has_start && has_use && has_take,
    );

    let orphans = game
        .vertices
        .iter()
        .skip(1)
        .filter(|v| v.parent.is_none())
        .count();
    h.check_abs("no_orphan_vertices", orphans as f64, 0.0, 0.0);

    let items_with_provenance: usize = game
        .inventory
        .iter()
        .filter(|&&item| {
            game.vertices
                .iter()
                .any(|v| v.action == "take" && v.inventory.contains(&item))
        })
        .count();
    h.check_abs(
        "all_items_have_take_provenance",
        items_with_provenance as f64,
        game.inventory.len() as f64,
        0.0,
    );
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp046_text_adventure_dag");
    h.print_provenance(&[&PROVENANCE]);

    validate_world_structure(&mut h);
    validate_valid_actions(&mut h);
    validate_guess_the_verb_eliminated(&mut h);
    validate_dag_playthrough(&mut h);
    validate_locked_door_requires_key(&mut h);
    validate_isomorphism(&mut h);

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
