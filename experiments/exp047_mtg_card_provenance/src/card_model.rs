// SPDX-License-Identifier: AGPL-3.0-or-later
//! MTG card domain model — certificates, zones, actions, board state, sessions.
//!
//! This module contains the game logic extracted from the validation binary.
//! The validation functions in `main.rs` use these types to verify:
//!   1. Card-as-certificate (loamSpine pattern)
//!   2. Game action DAG (tap, untap, cast, attack, block, damage, draw)
//!   3. Board state reconstruction from DAG
//!   4. Zone transitions (library → hand → battlefield → graveyard → exile)

use std::collections::HashMap;

// ── Card certificate (loamSpine pattern) ─────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CardCert {
    pub id: &'static str,
    pub name: &'static str,
    pub set_code: &'static str,
    pub collector_number: u16,
    pub card_type: CardType,
    pub mana_cost: &'static str,
    pub cmc: u8,
    pub power_toughness: Option<(i8, i8)>,
    pub oracle_text: &'static str,
}

#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all variants"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardType {
    Creature,
    Instant,
    Sorcery,
    Enchantment,
    Artifact,
    Land,
    Planeswalker,
}

/// Card condition for physical provenance.
#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all variants"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardCondition {
    Mint,
    NearMint,
    LightlyPlayed,
    ModeratelyPlayed,
    HeavilyPlayed,
    Damaged,
}

/// Ownership certificate — a physical card's provenance.
#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all fields"
)]
#[derive(Debug, Clone)]
pub struct OwnershipCert {
    pub card: &'static str,
    pub owner: &'static str,
    pub condition: CardCondition,
    pub acquired_from: &'static str,
    pub acquisition_method: AcquisitionMethod,
}

#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all variants"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcquisitionMethod {
    PackOpening { set: &'static str },
    Trade { from_owner: &'static str },
    Purchase { seller: &'static str },
    Prize { event: &'static str },
}

// ── Game zones ───────────────────────────────────────────────────────

#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all variants"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Zone {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Exile,
    Stack,
    #[expect(clippy::enum_variant_names, reason = "domain nomenclature")]
    CmdZone,
}

// ── Game action DAG ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[expect(dead_code, reason = "domain model completeness")]
pub struct GameVertex {
    pub id: usize,
    pub parent: Option<usize>,
    pub action: GameAction,
    pub player: &'static str,
    pub description: String,
}

#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all variants"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameAction {
    GameStart,
    Draw { card: &'static str },
    PlayLand { card: &'static str },
    TapLand { card: &'static str },
    CastSpell { card: &'static str, mana_paid: u8 },
    DeclareAttacker { card: &'static str },
    DeclareBlocker { blocker: &'static str, blocking: &'static str },
    DealDamage { source: &'static str, target: &'static str, amount: u8 },
    CreatureDies { card: &'static str },
    PhaseChange { phase: &'static str },
    PassPriority,
    Untap { card: &'static str },
}

// ── Board state ──────────────────────────────────────────────────────

/// Board state reconstructed from the DAG.
#[derive(Debug, Clone)]
pub struct BoardState {
    pub zones: HashMap<(&'static str, Zone), Vec<&'static str>>,
    pub life_totals: HashMap<&'static str, i32>,
    pub tapped: Vec<&'static str>,
}

impl BoardState {
    pub fn new(players: &[&'static str]) -> Self {
        let mut zones = HashMap::new();
        let mut life_totals = HashMap::new();
        for &player in players {
            zones.insert((player, Zone::Library), Vec::new());
            zones.insert((player, Zone::Hand), Vec::new());
            zones.insert((player, Zone::Battlefield), Vec::new());
            zones.insert((player, Zone::Graveyard), Vec::new());
            zones.insert((player, Zone::Exile), Vec::new());
            life_totals.insert(player, 20);
        }
        Self {
            zones,
            life_totals,
            tapped: Vec::new(),
        }
    }

    pub fn move_card(&mut self, player: &'static str, card: &'static str, from: Zone, to: Zone) {
        if let Some(from_zone) = self.zones.get_mut(&(player, from)) {
            from_zone.retain(|&c| c != card);
        }
        self.zones.entry((player, to)).or_default().push(card);
    }

    pub fn cards_in(&self, player: &'static str, zone: Zone) -> &[&'static str] {
        self.zones.get(&(player, zone)).map_or(&[], Vec::as_slice)
    }

    pub fn apply(&mut self, vertex: &GameVertex) {
        let player = vertex.player;
        match &vertex.action {
            GameAction::Draw { card } => {
                self.move_card(player, card, Zone::Library, Zone::Hand);
            }
            GameAction::PlayLand { card } | GameAction::CastSpell { card, .. } => {
                self.move_card(player, card, Zone::Hand, Zone::Battlefield);
            }
            GameAction::TapLand { card } | GameAction::DeclareAttacker { card } => {
                self.tapped.push(card);
            }
            GameAction::CreatureDies { card } => {
                self.move_card(player, card, Zone::Battlefield, Zone::Graveyard);
                self.tapped.retain(|&c| c != *card);
            }
            GameAction::DealDamage { target, amount, .. } => {
                if let Some(life) = self.life_totals.get_mut(target) {
                    *life -= i32::from(*amount);
                }
            }
            GameAction::Untap { card } => {
                self.tapped.retain(|&c| c != *card);
            }
            _ => {}
        }
    }
}

// ── Game session ─────────────────────────────────────────────────────

pub struct GameSession {
    pub vertices: Vec<GameVertex>,
    pub board: BoardState,
    pub cards: HashMap<&'static str, CardCert>,
}

impl GameSession {
    #[expect(
        clippy::too_many_lines,
        reason = "session setup — card catalog and initial state"
    )]
    pub fn new() -> Self {
        let players = &["alice", "bob"];
        let mut board = BoardState::new(players);
        let mut cards = HashMap::new();

        let alice_cards = vec![
            CardCert {
                id: "a_forest_1",
                name: "Forest",
                set_code: "MKM",
                collector_number: 280,
                card_type: CardType::Land,
                mana_cost: "",
                cmc: 0,
                power_toughness: None,
                oracle_text: "({T}: Add {G}.)",
            },
            CardCert {
                id: "a_forest_2",
                name: "Forest",
                set_code: "MKM",
                collector_number: 280,
                card_type: CardType::Land,
                mana_cost: "",
                cmc: 0,
                power_toughness: None,
                oracle_text: "({T}: Add {G}.)",
            },
            CardCert {
                id: "a_bear",
                name: "Grizzly Bears",
                set_code: "10E",
                collector_number: 268,
                card_type: CardType::Creature,
                mana_cost: "{1}{G}",
                cmc: 2,
                power_toughness: Some((2, 2)),
                oracle_text: "",
            },
            CardCert {
                id: "a_giant",
                name: "Giant Growth",
                set_code: "M10",
                collector_number: 178,
                card_type: CardType::Instant,
                mana_cost: "{G}",
                cmc: 1,
                power_toughness: None,
                oracle_text: "Target creature gets +3/+3 until end of turn.",
            },
        ];

        let bob_cards = vec![
            CardCert {
                id: "b_mountain_1",
                name: "Mountain",
                set_code: "MKM",
                collector_number: 283,
                card_type: CardType::Land,
                mana_cost: "",
                cmc: 0,
                power_toughness: None,
                oracle_text: "({T}: Add {R}.)",
            },
            CardCert {
                id: "b_goblin",
                name: "Goblin Piker",
                set_code: "M10",
                collector_number: 142,
                card_type: CardType::Creature,
                mana_cost: "{1}{R}",
                cmc: 2,
                power_toughness: Some((2, 1)),
                oracle_text: "",
            },
            CardCert {
                id: "b_bolt",
                name: "Lightning Bolt",
                set_code: "M10",
                collector_number: 146,
                card_type: CardType::Instant,
                mana_cost: "{R}",
                cmc: 1,
                power_toughness: None,
                oracle_text: "Lightning Bolt deals 3 damage to any target.",
            },
        ];

        for card in &alice_cards {
            board
                .zones
                .entry(("alice", Zone::Hand))
                .or_default()
                .push(card.id);
            cards.insert(card.id, card.clone());
        }
        for card in &bob_cards {
            board
                .zones
                .entry(("bob", Zone::Hand))
                .or_default()
                .push(card.id);
            cards.insert(card.id, card.clone());
        }

        let initial = GameVertex {
            id: 0,
            parent: None,
            action: GameAction::GameStart,
            player: "system",
            description: "Game begins. Alice vs Bob.".into(),
        };

        Self {
            vertices: vec![initial],
            board,
            cards,
        }
    }

    pub fn add_action(&mut self, player: &'static str, action: GameAction, description: &str) {
        let parent = self.vertices.len() - 1;
        let vertex = GameVertex {
            id: self.vertices.len(),
            parent: Some(parent),
            action,
            player,
            description: description.into(),
        };
        self.board.apply(&vertex);
        self.vertices.push(vertex);
    }

    pub fn simulate_turn_sequence(&mut self) {
        self.add_action("alice", GameAction::PlayLand { card: "a_forest_1" }, "Alice plays Forest.");
        self.add_action("alice", GameAction::PassPriority, "Alice passes.");

        self.add_action("bob", GameAction::PlayLand { card: "b_mountain_1" }, "Bob plays Mountain.");
        self.add_action("bob", GameAction::PassPriority, "Bob passes.");

        self.add_action("alice", GameAction::PlayLand { card: "a_forest_2" }, "Alice plays Forest.");
        self.add_action("alice", GameAction::TapLand { card: "a_forest_1" }, "Alice taps Forest for {G}.");
        self.add_action("alice", GameAction::TapLand { card: "a_forest_2" }, "Alice taps Forest for {G}.");
        self.add_action("alice", GameAction::CastSpell { card: "a_bear", mana_paid: 2 }, "Alice casts Grizzly Bears.");
        self.add_action("alice", GameAction::PassPriority, "Alice passes.");

        self.add_action("bob", GameAction::TapLand { card: "b_mountain_1" }, "Bob taps Mountain for {R}.");
        self.add_action("bob", GameAction::CastSpell { card: "b_bolt", mana_paid: 1 }, "Bob casts Lightning Bolt targeting Grizzly Bears.");
        self.add_action("bob", GameAction::DealDamage { source: "b_bolt", target: "a_bear", amount: 3 }, "Lightning Bolt deals 3 damage to Grizzly Bears.");
        self.add_action("alice", GameAction::CreatureDies { card: "a_bear" }, "Grizzly Bears dies (3 damage, 2 toughness).");

        self.add_action("alice", GameAction::Untap { card: "a_forest_1" }, "Alice untaps Forest.");
        self.add_action("alice", GameAction::Untap { card: "a_forest_2" }, "Alice untaps Forest.");
        self.add_action("alice", GameAction::PassPriority, "Alice passes.");
    }
}
