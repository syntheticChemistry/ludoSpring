// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ruleset abstraction — machine-readable game rule systems.
//!
//! Rulesets are not hard-coded to any system. The trait and types here
//! support d20 (Pathfinder 2e), Fudge dice (FATE Core), roll-under
//! (Cairn/Into the Odd), dice pools (Year Zero), and single-target
//! (Cypher) — any open system can be ingested.
//!
//! In the ecoPrimals architecture these become loamSpine certificates:
//! immutable, machine-readable constraint documents that an AI narration
//! engine must respect.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Dice
// ---------------------------------------------------------------------------

/// How the ruleset resolves randomness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiceSystem {
    /// Roll 1d20 + modifier vs DC (Pathfinder, D&D, Cypher).
    D20,
    /// Roll 4 Fudge dice (−1/0/+1 each) + skill vs difficulty (FATE).
    FudgeDice,
    /// Roll d20, succeed if result ≤ ability score (Cairn, Into the Odd).
    RollUnder,
    /// Roll a pool of d6; count successes ≥ threshold (Year Zero, Shadowrun).
    D6Pool {
        /// Minimum die value that counts as a success.
        threshold: u8,
    },
    /// Percentile: roll d100 vs target (Call of Cthulhu, Warhammer).
    D100,
}

/// Result of rolling dice before modifiers.
#[derive(Debug, Clone)]
pub struct DiceResult {
    /// Raw value(s) from the dice.
    pub values: Vec<i32>,
    /// Sum of the dice (for systems that care).
    pub total: i32,
}

impl DiceResult {
    /// Single die result (d20, d100).
    #[must_use]
    pub fn single(value: i32) -> Self {
        Self {
            values: vec![value],
            total: value,
        }
    }

    /// Multiple dice result (4dF, dice pool).
    #[must_use]
    pub fn multi(values: Vec<i32>) -> Self {
        let total = values.iter().sum();
        Self { values, total }
    }
}

// ---------------------------------------------------------------------------
// Outcome resolution
// ---------------------------------------------------------------------------

/// Degree of success from a check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DegreeOfSuccess {
    /// Catastrophic failure (nat 1 in `PF2e`, worst-roll in FATE).
    CriticalFailure,
    /// Standard failure (below DC).
    Failure,
    /// Partial success / tie (FATE tie, `PbtA` 7-9).
    PartialSuccess,
    /// Standard success (meet or beat DC).
    Success,
    /// Exceptional success (DC+10 in `PF2e`, succeed with style in FATE).
    CriticalSuccess,
}

impl DegreeOfSuccess {
    /// Numeric value for validation comparisons.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::CriticalFailure => -2,
            Self::Failure => -1,
            Self::PartialSuccess => 0,
            Self::Success => 1,
            Self::CriticalSuccess => 2,
        }
    }
}

// ---------------------------------------------------------------------------
// Abilities, skills, proficiency
// ---------------------------------------------------------------------------

/// A single ability score.
#[derive(Debug, Clone)]
pub struct AbilityScore {
    /// Ability name (e.g. "Strength", "Willpower").
    pub name: String,
    /// Raw value (e.g. 14 for `PF2e`, 12 for Cairn).
    pub value: i32,
    /// Derived modifier (`PF2e`: (value-10)/2; Cairn: value itself).
    pub modifier: i32,
}

impl AbilityScore {
    /// `PF2e` style: modifier = (value − 10) / 2 (round down).
    #[must_use]
    pub fn pf2e(name: &str, value: i32) -> Self {
        Self {
            name: name.into(),
            value,
            modifier: (value - 10) / 2,
        }
    }

    /// Direct-value style (FATE, Cairn): modifier = value.
    #[must_use]
    pub fn direct(name: &str, value: i32) -> Self {
        Self {
            name: name.into(),
            value,
            modifier: value,
        }
    }
}

/// Proficiency tier (`PF2e` model, generalizable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Proficiency {
    /// No formal training.
    Untrained,
    /// Basic competence.
    Trained,
    /// Advanced competence.
    Expert,
    /// High mastery.
    Master,
    /// Pinnacle of mortal skill.
    Legendary,
}

impl Proficiency {
    /// `PF2e` proficiency bonus (0, 2, 4, 6, 8) added to level.
    #[must_use]
    pub const fn bonus(self) -> i32 {
        match self {
            Self::Untrained => 0,
            Self::Trained => 2,
            Self::Expert => 4,
            Self::Master => 6,
            Self::Legendary => 8,
        }
    }
}

/// A skill with name and proficiency.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Skill name (e.g. "Perception", "Fight", "Athletics").
    pub name: String,
    /// Proficiency tier (`PF2e`).
    pub proficiency: Proficiency,
    /// Direct rating (FATE ladder: +0 Mediocre to +8 Legendary).
    pub rating: i32,
    /// Linked ability (`PF2e`: Thievery → DEX).
    pub linked_ability: Option<String>,
}

// ---------------------------------------------------------------------------
// Conditions
// ---------------------------------------------------------------------------

/// A condition affecting a character (Frightened, Wounded, Stressed...).
#[derive(Debug, Clone)]
pub struct Condition {
    /// Condition name.
    pub name: String,
    /// Severity value (e.g. Frightened 2 = value 2).
    pub value: u8,
    /// Decay per turn (`PF2e` Frightened decays by 1 each turn).
    pub decay_per_turn: u8,
    /// Turns remaining. `None` = until explicitly removed.
    pub turns_remaining: Option<u8>,
}

impl Condition {
    /// Advance one turn, applying decay. Returns true if condition is still active.
    pub const fn tick(&mut self) -> bool {
        if self.value > 0 && self.decay_per_turn > 0 {
            self.value = self.value.saturating_sub(self.decay_per_turn);
        }
        if let Some(ref mut t) = self.turns_remaining {
            *t = t.saturating_sub(1);
            if *t == 0 {
                self.value = 0;
            }
        }
        self.value > 0
    }
}

// ---------------------------------------------------------------------------
// Action economy
// ---------------------------------------------------------------------------

/// Per-turn action budget.
#[derive(Debug, Clone, Copy)]
pub struct ActionEconomy {
    /// Actions available per turn.
    pub actions: u8,
    /// Reactions available per round.
    pub reactions: u8,
    /// Free actions (unlimited in some systems).
    pub free_actions: u8,
}

impl ActionEconomy {
    /// `PF2e`: 3 actions, 1 reaction.
    pub const PF2E: Self = Self {
        actions: 3,
        reactions: 1,
        free_actions: 0,
    };

    /// FATE: one action per exchange, plus free invokes.
    pub const FATE: Self = Self {
        actions: 1,
        reactions: 0,
        free_actions: 255,
    };

    /// Cairn: 1 action per round (move and act).
    pub const CAIRN: Self = Self {
        actions: 1,
        reactions: 0,
        free_actions: 0,
    };
}

// ---------------------------------------------------------------------------
// Character
// ---------------------------------------------------------------------------

/// A character sheet — system-agnostic container.
#[derive(Debug, Clone)]
pub struct Character {
    /// Character name.
    pub name: String,
    /// Character level (0 for level-less systems).
    pub level: u8,
    /// Ability scores (6 for `PF2e`, 3 for Cairn, 0 for FATE).
    pub abilities: Vec<AbilityScore>,
    /// Skills with proficiency or rating.
    pub skills: Vec<Skill>,
    /// Active conditions.
    pub conditions: Vec<Condition>,
    /// Current hit points.
    pub hp_current: i32,
    /// Maximum hit points.
    pub hp_max: i32,
    /// Named resource tracks (stress, inventory, Fate Points).
    pub resource_tracks: HashMap<String, ResourceTrack>,
    /// Freeform tags (FATE Aspects, `PF2e` ancestry traits, Cairn background).
    pub tags: Vec<String>,
    /// Arbitrary system-specific data.
    pub metadata: HashMap<String, String>,
}

/// A depletable resource (HP, stress, inventory slots, Fate Points).
#[derive(Debug, Clone)]
pub struct ResourceTrack {
    /// Resource name.
    pub name: String,
    /// Current value.
    pub current: i32,
    /// Maximum value.
    pub max: i32,
}

// ---------------------------------------------------------------------------
// Ruleset trait
// ---------------------------------------------------------------------------

/// Summary of a ruleset's mechanical properties.
#[derive(Debug, Clone)]
pub struct RulesetSummary {
    /// Ruleset name and license.
    pub name: String,
    /// Dice resolution system.
    pub dice_system: DiceSystem,
    /// Per-turn action budget.
    pub action_economy: ActionEconomy,
    /// Number of ability scores (6 for `PF2e`, 3 for Cairn, 0 for FATE).
    pub ability_count: usize,
    /// Whether the system uses proficiency tiers.
    pub has_proficiency: bool,
    /// Whether the system uses freeform Aspects.
    pub has_aspects: bool,
    /// Number of distinct outcome tiers (4 for `PF2e`, 5 for FATE, 3 for Cairn).
    pub degree_count: usize,
    /// License identifier (ORC, CC-BY, etc.).
    pub license: String,
}

/// Core trait: any ruleset must resolve checks and describe its structure.
pub trait Ruleset {
    /// Human-readable name.
    fn name(&self) -> &str;

    /// Dice system used.
    fn dice_system(&self) -> &DiceSystem;

    /// Per-turn action budget.
    fn action_economy(&self) -> ActionEconomy;

    /// Resolve a skill check.
    ///
    /// `modifier`: total modifier (ability + proficiency + bonuses).
    /// `difficulty`: target number / DC.
    /// `roll`: dice result.
    fn resolve_check(&self, modifier: i32, difficulty: i32, roll: &DiceResult) -> DegreeOfSuccess;

    /// Build a default character for this system.
    fn default_character(&self, name: &str) -> Character;

    /// Structural summary for validation.
    fn summary(&self) -> RulesetSummary;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_result_single() {
        let r = DiceResult::single(15);
        assert_eq!(r.total, 15);
        assert_eq!(r.values.len(), 1);
    }

    #[test]
    fn dice_result_multi() {
        let r = DiceResult::multi(vec![-1, 0, 1, 1]);
        assert_eq!(r.total, 1);
        assert_eq!(r.values.len(), 4);
    }

    #[test]
    fn ability_score_pf2e_modifier() {
        let str_score = AbilityScore::pf2e("Strength", 16);
        assert_eq!(str_score.modifier, 3);

        let low = AbilityScore::pf2e("Charisma", 8);
        assert_eq!(low.modifier, -1);

        let ten = AbilityScore::pf2e("Wisdom", 10);
        assert_eq!(ten.modifier, 0);
    }

    #[test]
    fn proficiency_bonus_ordering() {
        assert!(Proficiency::Untrained < Proficiency::Legendary);
        assert_eq!(Proficiency::Trained.bonus(), 2);
        assert_eq!(Proficiency::Legendary.bonus(), 8);
    }

    #[test]
    fn condition_decays() {
        let mut frightened = Condition {
            name: "Frightened".into(),
            value: 3,
            decay_per_turn: 1,
            turns_remaining: None,
        };
        assert!(frightened.tick()); // 3 → 2, still active
        assert_eq!(frightened.value, 2);
        assert!(frightened.tick()); // 2 → 1, still active
        assert_eq!(frightened.value, 1);
        assert!(!frightened.tick()); // 1 → 0, no longer active
        assert_eq!(frightened.value, 0);
    }

    #[test]
    fn condition_timed_removal() {
        let mut stunned = Condition {
            name: "Stunned".into(),
            value: 1,
            decay_per_turn: 0,
            turns_remaining: Some(2),
        };
        assert!(stunned.tick());
        assert!(!stunned.tick());
    }

    #[test]
    fn degree_of_success_ordering() {
        assert!(DegreeOfSuccess::CriticalFailure < DegreeOfSuccess::Failure);
        assert!(DegreeOfSuccess::PartialSuccess < DegreeOfSuccess::Success);
        assert!(DegreeOfSuccess::Success < DegreeOfSuccess::CriticalSuccess);
    }

    #[test]
    fn action_economy_pf2e() {
        let ae = ActionEconomy::PF2E;
        assert_eq!(ae.actions, 3);
        assert_eq!(ae.reactions, 1);
    }

    #[test]
    fn action_economy_fate() {
        let ae = ActionEconomy::FATE;
        assert_eq!(ae.actions, 1);
        assert_eq!(ae.reactions, 0);
        assert_eq!(ae.free_actions, 255);
    }

    #[test]
    fn action_economy_cairn() {
        let ae = ActionEconomy::CAIRN;
        assert_eq!(ae.actions, 1);
        assert_eq!(ae.reactions, 0);
        assert_eq!(ae.free_actions, 0);
    }

    #[test]
    fn ability_score_direct_modifier_equals_value() {
        let courage = AbilityScore::direct("Courage", 3);
        assert_eq!(courage.modifier, 3);
        assert_eq!(courage.value, 3);
    }

    #[test]
    fn degree_of_success_as_i32() {
        assert_eq!(DegreeOfSuccess::CriticalFailure.as_i32(), -2);
        assert_eq!(DegreeOfSuccess::Failure.as_i32(), -1);
        assert_eq!(DegreeOfSuccess::PartialSuccess.as_i32(), 0);
        assert_eq!(DegreeOfSuccess::Success.as_i32(), 1);
        assert_eq!(DegreeOfSuccess::CriticalSuccess.as_i32(), 2);
    }

    #[test]
    fn proficiency_all_bonuses() {
        assert_eq!(Proficiency::Untrained.bonus(), 0);
        assert_eq!(Proficiency::Trained.bonus(), 2);
        assert_eq!(Proficiency::Expert.bonus(), 4);
        assert_eq!(Proficiency::Master.bonus(), 6);
        assert_eq!(Proficiency::Legendary.bonus(), 8);
    }

    #[test]
    fn resource_track_creation() {
        let rt = ResourceTrack {
            name: "Fate Points".into(),
            current: 3,
            max: 5,
        };
        assert_eq!(rt.current, 3);
        assert_eq!(rt.max, 5);
    }

    #[test]
    fn condition_no_decay_no_timer_stays_active() {
        let mut persistent = Condition {
            name: "Persistent Damage".into(),
            value: 5,
            decay_per_turn: 0,
            turns_remaining: None,
        };
        assert!(persistent.tick());
        assert_eq!(persistent.value, 5);
    }

    #[test]
    fn dice_system_d6_pool() {
        let ds = DiceSystem::D6Pool { threshold: 5 };
        assert_eq!(format!("{ds:?}"), "D6Pool { threshold: 5 }");
    }

    #[test]
    fn dice_system_d100() {
        let ds = DiceSystem::D100;
        assert_eq!(format!("{ds:?}"), "D100");
    }

    #[test]
    fn ruleset_summary_struct_fields() {
        let summary = RulesetSummary {
            name: "Test System".into(),
            dice_system: DiceSystem::D20,
            action_economy: ActionEconomy::PF2E,
            ability_count: 6,
            has_proficiency: true,
            has_aspects: false,
            degree_count: 4,
            license: "ORC".into(),
        };
        assert_eq!(summary.name, "Test System");
        assert_eq!(summary.ability_count, 6);
        assert!(summary.has_proficiency);
        assert!(!summary.has_aspects);
    }

    #[test]
    fn skill_creation() {
        let skill = Skill {
            name: "Athletics".into(),
            proficiency: Proficiency::Expert,
            rating: 4,
            linked_ability: Some("Strength".into()),
        };
        assert_eq!(skill.proficiency.bonus(), 4);
        assert_eq!(skill.rating, 4);
    }

    #[test]
    fn character_creation() {
        let mut c = Character {
            name: "Valeros".into(),
            level: 5,
            abilities: vec![AbilityScore::pf2e("Strength", 18)],
            skills: vec![],
            conditions: vec![],
            hp_current: 60,
            hp_max: 60,
            resource_tracks: HashMap::new(),
            tags: vec!["Human".into(), "Fighter".into()],
            metadata: HashMap::new(),
        };
        assert_eq!(c.abilities[0].modifier, 4);
        c.resource_tracks.insert(
            "focus".into(),
            ResourceTrack {
                name: "Focus Points".into(),
                current: 2,
                max: 3,
            },
        );
        assert_eq!(c.resource_tracks["focus"].max, 3);
    }
}
