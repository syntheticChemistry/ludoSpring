// SPDX-License-Identifier: AGPL-3.0-or-later
//! Condition mapping between RPGPT planes.
//!
//! When transitioning between planes (e.g., Dialogue → Tactical), game
//! conditions must be mapped between rulesets. This module defines the
//! mapping table and the transform logic.

use super::super::ruleset::Condition;

/// Rule for mapping a condition from one plane's ruleset to another's.
#[derive(Debug, Clone)]
pub struct ConditionMapping {
    /// Condition name in the source plane.
    pub source_name: String,
    /// Condition name in the target plane.
    pub target_name: String,
    /// Multiplier for the condition value (e.g. 1.0 = same severity).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogue_to_tactical_maps_frightened_with_decay() {
        let conditions = vec![Condition {
            name: "Frightened".into(),
            value: 2,
            decay_per_turn: 0,
            turns_remaining: Some(3),
        }];
        let mapped = map_conditions(&conditions, &dialogue_to_tactical_mappings());
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].name, "Frightened");
        assert_eq!(mapped[0].value, 2);
        assert_eq!(mapped[0].decay_per_turn, 1);
    }

    #[test]
    fn tactical_to_dialogue_preserves_wounded() {
        let conditions = vec![Condition {
            name: "Wounded".into(),
            value: 1,
            decay_per_turn: 0,
            turns_remaining: None,
        }];
        let mapped = map_conditions(&conditions, &tactical_to_dialogue_mappings());
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].name, "Wounded");
    }

    #[test]
    fn unmapped_conditions_are_dropped() {
        let conditions = vec![Condition {
            name: "Invisible".into(),
            value: 1,
            decay_per_turn: 0,
            turns_remaining: None,
        }];
        let mapped = map_conditions(&conditions, &dialogue_to_tactical_mappings());
        assert!(mapped.is_empty());
    }

    #[test]
    fn value_factor_scales_condition() {
        let conditions = vec![Condition {
            name: "Test".into(),
            value: 4,
            decay_per_turn: 0,
            turns_remaining: None,
        }];
        let mappings = vec![ConditionMapping {
            source_name: "Test".into(),
            target_name: "Mapped".into(),
            value_factor: 0.5,
            gains_decay: false,
            decay_per_turn: 0,
        }];
        let mapped = map_conditions(&conditions, &mappings);
        assert_eq!(mapped[0].value, 2);
        assert_eq!(mapped[0].name, "Mapped");
    }
}
