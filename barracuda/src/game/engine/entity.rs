// SPDX-License-Identifier: AGPL-3.0-or-later
//! Entity system — anything that exists in the 2D world.
//!
//! Entities are lightweight: an ID, a position, a kind, and a bag of
//! components. The entity system is *not* a full ECS — it's a simple
//! flat list suitable for the scale of tabletop/adventure games (tens
//! to low hundreds of entities, not millions).
//!
//! Entity kinds carry semantic meaning for petalTongue rendering and
//! audio narration: a `Player` is rendered differently from an `Npc`,
//! which is different from a `Trigger`.

use std::collections::HashMap;

/// Unique entity identifier within a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u32);

/// What kind of thing this entity is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityKind {
    /// The player character.
    Player,
    /// A non-player character (friendly, neutral, or hostile).
    Npc,
    /// A collectible or usable item.
    Item,
    /// An invisible trigger zone (conversation start, encounter, event).
    Trigger,
    /// A piece of evidence or clue (Investigation plane).
    Clue,
    /// A card or token (Card/Stack plane).
    Card,
    /// Terrain feature that can be interacted with (door, lever, chest).
    Interactable,
    /// Domain-specific entity.
    Custom(u16),
}

/// Faction alignment for combat and disposition coloring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Faction {
    /// Player's party.
    Party,
    /// Friendly to the party.
    Friendly,
    /// Neither hostile nor friendly.
    Neutral,
    /// Hostile to the party.
    Hostile,
}

/// A single entity in the world.
#[derive(Debug, Clone)]
pub struct Entity {
    /// Unique identifier.
    pub id: EntityId,
    /// What this entity is.
    pub kind: EntityKind,
    /// Display name.
    pub name: String,
    /// Grid X coordinate.
    pub x: u32,
    /// Grid Y coordinate.
    pub y: u32,
    /// Whether this entity blocks movement through its tile.
    pub blocking: bool,
    /// Whether this entity is currently visible (independent of fog).
    pub visible: bool,
    /// Faction for combat and coloring.
    pub faction: Faction,
    /// Short description for audio narration ("a nervous innkeeper").
    pub description: String,
    /// Optional character sheet reference (for Player/Npc with stats).
    pub character_id: Option<String>,
    /// Optional NPC personality cert reference (for RPGPT NPCs).
    pub npc_personality_id: Option<String>,
    /// Arbitrary key-value properties.
    pub properties: HashMap<String, String>,
}

impl Entity {
    /// Create a player entity.
    #[must_use]
    pub fn player(id: u32, name: &str, x: u32, y: u32) -> Self {
        Self {
            id: EntityId(id),
            kind: EntityKind::Player,
            name: name.into(),
            x,
            y,
            blocking: true,
            visible: true,
            faction: Faction::Party,
            description: format!("You — {name}"),
            character_id: None,
            npc_personality_id: None,
            properties: HashMap::new(),
        }
    }

    /// Create an NPC entity.
    #[must_use]
    pub fn npc(id: u32, name: &str, x: u32, y: u32, faction: Faction) -> Self {
        Self {
            id: EntityId(id),
            kind: EntityKind::Npc,
            name: name.into(),
            x,
            y,
            blocking: true,
            visible: true,
            faction,
            description: name.into(),
            character_id: None,
            npc_personality_id: None,
            properties: HashMap::new(),
        }
    }

    /// Create an item entity.
    #[must_use]
    pub fn item(id: u32, name: &str, x: u32, y: u32) -> Self {
        Self {
            id: EntityId(id),
            kind: EntityKind::Item,
            name: name.into(),
            x,
            y,
            blocking: false,
            visible: true,
            faction: Faction::Neutral,
            description: name.into(),
            character_id: None,
            npc_personality_id: None,
            properties: HashMap::new(),
        }
    }

    /// Create an invisible trigger.
    #[must_use]
    pub fn trigger(id: u32, name: &str, x: u32, y: u32) -> Self {
        Self {
            id: EntityId(id),
            kind: EntityKind::Trigger,
            name: name.into(),
            x,
            y,
            blocking: false,
            visible: false,
            faction: Faction::Neutral,
            description: String::new(),
            character_id: None,
            npc_personality_id: None,
            properties: HashMap::new(),
        }
    }

    /// Manhattan distance to another position.
    #[must_use]
    pub const fn distance_to(&self, x: u32, y: u32) -> u32 {
        self.x.abs_diff(x) + self.y.abs_diff(y)
    }

    /// Whether this entity is adjacent (including diagonal) to a position.
    #[must_use]
    pub const fn is_adjacent_to(&self, x: u32, y: u32) -> bool {
        self.x.abs_diff(x) <= 1 && self.y.abs_diff(y) <= 1 && (self.x != x || self.y != y)
    }
}

/// Registry of entities in a session.
#[derive(Debug, Clone, Default)]
pub struct EntityRegistry {
    entities: Vec<Entity>,
    next_id: u32,
}

impl EntityRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an entity, returning its assigned ID.
    pub fn spawn(&mut self, mut entity: Entity) -> EntityId {
        let id = EntityId(self.next_id);
        entity.id = id;
        self.next_id += 1;
        self.entities.push(entity);
        id
    }

    /// Get an entity by ID.
    #[must_use]
    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == id)
    }

    /// Get a mutable entity by ID.
    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    /// Remove an entity by ID. Returns the entity if found.
    pub fn despawn(&mut self, id: EntityId) -> Option<Entity> {
        if let Some(pos) = self.entities.iter().position(|e| e.id == id) {
            Some(self.entities.swap_remove(pos))
        } else {
            None
        }
    }

    /// All entities at a position.
    pub fn at(&self, x: u32, y: u32) -> impl Iterator<Item = &Entity> {
        self.entities.iter().filter(move |e| e.x == x && e.y == y)
    }

    /// All entities of a given kind.
    pub fn of_kind(&self, kind: EntityKind) -> impl Iterator<Item = &Entity> {
        self.entities.iter().filter(move |e| e.kind == kind)
    }

    /// All entities within Manhattan distance of a point.
    pub fn within_range(&self, x: u32, y: u32, range: u32) -> impl Iterator<Item = &Entity> {
        self.entities
            .iter()
            .filter(move |e| e.distance_to(x, y) <= range)
    }

    /// The player entity (assumes exactly one).
    #[must_use]
    pub fn player(&self) -> Option<&Entity> {
        self.entities.iter().find(|e| e.kind == EntityKind::Player)
    }

    /// Mutable player entity.
    pub fn player_mut(&mut self) -> Option<&mut Entity> {
        self.entities
            .iter_mut()
            .find(|e| e.kind == EntityKind::Player)
    }

    /// Whether any blocking entity occupies (x, y).
    #[must_use]
    pub fn is_blocked(&self, x: u32, y: u32) -> bool {
        self.entities
            .iter()
            .any(|e| e.x == x && e.y == y && e.blocking)
    }

    /// Total entity count.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.entities.len()
    }

    /// Iterate all entities.
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_creation() {
        let p = Entity::player(0, "Harlan", 5, 3);
        assert_eq!(p.kind, EntityKind::Player);
        assert_eq!(p.faction, Faction::Party);
        assert!(p.blocking);
        assert!(p.visible);
    }

    #[test]
    fn npc_creation() {
        let n = Entity::npc(1, "Maren", 3, 4, Faction::Neutral);
        assert_eq!(n.kind, EntityKind::Npc);
        assert_eq!(n.faction, Faction::Neutral);
    }

    #[test]
    fn trigger_is_invisible() {
        let t = Entity::trigger(2, "conversation_start", 3, 3);
        assert!(!t.visible);
        assert!(!t.blocking);
    }

    #[test]
    fn distance_and_adjacency() {
        let e = Entity::player(0, "P", 5, 5);
        assert_eq!(e.distance_to(5, 5), 0);
        assert_eq!(e.distance_to(7, 5), 2);
        assert!(e.is_adjacent_to(5, 6));
        assert!(e.is_adjacent_to(6, 6)); // diagonal
        assert!(!e.is_adjacent_to(5, 5)); // same position
        assert!(!e.is_adjacent_to(7, 7)); // too far
    }

    #[test]
    fn registry_spawn_and_query() {
        let mut reg = EntityRegistry::new();
        let pid = reg.spawn(Entity::player(0, "Harlan", 5, 5));
        let nid = reg.spawn(Entity::npc(0, "Maren", 5, 6, Faction::Neutral));
        let iid = reg.spawn(Entity::item(0, "Lantern", 5, 5));

        assert_eq!(reg.count(), 3);
        assert!(reg.get(pid).is_some());
        assert!(reg.player().is_some());
        assert_eq!(reg.player().unwrap().name, "Harlan");

        // Two entities at (5,5): player + lantern
        assert_eq!(reg.at(5, 5).count(), 2);

        // Position is blocked by player
        assert!(reg.is_blocked(5, 5));
        // Lantern doesn't block
        assert!(!reg.at(5, 5).any(|e| e.kind == EntityKind::Item && e.blocking));

        // Range query
        assert_eq!(reg.within_range(5, 5, 1).count(), 3); // all within 1

        // Despawn
        reg.despawn(nid);
        assert_eq!(reg.count(), 2);
        assert!(reg.get(nid).is_none());

        // Kind query
        assert_eq!(reg.of_kind(EntityKind::Item).count(), 1);
        let _ = iid;
    }

    #[test]
    fn registry_move_entity() {
        let mut reg = EntityRegistry::new();
        let pid = reg.spawn(Entity::player(0, "P", 3, 3));
        reg.get_mut(pid).unwrap().x = 4;
        assert_eq!(reg.get(pid).unwrap().x, 4);
    }

    #[test]
    fn entity_properties() {
        let mut e = Entity::item(0, "Ancient Key", 2, 2);
        e.properties.insert("use".into(), "unlock_crypt".into());
        assert_eq!(e.properties.get("use").unwrap(), "unlock_crypt");
    }
}
