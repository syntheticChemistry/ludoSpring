// SPDX-License-Identifier: AGPL-3.0-or-later
//! Plane transition — world state preservation across mode shifts.
//!
//! When transitioning between planes, the system must:
//! 1. Snapshot world state (inventory, NPC dispositions, conditions)
//! 2. Map conditions between rulesets (e.g., Dialogue Frightened -> PF2e Frightened 1)
//! 3. Record the transition as a DAG vertex
//! 4. Swap the active ruleset cert

use super::super::ruleset::{Character, Condition};
pub use super::condition_map::{
    ConditionMapping, dialogue_to_tactical_mappings, map_conditions, tactical_to_dialogue_mappings,
};
use super::plane::PlaneType;
use std::collections::HashMap;

/// Inventory item that persists across plane transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryItem {
    /// Unique item identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Whether this item has combat relevance.
    pub combat_relevant: bool,
    /// Plane-specific metadata (e.g., "damage: 1d6" for Tactical).
    pub plane_metadata: HashMap<String, String>,
}

/// NPC disposition snapshot for transition.
#[derive(Debug, Clone)]
pub struct NpcDisposition {
    /// NPC identifier.
    pub npc_id: String,
    /// Current trust value.
    pub trust: f64,
    /// Emotional state description.
    pub emotional_state: String,
    /// Whether this NPC is hostile.
    pub hostile: bool,
}

/// Knowledge entry that persists across planes.
#[derive(Debug, Clone)]
pub struct KnowledgeEntry {
    /// What was learned.
    pub fact: String,
    /// Which plane it was learned in.
    pub source_plane: PlaneType,
    /// Whether it's usable in the target plane.
    pub cross_plane: bool,
}

/// Complete world state snapshot at a plane transition.
#[derive(Debug, Clone)]
pub struct WorldStateSnapshot {
    /// Player character state.
    pub character: Character,
    /// Inventory items.
    pub inventory: Vec<InventoryItem>,
    /// NPC dispositions at transition time.
    pub npc_dispositions: Vec<NpcDisposition>,
    /// Knowledge gained so far.
    pub knowledge: Vec<KnowledgeEntry>,
    /// Active conditions on the player.
    pub conditions: Vec<Condition>,
    /// Source plane.
    pub from_plane: PlaneType,
    /// Target plane.
    pub to_plane: PlaneType,
    /// What triggered the transition.
    pub trigger: String,
}

/// Specific integrity check that can fail during a plane transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionIssue {
    /// One or more inventory items were lost.
    InventoryLost,
    /// NPC dispositions changed unexpectedly.
    DispositionChanged,
    /// Cross-plane knowledge was dropped.
    KnowledgeLost,
    /// Conditions were not correctly mapped between rulesets.
    ConditionMismatch,
    /// Hit points changed during the transition.
    HpChanged,
}

impl TransitionIssue {
    /// Human-readable description for diagnostics.
    #[must_use]
    pub const fn description(&self) -> &str {
        match self {
            Self::InventoryLost => "Inventory items missing after transition",
            Self::DispositionChanged => "NPC dispositions changed during transition",
            Self::KnowledgeLost => "Cross-plane knowledge lost during transition",
            Self::ConditionMismatch => "Conditions not correctly mapped during transition",
            Self::HpChanged => "HP changed during transition",
        }
    }
}

/// Result of verifying world state preservation across a plane transition.
///
/// An empty `issues` vec means the transition preserved all state correctly.
#[derive(Debug)]
pub struct TransitionVerification {
    /// Typed issues found during verification (empty = all passed).
    pub issues: Vec<TransitionIssue>,
}

impl TransitionVerification {
    /// Whether all integrity checks passed.
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.issues.is_empty()
    }

    /// Whether a specific check passed.
    #[must_use]
    pub fn check_passed(&self, issue: &TransitionIssue) -> bool {
        !self.issues.contains(issue)
    }

    /// Human-readable descriptions of all failures.
    #[must_use]
    pub fn failure_descriptions(&self) -> Vec<&str> {
        self.issues
            .iter()
            .map(TransitionIssue::description)
            .collect()
    }
}

/// Verify transition integrity between pre and post snapshots.
#[must_use]
pub fn verify_transition(
    pre: &WorldStateSnapshot,
    post: &WorldStateSnapshot,
    condition_mappings: &[ConditionMapping],
) -> TransitionVerification {
    let mut issues = Vec::new();

    let inventory_ok = pre.inventory.len() == post.inventory.len()
        && pre
            .inventory
            .iter()
            .all(|item| post.inventory.iter().any(|p| p.id == item.id));
    if !inventory_ok {
        issues.push(TransitionIssue::InventoryLost);
    }

    let dispositions_ok = pre.npc_dispositions.iter().all(|d| {
        post.npc_dispositions.iter().any(|p| {
            p.npc_id == d.npc_id
                && (p.trust - d.trust).abs() < crate::tolerances::TRUST_EQUALITY_TOL
        })
    });
    if !dispositions_ok {
        issues.push(TransitionIssue::DispositionChanged);
    }

    let knowledge_ok = pre
        .knowledge
        .iter()
        .filter(|k| k.cross_plane)
        .all(|k| post.knowledge.iter().any(|p| p.fact == k.fact));
    if !knowledge_ok {
        issues.push(TransitionIssue::KnowledgeLost);
    }

    let expected_conditions = map_conditions(&pre.conditions, condition_mappings);
    let conditions_ok = expected_conditions.iter().all(|ec| {
        post.conditions
            .iter()
            .any(|pc| pc.name == ec.name && pc.value == ec.value)
    });
    if !conditions_ok {
        issues.push(TransitionIssue::ConditionMismatch);
    }

    let hp_ok = pre.character.hp_current == post.character.hp_current
        && pre.character.hp_max == post.character.hp_max;
    if !hp_ok {
        issues.push(TransitionIssue::HpChanged);
    }

    TransitionVerification { issues }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::super::plane::PlaneType;
    use super::*;
    use std::collections::HashMap;

    fn test_character(hp_current: i32, hp_max: i32) -> Character {
        Character {
            name: "Tester".into(),
            level: 1,
            abilities: vec![],
            skills: vec![],
            conditions: vec![],
            hp_current,
            hp_max,
            resource_tracks: HashMap::new(),
            tags: vec![],
            metadata: HashMap::new(),
        }
    }

    fn base_snapshot(
        character: Character,
        inventory: Vec<InventoryItem>,
        npc_dispositions: Vec<NpcDisposition>,
        knowledge: Vec<KnowledgeEntry>,
        conditions: Vec<Condition>,
    ) -> WorldStateSnapshot {
        WorldStateSnapshot {
            character,
            inventory,
            npc_dispositions,
            knowledge,
            conditions,
            from_plane: PlaneType::Dialogue,
            to_plane: PlaneType::Tactical,
            trigger: "test".into(),
        }
    }

    #[test]
    fn transition_issue_descriptions_cover_all_variants() {
        let issues = [
            TransitionIssue::InventoryLost,
            TransitionIssue::DispositionChanged,
            TransitionIssue::KnowledgeLost,
            TransitionIssue::ConditionMismatch,
            TransitionIssue::HpChanged,
        ];
        let mut seen = std::collections::HashSet::new();
        for issue in &issues {
            let d = issue.description();
            assert!(!d.is_empty());
            assert!(seen.insert(d));
        }
    }

    #[test]
    fn transition_verification_passed_and_failure_descriptions() {
        let ok = TransitionVerification { issues: vec![] };
        assert!(ok.passed());
        assert!(ok.failure_descriptions().is_empty());

        let bad = TransitionVerification {
            issues: vec![TransitionIssue::InventoryLost, TransitionIssue::HpChanged],
        };
        assert!(!bad.passed());
        assert_eq!(bad.failure_descriptions().len(), 2);
        assert!(bad.check_passed(&TransitionIssue::KnowledgeLost));
        assert!(!bad.check_passed(&TransitionIssue::InventoryLost));
    }

    #[test]
    fn verify_transition_all_checks_pass() {
        let char_pre = test_character(10, 20);
        let char_post = char_pre.clone();
        let pre = base_snapshot(
            char_pre,
            vec![InventoryItem {
                id: "sword".into(),
                name: "Blade".into(),
                combat_relevant: true,
                plane_metadata: HashMap::new(),
            }],
            vec![NpcDisposition {
                npc_id: "npc1".into(),
                trust: 0.5,
                emotional_state: "calm".into(),
                hostile: false,
            }],
            vec![KnowledgeEntry {
                fact: "secret".into(),
                source_plane: PlaneType::Dialogue,
                cross_plane: true,
            }],
            vec![Condition {
                name: "Frightened".into(),
                value: 1,
                decay_per_turn: 0,
                turns_remaining: Some(3),
            }],
        );
        let mut post = pre.clone();
        post.character = char_post;
        post.conditions = vec![Condition {
            name: "Frightened".into(),
            value: 1,
            decay_per_turn: 1,
            turns_remaining: Some(3),
        }];
        let v = verify_transition(&pre, &post, &dialogue_to_tactical_mappings());
        assert!(v.passed(), "{:?}", v.issues);
    }

    #[test]
    fn verify_transition_detects_inventory_lost_by_count() {
        let pre = base_snapshot(
            test_character(5, 5),
            vec![InventoryItem {
                id: "a".into(),
                name: "A".into(),
                combat_relevant: false,
                plane_metadata: HashMap::new(),
            }],
            vec![],
            vec![],
            vec![],
        );
        let mut post = pre.clone();
        post.inventory.clear();
        let v = verify_transition(&pre, &post, &[]);
        assert_eq!(v.issues, vec![TransitionIssue::InventoryLost]);
    }

    #[test]
    fn verify_transition_detects_inventory_lost_by_missing_id() {
        let pre = base_snapshot(
            test_character(5, 5),
            vec![InventoryItem {
                id: "kept".into(),
                name: "K".into(),
                combat_relevant: false,
                plane_metadata: HashMap::new(),
            }],
            vec![],
            vec![],
            vec![],
        );
        let mut post = pre.clone();
        post.inventory[0].id = "replaced".into();
        let v = verify_transition(&pre, &post, &[]);
        assert_eq!(v.issues, vec![TransitionIssue::InventoryLost]);
    }

    #[test]
    fn verify_transition_detects_disposition_trust_change() {
        let pre = base_snapshot(
            test_character(5, 5),
            vec![],
            vec![NpcDisposition {
                npc_id: "n".into(),
                trust: 1.0,
                emotional_state: "x".into(),
                hostile: false,
            }],
            vec![],
            vec![],
        );
        let mut post = pre.clone();
        post.npc_dispositions[0].trust = 1e-6f64.mul_add(1000.0, 1.0);
        let v = verify_transition(&pre, &post, &[]);
        assert_eq!(v.issues, vec![TransitionIssue::DispositionChanged]);
    }

    #[test]
    fn verify_transition_allows_extra_post_dispositions() {
        let pre = base_snapshot(
            test_character(5, 5),
            vec![],
            vec![NpcDisposition {
                npc_id: "only_pre".into(),
                trust: 0.0,
                emotional_state: "x".into(),
                hostile: false,
            }],
            vec![],
            vec![],
        );
        let mut post = pre.clone();
        post.npc_dispositions.push(NpcDisposition {
            npc_id: "new_on_post".into(),
            trust: 0.0,
            emotional_state: "y".into(),
            hostile: false,
        });
        let v = verify_transition(&pre, &post, &[]);
        assert!(v.passed());
    }

    #[test]
    fn verify_transition_cross_plane_knowledge_must_persist() {
        let pre = base_snapshot(
            test_character(5, 5),
            vec![],
            vec![],
            vec![
                KnowledgeEntry {
                    fact: "local_only".into(),
                    source_plane: PlaneType::Dialogue,
                    cross_plane: false,
                },
                KnowledgeEntry {
                    fact: "must_follow".into(),
                    source_plane: PlaneType::Dialogue,
                    cross_plane: true,
                },
            ],
            vec![],
        );
        let mut post = pre.clone();
        post.knowledge.retain(|k| k.fact != "must_follow");
        let v = verify_transition(&pre, &post, &[]);
        assert_eq!(v.issues, vec![TransitionIssue::KnowledgeLost]);
    }

    #[test]
    fn verify_transition_non_cross_plane_knowledge_may_be_dropped() {
        let pre = base_snapshot(
            test_character(5, 5),
            vec![],
            vec![],
            vec![KnowledgeEntry {
                fact: "local_only".into(),
                source_plane: PlaneType::Dialogue,
                cross_plane: false,
            }],
            vec![],
        );
        let mut post = pre.clone();
        post.knowledge.clear();
        let v = verify_transition(&pre, &post, &[]);
        assert!(v.passed());
    }

    #[test]
    fn verify_transition_condition_mismatch_wrong_name_or_value() {
        let pre = base_snapshot(
            test_character(5, 5),
            vec![],
            vec![],
            vec![],
            vec![Condition {
                name: "Charmed".into(),
                value: 2,
                decay_per_turn: 5,
                turns_remaining: None,
            }],
        );
        let mappings = dialogue_to_tactical_mappings();
        let mut post = pre.clone();
        post.conditions = vec![Condition {
            name: "Fascinated".into(),
            value: 1,
            decay_per_turn: 5,
            turns_remaining: None,
        }];
        let v = verify_transition(&pre, &post, &mappings);
        assert_eq!(v.issues, vec![TransitionIssue::ConditionMismatch]);

        let mut post_ok = pre.clone();
        post_ok.conditions = map_conditions(&pre.conditions, &mappings);
        let v_ok = verify_transition(&pre, &post_ok, &mappings);
        assert!(v_ok.passed());

        let mut post_bad_value = pre.clone();
        post_bad_value.conditions = vec![Condition {
            name: "Fascinated".into(),
            value: 99,
            decay_per_turn: 5,
            turns_remaining: None,
        }];
        let v_bad = verify_transition(&pre, &post_bad_value, &mappings);
        assert_eq!(v_bad.issues, vec![TransitionIssue::ConditionMismatch]);
    }

    #[test]
    fn verify_transition_hp_changed_current_or_max() {
        let pre = base_snapshot(test_character(10, 20), vec![], vec![], vec![], vec![]);
        let mut post = pre.clone();
        post.character.hp_current = 9;
        assert_eq!(
            verify_transition(&pre, &post, &[]).issues,
            vec![TransitionIssue::HpChanged]
        );
        let mut post2 = pre.clone();
        post2.character.hp_max = 21;
        assert_eq!(
            verify_transition(&pre, &post2, &[]).issues,
            vec![TransitionIssue::HpChanged]
        );
    }

    #[test]
    fn verify_transition_multiple_issues_accumulate() {
        let pre = base_snapshot(
            test_character(1, 1),
            vec![InventoryItem {
                id: "x".into(),
                name: "X".into(),
                combat_relevant: false,
                plane_metadata: HashMap::new(),
            }],
            vec![NpcDisposition {
                npc_id: "n".into(),
                trust: 0.0,
                emotional_state: String::new(),
                hostile: true,
            }],
            vec![KnowledgeEntry {
                fact: "k".into(),
                source_plane: PlaneType::Tactical,
                cross_plane: true,
            }],
            vec![Condition {
                name: "Frightened".into(),
                value: 1,
                decay_per_turn: 0,
                turns_remaining: None,
            }],
        );
        let post = WorldStateSnapshot {
            character: test_character(2, 1),
            inventory: vec![],
            npc_dispositions: vec![],
            knowledge: vec![],
            conditions: vec![],
            from_plane: pre.from_plane,
            to_plane: pre.to_plane,
            trigger: pre.trigger.clone(),
        };
        let v = verify_transition(&pre, &post, &dialogue_to_tactical_mappings());
        assert_eq!(v.issues.len(), 5);
        assert!(!v.passed());
    }

    #[test]
    fn map_conditions_preserves_decay_when_mapping_does_not_gain_decay() {
        let conditions = vec![Condition {
            name: "Wounded".into(),
            value: 1,
            decay_per_turn: 7,
            turns_remaining: Some(2),
        }];
        let mapped = map_conditions(&conditions, &tactical_to_dialogue_mappings());
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].decay_per_turn, 7);
        assert_eq!(mapped[0].turns_remaining, Some(2));
    }

    #[test]
    fn map_conditions_rounds_value_factor_and_clamps_to_min_one() {
        let mappings = vec![ConditionMapping {
            source_name: "X".into(),
            target_name: "Y".into(),
            value_factor: 0.2,
            gains_decay: false,
            decay_per_turn: 0,
        }];
        let conditions = vec![Condition {
            name: "X".into(),
            value: 1,
            decay_per_turn: 3,
            turns_remaining: None,
        }];
        let mapped = map_conditions(&conditions, &mappings);
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].name, "Y");
        assert_eq!(mapped[0].value, 1);
    }

    #[test]
    fn map_conditions_confused_to_stupefied_and_charmed_to_fascinated() {
        let conditions = vec![
            Condition {
                name: "Confused".into(),
                value: 3,
                decay_per_turn: 2,
                turns_remaining: None,
            },
            Condition {
                name: "Charmed".into(),
                value: 1,
                decay_per_turn: 0,
                turns_remaining: Some(5),
            },
        ];
        let mapped = map_conditions(&conditions, &dialogue_to_tactical_mappings());
        assert_eq!(mapped[0].name, "Stupefied");
        assert_eq!(mapped[0].value, 3);
        assert_eq!(mapped[0].decay_per_turn, 2);
        assert_eq!(mapped[1].name, "Fascinated");
        assert_eq!(mapped[1].turns_remaining, Some(5));
    }

    #[test]
    fn map_conditions_exhausted_to_fatigued() {
        let conditions = vec![Condition {
            name: "Exhausted".into(),
            value: 2,
            decay_per_turn: 1,
            turns_remaining: None,
        }];
        let mapped = map_conditions(&conditions, &dialogue_to_tactical_mappings());
        assert_eq!(mapped[0].name, "Fatigued");
        assert_eq!(mapped[0].value, 2);
        assert_eq!(mapped[0].decay_per_turn, 1);
    }

    #[test]
    fn tactical_to_dialogue_maps_dying() {
        let conditions = vec![Condition {
            name: "Dying".into(),
            value: 4,
            decay_per_turn: 0,
            turns_remaining: Some(1),
        }];
        let mapped = map_conditions(&conditions, &tactical_to_dialogue_mappings());
        assert_eq!(mapped[0].name, "Dying");
        assert_eq!(mapped[0].value, 4);
    }

    #[test]
    fn dialogue_frightened_maps_to_pf2e_frightened() {
        let conditions = vec![Condition {
            name: "Frightened".into(),
            value: 2,
            decay_per_turn: 0,
            turns_remaining: None,
        }];
        let mappings = dialogue_to_tactical_mappings();
        let mapped = map_conditions(&conditions, &mappings);
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].name, "Frightened");
        assert_eq!(mapped[0].value, 2);
        assert_eq!(mapped[0].decay_per_turn, 1); // gains decay in PF2e
    }

    #[test]
    fn unmapped_conditions_dropped() {
        let conditions = vec![Condition {
            name: "CustomDialogueOnly".into(),
            value: 1,
            decay_per_turn: 0,
            turns_remaining: None,
        }];
        let mappings = dialogue_to_tactical_mappings();
        let mapped = map_conditions(&conditions, &mappings);
        assert!(mapped.is_empty());
    }

    #[test]
    fn tactical_wounded_maps_to_dialogue() {
        let conditions = vec![Condition {
            name: "Wounded".into(),
            value: 1,
            decay_per_turn: 0,
            turns_remaining: None,
        }];
        let mappings = tactical_to_dialogue_mappings();
        let mapped = map_conditions(&conditions, &mappings);
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].name, "Wounded");
    }

    #[test]
    fn multiple_conditions_map() {
        let conditions = vec![
            Condition {
                name: "Frightened".into(),
                value: 1,
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
        let mappings = dialogue_to_tactical_mappings();
        let mapped = map_conditions(&conditions, &mappings);
        assert_eq!(mapped.len(), 2);
    }

    #[test]
    fn condition_mapping_count() {
        assert_eq!(dialogue_to_tactical_mappings().len(), 4);
        assert_eq!(tactical_to_dialogue_mappings().len(), 3);
    }
}
