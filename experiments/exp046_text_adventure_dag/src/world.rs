// SPDX-License-Identifier: AGPL-3.0-or-later
//! World model for the text adventure DAG.

use std::collections::{HashMap, HashSet};
use std::fmt::Write;

pub type RoomId = &'static str;
pub type ItemId = &'static str;
pub type ActionId = &'static str;

/// A room in the world.
#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all fields"
)]
#[derive(Debug, Clone)]
pub struct Room {
    pub id: RoomId,
    pub name: &'static str,
    pub description: &'static str,
    pub exits: Vec<Exit>,
    pub items: Vec<ItemId>,
}

/// A directional exit from a room.
#[derive(Debug, Clone)]
pub struct Exit {
    pub direction: &'static str,
    pub target: RoomId,
    pub requires: Option<ItemId>,
    pub locked_message: &'static str,
}

/// An item that can be picked up.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub name: &'static str,
    pub description: &'static str,
    pub usable_at: Option<RoomId>,
    pub use_effect: Option<&'static str>,
}

// ===========================================================================
// DAG vertex — every game state transition is recorded
// ===========================================================================

#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all fields"
)]
#[derive(Debug, Clone)]
pub struct Vertex {
    pub id: usize,
    pub parent: Option<usize>,
    pub room: RoomId,
    pub action: ActionId,
    pub description: String,
    pub inventory: Vec<ItemId>,
    pub flags: HashSet<String>,
}

/// An action available from the current state.
#[derive(Debug, Clone)]
pub struct AvailableAction {
    pub id: ActionId,
    pub display: String,
    pub category: ActionCategory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionCategory {
    Move,
    Take,
    Use,
    Look,
    Examine,
}

// ===========================================================================
// Game state
// ===========================================================================

pub struct GameState {
    pub rooms: HashMap<RoomId, Room>,
    pub items: HashMap<ItemId, Item>,
    pub vertices: Vec<Vertex>,
    pub current_room: RoomId,
    pub inventory: Vec<ItemId>,
    pub flags: HashSet<String>,
}

impl GameState {
    #[expect(clippy::too_many_lines, reason = "world builder — rooms, items, exits")]
    pub fn build_world() -> Self {
        let mut rooms = HashMap::new();
        let mut items = HashMap::new();

        rooms.insert(
            "entrance",
            Room {
                id: "entrance",
                name: "Cave Entrance",
                description: "You stand at the mouth of a dark cave. Sunlight filters through the trees behind you. A narrow passage leads north into darkness.",
                exits: vec![
                    Exit { direction: "north", target: "corridor", requires: None, locked_message: "" },
                ],
                items: vec!["torch"],
            },
        );

        rooms.insert(
            "corridor",
            Room {
                id: "corridor",
                name: "Stone Corridor",
                description: "A rough stone corridor stretches ahead. Water drips from the ceiling. Passages lead north and east. The entrance is to the south.",
                exits: vec![
                    Exit { direction: "south", target: "entrance", requires: None, locked_message: "" },
                    Exit { direction: "north", target: "locked_door", requires: None, locked_message: "" },
                    Exit { direction: "east", target: "pool_room", requires: None, locked_message: "" },
                ],
                items: vec![],
            },
        );

        rooms.insert(
            "pool_room",
            Room {
                id: "pool_room",
                name: "Underground Pool",
                description: "A still pool of dark water fills most of this chamber. Something glints at the bottom. The corridor is to the west.",
                exits: vec![
                    Exit { direction: "west", target: "corridor", requires: None, locked_message: "" },
                ],
                items: vec!["key"],
            },
        );

        rooms.insert(
            "locked_door",
            Room {
                id: "locked_door",
                name: "Iron Door",
                description: "A massive iron door blocks the passage. There is a keyhole. The corridor is to the south.",
                exits: vec![
                    Exit { direction: "south", target: "corridor", requires: None, locked_message: "" },
                    Exit {
                        direction: "north",
                        target: "treasure_room",
                        requires: Some("key"),
                        locked_message: "The iron door is locked. You need a key.",
                    },
                ],
                items: vec![],
            },
        );

        rooms.insert(
            "treasure_room",
            Room {
                id: "treasure_room",
                name: "Treasure Chamber",
                description: "Gold coins and ancient artifacts fill this chamber. A beam of light falls from a crack in the ceiling. You've found the treasure!",
                exits: vec![
                    Exit { direction: "south", target: "locked_door", requires: None, locked_message: "" },
                ],
                items: vec!["crown"],
            },
        );

        items.insert(
            "torch",
            Item {
                id: "torch",
                name: "Wooden Torch",
                description: "A sturdy torch wrapped in oil-soaked cloth.",
                usable_at: None,
                use_effect: None,
            },
        );

        items.insert(
            "key",
            Item {
                id: "key",
                name: "Iron Key",
                description: "A heavy iron key, crusted with age.",
                usable_at: Some("locked_door"),
                use_effect: Some(
                    "You insert the key into the lock. The iron door swings open with a groan.",
                ),
            },
        );

        items.insert(
            "crown",
            Item {
                id: "crown",
                name: "Ancient Crown",
                description: "A golden crown set with deep red gems. It pulses faintly with warmth.",
                usable_at: None,
                use_effect: None,
            },
        );

        let initial_vertex = Vertex {
            id: 0,
            parent: None,
            room: "entrance",
            action: "start",
            description: "Adventure begins at the cave entrance.".into(),
            inventory: vec![],
            flags: HashSet::new(),
        };

        Self {
            rooms,
            items,
            vertices: vec![initial_vertex],
            current_room: "entrance",
            inventory: vec![],
            flags: HashSet::new(),
        }
    }

    /// The core innovation: return only valid actions from the current state.
    /// No "guess the verb". No near-infinite command space.
    pub fn available_actions(&self) -> Vec<AvailableAction> {
        let room = &self.rooms[self.current_room];
        let mut actions = Vec::new();

        actions.push(AvailableAction {
            id: "look",
            display: format!("Look around ({})", room.name),
            category: ActionCategory::Look,
        });

        for exit in &room.exits {
            if let Some(required) = exit.requires {
                if !self.flags.contains(&format!("used_{required}")) {
                    if self.inventory.contains(&required) {
                        actions.push(AvailableAction {
                            id: "use_item",
                            display: format!(
                                "Use {} to go {}",
                                self.items[required].name, exit.direction
                            ),
                            category: ActionCategory::Use,
                        });
                    }
                    continue;
                }
            }
            actions.push(AvailableAction {
                id: exit.direction,
                display: format!("Go {}", exit.direction),
                category: ActionCategory::Move,
            });
        }

        let room_items: Vec<_> = room
            .items
            .iter()
            .filter(|i| !self.inventory.contains(i) && !self.flags.contains(&format!("taken_{i}")))
            .collect();
        for &item_id in &room_items {
            actions.push(AvailableAction {
                id: item_id,
                display: format!("Take {}", self.items[item_id].name),
                category: ActionCategory::Take,
            });
        }

        for &item_id in &self.inventory {
            let item = &self.items[item_id];
            if item.usable_at == Some(self.current_room)
                && !self.flags.contains(&format!("used_{item_id}"))
            {
                actions.push(AvailableAction {
                    id: "use_item",
                    display: format!("Use {}", item.name),
                    category: ActionCategory::Use,
                });
            }
        }

        for &item_id in &self.inventory {
            actions.push(AvailableAction {
                id: item_id,
                display: format!("Examine {}", self.items[item_id].name),
                category: ActionCategory::Examine,
            });
        }

        actions
    }

    pub fn execute_move(&mut self, direction: &'static str) -> String {
        let room = &self.rooms[self.current_room];
        let exit = room.exits.iter().find(|e| e.direction == direction);

        match exit {
            Some(e) => {
                if let Some(req) = e.requires {
                    if !self.flags.contains(&format!("used_{req}")) {
                        return e.locked_message.to_string();
                    }
                }
                self.current_room = e.target;
                let new_room = &self.rooms[self.current_room];
                let desc = format!(
                    "You move {}.\n\n{}\n{}",
                    direction, new_room.name, new_room.description
                );
                self.add_vertex(direction, &desc);
                desc
            }
            None => "You can't go that way.".into(),
        }
    }

    pub fn execute_take(&mut self, item_id: ItemId) -> String {
        if self.rooms[self.current_room].items.contains(&item_id)
            && !self.inventory.contains(&item_id)
            && !self.flags.contains(&format!("taken_{item_id}"))
        {
            self.inventory.push(item_id);
            self.flags.insert(format!("taken_{item_id}"));
            let item = &self.items[item_id];
            let desc = format!("You pick up the {}.", item.name);
            self.add_vertex("take", &desc);
            desc
        } else {
            "Nothing to take.".into()
        }
    }

    pub fn execute_use(&mut self, item_id: ItemId) -> String {
        let item = &self.items[item_id];
        if item.usable_at == Some(self.current_room)
            && !self.flags.contains(&format!("used_{item_id}"))
        {
            self.flags.insert(format!("used_{item_id}"));
            let effect = item.use_effect.unwrap_or("Nothing happens.");
            let desc = effect.to_string();
            self.add_vertex("use", &desc);
            desc
        } else {
            "You can't use that here.".into()
        }
    }

    pub fn execute_look(&mut self) -> String {
        let room = &self.rooms[self.current_room];
        let mut desc = format!("{}\n{}", room.name, room.description);

        let visible_items: Vec<_> = room
            .items
            .iter()
            .filter(|i| !self.inventory.contains(i) && !self.flags.contains(&format!("taken_{i}")))
            .collect();
        if !visible_items.is_empty() {
            desc.push_str("\n\nYou see:");
            for &item_id in &visible_items {
                let _ = write!(&mut desc, "\n  - {}", self.items[item_id].name);
            }
        }

        self.add_vertex("look", &desc);
        desc
    }

    fn add_vertex(&mut self, action: ActionId, description: &str) {
        let parent = self.vertices.len() - 1;
        self.vertices.push(Vertex {
            id: self.vertices.len(),
            parent: Some(parent),
            room: self.current_room,
            action,
            description: description.to_string(),
            inventory: self.inventory.clone(),
            flags: self.flags.clone(),
        });
    }

    pub const fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn dag_depth(&self) -> usize {
        let mut depth = 0;
        let mut current = self.vertices.last();
        while let Some(v) = current {
            depth += 1;
            current = v.parent.and_then(|p| self.vertices.get(p));
        }
        depth
    }
}
