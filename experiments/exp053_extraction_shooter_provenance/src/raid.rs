// SPDX-License-Identifier: AGPL-3.0-or-later
//! Extraction shooter raid model wired to the provenance trio.
//!
//! Every raid action is a rhizoCrypt DAG vertex. Every item is a loamSpine
//! certificate. Every player action is a sweetGrass braid. Fraud detection
//! reduces to checking provenance chain integrity.
//!
//! ## Consumable Lifecycle
//!
//! Items fall into two categories:
//! - **Static**: weapons, armor, keys, barter items — persist until extracted or lost on death.
//! - **Consumable**: ammo rounds, medical, food — consumed on use. The cert transitions
//!   to `Consumed` state. The DAG vertex proving existence persists forever.
//!
//! A round fired without a matching cert = phantom round = provenance gap = fraud.
//! A medkit consumed twice = overconsumption = provenance violation.

use std::collections::HashMap;

// ============================================================================
// Domain Types
// ============================================================================

pub type PlayerId = &'static str;
pub type ItemId = String;
pub type ZoneId = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Pmc,
    Scav,
}

#[derive(Debug, Clone)]
pub struct RaidEntity {
    pub id: PlayerId,
    pub kind: EntityKind,
    pub zone: ZoneId,
    pub health: i32,
    pub alive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemCategory {
    Weapon,
    Ammo,
    Armor,
    Medical,
    Food,
    Key,
    Barter,
    Container,
}

#[derive(Debug, Clone)]
pub struct LootItem {
    pub id: ItemId,
    pub name: &'static str,
    pub category: ItemCategory,
    pub value_roubles: u32,
}

#[derive(Debug, Clone)]
pub struct BulletEvent {
    pub shooter: PlayerId,
    pub target: PlayerId,
    pub weapon: &'static str,
    pub distance_m: f64,
    pub headshot: bool,
    pub damage: i32,
    /// If tracked, the specific round cert consumed by this shot.
    pub round_id: Option<ItemId>,
}

// ============================================================================
// Consumable State
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumableState {
    Intact,
    Consumed { consumed_at_tick: u64 },
}

/// A magazine holding individually-tracked rounds. Each round is a `LootItem`
/// with its own certificate and provenance vertex.
#[derive(Debug, Clone)]
pub struct Magazine {
    pub id: ItemId,
    pub caliber: &'static str,
    pub weapon_id: ItemId,
    pub round_ids: Vec<ItemId>,
    pub capacity: u16,
}

// ============================================================================
// Raid Actions — every action becomes a DAG vertex
// ============================================================================

#[derive(Debug, Clone)]
pub enum RaidAction {
    Spawn {
        entity: PlayerId,
        zone: ZoneId,
    },
    Move {
        entity: PlayerId,
        from: ZoneId,
        to: ZoneId,
    },
    LootPickup {
        entity: PlayerId,
        item: ItemId,
        source: LootSource,
    },
    LootDrop {
        entity: PlayerId,
        item: ItemId,
    },
    Fire(BulletEvent),
    Kill {
        killer: PlayerId,
        victim: PlayerId,
    },
    Extract {
        entity: PlayerId,
        zone: ZoneId,
    },
    OpenContainer {
        entity: PlayerId,
        container: &'static str,
    },
    Heal {
        entity: PlayerId,
        item: ItemId,
        hp_restored: i32,
    },
    Consume {
        entity: PlayerId,
        item: ItemId,
    },
    LoadMagazine {
        entity: PlayerId,
        mag_id: ItemId,
        round_ids: Vec<ItemId>,
    },
    TopUpMagazine {
        entity: PlayerId,
        mag_id: ItemId,
        added_round_ids: Vec<ItemId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootSource {
    Ground,
    Container(&'static str),
    Corpse(PlayerId),
    Spawn,
}

// ============================================================================
// Raid Session — provenance-tracked game state
// ============================================================================

pub struct RaidSession {
    pub entities: HashMap<PlayerId, RaidEntity>,
    pub inventories: HashMap<PlayerId, Vec<ItemId>>,
    pub items: HashMap<ItemId, LootItem>,
    pub action_log: Vec<TimestampedAction>,
    pub vertex_ids: Vec<rhizo_crypt_core::VertexId>,
    pub certificates: HashMap<ItemId, loam_spine_core::Certificate>,
    /// Items that were part of a player's spawn loadout (immutable record).
    pub spawn_loadout_items: HashMap<PlayerId, Vec<ItemId>>,
    /// Consumable state for each item. Static items are absent (implicitly intact).
    pub consumable_states: HashMap<ItemId, ConsumableState>,
    /// Magazines indexed by mag ID.
    pub magazines: HashMap<ItemId, Magazine>,
    /// Zone adjacency and line-of-sight for spatial fraud detection.
    pub zone_topology: Option<ZoneTopology>,
    /// Spoof markers: `(claimed_player, tick)` where identity was spoofed.
    pub spoof_claims: Vec<(PlayerId, u64)>,
    /// Actual zone of the real actor at spoof time.
    pub spoof_actual_zones: HashMap<(PlayerId, u64), ZoneId>,
    rhizo_session: rhizo_crypt_core::Session,
    rhizo_frontier: Vec<rhizo_crypt_core::VertexId>,
    spine_id: uuid::Uuid,
    tick_ms: u64,
}

#[derive(Debug, Clone)]
pub struct TimestampedAction {
    pub tick_ms: u64,
    pub action: RaidAction,
    /// The actor's zone at the time of the action (for spatial fraud detection).
    pub actor_zone: Option<ZoneId>,
}

impl RaidSession {
    #[must_use]
    pub fn new(map_name: &str) -> Self {
        let rhizo_session =
            rhizo_crypt_core::SessionBuilder::new(rhizo_crypt_core::session::SessionType::Gaming {
                game_id: "extraction_shooter".into(),
            })
            .with_name(format!("Raid: {map_name}"))
            .build();

        Self {
            entities: HashMap::new(),
            inventories: HashMap::new(),
            items: HashMap::new(),
            action_log: Vec::new(),
            vertex_ids: Vec::new(),
            certificates: HashMap::new(),
            spawn_loadout_items: HashMap::new(),
            consumable_states: HashMap::new(),
            magazines: HashMap::new(),
            zone_topology: None,
            spoof_claims: Vec::new(),
            spoof_actual_zones: HashMap::new(),
            rhizo_session,
            rhizo_frontier: Vec::new(),
            spine_id: uuid::Uuid::now_v7(),
            tick_ms: 0,
        }
    }

    #[must_use]
    pub const fn rhizo_session_active(&self) -> bool {
        self.rhizo_session.is_active()
    }

    pub const fn advance_time(&mut self, delta_ms: u64) {
        self.tick_ms += delta_ms;
    }

    fn push_action(&mut self, action: RaidAction, actor: Option<PlayerId>) {
        let actor_zone = actor.and_then(|id| self.entities.get(id).map(|e| e.zone));
        self.action_log.push(TimestampedAction {
            tick_ms: self.tick_ms,
            action,
            actor_zone,
        });
    }

    fn append_vertex(&mut self, action_str: &str, agent_did: &str) -> rhizo_crypt_core::VertexId {
        let agent = rhizo_crypt_core::Did::new(format!("did:key:{agent_did}"));
        let mut builder =
            rhizo_crypt_core::VertexBuilder::new(rhizo_crypt_core::EventType::AgentAction {
                action: action_str.into(),
            })
            .with_agent(agent);

        for &parent in &self.rhizo_frontier {
            builder = builder.with_parent(parent);
        }

        let vertex = builder.build();
        let vid = vertex.compute_id().expect("vertex id computation");
        self.rhizo_session
            .update_frontier(vid, &self.rhizo_frontier);
        self.rhizo_frontier = vec![vid];
        self.vertex_ids.push(vid);
        vid
    }

    fn mint_item_cert(&mut self, item: &LootItem, owner_id: PlayerId) {
        let owner = loam_spine_core::Did::new(format!("did:key:{owner_id}"));
        let cert_id = uuid::Uuid::now_v7();
        let entry_hash = [0u8; 32];
        let mint_info =
            loam_spine_core::certificate::MintInfo::new(owner.clone(), self.spine_id, entry_hash);

        let mut attrs = HashMap::new();
        attrs.insert("item_name".into(), item.name.into());
        attrs.insert("value".into(), item.value_roubles.to_string());

        let cert = loam_spine_core::Certificate::new(
            cert_id,
            loam_spine_core::certificate::CertificateType::GameItem {
                game_id: "extraction_shooter".into(),
                item_type: format!("{:?}", item.category),
                item_id: item.id.clone(),
                attributes: attrs,
            },
            &owner,
            &mint_info,
        );
        self.certificates.insert(item.id.clone(), cert);
    }

    // === Public action methods ===

    pub fn spawn_entity(
        &mut self,
        id: PlayerId,
        kind: EntityKind,
        zone: ZoneId,
        loadout: &[LootItem],
    ) {
        self.entities.insert(
            id,
            RaidEntity {
                id,
                kind,
                zone,
                health: 100,
                alive: true,
            },
        );
        let mut inv = Vec::new();
        for item in loadout {
            self.items.insert(item.id.clone(), item.clone());
            self.mint_item_cert(item, id);
            if item.category == ItemCategory::Medical
                || item.category == ItemCategory::Food
                || item.category == ItemCategory::Ammo
            {
                self.consumable_states
                    .insert(item.id.clone(), ConsumableState::Intact);
            }
            inv.push(item.id.clone());
        }
        self.spawn_loadout_items.insert(id, inv.clone());
        self.inventories.insert(id, inv);
        self.append_vertex("spawn", id);
        self.push_action(RaidAction::Spawn { entity: id, zone }, Some(id));
    }

    /// Add an item to a player's inventory after spawn (e.g., ammo rounds).
    /// Mints a cert and marks consumables appropriately.
    pub fn spawn_item_in_inventory(&mut self, player: PlayerId, item: &LootItem) {
        self.items.insert(item.id.clone(), item.clone());
        self.mint_item_cert(item, player);
        if item.category == ItemCategory::Medical
            || item.category == ItemCategory::Food
            || item.category == ItemCategory::Ammo
        {
            self.consumable_states
                .insert(item.id.clone(), ConsumableState::Intact);
        }
        self.inventories
            .entry(player)
            .or_default()
            .push(item.id.clone());
        self.spawn_loadout_items
            .entry(player)
            .or_default()
            .push(item.id.clone());
    }

    pub fn move_entity(&mut self, id: PlayerId, to: ZoneId) {
        let from = self.entities[id].zone;
        if let Some(e) = self.entities.get_mut(id) {
            e.zone = to;
        }
        self.append_vertex(&format!("move:{to}"), id);
        self.push_action(
            RaidAction::Move {
                entity: id,
                from,
                to,
            },
            Some(id),
        );
    }

    pub fn open_container(&mut self, id: PlayerId, container: &'static str) {
        self.append_vertex(&format!("open:{container}"), id);
        self.push_action(
            RaidAction::OpenContainer {
                entity: id,
                container,
            },
            Some(id),
        );
    }

    pub fn spawn_ground_loot(&mut self, item: LootItem) {
        self.items.insert(item.id.clone(), item);
    }

    pub fn pickup_loot(&mut self, id: PlayerId, item_id: &str, source: LootSource) {
        self.inventories.entry(id).or_default().push(item_id.into());
        if !self.certificates.contains_key(item_id) {
            if let Some(item) = self.items.get(item_id) {
                let item_clone = item.clone();
                self.mint_item_cert(&item_clone, id);
            }
        }
        if let Some(item) = self.items.get(item_id) {
            if item.category == ItemCategory::Medical
                || item.category == ItemCategory::Food
                || item.category == ItemCategory::Ammo
            {
                self.consumable_states
                    .entry(item_id.into())
                    .or_insert(ConsumableState::Intact);
            }
        }
        self.append_vertex(&format!("loot:{item_id}"), id);
        self.push_action(
            RaidAction::LootPickup {
                entity: id,
                item: item_id.into(),
                source,
            },
            Some(id),
        );
    }

    /// Fire without round tracking (legacy path for non-consumable scenarios).
    pub fn fire_bullet(&mut self, event: BulletEvent) {
        if let Some(target) = self.entities.get_mut(event.target) {
            target.health -= event.damage;
        }
        let shooter = event.shooter;
        self.append_vertex(
            &format!("fire:{}->{}:{}", event.shooter, event.target, event.weapon),
            shooter,
        );
        self.push_action(RaidAction::Fire(event), Some(shooter));
    }

    /// Fire a tracked round. The round cert transitions to Consumed. If the
    /// round was never in the shooter's inventory, the DAG still records the
    /// fire vertex — fraud detection will flag the phantom round.
    pub fn fire_round(&mut self, event: BulletEvent, round_id: &str) {
        if let Some(target) = self.entities.get_mut(event.target) {
            target.health -= event.damage;
        }
        self.consumable_states.insert(
            round_id.into(),
            ConsumableState::Consumed {
                consumed_at_tick: self.tick_ms,
            },
        );
        // Remove from magazine if present
        for mag in self.magazines.values_mut() {
            mag.round_ids.retain(|r| r != round_id);
        }
        let shooter = event.shooter;
        self.append_vertex(
            &format!(
                "fire_round:{}->{}:{}:{}",
                event.shooter, event.target, event.weapon, round_id
            ),
            shooter,
        );
        self.push_action(RaidAction::Fire(event), Some(shooter));
    }

    /// Load a magazine with specific rounds. Each round must already be in
    /// the player's inventory with a valid cert.
    pub fn load_magazine(
        &mut self,
        player: PlayerId,
        mag_id: &str,
        caliber: &'static str,
        weapon_id: &str,
        rounds: &[LootItem],
    ) {
        let round_ids: Vec<ItemId> = rounds.iter().map(|r| r.id.clone()).collect();
        let count = round_ids.len();
        let mag = Magazine {
            id: mag_id.into(),
            caliber,
            weapon_id: weapon_id.into(),
            round_ids: round_ids.clone(),
            capacity: 30,
        };
        self.magazines.insert(mag_id.into(), mag);
        self.append_vertex(&format!("load_mag:{mag_id}:{count}"), player);
        self.push_action(
            RaidAction::LoadMagazine {
                entity: player,
                mag_id: mag_id.into(),
                round_ids,
            },
            Some(player),
        );
    }

    /// Add found rounds to an existing magazine.
    pub fn top_up_magazine(&mut self, player: PlayerId, mag_id: &str, rounds: &[LootItem]) {
        let added_ids: Vec<ItemId> = rounds.iter().map(|r| r.id.clone()).collect();
        if let Some(mag) = self.magazines.get_mut(mag_id) {
            mag.round_ids.extend(added_ids.clone());
        }
        self.append_vertex(&format!("topup_mag:{mag_id}:{}", added_ids.len()), player);
        self.push_action(
            RaidAction::TopUpMagazine {
                entity: player,
                mag_id: mag_id.into(),
                added_round_ids: added_ids,
            },
            Some(player),
        );
    }

    /// Consume an item (medical, food, ammo used outside combat). Marks cert
    /// as consumed. Multiple calls on the same item = overconsumption fraud.
    pub fn consume_item(&mut self, player: PlayerId, item_id: &str, uses: u16) {
        for _ in 0..uses {
            self.consumable_states.insert(
                item_id.into(),
                ConsumableState::Consumed {
                    consumed_at_tick: self.tick_ms,
                },
            );
        }
        self.append_vertex(&format!("consume:{item_id}"), player);
        self.push_action(
            RaidAction::Consume {
                entity: player,
                item: item_id.into(),
            },
            Some(player),
        );
    }

    pub fn kill(&mut self, killer: PlayerId, victim: PlayerId) {
        if let Some(v) = self.entities.get_mut(victim) {
            v.alive = false;
            v.health = 0;
        }
        self.append_vertex(&format!("kill:{victim}"), killer);
        self.push_action(RaidAction::Kill { killer, victim }, Some(killer));
    }

    pub fn extract(&mut self, id: PlayerId, zone: ZoneId) {
        self.append_vertex(&format!("extract:{zone}"), id);
        self.push_action(RaidAction::Extract { entity: id, zone }, Some(id));
    }

    pub fn set_topology(&mut self, topology: ZoneTopology) {
        self.zone_topology = Some(topology);
    }

    /// Directly set a player's zone (for ghost/spoof scenario setup).
    pub fn set_entity_zone(&mut self, id: PlayerId, zone: ZoneId) {
        if let Some(e) = self.entities.get_mut(id) {
            e.zone = zone;
        }
    }

    // === Fraud injection (for testing) ===

    /// Inject an item into inventory WITHOUT a loot vertex. Simulates duplication exploit.
    pub fn inject_orphan_item(&mut self, player: PlayerId, item: &LootItem) {
        self.items.insert(item.id.clone(), item.clone());
        let item_id = item.id.clone();
        self.mint_item_cert(item, player);
        self.inventories.entry(player).or_default().push(item_id);
    }

    /// Mark a fire event as identity-spoofed. The real actor is elsewhere.
    pub fn mark_spoof(&mut self, claimed: PlayerId, actual_zone: ZoneId) {
        self.spoof_claims.push((claimed, self.tick_ms));
        self.spoof_actual_zones
            .insert((claimed, self.tick_ms), actual_zone);
    }

    /// Duplicate an existing certificate. Simulates item duplication glitch.
    pub fn duplicate_cert(&mut self, original_id: &str, new_id: &str) {
        if let Some(cert) = self.certificates.get(original_id) {
            let mut dupe = cert.clone();
            dupe.id = uuid::Uuid::now_v7();
            self.certificates.insert(new_id.into(), dupe);
        }
    }
}

// ============================================================================
// Zone Topology — adjacency and line-of-sight for spatial fraud detection
// ============================================================================

/// Map topology defining which zones connect and which have line-of-sight.
/// Required for through-wall, teleport, and ghost action detection.
#[derive(Debug, Clone)]
pub struct ZoneTopology {
    /// Bidirectional adjacency: `(a, b)` means a player can walk between them.
    adjacency: Vec<(ZoneId, ZoneId)>,
    /// Bidirectional line-of-sight: `(a, b)` means shots can travel between them.
    /// Superset of adjacency (adjacent zones always have `LoS`).
    line_of_sight: Vec<(ZoneId, ZoneId)>,
}

impl ZoneTopology {
    #[must_use]
    pub const fn new(
        adjacency: Vec<(ZoneId, ZoneId)>,
        line_of_sight: Vec<(ZoneId, ZoneId)>,
    ) -> Self {
        Self {
            adjacency,
            line_of_sight,
        }
    }

    #[must_use]
    pub fn is_adjacent(&self, a: ZoneId, b: ZoneId) -> bool {
        a == b
            || self
                .adjacency
                .iter()
                .any(|&(x, y)| (x == a && y == b) || (x == b && y == a))
    }

    #[must_use]
    pub fn has_line_of_sight(&self, a: ZoneId, b: ZoneId) -> bool {
        a == b
            || self.is_adjacent(a, b)
            || self
                .line_of_sight
                .iter()
                .any(|&(x, y)| (x == a && y == b) || (x == b && y == a))
    }
}

/// Build the Customs map topology.
///
/// ```text
///  big_red ──── crossroads ──── gas_station
///                   │                │
///              checkpoint         sniper_rock
///                   │
///              extract_zone
/// ```
///
/// Line-of-sight extends across some non-adjacent zones (e.g.,
/// `big_red` can see `gas_station` across the open field) but
/// `sniper_rock` is behind the station (no `LoS` to `big_red`).
#[must_use]
pub fn customs_topology() -> ZoneTopology {
    ZoneTopology::new(
        vec![
            ("big_red", "crossroads"),
            ("crossroads", "gas_station"),
            ("crossroads", "checkpoint"),
            ("checkpoint", "extract_zone"),
            ("gas_station", "sniper_rock"),
        ],
        vec![("big_red", "gas_station"), ("crossroads", "sniper_rock")],
    )
}
