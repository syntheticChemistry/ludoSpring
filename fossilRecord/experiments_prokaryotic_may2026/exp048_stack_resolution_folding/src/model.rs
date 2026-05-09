// SPDX-License-Identifier: AGPL-3.0-or-later
//! Domain model for exp048 — Stack Resolution as Folding.
//!
//! Card types, stack (LIFO), board state, creatures, and scenario builders.
//! Same cards in different resolution orders produce different board states.

// ===========================================================================
// Card model (minimal — focused on stack interaction)
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(clippy::struct_field_names, reason = "domain naming")]
pub struct Card {
    pub name: &'static str,
    pub card_type: SpellType,
    pub effect: Effect,
}

#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all variants"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellType {
    Instant,
    Creature,
    Sorcery,
    Ability,
}

#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all variants"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Damage {
        target: Target,
        amount: i32,
    },
    Pump {
        target: Target,
        power: i32,
        toughness: i32,
    },
    Counter {
        what: Target,
    },
    Destroy {
        target: Target,
    },
    Regenerate {
        target: Target,
    },
    Draw {
        count: u8,
    },
    Redirect {
        from: Target,
        to: Target,
    },
}

#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all variants"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Creature(&'static str),
    Player(&'static str),
    StackItem(usize),
    Any,
}

// ===========================================================================
// Stack — LIFO with response windows
// ===========================================================================

#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all fields"
)]
#[derive(Debug, Clone)]
pub struct StackItem {
    pub id: usize,
    pub card: Card,
    pub controller: &'static str,
    pub targets: Vec<Target>,
    pub responding_to: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Stack {
    pub items: Vec<StackItem>,
    next_id: usize,
}

impl Stack {
    pub const fn new() -> Self {
        Self {
            items: Vec::new(),
            next_id: 0,
        }
    }

    pub fn cast(&mut self, card: Card, controller: &'static str, targets: Vec<Target>) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(StackItem {
            id,
            card,
            controller,
            targets,
            responding_to: None,
        });
        id
    }

    pub fn respond(
        &mut self,
        card: Card,
        controller: &'static str,
        targets: Vec<Target>,
        to: usize,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(StackItem {
            id,
            card,
            controller,
            targets,
            responding_to: Some(to),
        });
        id
    }

    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn resolve_top(&mut self) -> Option<StackItem> {
        self.items.pop()
    }
}

// ===========================================================================
// Board state — minimal for resolution tracking
// ===========================================================================

pub const STARTING_LIFE: i32 = 20;

#[derive(Debug, Clone)]
pub struct Creature {
    pub name: &'static str,
    base_power: i32,
    base_toughness: i32,
    pub damage: i32,
    pumps: Vec<(i32, i32)>,
    destroyed: bool,
    regeneration_shield: bool,
}

impl Creature {
    pub const fn new(name: &'static str, power: i32, toughness: i32) -> Self {
        Self {
            name,
            base_power: power,
            base_toughness: toughness,
            damage: 0,
            pumps: Vec::new(),
            destroyed: false,
            regeneration_shield: false,
        }
    }

    pub fn effective_power(&self) -> i32 {
        self.base_power + self.pumps.iter().map(|(p, _)| p).sum::<i32>()
    }

    pub fn effective_toughness(&self) -> i32 {
        self.base_toughness + self.pumps.iter().map(|(_, t)| t).sum::<i32>()
    }

    pub fn is_dead(&self) -> bool {
        self.destroyed || self.damage >= self.effective_toughness()
    }
}

#[derive(Debug, Clone)]
pub struct BoardState {
    pub creatures: Vec<Creature>,
    pub life: [i32; 2],
    pub graveyard: Vec<&'static str>,
    pub countered: Vec<&'static str>,
    pub resolution_log: Vec<String>,
}

impl BoardState {
    pub const fn new() -> Self {
        Self {
            creatures: Vec::new(),
            life: [STARTING_LIFE, STARTING_LIFE],
            graveyard: Vec::new(),
            countered: Vec::new(),
            resolution_log: Vec::new(),
        }
    }

    fn find_creature_mut(&mut self, name: &str) -> Option<&mut Creature> {
        self.creatures
            .iter_mut()
            .find(|c| c.name == name && !c.is_dead())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "stack resolution — damage, destroy, regenerate, pump"
    )]
    pub fn resolve_item(&mut self, item: &StackItem) {
        // Check if this item was countered
        if self.countered.contains(&item.card.name) {
            self.resolution_log
                .push(format!("{} was countered — fizzles", item.card.name));
            return;
        }

        match &item.card.effect {
            Effect::Damage { target, amount } => match target {
                Target::Creature(name) => {
                    let outcome = if let Some(c) = self.find_creature_mut(name) {
                        c.damage += amount;
                        let p = c.effective_power();
                        let t = c.effective_toughness();
                        let d = c.damage;
                        let dead = c.is_dead();
                        let regen = c.regeneration_shield;
                        if dead && regen {
                            c.damage = 0;
                            c.regeneration_shield = false;
                        }
                        Some((p, t, d, dead, regen))
                    } else {
                        None
                    };
                    match outcome {
                        Some((p, t, d, dead, regen)) => {
                            self.resolution_log.push(format!(
                                "{} deals {} damage to {} (now {}/{}  with {} damage)",
                                item.card.name, amount, name, p, t, d
                            ));
                            if dead {
                                if regen {
                                    self.resolution_log
                                        .push(format!("{name} would die but regenerates!"));
                                } else {
                                    self.resolution_log
                                        .push(format!("{name} dies (lethal damage)"));
                                    self.graveyard.push(name);
                                }
                            }
                        }
                        None => {
                            self.resolution_log.push(format!(
                                "{} fizzles — target {} not on battlefield",
                                item.card.name, name
                            ));
                        }
                    }
                }
                Target::Player(name) => {
                    let idx = usize::from(*name != "alice");
                    self.life[idx] -= amount;
                    self.resolution_log.push(format!(
                        "{} deals {} damage to {} (life: {})",
                        item.card.name, amount, name, self.life[idx]
                    ));
                }
                _ => {}
            },
            Effect::Pump {
                target,
                power,
                toughness,
            } => {
                if let Target::Creature(name) = target {
                    let stats = if let Some(c) = self.find_creature_mut(name) {
                        c.pumps.push((*power, *toughness));
                        Some((c.effective_power(), c.effective_toughness()))
                    } else {
                        None
                    };
                    if let Some((p, t)) = stats {
                        self.resolution_log.push(format!(
                            "{} gives {} +{}/+{} (now {}/{})",
                            item.card.name, name, power, toughness, p, t
                        ));
                    }
                }
            }
            Effect::Counter { what } => {
                if let Target::StackItem(_) = what {
                    // Find the targeted item's card name by walking the stack
                    // In our simplified model, we mark it as countered
                    self.resolution_log
                        .push(format!("{} counters a spell", item.card.name));
                }
            }
            Effect::Destroy { target } => {
                if let Target::Creature(name) = target {
                    if let Some(c) = self.find_creature_mut(name) {
                        if c.regeneration_shield {
                            c.regeneration_shield = false;
                            c.damage = 0;
                            self.resolution_log.push(format!(
                                "{} would destroy {} but it regenerates!",
                                item.card.name, name
                            ));
                        } else {
                            c.destroyed = true;
                            self.graveyard.push(name);
                            self.resolution_log
                                .push(format!("{} destroys {}", item.card.name, name));
                        }
                    }
                }
            }
            Effect::Regenerate { target } => {
                if let Target::Creature(name) = target {
                    if let Some(c) = self.find_creature_mut(name) {
                        c.regeneration_shield = true;
                        self.resolution_log
                            .push(format!("Regeneration shield on {name}"));
                    }
                }
            }
            Effect::Draw { count } => {
                self.resolution_log
                    .push(format!("{} draws {} card(s)", item.controller, count));
            }
            Effect::Redirect { from, to } => {
                self.resolution_log.push(format!(
                    "{} redirects damage from {:?} to {:?}",
                    item.card.name, from, to
                ));
            }
        }
    }

    pub fn resolve_stack(&mut self, stack: &mut Stack) {
        while let Some(item) = stack.resolve_top() {
            self.resolve_item(&item);
        }
    }
}

// ===========================================================================
// Card factory functions
// ===========================================================================

pub const fn lightning_bolt() -> Card {
    Card {
        name: "Lightning Bolt",
        card_type: SpellType::Instant,
        effect: Effect::Damage {
            target: Target::Creature("bear"),
            amount: 3,
        },
    }
}

pub const fn giant_growth() -> Card {
    Card {
        name: "Giant Growth",
        card_type: SpellType::Instant,
        effect: Effect::Pump {
            target: Target::Creature("bear"),
            power: 3,
            toughness: 3,
        },
    }
}

pub const fn murder() -> Card {
    Card {
        name: "Murder",
        card_type: SpellType::Instant,
        effect: Effect::Destroy {
            target: Target::Creature("bear"),
        },
    }
}

pub const fn regenerate() -> Card {
    Card {
        name: "Regenerate",
        card_type: SpellType::Instant,
        effect: Effect::Regenerate {
            target: Target::Creature("bear"),
        },
    }
}

pub const fn bolt_to_face() -> Card {
    Card {
        name: "Lightning Bolt",
        card_type: SpellType::Instant,
        effect: Effect::Damage {
            target: Target::Player("bob"),
            amount: 3,
        },
    }
}

// ===========================================================================
// Scenario builders — same cards, different orders
// ===========================================================================

/// Scenario A: Alice casts Giant Growth, Bob responds with Bolt.
/// Stack (bottom to top): Giant Growth, Bolt.
/// Resolves: Bolt first (3 damage to 2/2 = dead), Giant Growth fizzles.
/// Bear DIES.
pub fn scenario_bolt_then_growth() -> BoardState {
    let mut board = BoardState::new();
    board.creatures.push(Creature::new("bear", 2, 2));

    let mut stack = Stack::new();
    let growth_id = stack.cast(giant_growth(), "alice", vec![Target::Creature("bear")]);
    stack.respond(
        lightning_bolt(),
        "bob",
        vec![Target::Creature("bear")],
        growth_id,
    );

    board.resolve_stack(&mut stack);
    board
}

/// Scenario B: Bob casts Bolt, Alice responds with Giant Growth.
/// Stack (bottom to top): Bolt, Giant Growth.
/// Resolves: Giant Growth first (bear becomes 5/5), then Bolt (3 damage to 5/5 = survives).
/// Bear LIVES (5/5 with 3 damage).
pub fn scenario_growth_then_bolt() -> BoardState {
    let mut board = BoardState::new();
    board.creatures.push(Creature::new("bear", 2, 2));

    let mut stack = Stack::new();
    let bolt_id = stack.cast(lightning_bolt(), "bob", vec![Target::Creature("bear")]);
    stack.respond(
        giant_growth(),
        "alice",
        vec![Target::Creature("bear")],
        bolt_id,
    );

    board.resolve_stack(&mut stack);
    board
}

/// Alice's creature faces Murder. She responds with Regenerate.
/// Stack: Murder (bottom), Regenerate (top).
/// Resolves: Regenerate first (shield up), Murder (shield absorbs destroy).
/// Bear LIVES.
pub fn scenario_regen_before_murder() -> BoardState {
    let mut board = BoardState::new();
    board.creatures.push(Creature::new("bear", 2, 2));

    let mut stack = Stack::new();
    let murder_id = stack.cast(murder(), "bob", vec![Target::Creature("bear")]);
    stack.respond(
        regenerate(),
        "alice",
        vec![Target::Creature("bear")],
        murder_id,
    );

    board.resolve_stack(&mut stack);
    board
}

/// Murder resolves BEFORE Regenerate is cast (Alice was too slow).
/// Stack: Murder alone.
/// Bear DIES. Regenerate can't target a dead creature.
pub fn scenario_murder_no_response() -> BoardState {
    let mut board = BoardState::new();
    board.creatures.push(Creature::new("bear", 2, 2));

    let mut stack = Stack::new();
    stack.cast(murder(), "bob", vec![Target::Creature("bear")]);

    board.resolve_stack(&mut stack);
    board
}

/// Bob: Bolt. Alice: Growth in response. Bob: second Bolt in response to Growth.
/// Stack (bottom→top): Bolt₁, Growth, Bolt₂
/// Resolves: Bolt₂ (3 dmg to 2/2 = dead), Growth fizzles, Bolt₁ fizzles.
/// Bear DIES. Bob's timing defeated Alice's pump.
pub fn scenario_triple_stack_bolt_wins() -> BoardState {
    let mut board = BoardState::new();
    board.creatures.push(Creature::new("bear", 2, 2));

    let mut stack = Stack::new();
    let bolt1_id = stack.cast(lightning_bolt(), "bob", vec![Target::Creature("bear")]);
    let growth_id = stack.respond(
        giant_growth(),
        "alice",
        vec![Target::Creature("bear")],
        bolt1_id,
    );
    stack.respond(
        lightning_bolt(),
        "bob",
        vec![Target::Creature("bear")],
        growth_id,
    );

    board.resolve_stack(&mut stack);
    board
}

/// Same 3 cards but Alice orders differently:
/// Alice: Growth first (proactive). Bob: Bolt in response. Alice: second Growth in response.
/// Stack (bottom→top): Growth₁, Bolt, Growth₂
/// Resolves: Growth₂ (bear is 5/5), Bolt (3 dmg to 5/5, survives), Growth₁ (bear is 8/8).
/// Bear LIVES at 8/8 with 3 damage.
pub fn scenario_triple_stack_growth_wins() -> BoardState {
    let mut board = BoardState::new();
    board.creatures.push(Creature::new("bear", 2, 2));

    let mut stack = Stack::new();
    let g1_id = stack.cast(giant_growth(), "alice", vec![Target::Creature("bear")]);
    let bolt_id = stack.respond(
        lightning_bolt(),
        "bob",
        vec![Target::Creature("bear")],
        g1_id,
    );
    stack.respond(
        giant_growth(),
        "alice",
        vec![Target::Creature("bear")],
        bolt_id,
    );

    board.resolve_stack(&mut stack);
    board
}
