// SPDX-License-Identifier: AGPL-3.0-or-later
//! Planes — game modes as swappable rulesets on a continuous DAG.
//!
//! Each plane governs how new vertices are added to the rhizoCrypt session DAG.
//! Plane transitions are themselves DAG vertices, preserving world state across
//! mode shifts.

use super::super::ruleset::{ActionEconomy, DiceSystem};

/// The seven planes of play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaneType {
    /// Open world, free movement, ambient discovery. Cairn rules.
    Exploration,
    /// Skill-as-voice, persuasion, deception, empathy. Custom Disco Elysium model.
    Dialogue,
    /// Grid/zone combat, action economy, positioning. PF2e ORC.
    Tactical,
    /// Clue gathering, evidence DAG, deduction chains. GUMSHOE-inspired.
    Investigation,
    /// Faction reputation, alliances, betrayal. FATE Core Aspects.
    Political,
    /// Material transformation, recipe discovery, alchemy. Reaction kinetics.
    Crafting,
    /// Zone management, stack resolution, resource economy. MTG rules.
    CardStack,
}

impl PlaneType {
    /// Human-readable label for this plane.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Exploration => "Exploration",
            Self::Dialogue => "Dialogue",
            Self::Tactical => "Tactical",
            Self::Investigation => "Investigation",
            Self::Political => "Political",
            Self::Crafting => "Crafting",
            Self::CardStack => "Card/Stack",
        }
    }

    /// Default dice system for this plane.
    #[must_use]
    pub const fn default_dice_system(self) -> DiceSystem {
        match self {
            Self::Exploration => DiceSystem::RollUnder,
            Self::Dialogue | Self::Investigation => DiceSystem::D6Pool { threshold: 4 },
            Self::Tactical | Self::Crafting | Self::CardStack => DiceSystem::D20,
            Self::Political => DiceSystem::FudgeDice,
        }
    }

    /// Default action economy for this plane.
    #[must_use]
    pub const fn default_action_economy(self) -> ActionEconomy {
        match self {
            Self::Exploration => ActionEconomy::CAIRN,
            Self::Dialogue | Self::Investigation | Self::CardStack => ActionEconomy {
                actions: 0,
                reactions: 0,
                free_actions: 255,
            },
            Self::Tactical => ActionEconomy::PF2E,
            Self::Political => ActionEconomy::FATE,
            Self::Crafting => ActionEconomy {
                actions: 1,
                reactions: 0,
                free_actions: 0,
            },
        }
    }
}

/// Pacing hint for AI narration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NarrationPacing {
    /// Slow, atmospheric, exploratory.
    Slow,
    /// Measured, thoughtful, conversational.
    Measured,
    /// Urgent, tense, time-pressured.
    Urgent,
    /// Frantic, chaotic, overwhelming.
    Frantic,
}

/// Narration style guide for a plane.
#[derive(Debug, Clone)]
pub struct NarrationGuide {
    /// Descriptive tone hint.
    pub tone: String,
    /// Pacing hint.
    pub pacing: NarrationPacing,
    /// Vocabulary register (literary, colloquial, technical, archaic).
    pub vocabulary: String,
    /// Narrative perspective (second_person, third_person).
    pub perspective: String,
    /// Senses to prioritize in descriptions.
    pub sensory_emphasis: Vec<String>,
}

/// Resolution method for skill checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionMethod {
    /// Four degrees: crit fail / fail / success / crit success (PF2e).
    DegreeOfSuccess,
    /// Binary: pass or fail (Cairn).
    BinaryPass,
    /// Three tiers: fail / partial / success (PbtA, FATE ties).
    PartialSuccess,
    /// Opposed roll (contested checks).
    Contested,
    /// No roll — automatic success for core actions (GUMSHOE core clues).
    Automatic,
}

/// A machine-readable constraint the AI must respect.
#[derive(Debug, Clone)]
pub struct RuleConstraint {
    /// The rule as a human-readable string.
    pub rule: String,
    /// Whether the constraint is hard (enforced) or soft (advisory).
    pub enforcement: ConstraintEnforcement,
    /// Citation for the rule's origin.
    pub source: Option<String>,
}

/// Enforcement level for a rule constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintEnforcement {
    /// The system mechanically prevents violation.
    Hard,
    /// The AI should follow but the system does not block violations.
    Soft,
}

/// A passive check that triggers a voice without player action.
#[derive(Debug, Clone)]
pub struct PassiveCheck {
    /// Which voice/skill triggers.
    pub skill: String,
    /// Condition that activates the check.
    pub trigger_condition: String,
    /// Difficulty class.
    pub dc: u8,
    /// Priority level for display ordering.
    pub priority: PassiveCheckPriority,
    /// Hint text when the check succeeds.
    pub on_success_hint: String,
}

/// Priority for passive check display ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PassiveCheckPriority {
    /// Fires if no higher-priority voices fired.
    Low = 0,
    /// Fires if no high-priority voices fired.
    Medium = 1,
    /// Fires if check succeeds, appended after narration.
    High = 2,
    /// Always fires if check succeeds, interrupts narration.
    Critical = 3,
}

/// An action template available in a plane.
#[derive(Debug, Clone)]
pub struct ActionTemplate {
    /// Machine identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Action cost (0 = free/unlimited).
    pub cost: u8,
    /// Required skill or condition.
    pub requires: Option<String>,
    /// Vertex type produced in the DAG.
    pub produces: String,
    /// AI narration hint.
    pub description: String,
}

/// A complete ruleset certificate for a plane.
#[derive(Debug, Clone)]
pub struct RulesetCert {
    /// Which plane this ruleset governs.
    pub plane: PlaneType,
    /// Dice resolution system.
    pub dice_system: DiceSystem,
    /// Action economy.
    pub action_economy: ActionEconomy,
    /// How checks resolve.
    pub resolution: ResolutionMethod,
    /// Available actions in this plane.
    pub available_actions: Vec<ActionTemplate>,
    /// Passive checks that trigger voices.
    pub passive_checks: Vec<PassiveCheck>,
    /// Hard and soft rule constraints.
    pub constraints: Vec<RuleConstraint>,
    /// Narration style guide.
    pub narration_style: NarrationGuide,
}

/// A plane transition recorded as a DAG vertex.
#[derive(Debug, Clone)]
pub struct PlaneTransition {
    /// Plane being left.
    pub from: PlaneType,
    /// Plane being entered.
    pub to: PlaneType,
    /// What triggered the transition.
    pub trigger: String,
    /// Hash of world state at transition point.
    pub world_state_hash: Option<String>,
}

impl PlaneTransition {
    /// Create a new plane transition.
    #[must_use]
    pub fn new(from: PlaneType, to: PlaneType, trigger: impl Into<String>) -> Self {
        Self {
            from,
            to,
            trigger: trigger.into(),
            world_state_hash: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_planes_have_labels() {
        let planes = [
            PlaneType::Exploration,
            PlaneType::Dialogue,
            PlaneType::Tactical,
            PlaneType::Investigation,
            PlaneType::Political,
            PlaneType::Crafting,
            PlaneType::CardStack,
        ];
        for p in planes {
            assert!(!p.label().is_empty(), "plane {p:?} has empty label");
        }
    }

    #[test]
    fn seven_planes_distinct() {
        let planes = [
            PlaneType::Exploration,
            PlaneType::Dialogue,
            PlaneType::Tactical,
            PlaneType::Investigation,
            PlaneType::Political,
            PlaneType::Crafting,
            PlaneType::CardStack,
        ];
        for (i, a) in planes.iter().enumerate() {
            for (j, b) in planes.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn tactical_uses_pf2e_three_action() {
        let ae = PlaneType::Tactical.default_action_economy();
        assert_eq!(ae.actions, 3);
        assert_eq!(ae.reactions, 1);
    }

    #[test]
    fn dialogue_is_freeform() {
        let ae = PlaneType::Dialogue.default_action_economy();
        assert_eq!(ae.actions, 0);
        assert_eq!(ae.free_actions, 255);
    }

    #[test]
    fn exploration_uses_roll_under() {
        assert_eq!(
            PlaneType::Exploration.default_dice_system(),
            DiceSystem::RollUnder
        );
    }

    #[test]
    fn political_uses_fudge_dice() {
        assert_eq!(
            PlaneType::Political.default_dice_system(),
            DiceSystem::FudgeDice
        );
    }

    #[test]
    fn plane_transition_records_trigger() {
        let t = PlaneTransition::new(PlaneType::Dialogue, PlaneType::Tactical, "guard_alerted");
        assert_eq!(t.from, PlaneType::Dialogue);
        assert_eq!(t.to, PlaneType::Tactical);
        assert_eq!(t.trigger, "guard_alerted");
        assert!(t.world_state_hash.is_none());
    }

    #[test]
    fn passive_check_priority_ordering() {
        assert!(PassiveCheckPriority::Low < PassiveCheckPriority::Medium);
        assert!(PassiveCheckPriority::Medium < PassiveCheckPriority::High);
        assert!(PassiveCheckPriority::High < PassiveCheckPriority::Critical);
    }

    #[test]
    fn constraint_enforcement_variants() {
        assert_ne!(ConstraintEnforcement::Hard, ConstraintEnforcement::Soft);
    }

    #[test]
    fn resolution_method_variants() {
        let methods = [
            ResolutionMethod::DegreeOfSuccess,
            ResolutionMethod::BinaryPass,
            ResolutionMethod::PartialSuccess,
            ResolutionMethod::Contested,
            ResolutionMethod::Automatic,
        ];
        assert_eq!(methods.len(), 5);
    }
}
