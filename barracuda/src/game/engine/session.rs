// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game session — ties world, entities, plane, and ruleset together.
//!
//! A `GameSession` is the top-level game state container. It owns the
//! tile world, entity registry, active plane, turn tracking, and the
//! command history (for replay and provenance). The session drives the
//! game loop: receive command → validate → resolve → apply → render.
//!
//! Sessions are plane-aware: the active plane determines which commands
//! are legal, how they resolve, and what the AI narration style is.
//! Plane transitions are first-class operations that swap the ruleset
//! while preserving the world state.

use super::action::{ActionCost, ActionOutcome, Command, Effect, TriggerEvent, TurnBudget};
use super::entity::{Entity, EntityId, EntityKind, EntityRegistry};
use super::world::TileWorld;
use crate::game::rpgpt::plane::PlaneType;

/// The game session — root of all game state.
#[derive(Debug, Clone)]
pub struct GameSession {
    /// The 2D tile world.
    pub world: TileWorld,
    /// All entities in the world.
    pub entities: EntityRegistry,
    /// Currently active game plane.
    pub active_plane: PlaneType,
    /// Current turn number (increments after each full turn cycle).
    pub turn: u32,
    /// Turn budget for the currently acting entity.
    pub turn_budget: Option<TurnBudget>,
    /// Initiative order for Tactical plane (entity IDs in order).
    pub initiative: Vec<EntityId>,
    /// Index into initiative order (who's acting now).
    pub initiative_index: usize,
    /// Command history for replay and provenance.
    pub history: Vec<ResolvedCommand>,
    /// Player's sight radius for fog of war.
    pub sight_radius: u32,
    /// Session identifier for provenance DAG.
    pub session_id: String,
}

/// A command paired with its outcome for history tracking.
#[derive(Debug, Clone)]
pub struct ResolvedCommand {
    /// The command that was issued.
    pub command: Command,
    /// The outcome after resolution.
    pub outcome: ActionOutcome,
    /// Turn number when this was resolved.
    pub turn: u32,
}

impl GameSession {
    /// Create a new session in the given plane.
    #[must_use]
    pub fn new(world: TileWorld, plane: PlaneType) -> Self {
        Self {
            world,
            entities: EntityRegistry::new(),
            active_plane: plane,
            turn: 0,
            turn_budget: None,
            initiative: Vec::new(),
            initiative_index: 0,
            history: Vec::new(),
            sight_radius: 5,
            session_id: String::new(),
        }
    }

    /// Spawn an entity into the session. Returns its ID.
    pub fn spawn(&mut self, entity: Entity) -> EntityId {
        self.entities.spawn(entity)
    }

    /// Process a command through the resolution pipeline.
    ///
    /// Returns the outcome. The caller is responsible for rendering
    /// the outcome via petalTongue and routing narration to Squirrel.
    pub fn process(&mut self, command: Command) -> ActionOutcome {
        let outcome = self.resolve(&command);
        self.apply(&outcome);
        self.history.push(ResolvedCommand {
            command,
            outcome: outcome.clone(),
            turn: self.turn,
        });
        outcome
    }

    /// Resolve a command against the current state (pure — no mutation).
    #[must_use]
    #[allow(clippy::too_many_lines)]
    fn resolve(&self, command: &Command) -> ActionOutcome {
        match command {
            Command::Move { entity, direction } => self.resolve_move(*entity, *direction),
            Command::Interact { actor, target } => self.resolve_interact(*actor, *target),
            Command::Talk {
                actor,
                target,
                choice_id,
            } => self.resolve_talk(*actor, *target, choice_id.as_deref()),
            Command::Examine { actor, target } => self.resolve_examine(*actor, target),
            Command::Wait { entity } => ActionOutcome {
                effect: Effect::TurnEnded { entity: *entity },
                cost: ActionCost::One,
                narration: Some("You wait, watching and listening.".into()),
                triggers: Vec::new(),
            },
            Command::EndTurn { entity } => {
                ActionOutcome {
                    effect: Effect::TurnEnded { entity: *entity },
                    cost: ActionCost::Free,
                    narration: None,
                    triggers: Vec::new(),
                }
            }
            Command::Attack { actor, target, .. } => self.resolve_attack(*actor, *target),
            Command::UseItem {
                actor, item_name, ..
            } => ActionOutcome {
                effect: Effect::ItemUsed {
                    actor: *actor,
                    item_name: item_name.clone(),
                    result: "Used".into(),
                },
                cost: ActionCost::One,
                narration: Some(format!("You use the {item_name}.")),
                triggers: Vec::new(),
            },
            Command::Custom { actor: _, verb, args } => ActionOutcome {
                effect: Effect::NoEffect {
                    reason: format!("custom command: {verb} {}", args.join(" ")),
                },
                cost: ActionCost::One,
                narration: Some(format!("You attempt to {verb}.")),
                triggers: Vec::new(),
            },
        }
    }

    fn resolve_move(&self, entity: EntityId, direction: super::world::Direction) -> ActionOutcome {
        let Some(e) = self.entities.get(entity) else {
            return ActionOutcome {
                effect: Effect::NoEffect {
                    reason: "entity not found".into(),
                },
                cost: ActionCost::Free,
                narration: None,
                triggers: Vec::new(),
            };
        };

        let (ex, ey) = (e.x, e.y);

        if let Some((nx, ny)) = self.world.move_in(ex, ey, direction) {
            if self.entities.is_blocked(nx, ny) {
                return ActionOutcome {
                    effect: Effect::NoEffect {
                        reason: "destination occupied".into(),
                    },
                    cost: ActionCost::Free,
                    narration: Some("Something blocks your way.".into()),
                    triggers: Vec::new(),
                };
            }

            let mut triggers = Vec::new();
            for adj in self.entities.within_range(nx, ny, 0) {
                if adj.kind == EntityKind::Trigger {
                    if let Some(zone) = adj.properties.get("zone_transition") {
                        triggers.push(TriggerEvent::ZoneTransition {
                            zone_name: zone.clone(),
                        });
                    }
                    if let Some(plane) = adj.properties.get("plane_transition") {
                        triggers.push(TriggerEvent::PlaneTransition {
                            to_plane: plane.clone(),
                            trigger: adj.name.clone(),
                        });
                    }
                }
            }

            ActionOutcome {
                effect: Effect::Moved {
                    entity,
                    to_x: nx,
                    to_y: ny,
                },
                cost: ActionCost::One,
                narration: None,
                triggers,
            }
        } else {
            ActionOutcome {
                effect: Effect::NoEffect {
                    reason: "blocked terrain".into(),
                },
                cost: ActionCost::Free,
                narration: Some("You can't go that way.".into()),
                triggers: Vec::new(),
            }
        }
    }

    fn resolve_interact(&self, actor: EntityId, target: EntityId) -> ActionOutcome {
        let Some(target_entity) = self.entities.get(target) else {
            return ActionOutcome {
                effect: Effect::NoEffect {
                    reason: "target not found".into(),
                },
                cost: ActionCost::Free,
                narration: None,
                triggers: Vec::new(),
            };
        };

        match target_entity.kind {
            EntityKind::Npc => ActionOutcome {
                effect: Effect::DialogueAdvanced {
                    speaker: target,
                    listener: actor,
                    exchange_id: format!("conv_{}", self.history.len()),
                },
                cost: ActionCost::One,
                narration: Some(format!("{} turns to face you.", target_entity.name)),
                triggers: vec![TriggerEvent::ConversationStart { npc: target }],
            },
            EntityKind::Item => ActionOutcome {
                effect: Effect::ItemAcquired {
                    actor,
                    item_name: target_entity.name.clone(),
                },
                cost: ActionCost::One,
                narration: Some(format!("You pick up the {}.", target_entity.name)),
                triggers: Vec::new(),
            },
            EntityKind::Interactable => ActionOutcome {
                effect: Effect::Interacted {
                    actor,
                    target,
                    result: "interacted".into(),
                },
                cost: ActionCost::One,
                narration: Some(format!("You interact with the {}.", target_entity.name)),
                triggers: Vec::new(),
            },
            EntityKind::Clue => ActionOutcome {
                effect: Effect::Revealed {
                    actor,
                    info: target_entity.description.clone(),
                },
                cost: ActionCost::One,
                narration: Some(format!(
                    "You examine the evidence: {}",
                    target_entity.description
                )),
                triggers: vec![TriggerEvent::ClueFound {
                    clue_id: target_entity.name.clone(),
                }],
            },
            _ => ActionOutcome {
                effect: Effect::NoEffect {
                    reason: "nothing to interact with".into(),
                },
                cost: ActionCost::Free,
                narration: Some("There's nothing to do here.".into()),
                triggers: Vec::new(),
            },
        }
    }

    fn resolve_talk(
        &self,
        actor: EntityId,
        target: EntityId,
        _choice_id: Option<&str>,
    ) -> ActionOutcome {
        let Some(npc) = self.entities.get(target) else {
            return ActionOutcome {
                effect: Effect::NoEffect {
                    reason: "NPC not found".into(),
                },
                cost: ActionCost::Free,
                narration: None,
                triggers: Vec::new(),
            };
        };

        ActionOutcome {
            effect: Effect::DialogueAdvanced {
                speaker: target,
                listener: actor,
                exchange_id: format!("talk_{}", self.history.len()),
            },
            cost: ActionCost::One,
            narration: Some(format!("You speak with {}.", npc.name)),
            triggers: Vec::new(),
        }
    }

    fn resolve_examine(
        &self,
        actor: EntityId,
        target: &super::action::ExamineTarget,
    ) -> ActionOutcome {
        let info = match target {
            super::action::ExamineTarget::Entity(eid) => self
                .entities
                .get(*eid)
                .map_or_else(|| "Nothing here.".into(), |e| e.description.clone()),
            super::action::ExamineTarget::Tile(x, y) => self
                .world
                .get(*x, *y)
                .and_then(|t| t.description.clone())
                .unwrap_or_else(|| "An unremarkable spot.".into()),
            super::action::ExamineTarget::Direction(dir) => {
                format!("You peer {dir:?}ward. The way looks clear.")
            }
        };

        ActionOutcome {
            effect: Effect::Revealed { actor, info },
            cost: ActionCost::Free,
            narration: None,
            triggers: Vec::new(),
        }
    }

    fn resolve_attack(&self, _actor: EntityId, target: EntityId) -> ActionOutcome {
        let Some(target_entity) = self.entities.get(target) else {
            return ActionOutcome {
                effect: Effect::NoEffect {
                    reason: "target not found".into(),
                },
                cost: ActionCost::Free,
                narration: None,
                triggers: Vec::new(),
            };
        };

        ActionOutcome {
            effect: Effect::Damaged {
                target,
                amount: 0, // actual damage resolved via dice + ruleset
                source: "attack".into(),
            },
            cost: ActionCost::One,
            narration: Some(format!("You strike at {}!", target_entity.name)),
            triggers: Vec::new(),
        }
    }

    /// Apply an outcome to the session state.
    fn apply(&mut self, outcome: &ActionOutcome) {
        match &outcome.effect {
            Effect::Moved { entity, to_x, to_y } => {
                if let Some(e) = self.entities.get_mut(*entity) {
                    e.x = *to_x;
                    e.y = *to_y;
                }
                // Update fog of war if player moved
                if self
                    .entities
                    .get(*entity)
                    .is_some_and(|e| e.kind == EntityKind::Player)
                {
                    self.world.reveal_radius(*to_x, *to_y, self.sight_radius);
                }
            }
            Effect::TurnEnded { .. } => {
                self.advance_turn();
            }
            Effect::ItemAcquired { .. }
            | Effect::Interacted { .. }
            | Effect::DialogueAdvanced { .. }
            | Effect::Damaged { .. }
            | Effect::ItemUsed { .. }
            | Effect::Revealed { .. }
            | Effect::NoEffect { .. } => {
                // In a full implementation, ItemAcquired would remove entity and add to inventory
            }
        }
    }

    /// Advance to the next turn.
    const fn advance_turn(&mut self) {
        if self.initiative.is_empty() {
            self.turn += 1;
        } else {
            self.initiative_index += 1;
            if self.initiative_index >= self.initiative.len() {
                self.initiative_index = 0;
                self.turn += 1;
            }
        }
    }

    /// The currently acting entity (initiative order or player).
    #[must_use]
    pub fn active_entity(&self) -> Option<EntityId> {
        if self.initiative.is_empty() {
            self.entities.player().map(|p| p.id)
        } else {
            self.initiative.get(self.initiative_index).copied()
        }
    }

    /// Transition to a new plane, preserving world and entity state.
    pub fn transition_plane(&mut self, new_plane: PlaneType) {
        self.active_plane = new_plane;
        self.initiative.clear();
        self.initiative_index = 0;
        self.turn_budget = None;
    }

    /// Number of resolved commands in the history.
    #[must_use]
    pub const fn history_len(&self) -> usize {
        self.history.len()
    }

    /// All visible NPC entities (for narration and UI).
    pub fn visible_npcs(&self) -> impl Iterator<Item = &Entity> {
        self.entities
            .of_kind(EntityKind::Npc)
            .filter(|e| e.visible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::engine::entity::{Entity, Faction};
    use crate::game::engine::world::{Direction, Terrain, TileWorld, Visibility};

    fn tavern_session() -> GameSession {
        let mut world = TileWorld::new(10, 10, "The Drowned Rat", Terrain::Open);
        // Walls around the edges
        for x in 0..10 {
            world.set_terrain(x, 0, Terrain::Wall);
            world.set_terrain(x, 9, Terrain::Wall);
        }
        for y in 0..10 {
            world.set_terrain(0, y, Terrain::Wall);
            world.set_terrain(9, y, Terrain::Wall);
        }
        // Door
        world.set_terrain(5, 9, Terrain::Door { open: true });
        // Table
        world.set_terrain(3, 3, Terrain::Surface);

        let mut session = GameSession::new(world, PlaneType::Dialogue);
        session.spawn(Entity::player(0, "Harlan Cole", 5, 5));
        session.spawn(Entity::npc(0, "Maren", 3, 4, Faction::Neutral));
        session.spawn(Entity::item(0, "Old Lantern", 7, 3));
        session.spawn(Entity::trigger(0, "cellar_entrance", 2, 8));
        session
    }

    #[test]
    fn session_creation() {
        let s = tavern_session();
        assert_eq!(s.active_plane, PlaneType::Dialogue);
        assert_eq!(s.entities.count(), 4);
        assert_eq!(s.turn, 0);
    }

    #[test]
    fn player_movement() {
        let mut s = tavern_session();
        let outcome = s.process(Command::Move {
            entity: s.entities.player().unwrap().id,
            direction: Direction::North,
        });
        assert!(matches!(outcome.effect, Effect::Moved { to_y: 4, .. }));
        assert_eq!(s.entities.player().unwrap().y, 4);
        assert_eq!(s.history_len(), 1);
    }

    #[test]
    fn blocked_by_wall() {
        let mut s = tavern_session();
        let pid = s.entities.player().unwrap().id;
        // Move player to (1,1) which is adjacent to walls
        s.entities.get_mut(pid).unwrap().x = 1;
        s.entities.get_mut(pid).unwrap().y = 1;

        let outcome = s.process(Command::Move {
            entity: pid,
            direction: Direction::North,
        });
        assert!(matches!(outcome.effect, Effect::NoEffect { .. }));
    }

    #[test]
    fn interact_with_npc_starts_conversation() {
        let mut s = tavern_session();
        let pid = s.entities.player().unwrap().id;
        let npc_id = s
            .entities
            .of_kind(EntityKind::Npc)
            .next()
            .unwrap()
            .id;

        let outcome = s.process(Command::Interact {
            actor: pid,
            target: npc_id,
        });
        assert!(matches!(outcome.effect, Effect::DialogueAdvanced { .. }));
        assert_eq!(outcome.triggers.len(), 1);
        assert!(matches!(
            outcome.triggers[0],
            TriggerEvent::ConversationStart { .. }
        ));
    }

    #[test]
    fn pick_up_item() {
        let mut s = tavern_session();
        let pid = s.entities.player().unwrap().id;
        let item_id = s
            .entities
            .of_kind(EntityKind::Item)
            .next()
            .unwrap()
            .id;

        let outcome = s.process(Command::Interact {
            actor: pid,
            target: item_id,
        });
        assert!(matches!(outcome.effect, Effect::ItemAcquired { .. }));
    }

    #[test]
    fn examine_entity() {
        let mut s = tavern_session();
        let pid = s.entities.player().unwrap().id;
        let npc_id = s
            .entities
            .of_kind(EntityKind::Npc)
            .next()
            .unwrap()
            .id;

        let outcome = s.process(Command::Examine {
            actor: pid,
            target: super::super::action::ExamineTarget::Entity(npc_id),
        });
        assert!(matches!(outcome.effect, Effect::Revealed { .. }));
    }

    #[test]
    fn plane_transition() {
        let mut s = tavern_session();
        assert_eq!(s.active_plane, PlaneType::Dialogue);
        s.transition_plane(PlaneType::Tactical);
        assert_eq!(s.active_plane, PlaneType::Tactical);
    }

    #[test]
    fn fog_of_war_updates_on_move() {
        let mut s = tavern_session();
        let pid = s.entities.player().unwrap().id;

        // Initially all hidden
        let before = s.world.count_where(|t| t.visibility == Visibility::Visible);
        assert_eq!(before, 0);

        // Move triggers fog reveal
        s.process(Command::Move {
            entity: pid,
            direction: Direction::South,
        });

        let after = s.world.count_where(|t| t.visibility == Visibility::Visible);
        assert!(after > 0);
    }

    #[test]
    fn turn_advances_on_end_turn() {
        let mut s = tavern_session();
        let pid = s.entities.player().unwrap().id;
        assert_eq!(s.turn, 0);

        s.process(Command::EndTurn { entity: pid });
        assert_eq!(s.turn, 1);
    }

    #[test]
    fn active_entity_defaults_to_player() {
        let s = tavern_session();
        let active = s.active_entity();
        assert_eq!(active, Some(s.entities.player().unwrap().id));
    }

    #[test]
    fn visible_npcs() {
        let s = tavern_session();
        let npcs: Vec<_> = s.visible_npcs().collect();
        assert_eq!(npcs.len(), 1);
        assert_eq!(npcs[0].name, "Maren");
    }

    #[test]
    fn wait_costs_action_and_narrates() {
        let mut s = tavern_session();
        let pid = s.entities.player().unwrap().id;
        let outcome = s.process(Command::Wait { entity: pid });
        assert_eq!(outcome.cost, ActionCost::One);
        assert!(outcome.narration.is_some());
    }
}
