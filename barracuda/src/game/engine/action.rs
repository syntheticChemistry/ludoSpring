// SPDX-License-Identifier: AGPL-3.0-or-later
//! Action types and resolution pipeline.
//!
//! Every player input becomes a `Command`. Commands are validated against
//! the current plane's ruleset, resolved (possibly involving dice rolls,
//! skill checks, or AI narration), and produce `ActionOutcome`s that
//! update the world and entity state.
//!
//! The pipeline is: Input → Command → Validate → Resolve → Outcome → Apply.
//! Each stage is pure and testable. Side effects (rendering, narration,
//! provenance) happen *after* the outcome is determined.

use super::entity::EntityId;
use super::world::Direction;

/// A command from the player or an NPC.
///
/// Commands are plane-agnostic at the structural level — every plane
/// can express movement, interaction, and communication. The ruleset
/// determines which commands are legal and how they resolve.
#[derive(Debug, Clone)]
pub enum Command {
    /// Move in a direction (all planes).
    Move {
        /// Entity to move.
        entity: EntityId,
        /// Direction to move in.
        direction: Direction,
    },

    /// Interact with an adjacent entity (examine, open, pick up).
    Interact {
        /// Entity performing the interaction.
        actor: EntityId,
        /// Entity being interacted with.
        target: EntityId,
    },

    /// Start or continue a conversation (Dialogue plane).
    Talk {
        /// Entity initiating the conversation.
        actor: EntityId,
        /// NPC being spoken to.
        target: EntityId,
        /// Optional dialogue choice ID when advancing.
        choice_id: Option<String>,
    },

    /// Use an item from inventory on a target or self.
    UseItem {
        /// Entity using the item.
        actor: EntityId,
        /// Name of the item.
        item_name: String,
        /// Optional target entity (or self if None).
        target: Option<EntityId>,
    },

    /// Examine something (get description, trigger Investigation clues).
    Examine {
        /// Entity performing the examination.
        actor: EntityId,
        /// What to examine (entity, tile, or direction).
        target: ExamineTarget,
    },

    /// Attack a target (Tactical plane).
    Attack {
        /// Entity performing the attack.
        actor: EntityId,
        /// Entity being attacked.
        target: EntityId,
        /// Optional weapon name.
        weapon: Option<String>,
    },

    /// End turn voluntarily (Tactical plane action economy).
    EndTurn {
        /// Entity ending its turn.
        entity: EntityId,
    },

    /// Wait / pass (costs an action in some systems, free in others).
    Wait {
        /// Entity waiting.
        entity: EntityId,
    },

    /// Custom command for domain-specific actions.
    Custom {
        /// Entity performing the action.
        actor: EntityId,
        /// Verb or action name.
        verb: String,
        /// Additional arguments.
        args: Vec<String>,
    },
}

impl Command {
    /// Machine-readable verb for ruleset validation.
    #[must_use]
    pub fn verb(&self) -> &str {
        match self {
            Self::Move { .. } => "move",
            Self::Interact { .. } => "interact",
            Self::Talk { .. } => "talk",
            Self::UseItem { .. } => "use_item",
            Self::Examine { .. } => "examine",
            Self::Attack { .. } => "attack",
            Self::EndTurn { .. } => "end_turn",
            Self::Wait { .. } => "wait",
            Self::Custom { verb, .. } => verb,
        }
    }
}

/// What to examine — an entity, a tile, or a direction.
#[derive(Debug, Clone)]
pub enum ExamineTarget {
    /// Examine a specific entity.
    Entity(EntityId),
    /// Examine the tile at a position.
    Tile(u32, u32),
    /// Look in a direction (narrate what's visible).
    Direction(Direction),
}

/// Outcome of resolving a command against the ruleset.
#[derive(Debug, Clone)]
pub struct ActionOutcome {
    /// What happened (for narration and state updates).
    pub effect: Effect,
    /// Whether the action consumed the actor's turn/action.
    pub cost: ActionCost,
    /// Optional narration text (filled by AI or template).
    pub narration: Option<String>,
    /// Triggered events (conversation start, encounter, etc.).
    pub triggers: Vec<TriggerEvent>,
}

/// The concrete effect of an action on the world.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Entity moved to a new position.
    Moved {
        /// Entity that moved.
        entity: EntityId,
        /// New X coordinate.
        to_x: u32,
        /// New Y coordinate.
        to_y: u32,
    },

    /// Entity interacted with target (door opened, item picked up, etc.).
    Interacted {
        /// Entity that performed the interaction.
        actor: EntityId,
        /// Entity that was interacted with.
        target: EntityId,
        /// Description of the result.
        result: String,
    },

    /// Conversation started or advanced.
    DialogueAdvanced {
        /// NPC speaking.
        speaker: EntityId,
        /// Entity being spoken to.
        listener: EntityId,
        /// Exchange or dialogue node ID.
        exchange_id: String,
    },

    /// Damage dealt.
    Damaged {
        /// Entity that took damage.
        target: EntityId,
        /// Amount of damage.
        amount: i32,
        /// Source of the damage (e.g. "attack", "trap").
        source: String,
    },

    /// Item acquired by actor.
    ItemAcquired {
        /// Entity that acquired the item.
        actor: EntityId,
        /// Name of the item.
        item_name: String,
    },

    /// Item consumed/used.
    ItemUsed {
        /// Entity that used the item.
        actor: EntityId,
        /// Name of the item.
        item_name: String,
        /// Description of the result.
        result: String,
    },

    /// Information revealed (clue found, secret learned).
    Revealed {
        /// Entity that discovered the information.
        actor: EntityId,
        /// The revealed information.
        info: String,
    },

    /// Nothing happened (blocked, invalid, no effect).
    NoEffect {
        /// Reason the action had no effect.
        reason: String,
    },

    /// Entity's turn ended.
    TurnEnded {
        /// Entity whose turn ended.
        entity: EntityId,
    },
}

/// How much the action costs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCost {
    /// Free action (no cost).
    Free,
    /// One action (PF2e: 1 of 3; Cairn: the entire turn).
    One,
    /// Two actions (PF2e multi-action activities).
    Two,
    /// Three actions (full turn in PF2e).
    Three,
    /// Reaction (PF2e: 1 per round).
    Reaction,
}

impl ActionCost {
    /// Numeric cost for budget tracking.
    #[must_use]
    pub const fn actions(self) -> u8 {
        match self {
            Self::Free | Self::Reaction => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }
}

/// An event triggered as a side effect of an action.
#[derive(Debug, Clone)]
pub enum TriggerEvent {
    /// Start a conversation with an NPC.
    ConversationStart {
        /// NPC to converse with.
        npc: EntityId,
    },
    /// Encounter begins (transition to Tactical plane).
    EncounterStart {
        /// Hostile entities in the encounter.
        hostiles: Vec<EntityId>,
    },
    /// Clue discovered (Investigation plane).
    ClueFound {
        /// ID of the clue.
        clue_id: String,
    },
    /// Zone transition (move to a different area/map).
    ZoneTransition {
        /// Name of the destination zone.
        zone_name: String,
    },
    /// Plane transition (swap active ruleset).
    PlaneTransition {
        /// Target plane name.
        to_plane: String,
        /// What triggered the transition.
        trigger: String,
    },
    /// Custom event.
    Custom {
        /// Event name.
        name: String,
        /// Event payload.
        data: String,
    },
}

/// Tracks remaining actions in a turn (for action-economy systems).
#[derive(Debug, Clone)]
pub struct TurnBudget {
    /// Actions remaining this turn.
    pub actions_remaining: u8,
    /// Reactions remaining this round.
    pub reactions_remaining: u8,
    /// Whether a free action has been used this turn.
    pub free_used: bool,
}

impl TurnBudget {
    /// Create from an action economy definition.
    #[must_use]
    pub const fn from_economy(economy: &crate::game::ruleset::ActionEconomy) -> Self {
        Self {
            actions_remaining: economy.actions,
            reactions_remaining: economy.reactions,
            free_used: false,
        }
    }

    /// Whether the actor can afford this cost.
    #[must_use]
    pub const fn can_afford(&self, cost: ActionCost) -> bool {
        match cost {
            ActionCost::Free => true,
            ActionCost::One => self.actions_remaining >= 1,
            ActionCost::Two => self.actions_remaining >= 2,
            ActionCost::Three => self.actions_remaining >= 3,
            ActionCost::Reaction => self.reactions_remaining >= 1,
        }
    }

    /// Spend the cost. Returns `false` if insufficient budget.
    pub const fn spend(&mut self, cost: ActionCost) -> bool {
        if !self.can_afford(cost) {
            return false;
        }
        match cost {
            ActionCost::Free => self.free_used = true,
            ActionCost::One => self.actions_remaining -= 1,
            ActionCost::Two => self.actions_remaining -= 2,
            ActionCost::Three => self.actions_remaining -= 3,
            ActionCost::Reaction => self.reactions_remaining -= 1,
        }
        true
    }

    /// Whether the turn is exhausted (no actions remaining).
    #[must_use]
    pub const fn is_exhausted(&self) -> bool {
        self.actions_remaining == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ruleset::ActionEconomy;

    #[test]
    fn action_cost_values() {
        assert_eq!(ActionCost::Free.actions(), 0);
        assert_eq!(ActionCost::One.actions(), 1);
        assert_eq!(ActionCost::Three.actions(), 3);
    }

    #[test]
    fn turn_budget_pf2e() {
        let mut budget = TurnBudget::from_economy(&ActionEconomy::PF2E);
        assert_eq!(budget.actions_remaining, 3);
        assert_eq!(budget.reactions_remaining, 1);

        assert!(budget.can_afford(ActionCost::Two));
        assert!(budget.spend(ActionCost::Two));
        assert_eq!(budget.actions_remaining, 1);

        assert!(!budget.can_afford(ActionCost::Two));
        assert!(budget.spend(ActionCost::One));
        assert!(budget.is_exhausted());
    }

    #[test]
    fn turn_budget_cairn() {
        let mut budget = TurnBudget::from_economy(&ActionEconomy::CAIRN);
        assert_eq!(budget.actions_remaining, 1);
        assert!(budget.spend(ActionCost::One));
        assert!(budget.is_exhausted());
    }

    #[test]
    fn reaction_spending() {
        let mut budget = TurnBudget::from_economy(&ActionEconomy::PF2E);
        assert!(budget.can_afford(ActionCost::Reaction));
        assert!(budget.spend(ActionCost::Reaction));
        assert!(!budget.can_afford(ActionCost::Reaction));
        // Actions unaffected
        assert_eq!(budget.actions_remaining, 3);
    }

    #[test]
    fn command_variants() {
        let move_cmd = Command::Move {
            entity: EntityId(0),
            direction: Direction::North,
        };
        assert!(matches!(move_cmd, Command::Move { .. }));

        let talk_cmd = Command::Talk {
            actor: EntityId(0),
            target: EntityId(1),
            choice_id: Some("ask_about_disappearances".into()),
        };
        assert!(matches!(talk_cmd, Command::Talk { .. }));
    }

    #[test]
    fn effect_variants() {
        let moved = Effect::Moved {
            entity: EntityId(0),
            to_x: 5,
            to_y: 3,
        };
        assert!(matches!(moved, Effect::Moved { to_x: 5, .. }));

        let revealed = Effect::Revealed {
            actor: EntityId(0),
            info: "The bookshelf conceals a passage.".into(),
        };
        assert!(matches!(revealed, Effect::Revealed { .. }));
    }

    #[test]
    fn trigger_events() {
        let conv = TriggerEvent::ConversationStart {
            npc: EntityId(5),
        };
        assert!(matches!(conv, TriggerEvent::ConversationStart { .. }));

        let plane = TriggerEvent::PlaneTransition {
            to_plane: "Tactical".into(),
            trigger: "guard_alerted".into(),
        };
        assert!(matches!(plane, TriggerEvent::PlaneTransition { .. }));
    }

    #[test]
    fn outcome_composition() {
        let outcome = ActionOutcome {
            effect: Effect::Moved {
                entity: EntityId(0),
                to_x: 4,
                to_y: 3,
            },
            cost: ActionCost::One,
            narration: Some("You step cautiously into the dim corridor.".into()),
            triggers: vec![TriggerEvent::ConversationStart {
                npc: EntityId(2),
            }],
        };
        assert_eq!(outcome.triggers.len(), 1);
        assert!(outcome.narration.is_some());
    }
}
