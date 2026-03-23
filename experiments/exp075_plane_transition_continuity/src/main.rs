// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp075 — Plane Transition Continuity (Dialogue <-> Tactical)
//!
//! Validates world state preservation across plane transitions:
//! 1. PlaneTransition records correct metadata
//! 2. Inventory persists
//! 3. NPC dispositions persist (trust unchanged by transition)
//! 4. Conditions map between rulesets
//! 5. Knowledge carries forward
//! 6. Round-trip (Dialogue -> Tactical -> Dialogue) preserves state

use std::collections::HashMap;

use ludospring_barracuda::game::rpgpt::plane::{PlaneTransition, PlaneType};
use ludospring_barracuda::game::rpgpt::transition::{
    InventoryItem, KnowledgeEntry, NpcDisposition, TransitionIssue, WorldStateSnapshot,
    dialogue_to_tactical_mappings, map_conditions, tactical_to_dialogue_mappings,
    verify_transition,
};
use ludospring_barracuda::game::ruleset::{AbilityScore, Character, Condition};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const EXP: &str = "exp075_plane_transition_continuity";

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "specs/RPGPT_PLANES_SCHEMA.md",
    commit: "74cf9488",
    date: "2026-03-15",
    command: "cargo run -p exp075_plane_transition_continuity",
};

fn test_character() -> Character {
    Character {
        name: "Elara".into(),
        level: 5,
        abilities: vec![
            AbilityScore::pf2e("Strength", 14),
            AbilityScore::pf2e("Charisma", 18),
        ],
        skills: vec![],
        conditions: vec![],
        hp_current: 45,
        hp_max: 52,
        resource_tracks: HashMap::new(),
        tags: vec!["Human".into(), "Diplomat".into()],
        metadata: HashMap::new(),
    }
}

fn test_inventory() -> Vec<InventoryItem> {
    vec![
        InventoryItem {
            id: "sword_01".into(),
            name: "Fine Longsword".into(),
            combat_relevant: true,
            plane_metadata: {
                let mut m = HashMap::new();
                m.insert("damage".into(), "1d8+3".into());
                m
            },
        },
        InventoryItem {
            id: "letter_02".into(),
            name: "Sealed Letter".into(),
            combat_relevant: false,
            plane_metadata: HashMap::new(),
        },
        InventoryItem {
            id: "potion_03".into(),
            name: "Healing Potion".into(),
            combat_relevant: true,
            plane_metadata: HashMap::new(),
        },
    ]
}

fn test_dispositions() -> Vec<NpcDisposition> {
    vec![
        NpcDisposition {
            npc_id: "guard_captain".into(),
            trust: -2.0,
            emotional_state: "hostile".into(),
            hostile: true,
        },
        NpcDisposition {
            npc_id: "maren_blacksmith".into(),
            trust: 3.5,
            emotional_state: "warm".into(),
            hostile: false,
        },
    ]
}

fn test_knowledge() -> Vec<KnowledgeEntry> {
    vec![
        KnowledgeEntry {
            fact: "Guard's family is threatened by the guild".into(),
            source_plane: PlaneType::Dialogue,
            cross_plane: true,
        },
        KnowledgeEntry {
            fact: "The cellar door has a hidden mechanism".into(),
            source_plane: PlaneType::Investigation,
            cross_plane: true,
        },
        KnowledgeEntry {
            fact: "Maren's emotional state during persuasion".into(),
            source_plane: PlaneType::Dialogue,
            cross_plane: false, // dialogue-only insight
        },
    ]
}

fn pre_transition_snapshot() -> WorldStateSnapshot {
    WorldStateSnapshot {
        character: test_character(),
        inventory: test_inventory(),
        npc_dispositions: test_dispositions(),
        knowledge: test_knowledge(),
        conditions: vec![Condition {
            name: "Frightened".into(),
            value: 1,
            decay_per_turn: 0,
            turns_remaining: None,
        }],
        from_plane: PlaneType::Dialogue,
        to_plane: PlaneType::Tactical,
        trigger: "guard_draws_weapon".into(),
    }
}

fn validate_transition_vertex(h: &mut ValidationHarness) {
    let transition = PlaneTransition::new(
        PlaneType::Dialogue,
        PlaneType::Tactical,
        "guard_draws_weapon",
    );

    h.check_bool(
        "transition_from_dialogue",
        transition.from == PlaneType::Dialogue,
    );
    h.check_bool(
        "transition_to_tactical",
        transition.to == PlaneType::Tactical,
    );
    h.check_bool(
        "transition_trigger_correct",
        transition.trigger == "guard_draws_weapon",
    );
    h.check_bool(
        "transition_hash_none_initially",
        transition.world_state_hash.is_none(),
    );
}

fn validate_inventory_preservation(h: &mut ValidationHarness) {
    let pre = pre_transition_snapshot();
    let mut post = pre.clone();
    post.from_plane = PlaneType::Tactical;
    post.to_plane = PlaneType::Dialogue;

    let mappings = dialogue_to_tactical_mappings();
    post.conditions = map_conditions(&pre.conditions, &mappings);

    let v = verify_transition(&pre, &post, &mappings);
    h.check_bool(
        "inventory_preserved",
        v.check_passed(&TransitionIssue::InventoryLost),
    );
    h.check_abs(
        "inventory_count_matches",
        post.inventory.len() as f64,
        3.0,
        0.0,
    );
}

fn validate_disposition_preservation(h: &mut ValidationHarness) {
    let pre = pre_transition_snapshot();
    let mut post = pre.clone();
    post.conditions = map_conditions(&pre.conditions, &dialogue_to_tactical_mappings());

    let v = verify_transition(&pre, &post, &dialogue_to_tactical_mappings());
    h.check_bool(
        "dispositions_preserved",
        v.check_passed(&TransitionIssue::DispositionChanged),
    );

    let guard = post
        .npc_dispositions
        .iter()
        .find(|d| d.npc_id == "guard_captain");
    h.check_bool("guard_still_hostile", guard.is_some_and(|g| g.hostile));
    h.check_abs(
        "guard_trust_preserved",
        guard.map_or(0.0, |g| g.trust),
        -2.0,
        tolerances::GAME_STATE_TOL,
    );
}

fn validate_condition_mapping_dialogue_to_tactical(h: &mut ValidationHarness) {
    let conditions = vec![
        Condition {
            name: "Frightened".into(),
            value: 2,
            decay_per_turn: 0,
            turns_remaining: None,
        },
        Condition {
            name: "Exhausted".into(),
            value: 1,
            decay_per_turn: 0,
            turns_remaining: None,
        },
    ];
    let mapped = map_conditions(&conditions, &dialogue_to_tactical_mappings());

    h.check_abs("mapped_condition_count", mapped.len() as f64, 2.0, 0.0);

    let frightened = mapped.iter().find(|c| c.name == "Frightened");
    h.check_bool("frightened_mapped", frightened.is_some());
    h.check_abs(
        "frightened_value_preserved",
        f64::from(frightened.map_or(0, |c| c.value)),
        2.0,
        0.0,
    );
    h.check_abs(
        "frightened_gains_decay",
        f64::from(frightened.map_or(0, |c| c.decay_per_turn)),
        1.0,
        0.0,
    );

    let fatigued = mapped.iter().find(|c| c.name == "Fatigued");
    h.check_bool("exhausted_becomes_fatigued", fatigued.is_some());
}

fn validate_condition_mapping_tactical_to_dialogue(h: &mut ValidationHarness) {
    let conditions = vec![
        Condition {
            name: "Wounded".into(),
            value: 1,
            decay_per_turn: 0,
            turns_remaining: None,
        },
        Condition {
            name: "Frightened".into(),
            value: 3,
            decay_per_turn: 1,
            turns_remaining: None,
        },
    ];
    let mapped = map_conditions(&conditions, &tactical_to_dialogue_mappings());

    h.check_abs("return_mapped_count", mapped.len() as f64, 2.0, 0.0);

    let wounded = mapped.iter().find(|c| c.name == "Wounded");
    h.check_bool("wounded_persists_to_dialogue", wounded.is_some());
}

fn validate_knowledge_carries_forward(h: &mut ValidationHarness) {
    let pre = pre_transition_snapshot();
    let mut post = pre.clone();
    post.conditions = map_conditions(&pre.conditions, &dialogue_to_tactical_mappings());

    let v = verify_transition(&pre, &post, &dialogue_to_tactical_mappings());
    h.check_bool(
        "knowledge_preserved",
        v.check_passed(&TransitionIssue::KnowledgeLost),
    );

    let cross_plane: Vec<&KnowledgeEntry> =
        post.knowledge.iter().filter(|k| k.cross_plane).collect();
    h.check_abs(
        "cross_plane_knowledge_count",
        cross_plane.len() as f64,
        2.0,
        0.0,
    );
}

fn validate_hp_preservation(h: &mut ValidationHarness) {
    let pre = pre_transition_snapshot();
    let mut post = pre.clone();
    post.conditions = map_conditions(&pre.conditions, &dialogue_to_tactical_mappings());

    let v = verify_transition(&pre, &post, &dialogue_to_tactical_mappings());
    h.check_bool("hp_preserved", v.check_passed(&TransitionIssue::HpChanged));
    h.check_abs(
        "hp_current_45",
        f64::from(post.character.hp_current),
        45.0,
        0.0,
    );
    h.check_abs("hp_max_52", f64::from(post.character.hp_max), 52.0, 0.0);
}

fn validate_round_trip(h: &mut ValidationHarness) {
    // Dialogue -> Tactical
    let pre = pre_transition_snapshot();
    let mut tactical_state = pre.clone();
    tactical_state.conditions = map_conditions(&pre.conditions, &dialogue_to_tactical_mappings());

    // Simulate combat: take damage, gain Wounded
    tactical_state.character.hp_current = 30;
    tactical_state.conditions.push(Condition {
        name: "Wounded".into(),
        value: 1,
        decay_per_turn: 0,
        turns_remaining: None,
    });

    // Tactical -> Dialogue
    let mut post_dialogue = tactical_state.clone();
    post_dialogue.conditions =
        map_conditions(&tactical_state.conditions, &tactical_to_dialogue_mappings());
    post_dialogue.from_plane = PlaneType::Tactical;
    post_dialogue.to_plane = PlaneType::Dialogue;

    h.check_abs(
        "post_combat_hp",
        f64::from(post_dialogue.character.hp_current),
        30.0,
        0.0,
    );

    let has_wounded = post_dialogue.conditions.iter().any(|c| c.name == "Wounded");
    h.check_bool("wounded_persists_to_dialogue_roundtrip", has_wounded);

    h.check_abs(
        "inventory_still_three",
        post_dialogue.inventory.len() as f64,
        3.0,
        0.0,
    );

    let guard = post_dialogue
        .npc_dispositions
        .iter()
        .find(|d| d.npc_id == "guard_captain");
    h.check_bool(
        "guard_still_hostile_after_roundtrip",
        guard.is_some_and(|g| g.hostile),
    );
}

fn validate_unmapped_conditions_dropped(h: &mut ValidationHarness) {
    let conditions = vec![Condition {
        name: "DialogueOnlyCondition".into(),
        value: 1,
        decay_per_turn: 0,
        turns_remaining: None,
    }];
    let mapped = map_conditions(&conditions, &dialogue_to_tactical_mappings());
    h.check_bool("unmapped_dropped", mapped.is_empty());
}

fn validate_no_state_leak(h: &mut ValidationHarness) {
    let pre = pre_transition_snapshot();
    let post = pre.clone();

    // Plane-specific metadata should not leak
    let sword = post.inventory.iter().find(|i| i.id == "sword_01");
    h.check_bool(
        "sword_has_damage_metadata",
        sword.is_some_and(|s| s.plane_metadata.contains_key("damage")),
    );

    // Non-cross-plane knowledge should exist but be marked
    let dialogue_only = post.knowledge.iter().filter(|k| !k.cross_plane).count();
    h.check_abs(
        "dialogue_only_knowledge_count",
        dialogue_only as f64,
        1.0,
        0.0,
    );
}

fn validate_verification_detects_issues(h: &mut ValidationHarness) {
    let pre = pre_transition_snapshot();
    let mut post = pre.clone();
    post.conditions = map_conditions(&pre.conditions, &dialogue_to_tactical_mappings());

    // Remove an item to trigger inventory issue
    post.inventory.pop();
    let v = verify_transition(&pre, &post, &dialogue_to_tactical_mappings());
    h.check_bool(
        "detects_missing_inventory",
        !v.check_passed(&TransitionIssue::InventoryLost),
    );
    h.check_bool("issues_not_empty", !v.passed());

    // Change HP to trigger HP issue
    let mut post2 = pre.clone();
    post2.conditions = map_conditions(&pre.conditions, &dialogue_to_tactical_mappings());
    post2.character.hp_current = 10;
    let v2 = verify_transition(&pre, &post2, &dialogue_to_tactical_mappings());
    h.check_bool(
        "detects_hp_change",
        !v2.check_passed(&TransitionIssue::HpChanged),
    );
}

fn main() {
    let mut h = ValidationHarness::new(EXP);
    h.print_provenance(&[&PROVENANCE]);

    validate_transition_vertex(&mut h);
    validate_inventory_preservation(&mut h);
    validate_disposition_preservation(&mut h);
    validate_condition_mapping_dialogue_to_tactical(&mut h);
    validate_condition_mapping_tactical_to_dialogue(&mut h);
    validate_knowledge_carries_forward(&mut h);
    validate_hp_preservation(&mut h);
    validate_round_trip(&mut h);
    validate_unmapped_conditions_dropped(&mut h);
    validate_no_state_leak(&mut h);
    validate_verification_detects_issues(&mut h);

    h.finish();
}
