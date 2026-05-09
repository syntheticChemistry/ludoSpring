// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fraud detection via provenance chain analysis.
//!
//! Every detector takes an immutable reference to a [`RaidSession`] and returns
//! zero or more violation records. The detectors compose into a [`FraudReport`].
//!
//! ## Detection categories
//!
//! | Level | Fraud Type | What breaks in the DAG |
//! |-------|-----------|----------------------|
//! | Basic | Orphan item | Item in inventory with no loot/spawn vertex |
//! | Basic | Duplicate cert | Two certs with identical type but different IDs |
//! | Basic | Speed hack | >15 actions/second |
//! | Basic | Impossible kill | Shot beyond weapon range |
//! | Basic | Unattributed loot | Container looted without open vertex |
//! | Basic | Aimbot | >90% headshot ratio over 10+ shots |
//! | Consumable | Phantom round | Fire event references round never in inventory |
//! | Consumable | Overconsumption | Same single-use item consumed more than once |
//! | Advanced | Identity spoof | Fire attributed to player with no vertex at that zone/tick |
//! | Advanced | Ghost action | Action in a zone player never entered |
//! | Advanced | Through-wall shot | Shooter and target in non-adjacent zones |
//! | Advanced | Teleport | Move between non-adjacent zones (skipped intermediate) |

use std::collections::HashMap;

use crate::raid::{LootSource, PlayerId, RaidAction, RaidSession, ZoneId};

// ============================================================================
// Fraud Report
// ============================================================================

#[derive(Debug)]
pub struct FraudReport {
    pub orphan_items: Vec<OrphanItem>,
    pub duplicate_certs: Vec<DuplicateCert>,
    pub speed_violations: Vec<SpeedViolation>,
    pub impossible_kills: Vec<ImpossibleKill>,
    pub unattributed_loots: Vec<UnattributedLoot>,
    pub headshot_anomalies: Vec<HeadshotAnomaly>,
    pub phantom_rounds: Vec<PhantomRound>,
    pub overconsumptions: Vec<OverConsumption>,
    pub identity_spoofs: Vec<IdentitySpoof>,
    pub ghost_actions: Vec<GhostAction>,
    pub through_wall_shots: Vec<ThroughWallShot>,
    pub teleports: Vec<Teleport>,
}

impl FraudReport {
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.orphan_items.is_empty()
            && self.duplicate_certs.is_empty()
            && self.speed_violations.is_empty()
            && self.impossible_kills.is_empty()
            && self.unattributed_loots.is_empty()
            && self.headshot_anomalies.is_empty()
            && self.phantom_rounds.is_empty()
            && self.overconsumptions.is_empty()
            && self.identity_spoofs.is_empty()
            && self.ghost_actions.is_empty()
            && self.through_wall_shots.is_empty()
            && self.teleports.is_empty()
    }

    #[must_use]
    pub const fn total_violations(&self) -> usize {
        self.orphan_items.len()
            + self.duplicate_certs.len()
            + self.speed_violations.len()
            + self.impossible_kills.len()
            + self.unattributed_loots.len()
            + self.headshot_anomalies.len()
            + self.phantom_rounds.len()
            + self.overconsumptions.len()
            + self.identity_spoofs.len()
            + self.ghost_actions.len()
            + self.through_wall_shots.len()
            + self.teleports.len()
    }
}

// ============================================================================
// Violation types — basic
// ============================================================================

#[derive(Debug)]
pub struct OrphanItem {
    pub player: PlayerId,
    pub item_id: String,
}

#[derive(Debug)]
pub struct DuplicateCert {
    pub item_id_a: String,
    pub item_id_b: String,
}

#[derive(Debug)]
pub struct SpeedViolation {
    pub player: PlayerId,
    pub actions_in_window: usize,
    pub window_ms: u64,
}

#[derive(Debug)]
pub struct ImpossibleKill {
    pub killer: PlayerId,
    pub victim: PlayerId,
    pub reason: &'static str,
}

#[derive(Debug)]
pub struct UnattributedLoot {
    pub player: PlayerId,
    pub item_id: String,
}

#[derive(Debug)]
pub struct HeadshotAnomaly {
    pub player: PlayerId,
    pub headshot_ratio: f64,
    pub total_shots: usize,
}

// ============================================================================
// Violation types — consumable
// ============================================================================

#[derive(Debug)]
pub struct PhantomRound {
    pub shooter: PlayerId,
    pub claimed_round_id: String,
}

#[derive(Debug)]
pub struct OverConsumption {
    pub player: PlayerId,
    pub item_id: String,
    pub consume_count: usize,
}

// ============================================================================
// Violation types — advanced
// ============================================================================

/// Fire event claims shooter is player X, but player X's last known zone
/// doesn't match the zone where the shot originated.
#[derive(Debug)]
pub struct IdentitySpoof {
    pub claimed_shooter: PlayerId,
    pub actual_zone: ZoneId,
    pub expected_zone: ZoneId,
}

/// Action performed in a zone the player never entered (no Spawn or Move
/// vertex places them there).
#[derive(Debug)]
pub struct GhostAction {
    pub player: PlayerId,
    pub zone: ZoneId,
    pub action_desc: String,
}

/// Shooter and target in zones that have no line-of-sight according to
/// the map topology.
#[derive(Debug)]
pub struct ThroughWallShot {
    pub shooter: PlayerId,
    pub shooter_zone: ZoneId,
    pub target: PlayerId,
    pub target_zone: ZoneId,
}

/// Move between non-adjacent zones with no intermediate move vertices.
#[derive(Debug)]
pub struct Teleport {
    pub player: PlayerId,
    pub from: ZoneId,
    pub to: ZoneId,
}

// ============================================================================
// Thresholds
// ============================================================================

const HEADSHOT_ANOMALY_THRESHOLD: f64 = 0.9;
const HEADSHOT_MIN_SHOTS: usize = 10;
const MAX_ACTIONS_PER_SECOND: usize = 15;
const MAX_WEAPON_RANGE_M: f64 = 500.0;

// ============================================================================
// Main entry point
// ============================================================================

#[must_use]
pub fn analyze_raid(session: &RaidSession) -> FraudReport {
    FraudReport {
        orphan_items: detect_orphan_items(session),
        duplicate_certs: detect_duplicate_certs(session),
        speed_violations: detect_speed_violations(session),
        impossible_kills: detect_impossible_kills(session),
        unattributed_loots: detect_unattributed_loots(session),
        headshot_anomalies: detect_headshot_anomalies(session),
        phantom_rounds: detect_phantom_rounds(session),
        overconsumptions: detect_overconsumption(session),
        identity_spoofs: detect_identity_spoofs(session),
        ghost_actions: detect_ghost_actions(session),
        through_wall_shots: detect_through_wall_shots(session),
        teleports: detect_teleports(session),
    }
}

// ============================================================================
// Basic detectors
// ============================================================================

fn detect_orphan_items(session: &RaidSession) -> Vec<OrphanItem> {
    let mut orphans = Vec::new();

    let looted_items: Vec<String> = session
        .action_log
        .iter()
        .filter_map(|a| match &a.action {
            RaidAction::LootPickup { item, .. } => Some(item.clone()),
            _ => None,
        })
        .collect();

    for (player, inv) in &session.inventories {
        let spawn_items = session.spawn_loadout_items.get(player);
        for item_id in inv {
            let in_spawn = spawn_items.is_some_and(|s| s.contains(item_id));
            let in_loot = looted_items.contains(item_id);
            if !in_spawn && !in_loot {
                orphans.push(OrphanItem {
                    player,
                    item_id: item_id.clone(),
                });
            }
        }
    }
    orphans
}

fn detect_duplicate_certs(session: &RaidSession) -> Vec<DuplicateCert> {
    let mut dupes = Vec::new();
    let certs: Vec<_> = session.certificates.values().collect();
    for (i, a) in certs.iter().enumerate() {
        for b in &certs[i + 1..] {
            let a_type = format!("{:?}", a.cert_type);
            let b_type = format!("{:?}", b.cert_type);
            if a_type == b_type && a.id != b.id {
                let a_id: String = session
                    .certificates
                    .iter()
                    .find(|(_, c)| c.id == a.id)
                    .map(|(k, _)| k.clone())
                    .unwrap_or_default();
                let b_id: String = session
                    .certificates
                    .iter()
                    .find(|(_, c)| c.id == b.id)
                    .map(|(k, _)| k.clone())
                    .unwrap_or_default();
                if a_id != b_id {
                    dupes.push(DuplicateCert {
                        item_id_a: a_id,
                        item_id_b: b_id,
                    });
                }
            }
        }
    }
    dupes
}

fn detect_speed_violations(session: &RaidSession) -> Vec<SpeedViolation> {
    let mut violations = Vec::new();
    let mut player_actions: HashMap<PlayerId, Vec<u64>> = HashMap::new();

    for entry in &session.action_log {
        let player = match &entry.action {
            RaidAction::Fire(b) => Some(b.shooter),
            RaidAction::Move { entity, .. }
            | RaidAction::LootPickup { entity, .. }
            | RaidAction::Kill { killer: entity, .. } => Some(*entity),
            _ => None,
        };
        if let Some(p) = player {
            player_actions.entry(p).or_default().push(entry.tick_ms);
        }
    }

    for (player, timestamps) in &player_actions {
        for window in timestamps.windows(MAX_ACTIONS_PER_SECOND) {
            let span = window.last().copied().unwrap_or(0) - window.first().copied().unwrap_or(0);
            if span < 1000 && window.len() >= MAX_ACTIONS_PER_SECOND {
                violations.push(SpeedViolation {
                    player,
                    actions_in_window: window.len(),
                    window_ms: span,
                });
                break;
            }
        }
    }
    violations
}

fn detect_impossible_kills(session: &RaidSession) -> Vec<ImpossibleKill> {
    let mut impossible = Vec::new();
    for entry in &session.action_log {
        if let RaidAction::Fire(bullet) = &entry.action {
            if bullet.distance_m > MAX_WEAPON_RANGE_M {
                impossible.push(ImpossibleKill {
                    killer: bullet.shooter,
                    victim: bullet.target,
                    reason: "shot beyond max weapon range",
                });
            }
        }
    }
    impossible
}

fn detect_unattributed_loots(session: &RaidSession) -> Vec<UnattributedLoot> {
    let mut unattributed = Vec::new();
    let container_opens: Vec<(&str, PlayerId)> = session
        .action_log
        .iter()
        .filter_map(|a| match &a.action {
            RaidAction::OpenContainer { entity, container } => Some((*container, *entity)),
            _ => None,
        })
        .collect();

    for entry in &session.action_log {
        if let RaidAction::LootPickup {
            entity,
            item,
            source: LootSource::Container(container),
        } = &entry.action
        {
            let opened = container_opens
                .iter()
                .any(|(c, p)| *c == *container && *p == *entity);
            if !opened {
                unattributed.push(UnattributedLoot {
                    player: entity,
                    item_id: item.clone(),
                });
            }
        }
    }
    unattributed
}

#[expect(
    clippy::cast_precision_loss,
    reason = "shot counts fit in f64 mantissa"
)]
fn detect_headshot_anomalies(session: &RaidSession) -> Vec<HeadshotAnomaly> {
    let mut anomalies = Vec::new();
    let mut player_shots: HashMap<PlayerId, (usize, usize)> = HashMap::new();

    for entry in &session.action_log {
        if let RaidAction::Fire(bullet) = &entry.action {
            let (total, headshots) = player_shots.entry(bullet.shooter).or_insert((0, 0));
            *total += 1;
            if bullet.headshot {
                *headshots += 1;
            }
        }
    }

    for (player, (total, headshots)) in &player_shots {
        if *total >= HEADSHOT_MIN_SHOTS {
            let ratio = *headshots as f64 / *total as f64;
            if ratio > HEADSHOT_ANOMALY_THRESHOLD {
                anomalies.push(HeadshotAnomaly {
                    player,
                    headshot_ratio: ratio,
                    total_shots: *total,
                });
            }
        }
    }
    anomalies
}

// ============================================================================
// Consumable detectors
// ============================================================================

fn detect_phantom_rounds(session: &RaidSession) -> Vec<PhantomRound> {
    let mut phantoms = Vec::new();
    for entry in &session.action_log {
        if let RaidAction::Fire(bullet) = &entry.action {
            if let Some(round_id) = &bullet.round_id {
                let has_cert = session.certificates.contains_key(round_id);
                let in_spawn = session
                    .spawn_loadout_items
                    .get(bullet.shooter)
                    .is_some_and(|items| items.iter().any(|id| id == round_id));
                let in_loot = session.action_log.iter().any(|a| {
                    matches!(
                        &a.action,
                        RaidAction::LootPickup { item, .. } if item == round_id
                    )
                });
                if !has_cert || (!in_spawn && !in_loot) {
                    phantoms.push(PhantomRound {
                        shooter: bullet.shooter,
                        claimed_round_id: round_id.clone(),
                    });
                }
            }
        }
    }
    phantoms
}

fn detect_overconsumption(session: &RaidSession) -> Vec<OverConsumption> {
    let mut consume_counts: HashMap<(PlayerId, &str), usize> = HashMap::new();

    for entry in &session.action_log {
        if let RaidAction::Consume { entity, item } = &entry.action {
            *consume_counts.entry((*entity, item)).or_insert(0) += 1;
        }
    }

    consume_counts
        .into_iter()
        .filter(|&(_, count)| count > 1)
        .map(|((player, item_id), count)| OverConsumption {
            player,
            item_id: item_id.into(),
            consume_count: count,
        })
        .collect()
}

// ============================================================================
// Advanced detectors
// ============================================================================

/// Build a timeline of each player's zone presence from the action log.
fn build_zone_timeline(session: &RaidSession) -> HashMap<PlayerId, Vec<(u64, ZoneId)>> {
    let mut timelines: HashMap<PlayerId, Vec<(u64, ZoneId)>> = HashMap::new();

    for entry in &session.action_log {
        match &entry.action {
            RaidAction::Spawn { entity, zone } => {
                timelines
                    .entry(entity)
                    .or_default()
                    .push((entry.tick_ms, *zone));
            }
            RaidAction::Move { entity, to, .. } => {
                timelines
                    .entry(entity)
                    .or_default()
                    .push((entry.tick_ms, *to));
            }
            _ => {}
        }
    }
    timelines
}

/// Get a player's zone at a given tick from their timeline.
fn zone_at_tick(timeline: &[(u64, ZoneId)], tick: u64) -> Option<ZoneId> {
    timeline
        .iter()
        .rev()
        .find(|(t, _)| *t <= tick)
        .map(|(_, z)| *z)
}

/// Detect actions attributed to a player who isn't in the zone where the
/// action claims to originate.
fn detect_identity_spoofs(session: &RaidSession) -> Vec<IdentitySpoof> {
    let mut spoofs = Vec::new();
    let timelines = build_zone_timeline(session);

    for entry in &session.action_log {
        if let RaidAction::Fire(bullet) = &entry.action {
            let shooter_timeline = timelines.get(bullet.shooter);
            let target_zone = session.entities.get(bullet.target).map(|e| e.zone);

            if let (Some(timeline), Some(_target_z)) = (shooter_timeline, target_zone) {
                let shooter_zone = zone_at_tick(timeline, entry.tick_ms);
                if let Some(sz) = shooter_zone {
                    if session
                        .spoof_claims
                        .contains(&(bullet.shooter, entry.tick_ms))
                    {
                        let actual = session
                            .spoof_actual_zones
                            .get(&(bullet.shooter, entry.tick_ms));
                        if let Some(&actual_zone) = actual {
                            if actual_zone != sz {
                                spoofs.push(IdentitySpoof {
                                    claimed_shooter: bullet.shooter,
                                    actual_zone,
                                    expected_zone: sz,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    spoofs
}

/// Detect actions in zones the player never entered according to the DAG.
/// Uses `actor_zone` from the timestamped action (the entity's zone at action
/// time) and checks against the zone timeline (Spawn/Move vertices only).
fn detect_ghost_actions(session: &RaidSession) -> Vec<GhostAction> {
    let mut ghosts = Vec::new();
    let timelines = build_zone_timeline(session);

    for entry in &session.action_log {
        let (player, desc) = match &entry.action {
            RaidAction::LootPickup { entity, item, .. } => (Some(*entity), format!("loot:{item}")),
            RaidAction::Kill { killer, victim } => (Some(*killer), format!("kill:{victim}")),
            _ => (None, String::new()),
        };

        if let (Some(player), Some(zone)) = (player, entry.actor_zone) {
            let timeline = timelines.get(player);
            let entered_before = timeline
                .is_some_and(|tl| tl.iter().any(|(t, z)| *z == zone && *t <= entry.tick_ms));
            if !entered_before {
                ghosts.push(GhostAction {
                    player,
                    zone,
                    action_desc: desc,
                });
            }
        }
    }
    ghosts
}

/// Detect shots where shooter and target are in non-adjacent zones with no `LoS`.
fn detect_through_wall_shots(session: &RaidSession) -> Vec<ThroughWallShot> {
    let mut violations = Vec::new();
    let Some(topology) = &session.zone_topology else {
        return violations;
    };
    let timelines = build_zone_timeline(session);

    for entry in &session.action_log {
        if let RaidAction::Fire(bullet) = &entry.action {
            let shooter_zone = entry.actor_zone.or_else(|| {
                timelines
                    .get(bullet.shooter)
                    .and_then(|tl| zone_at_tick(tl, entry.tick_ms))
            });
            let target_zone = timelines
                .get(bullet.target)
                .and_then(|tl| zone_at_tick(tl, entry.tick_ms));

            if let (Some(sz), Some(tz)) = (shooter_zone, target_zone) {
                if sz != tz && !topology.has_line_of_sight(sz, tz) {
                    violations.push(ThroughWallShot {
                        shooter: bullet.shooter,
                        shooter_zone: sz,
                        target: bullet.target,
                        target_zone: tz,
                    });
                }
            }
        }
    }
    violations
}

/// Detect moves between non-adjacent zones (teleportation).
fn detect_teleports(session: &RaidSession) -> Vec<Teleport> {
    let mut teleports = Vec::new();
    let Some(topology) = &session.zone_topology else {
        return teleports;
    };

    for entry in &session.action_log {
        if let RaidAction::Move { entity, from, to } = &entry.action {
            if !topology.is_adjacent(from, to) {
                teleports.push(Teleport {
                    player: entity,
                    from,
                    to,
                });
            }
        }
    }
    teleports
}
