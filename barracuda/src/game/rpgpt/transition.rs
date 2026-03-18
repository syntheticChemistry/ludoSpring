// SPDX-License-Identifier: AGPL-3.0-or-later
//! Plane transition — world state preservation across mode shifts.
//!
//! When transitioning between planes, the system must:
//! 1. Snapshot world state (inventory, NPC dispositions, conditions)
//! 2. Map conditions between rulesets (e.g., Dialogue Frightened -> PF2e Frightened 1)
//! 3. Record the transition as a DAG vertex
//! 4. Swap the active ruleset cert

use super::super::ruleset::{Character, Condition};
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

/// A condition mapping rule between two planes.
#[derive(Debug, Clone)]
pub struct ConditionMapping {
    /// Source condition name.
    pub source_name: String,
    /// Target condition name.
    pub target_name: String,
    /// Value transformation factor (1.0 = direct mapping).
    pub value_factor: f64,
    /// Whether the condition gains per-turn decay in the target plane.
    pub gains_decay: bool,
    /// Decay rate if gained.
    pub decay_per_turn: u8,
}

/// Default condition mappings between Dialogue and Tactical (PF2e).
#[must_use]
pub fn dialogue_to_tactical_mappings() -> Vec<ConditionMapping> {
    vec![
        ConditionMapping {
            source_name: "Frightened".into(),
            target_name: "Frightened".into(),
            value_factor: 1.0,
            gains_decay: true,
            decay_per_turn: 1,
        },
        ConditionMapping {
            source_name: "Confused".into(),
            target_name: "Stupefied".into(),
            value_factor: 1.0,
            gains_decay: false,
            decay_per_turn: 0,
        },
        ConditionMapping {
            source_name: "Charmed".into(),
            target_name: "Fascinated".into(),
            value_factor: 1.0,
            gains_decay: false,
            decay_per_turn: 0,
        },
        ConditionMapping {
            source_name: "Exhausted".into(),
            target_name: "Fatigued".into(),
            value_factor: 1.0,
            gains_decay: false,
            decay_per_turn: 0,
        },
    ]
}

/// Default condition mappings from Tactical back to Dialogue.
#[must_use]
pub fn tactical_to_dialogue_mappings() -> Vec<ConditionMapping> {
    vec![
        ConditionMapping {
            source_name: "Frightened".into(),
            target_name: "Frightened".into(),
            value_factor: 1.0,
            gains_decay: false,
            decay_per_turn: 0,
        },
        ConditionMapping {
            source_name: "Wounded".into(),
            target_name: "Wounded".into(),
            value_factor: 1.0,
            gains_decay: false,
            decay_per_turn: 0,
        },
        ConditionMapping {
            source_name: "Dying".into(),
            target_name: "Dying".into(),
            value_factor: 1.0,
            gains_decay: false,
            decay_per_turn: 0,
        },
    ]
}

/// Map conditions from one plane to another using a mapping table.
#[must_use]
pub fn map_conditions(conditions: &[Condition], mappings: &[ConditionMapping]) -> Vec<Condition> {
    conditions
        .iter()
        .filter_map(|c| {
            mappings.iter().find(|m| m.source_name == c.name).map(|m| {
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "condition values are small integers"
                )]
                let mapped_value = (f64::from(c.value) * m.value_factor).round() as u8;
                Condition {
                    name: m.target_name.clone(),
                    value: mapped_value.max(1),
                    decay_per_turn: if m.gains_decay {
                        m.decay_per_turn
                    } else {
                        c.decay_per_turn
                    },
                    turns_remaining: c.turns_remaining,
                }
            })
        })
        .collect()
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
        post.npc_dispositions
            .iter()
            .any(|p| p.npc_id == d.npc_id && (p.trust - d.trust).abs() < f64::EPSILON)
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
mod tests {
    use super::*;

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
