// SPDX-License-Identifier: AGPL-3.0-or-later
//! Two-player raid simulation with provenance tracking.
//!
//! Simulates a tick-based extraction raid between two players, tracking
//! every action as a rhizoCrypt DAG vertex and every item as a loamSpine
//! certificate. Produces [`RaidSnapshot`] for visualization.

#![forbid(unsafe_code)]

use std::collections::HashMap;

// ============================================================================
// Domain types
// ============================================================================

pub type PlayerId = &'static str;
pub type ZoneId = &'static str;
pub type ItemId = String;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub name: &'static str,
    pub zone: ZoneId,
    pub health: f64,
    pub alive: bool,
    pub inventory: Vec<ItemId>,
}

#[derive(Debug, Clone)]
#[expect(dead_code, reason = "domain model completeness")]
pub enum RaidAction {
    Spawn {
        player: PlayerId,
        zone: ZoneId,
    },
    Move {
        player: PlayerId,
        to: ZoneId,
    },
    Fire {
        shooter: PlayerId,
        target: PlayerId,
        damage: f64,
    },
    Loot {
        player: PlayerId,
        item: ItemId,
    },
    Heal {
        player: PlayerId,
        amount: f64,
    },
    Extract {
        player: PlayerId,
        zone: ZoneId,
    },
}

#[derive(Debug, Clone)]
pub struct TickEntry {
    pub tick: u64,
    pub action: RaidAction,
    pub vertex_id: rhizo_crypt_core::VertexId,
}

/// Snapshot of the raid state for visualization.
pub struct RaidSnapshot {
    pub raid_id: String,
    pub map_name: String,
    pub zone_names: Vec<&'static str>,
    pub zone_adjacency: Vec<(usize, usize)>,
    pub player_health: Vec<(&'static str, f64)>,
    pub action_timelines: Vec<(&'static str, Vec<f64>, Vec<f64>)>,
    pub fraud_categories: Vec<&'static str>,
    pub fraud_counts: Vec<f64>,
    pub inventories: Vec<(&'static str, Vec<&'static str>, Vec<f64>)>,
    pub total_vertices: usize,
    pub total_certs: usize,
    #[expect(dead_code, reason = "domain model completeness")]
    pub total_ticks: u64,
}

// ============================================================================
// Simulation engine
// ============================================================================

pub struct TwoPlayerRaid {
    pub players: HashMap<PlayerId, PlayerState>,
    pub action_log: Vec<TickEntry>,
    pub tick: u64,
    pub certs: HashMap<String, uuid::Uuid>,
    rhizo_session: rhizo_crypt_core::Session,
    rhizo_frontier: Vec<rhizo_crypt_core::VertexId>,
    vertex_ids: Vec<rhizo_crypt_core::VertexId>,
}

impl TwoPlayerRaid {
    #[must_use]
    pub fn new() -> Self {
        let rhizo_session =
            rhizo_crypt_core::SessionBuilder::new(rhizo_crypt_core::session::SessionType::Gaming {
                game_id: "extraction_raid_2p".into(),
            })
            .with_name("2-player raid")
            .build();

        Self {
            players: HashMap::new(),
            action_log: Vec::new(),
            tick: 0,
            certs: HashMap::new(),
            rhizo_session,
            rhizo_frontier: Vec::new(),
            vertex_ids: Vec::new(),
        }
    }

    fn append_vertex(&mut self, label: &str, agent: &str) -> rhizo_crypt_core::VertexId {
        let agent_did = rhizo_crypt_core::Did::new(format!("did:key:{agent}"));
        let mut builder =
            rhizo_crypt_core::VertexBuilder::new(rhizo_crypt_core::EventType::AgentAction {
                action: label.into(),
            })
            .with_agent(agent_did);

        for &parent in &self.rhizo_frontier {
            builder = builder.with_parent(parent);
        }

        let vertex = builder.build();
        let vid = vertex.compute_id();
        self.rhizo_session
            .update_frontier(vid, &self.rhizo_frontier);
        self.rhizo_frontier = vec![vid];
        self.vertex_ids.push(vid);
        vid
    }

    fn mint_cert(&mut self, item_id: &str) {
        self.certs.insert(item_id.into(), uuid::Uuid::now_v7());
    }

    pub fn spawn(&mut self, name: PlayerId, zone: ZoneId, items: &[&str]) {
        self.players.insert(
            name,
            PlayerState {
                name,
                zone,
                health: 100.0,
                alive: true,
                inventory: items.iter().map(|s| (*s).into()).collect(),
            },
        );
        for item in items {
            self.mint_cert(item);
        }
        let vid = self.append_vertex(&format!("spawn:{zone}"), name);
        self.action_log.push(TickEntry {
            tick: self.tick,
            action: RaidAction::Spawn { player: name, zone },
            vertex_id: vid,
        });
    }

    #[expect(clippy::missing_const_for_fn, reason = "mutates self")]
    pub fn advance(&mut self, ticks: u64) {
        self.tick += ticks;
    }

    pub fn move_player(&mut self, player: PlayerId, to: ZoneId) {
        if let Some(p) = self.players.get_mut(player) {
            p.zone = to;
        }
        let vid = self.append_vertex(&format!("move:{to}"), player);
        self.action_log.push(TickEntry {
            tick: self.tick,
            action: RaidAction::Move { player, to },
            vertex_id: vid,
        });
    }

    pub fn fire(&mut self, shooter: PlayerId, target: PlayerId, damage: f64) {
        if let Some(t) = self.players.get_mut(target) {
            t.health = (t.health - damage).max(0.0);
            if t.health <= 0.0 {
                t.alive = false;
            }
        }
        let vid = self.append_vertex(&format!("fire:{target}"), shooter);
        self.action_log.push(TickEntry {
            tick: self.tick,
            action: RaidAction::Fire {
                shooter,
                target,
                damage,
            },
            vertex_id: vid,
        });
    }

    pub fn loot(&mut self, player: PlayerId, item: &str) {
        self.mint_cert(item);
        if let Some(p) = self.players.get_mut(player) {
            p.inventory.push(item.into());
        }
        let vid = self.append_vertex(&format!("loot:{item}"), player);
        self.action_log.push(TickEntry {
            tick: self.tick,
            action: RaidAction::Loot {
                player,
                item: item.into(),
            },
            vertex_id: vid,
        });
    }

    pub fn heal(&mut self, player: PlayerId, amount: f64) {
        if let Some(p) = self.players.get_mut(player) {
            p.health = (p.health + amount).min(100.0);
        }
        let vid = self.append_vertex("heal", player);
        self.action_log.push(TickEntry {
            tick: self.tick,
            action: RaidAction::Heal { player, amount },
            vertex_id: vid,
        });
    }

    pub fn extract(&mut self, player: PlayerId, zone: ZoneId) {
        let vid = self.append_vertex(&format!("extract:{zone}"), player);
        self.action_log.push(TickEntry {
            tick: self.tick,
            action: RaidAction::Extract { player, zone },
            vertex_id: vid,
        });
    }

    #[expect(
        clippy::missing_const_for_fn,
        reason = "calls non-const Session method"
    )]
    pub fn rhizo_active(&self) -> bool {
        self.rhizo_session.is_active()
    }

    #[expect(clippy::missing_const_for_fn, reason = "Vec::len not const")]
    pub fn vertex_count(&self) -> usize {
        self.vertex_ids.len()
    }

    pub fn cert_count(&self) -> usize {
        self.certs.len()
    }

    /// Build a snapshot for visualization.
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "counts fit in f64")]
    pub fn snapshot(&self) -> RaidSnapshot {
        let zones = vec![
            "big_red",
            "crossroads",
            "gas_station",
            "checkpoint",
            "extract_zone",
            "sniper_rock",
        ];
        let adjacency = vec![(0, 1), (1, 2), (1, 3), (3, 4), (2, 5)];

        let player_health: Vec<_> = self.players.values().map(|p| (p.name, p.health)).collect();

        let mut action_counts: HashMap<PlayerId, Vec<(u64, usize)>> = HashMap::new();
        for entry in &self.action_log {
            let player = match &entry.action {
                RaidAction::Spawn { player, .. }
                | RaidAction::Move { player, .. }
                | RaidAction::Loot { player, .. }
                | RaidAction::Heal { player, .. }
                | RaidAction::Extract { player, .. } => *player,
                RaidAction::Fire { shooter, .. } => *shooter,
            };
            action_counts
                .entry(player)
                .or_default()
                .push((entry.tick, 1));
        }

        let action_timelines: Vec<_> = action_counts
            .into_iter()
            .map(|(name, entries)| {
                let ticks: Vec<f64> = entries.iter().map(|(t, _)| *t as f64).collect();
                let counts: Vec<f64> = entries.iter().map(|(_, c)| *c as f64).collect();
                (name, ticks, counts)
            })
            .collect();

        let inventories: Vec<_> = self
            .players
            .values()
            .map(|p| {
                let cats: Vec<&str> = vec!["Weapon", "Ammo", "Medical", "Other"];
                let counts = vec![
                    p.inventory
                        .iter()
                        .filter(|i| i.contains("ak") || i.contains("m4"))
                        .count() as f64,
                    p.inventory.iter().filter(|i| i.contains("mag")).count() as f64,
                    p.inventory.iter().filter(|i| i.contains("ifak")).count() as f64,
                    p.inventory
                        .iter()
                        .filter(|i| {
                            !i.contains("ak")
                                && !i.contains("m4")
                                && !i.contains("mag")
                                && !i.contains("ifak")
                        })
                        .count() as f64,
                ];
                (p.name, cats, counts)
            })
            .collect();

        RaidSnapshot {
            raid_id: "sim-2p-001".into(),
            map_name: "Customs".into(),
            zone_names: zones,
            zone_adjacency: adjacency,
            player_health,
            action_timelines,
            fraud_categories: vec![
                "Orphan",
                "DupeCert",
                "Speed",
                "Range",
                "Unattributed",
                "Aimbot",
                "Phantom",
                "OverConsume",
                "Spoof",
                "Ghost",
                "WallHack",
                "Teleport",
            ],
            fraud_counts: vec![0.0; 12],
            inventories,
            total_vertices: self.vertex_ids.len(),
            total_certs: self.certs.len(),
            total_ticks: self.tick,
        }
    }
}

// ============================================================================
// Scenario: honest 2-player raid
// ============================================================================

/// Two PMCs spawn, move through the map, fight a scav, loot, extract.
#[must_use]
pub fn run_honest_2p_raid() -> TwoPlayerRaid {
    let mut raid = TwoPlayerRaid::new();

    raid.spawn("player1", "big_red", &["p1_ak", "p1_mag", "p1_ifak"]);
    raid.advance(1);
    raid.spawn("player2", "gas_station", &["p2_m4", "p2_mag", "p2_ifak"]);
    raid.advance(5);

    // Player 1 moves big_red → crossroads
    raid.move_player("player1", "crossroads");
    raid.advance(3);

    // Player 2 moves gas_station → crossroads
    raid.move_player("player2", "crossroads");
    raid.advance(2);

    // Both engage a scav (not tracked as a player — just fire actions)
    raid.fire("player1", "player2", 0.0); // Friendly fire miss
    raid.advance(1);
    raid.fire("player1", "player2", 15.0); // Scav hits player2
    raid.advance(1);
    raid.heal("player2", 15.0);
    raid.advance(2);

    // Player 1 loots
    raid.loot("player1", "found_gpu");
    raid.advance(3);
    raid.loot("player2", "found_armor");
    raid.advance(2);

    // Move to extract
    raid.move_player("player1", "checkpoint");
    raid.advance(3);
    raid.move_player("player1", "extract_zone");
    raid.advance(2);
    raid.extract("player1", "extract_zone");

    raid.move_player("player2", "checkpoint");
    raid.advance(3);
    raid.move_player("player2", "extract_zone");
    raid.advance(2);
    raid.extract("player2", "extract_zone");

    raid
}
