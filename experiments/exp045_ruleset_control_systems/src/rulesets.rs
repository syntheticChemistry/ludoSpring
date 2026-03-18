// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ruleset implementations for exp045 — Pathfinder 2e, FATE Core, Cairn.
//!
//! Three structurally different open RPG rulesets implemented as control
//! systems, proving the type model handles d20, Fudge dice, and roll-under
//! through the same interface.

use std::collections::HashMap;

use ludospring_barracuda::game::ruleset::{
    AbilityScore, ActionEconomy, Character, DegreeOfSuccess, DiceResult, DiceSystem, Proficiency,
    ResourceTrack, Ruleset, RulesetSummary, Skill,
};

// ===========================================================================
// Pathfinder 2e (ORC License — open mechanical content)
// ===========================================================================

pub struct Pathfinder2e;

impl Ruleset for Pathfinder2e {
    fn name(&self) -> &'static str {
        "Pathfinder 2e (ORC)"
    }

    fn dice_system(&self) -> &DiceSystem {
        &DiceSystem::D20
    }

    fn action_economy(&self) -> ActionEconomy {
        ActionEconomy::PF2E
    }

    fn resolve_check(&self, modifier: i32, difficulty: i32, roll: &DiceResult) -> DegreeOfSuccess {
        let raw = roll.values[0];
        let total = raw + modifier;
        let delta = total - difficulty;

        // Natural 20/1 shift the degree by one step.
        let base_degree = if delta >= 10 {
            DegreeOfSuccess::CriticalSuccess
        } else if delta >= 0 {
            DegreeOfSuccess::Success
        } else if delta >= -10 {
            DegreeOfSuccess::Failure
        } else {
            DegreeOfSuccess::CriticalFailure
        };

        if raw == 20 {
            promote(base_degree)
        } else if raw == 1 {
            demote(base_degree)
        } else {
            base_degree
        }
    }

    fn default_character(&self, name: &str) -> Character {
        Character {
            name: name.into(),
            level: 1,
            abilities: vec![
                AbilityScore::pf2e("Strength", 14),
                AbilityScore::pf2e("Dexterity", 12),
                AbilityScore::pf2e("Constitution", 12),
                AbilityScore::pf2e("Intelligence", 10),
                AbilityScore::pf2e("Wisdom", 14),
                AbilityScore::pf2e("Charisma", 10),
            ],
            skills: vec![
                Skill {
                    name: "Perception".into(),
                    proficiency: Proficiency::Trained,
                    rating: 0,
                    linked_ability: Some("Wisdom".into()),
                },
                Skill {
                    name: "Athletics".into(),
                    proficiency: Proficiency::Trained,
                    rating: 0,
                    linked_ability: Some("Strength".into()),
                },
                Skill {
                    name: "Thievery".into(),
                    proficiency: Proficiency::Untrained,
                    rating: 0,
                    linked_ability: Some("Dexterity".into()),
                },
            ],
            conditions: Vec::new(),
            hp_current: 20,
            hp_max: 20,
            resource_tracks: HashMap::new(),
            tags: vec!["Human".into(), "Fighter".into()],
            metadata: HashMap::new(),
        }
    }

    fn summary(&self) -> RulesetSummary {
        RulesetSummary {
            name: self.name().into(),
            dice_system: DiceSystem::D20,
            action_economy: self.action_economy(),
            ability_count: 6,
            has_proficiency: true,
            has_aspects: false,
            degree_count: 4,
            license: "ORC (irrevocable)".into(),
        }
    }
}

/// `PF2e`: calculate total modifier for a skill check.
pub fn pf2e_skill_modifier(character: &Character, skill_name: &str) -> i32 {
    let skill = character
        .skills
        .iter()
        .find(|s| s.name == skill_name)
        .expect("skill not found");
    let ability_mod = skill
        .linked_ability
        .as_ref()
        .and_then(|a| character.abilities.iter().find(|ab| ab.name == *a))
        .map_or(0, |ab| ab.modifier);
    let prof_bonus = if skill.proficiency == Proficiency::Untrained {
        0
    } else {
        i32::from(character.level) + skill.proficiency.bonus()
    };
    ability_mod + prof_bonus
}

// ===========================================================================
// FATE Core (CC-BY — Evil Hat Productions)
// ===========================================================================

pub struct FateCore;

impl Ruleset for FateCore {
    fn name(&self) -> &'static str {
        "FATE Core (CC-BY)"
    }

    fn dice_system(&self) -> &DiceSystem {
        &DiceSystem::FudgeDice
    }

    fn action_economy(&self) -> ActionEconomy {
        ActionEconomy::FATE
    }

    fn resolve_check(&self, modifier: i32, difficulty: i32, roll: &DiceResult) -> DegreeOfSuccess {
        let total = roll.total + modifier;
        let shifts = total - difficulty;

        match shifts {
            s if s >= 3 => DegreeOfSuccess::CriticalSuccess,
            s if s >= 1 => DegreeOfSuccess::Success,
            0 => DegreeOfSuccess::PartialSuccess,
            s if s >= -2 => DegreeOfSuccess::Failure,
            _ => DegreeOfSuccess::CriticalFailure,
        }
    }

    fn default_character(&self, name: &str) -> Character {
        let mut resource_tracks = HashMap::new();
        resource_tracks.insert(
            "Physical Stress".into(),
            ResourceTrack {
                name: "Physical Stress".into(),
                current: 2,
                max: 2,
            },
        );
        resource_tracks.insert(
            "Mental Stress".into(),
            ResourceTrack {
                name: "Mental Stress".into(),
                current: 2,
                max: 2,
            },
        );
        resource_tracks.insert(
            "Fate Points".into(),
            ResourceTrack {
                name: "Fate Points".into(),
                current: 3,
                max: 3,
            },
        );

        Character {
            name: name.into(),
            level: 0,
            abilities: Vec::new(),
            skills: vec![
                Skill {
                    name: "Fight".into(),
                    proficiency: Proficiency::Untrained,
                    rating: 3,
                    linked_ability: None,
                },
                Skill {
                    name: "Investigate".into(),
                    proficiency: Proficiency::Untrained,
                    rating: 2,
                    linked_ability: None,
                },
                Skill {
                    name: "Athletics".into(),
                    proficiency: Proficiency::Untrained,
                    rating: 2,
                    linked_ability: None,
                },
                Skill {
                    name: "Will".into(),
                    proficiency: Proficiency::Untrained,
                    rating: 1,
                    linked_ability: None,
                },
                Skill {
                    name: "Notice".into(),
                    proficiency: Proficiency::Untrained,
                    rating: 1,
                    linked_ability: None,
                },
            ],
            conditions: Vec::new(),
            hp_current: 0,
            hp_max: 0,
            resource_tracks,
            tags: vec![
                "Haunted by the Ghost of My Mentor".into(),
                "Last of the Iron Legion".into(),
                "I Never Leave a Friend Behind".into(),
            ],
            metadata: HashMap::new(),
        }
    }

    fn summary(&self) -> RulesetSummary {
        RulesetSummary {
            name: self.name().into(),
            dice_system: DiceSystem::FudgeDice,
            action_economy: self.action_economy(),
            ability_count: 0,
            has_proficiency: false,
            has_aspects: true,
            degree_count: 5,
            license: "CC-BY 3.0".into(),
        }
    }
}

/// FATE: skill rating is the modifier directly.
pub fn fate_skill_modifier(character: &Character, skill_name: &str) -> i32 {
    character
        .skills
        .iter()
        .find(|s| s.name == skill_name)
        .map_or(0, |s| s.rating)
}

// ===========================================================================
// Cairn (CC-BY-SA — Yochai Gal)
// ===========================================================================

pub struct Cairn;

impl Ruleset for Cairn {
    fn name(&self) -> &'static str {
        "Cairn (CC-BY-SA)"
    }

    fn dice_system(&self) -> &DiceSystem {
        &DiceSystem::RollUnder
    }

    fn action_economy(&self) -> ActionEconomy {
        ActionEconomy::CAIRN
    }

    fn resolve_check(&self, modifier: i32, _difficulty: i32, roll: &DiceResult) -> DegreeOfSuccess {
        // Cairn: roll d20, succeed if roll ≤ ability score (modifier = ability value).
        let raw = roll.values[0];
        if raw <= modifier {
            if raw == 1 {
                DegreeOfSuccess::CriticalSuccess
            } else {
                DegreeOfSuccess::Success
            }
        } else if raw == 20 {
            DegreeOfSuccess::CriticalFailure
        } else {
            DegreeOfSuccess::Failure
        }
    }

    fn default_character(&self, name: &str) -> Character {
        let mut resource_tracks = HashMap::new();
        resource_tracks.insert(
            "Inventory Slots".into(),
            ResourceTrack {
                name: "Inventory Slots".into(),
                current: 10,
                max: 10,
            },
        );

        Character {
            name: name.into(),
            level: 0,
            abilities: vec![
                AbilityScore::direct("Strength", 12),
                AbilityScore::direct("Dexterity", 9),
                AbilityScore::direct("Willpower", 14),
            ],
            skills: Vec::new(),
            conditions: Vec::new(),
            hp_current: 4,
            hp_max: 4,
            resource_tracks,
            tags: vec!["Herbalist".into()],
            metadata: HashMap::new(),
        }
    }

    fn summary(&self) -> RulesetSummary {
        RulesetSummary {
            name: self.name().into(),
            dice_system: DiceSystem::RollUnder,
            action_economy: self.action_economy(),
            ability_count: 3,
            has_proficiency: false,
            has_aspects: false,
            degree_count: 3,
            license: "CC-BY-SA 4.0".into(),
        }
    }
}

/// Cairn: ability score is the target for roll-under.
pub fn cairn_ability_target(character: &Character, ability_name: &str) -> i32 {
    character
        .abilities
        .iter()
        .find(|a| a.name == ability_name)
        .map_or(10, |a| a.value)
}

// ===========================================================================
// Helpers
// ===========================================================================

#[expect(
    clippy::match_same_arms,
    reason = "explicit arm per variant for clarity"
)]
const fn promote(d: DegreeOfSuccess) -> DegreeOfSuccess {
    match d {
        DegreeOfSuccess::CriticalFailure => DegreeOfSuccess::Failure,
        DegreeOfSuccess::Failure => DegreeOfSuccess::Success,
        DegreeOfSuccess::PartialSuccess => DegreeOfSuccess::Success,
        DegreeOfSuccess::Success => DegreeOfSuccess::CriticalSuccess,
        DegreeOfSuccess::CriticalSuccess => DegreeOfSuccess::CriticalSuccess,
    }
}

#[expect(
    clippy::match_same_arms,
    reason = "explicit arm per variant for clarity"
)]
const fn demote(d: DegreeOfSuccess) -> DegreeOfSuccess {
    match d {
        DegreeOfSuccess::CriticalFailure => DegreeOfSuccess::CriticalFailure,
        DegreeOfSuccess::Failure => DegreeOfSuccess::CriticalFailure,
        DegreeOfSuccess::PartialSuccess => DegreeOfSuccess::Failure,
        DegreeOfSuccess::Success => DegreeOfSuccess::Failure,
        DegreeOfSuccess::CriticalSuccess => DegreeOfSuccess::Success,
    }
}

/// Convert bool to f64 for validation comparisons (1.0 = true, 0.0 = false).
#[expect(dead_code, reason = "kept for potential non-harness use")]
pub const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}
