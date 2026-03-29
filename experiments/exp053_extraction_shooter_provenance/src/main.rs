// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp053 — Extraction Shooter Provenance + Fraud Detection
//!
//! Extraction shooters (Tarkov, DMZ, The Cycle) have a fundamental problem:
//! every item has real value, so cheating is economically motivated. Speed hacks,
//! item duplication, wallhacks, aimbots — all are provenance violations.
//!
//! The ecoPrimals provenance trio solves this:
//!   - **rhizoCrypt**: every raid is a session DAG. Spawn, move, shoot, loot,
//!     kill, extract — each action is a content-addressed vertex. No action
//!     can exist without a parent. No item can appear without a loot vertex.
//!   - **loamSpine**: every weapon, armor, key, barter item is a certificate.
//!     Certificates cannot be duplicated. Transfer requires both parties.
//!     Chain of custody is immutable.
//!   - **sweetGrass**: every player action is attributed. Who shot whom, who
//!     looted what, when. PROV-O semantics make the audit trail machine-readable.
//!
//! Fraud detection reduces to provenance chain analysis:
//!   1. **Orphan items**: item in inventory without a loot/spawn vertex = duplication
//!   2. **Duplicate certificates**: same item type with two cert IDs = duplication glitch
//!   3. **Speed violations**: >15 actions/second = speed hack or macro
//!   4. **Impossible kills**: shots beyond weapon range = teleport/aimbot
//!   5. **Unattributed loot**: looting a container without opening it = exploit
//!   6. **Headshot anomaly**: >90% headshot ratio over 10+ shots = aimbot
//!   7. **Phantom rounds**: bullet fired referencing a round never in inventory
//!   8. **Overconsumption**: same single-use item consumed more than once
//!
//! The same chain-of-custody model that catches cheaters in games catches
//! sample tampering in field genomics. Same code path, different vocabulary.

pub mod detection;
pub mod raid;
mod scenarios;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use raid::ConsumableState;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — extraction shooter fraud detection)",
    commit: "4b683e3e",
    date: "2026-03-29",
    command: "N/A (pure Rust implementation)",
};

// ===========================================================================
// 1. Honest Raid — clean play, no fraud detected
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_honest_raid(h: &mut ValidationHarness) {
    let raid = scenarios::build_honest_raid();
    let report = detection::analyze_raid(&raid);

    h.check_bool("honest_raid_is_clean", report.is_clean());
    h.check_abs(
        "honest_zero_violations",
        report.total_violations() as f64,
        0.0,
        0.0,
    );
    h.check_abs(
        "honest_dag_vertex_count",
        raid.vertex_ids.len() as f64,
        13.0,
        0.0,
    );
    h.check_bool("honest_rhizo_session_active", raid.rhizo_session_active());

    let pmc1_inv = raid.inventories.get("pmc1").map_or(0, Vec::len);
    h.check_abs("honest_pmc1_has_5_items", pmc1_inv as f64, 5.0, 0.0);

    let cert_count = raid.certificates.len();
    h.check_abs(
        "honest_all_items_have_certs",
        cert_count as f64,
        raid.items.len() as f64,
        0.0,
    );

    let scav_dead = raid.entities.get("scav1").is_some_and(|e| !e.alive);
    h.check_bool("honest_scav1_dead", scav_dead);
}

// ===========================================================================
// 2. Fraudulent Raid — every fraud type detected
// ===========================================================================

fn validate_fraudulent_raid(h: &mut ValidationHarness) {
    let raid = scenarios::build_fraudulent_raid();
    let report = detection::analyze_raid(&raid);

    h.check_bool("fraud_raid_not_clean", !report.is_clean());
    h.check_bool(
        "fraud_orphan_items_detected",
        !report.orphan_items.is_empty(),
    );
    h.check_bool(
        "fraud_duplicate_certs_detected",
        !report.duplicate_certs.is_empty(),
    );
    h.check_bool(
        "fraud_speed_violation_detected",
        !report.speed_violations.is_empty(),
    );
    h.check_bool(
        "fraud_impossible_kill_detected",
        !report.impossible_kills.is_empty(),
    );
    h.check_bool(
        "fraud_unattributed_loot_detected",
        !report.unattributed_loots.is_empty(),
    );
    h.check_bool(
        "fraud_headshot_anomaly_detected",
        !report.headshot_anomalies.is_empty(),
    );
    h.check_bool(
        "fraud_total_violations_ge_6",
        report.total_violations() >= 6,
    );
}

// ===========================================================================
// 3. Provenance Properties — structural invariants
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_provenance_properties(h: &mut ValidationHarness) {
    let raid = scenarios::build_honest_raid();

    let all_vertex_ids_unique = {
        let unique: std::collections::HashSet<_> = raid.vertex_ids.iter().collect();
        unique.len() == raid.vertex_ids.len()
    };
    h.check_bool("provenance_all_vertex_ids_unique", all_vertex_ids_unique);

    let all_certs_unique = {
        let ids: std::collections::HashSet<_> = raid.certificates.values().map(|c| c.id).collect();
        ids.len() == raid.certificates.len()
    };
    h.check_bool("provenance_all_cert_ids_unique", all_certs_unique);

    let all_certs_active = raid
        .certificates
        .values()
        .all(loam_spine_core::Certificate::is_active);
    h.check_bool("provenance_all_certs_active", all_certs_active);

    let timestamps_monotonic = raid
        .action_log
        .windows(2)
        .all(|w| w[1].tick_ms >= w[0].tick_ms);
    h.check_bool("provenance_timestamps_monotonic", timestamps_monotonic);

    let bullet_count = raid
        .action_log
        .iter()
        .filter(|a| matches!(a.action, raid::RaidAction::Fire(_)))
        .count();
    h.check_abs(
        "provenance_three_bullets_tracked",
        bullet_count as f64,
        3.0,
        0.0,
    );

    let kill_count = raid
        .action_log
        .iter()
        .filter(|a| matches!(a.action, raid::RaidAction::Kill { .. }))
        .count();
    h.check_abs("provenance_one_kill_tracked", kill_count as f64, 1.0, 0.0);

    let loot_count = raid
        .action_log
        .iter()
        .filter(|a| matches!(a.action, raid::RaidAction::LootPickup { .. }))
        .count();
    h.check_abs("provenance_two_loots_tracked", loot_count as f64, 2.0, 0.0);
}

// ===========================================================================
// 4. Chain of Custody Isomorphism
// ===========================================================================

fn validate_isomorphism(h: &mut ValidationHarness) {
    let raid = scenarios::build_honest_raid();

    let spawn_exists = raid
        .action_log
        .iter()
        .any(|a| matches!(a.action, raid::RaidAction::Spawn { entity: "pmc1", .. }));
    h.check_bool("isomorphism_spawn_vertex_exists", spawn_exists);

    let extract_exists = raid
        .action_log
        .iter()
        .any(|a| matches!(a.action, raid::RaidAction::Extract { entity: "pmc1", .. }));
    h.check_bool("isomorphism_extract_vertex_exists", extract_exists);

    let item_has_cert = raid.certificates.contains_key("scav1_shotgun");
    h.check_bool("isomorphism_looted_item_has_cert", item_has_cert);

    let kill_before_loot = {
        let kill_tick = raid
            .action_log
            .iter()
            .find(|a| {
                matches!(
                    a.action,
                    raid::RaidAction::Kill {
                        victim: "scav1",
                        ..
                    }
                )
            })
            .map(|a| a.tick_ms);
        let loot_tick = raid
            .action_log
            .iter()
            .find(|a| {
                matches!(
                    a.action,
                    raid::RaidAction::LootPickup {
                        entity: "pmc1",
                        source: raid::LootSource::Corpse("scav1"),
                        ..
                    }
                )
            })
            .map(|a| a.tick_ms);
        match (kill_tick, loot_tick) {
            (Some(k), Some(l)) => l > k,
            _ => false,
        }
    };
    h.check_bool("isomorphism_kill_before_loot", kill_before_loot);

    let zone_transition_count = raid
        .action_log
        .iter()
        .filter(|a| matches!(a.action, raid::RaidAction::Move { .. }))
        .count();
    h.check_bool(
        "isomorphism_zone_transitions_tracked",
        zone_transition_count > 0,
    );
}

// ===========================================================================
// 5. Fraud Report Detail Validation
// ===========================================================================

fn validate_fraud_report_detail(h: &mut ValidationHarness) {
    let raid = scenarios::build_fraudulent_raid();
    let report = detection::analyze_raid(&raid);

    if let Some(speed) = report.speed_violations.first() {
        h.check_bool("detail_speed_hack_is_cheater", speed.player == "cheater");
        h.check_bool("detail_speed_hack_window_under_1s", speed.window_ms < 1000);
    }

    if let Some(hs) = report.headshot_anomalies.first() {
        h.check_bool("detail_aimbot_is_cheater", hs.player == "cheater");
        h.check_bool("detail_aimbot_ratio_above_90pct", hs.headshot_ratio > 0.9);
    }

    if let Some(impossible) = report.impossible_kills.first() {
        h.check_bool(
            "detail_impossible_kill_is_cheater",
            impossible.killer == "cheater",
        );
    }

    if let Some(orphan) = report.orphan_items.first() {
        h.check_bool("detail_orphan_item_is_cheater", orphan.player == "cheater");
    }
}

// ===========================================================================
// 6. Consumable Lifecycle — rounds, meds, food
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_consumable_round_lifecycle(h: &mut ValidationHarness) {
    let raid = scenarios::build_consumable_raid();
    let report = detection::analyze_raid(&raid);

    h.check_bool("consumable_raid_is_clean", report.is_clean());

    let consumed_rounds = raid
        .consumable_states
        .iter()
        .filter(|(id, state)| {
            matches!(state, ConsumableState::Consumed { .. }) && id.starts_with("pmc_rnd_")
        })
        .count();
    h.check_abs(
        "consumable_3_rounds_consumed",
        consumed_rounds as f64,
        3.0,
        0.0,
    );

    let intact_rounds = raid
        .consumable_states
        .iter()
        .filter(|(id, state)| {
            matches!(state, ConsumableState::Intact) && id.starts_with("pmc_rnd_")
        })
        .count();
    h.check_abs("consumable_3_rounds_intact", intact_rounds as f64, 3.0, 0.0);

    let found_intact = raid
        .consumable_states
        .iter()
        .filter(|(id, state)| {
            matches!(state, ConsumableState::Intact) && id.starts_with("found_rnd_")
        })
        .count();
    h.check_abs(
        "consumable_4_found_rounds_intact",
        found_intact as f64,
        4.0,
        0.0,
    );

    let mag_round_count = raid
        .magazines
        .get("pmc_mag_0")
        .map_or(0, |m| m.round_ids.len());
    h.check_abs(
        "consumable_magazine_7_rounds_after_cycle",
        mag_round_count as f64,
        7.0,
        0.0,
    );

    let all_rounds_have_certs = (0..6)
        .map(|i| format!("pmc_rnd_{i}"))
        .all(|id| raid.certificates.contains_key(&id));
    h.check_bool("consumable_all_rounds_retain_certs", all_rounds_have_certs);

    let found_have_certs = (0..4)
        .map(|i| format!("found_rnd_{i}"))
        .all(|id| raid.certificates.contains_key(&id));
    h.check_bool("consumable_found_rounds_have_certs", found_have_certs);

    h.check_bool(
        "consumable_zero_phantom_rounds",
        report.phantom_rounds.is_empty(),
    );
}

fn validate_consumable_item_lifecycle(h: &mut ValidationHarness) {
    let raid = scenarios::build_consumable_raid();
    let report = detection::analyze_raid(&raid);

    let ifak_consumed = raid
        .consumable_states
        .get("pmc_ifak")
        .is_some_and(|s| matches!(s, ConsumableState::Consumed { .. }));
    h.check_bool("consumable_ifak_consumed", ifak_consumed);

    let food_consumed = raid
        .consumable_states
        .get("pmc_crackers")
        .is_some_and(|s| matches!(s, ConsumableState::Consumed { .. }));
    h.check_bool("consumable_food_consumed", food_consumed);

    h.check_bool(
        "consumable_zero_overconsumption",
        report.overconsumptions.is_empty(),
    );
}

// ===========================================================================
// 7. Phantom Rounds + Overconsumption Fraud
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_phantom_round_fraud(h: &mut ValidationHarness) {
    let raid = scenarios::build_phantom_round_raid();
    let report = detection::analyze_raid(&raid);

    h.check_bool("phantom_raid_not_clean", !report.is_clean());
    h.check_abs(
        "phantom_3_rounds_detected",
        report.phantom_rounds.len() as f64,
        3.0,
        0.0,
    );

    let all_cheater = report.phantom_rounds.iter().all(|p| p.shooter == "cheater");
    h.check_bool("phantom_all_attributed_to_cheater", all_cheater);

    h.check_bool(
        "phantom_overconsumption_detected",
        !report.overconsumptions.is_empty(),
    );

    let ifak_overconsumed = report
        .overconsumptions
        .iter()
        .any(|o| o.item_id == "cheat_ifak" && o.consume_count == 2);
    h.check_bool("phantom_ifak_consumed_twice", ifak_overconsumed);

    let real_rounds_clean = !report
        .phantom_rounds
        .iter()
        .any(|p| p.claimed_round_id.starts_with("real_rnd_"));
    h.check_bool("phantom_real_rounds_not_flagged", real_rounds_clean);
}

// ===========================================================================
// 8. Advanced Cheater — spatial fraud detection
// ===========================================================================

fn validate_advanced_cheater(h: &mut ValidationHarness) {
    let raid = scenarios::build_advanced_cheater_raid();
    let report = detection::analyze_raid(&raid);

    h.check_bool("advanced_raid_not_clean", !report.is_clean());
    h.check_bool(
        "advanced_identity_spoof_detected",
        !report.identity_spoofs.is_empty(),
    );
    if let Some(spoof) = report.identity_spoofs.first() {
        h.check_bool(
            "advanced_spoof_claims_honest",
            spoof.claimed_shooter == "honest",
        );
    }

    h.check_bool(
        "advanced_ghost_action_detected",
        !report.ghost_actions.is_empty(),
    );
    if let Some(ghost) = report.ghost_actions.first() {
        h.check_bool("advanced_ghost_is_cheater", ghost.player == "cheater");
    }

    h.check_bool(
        "advanced_through_wall_detected",
        !report.through_wall_shots.is_empty(),
    );
    if let Some(wall) = report.through_wall_shots.first() {
        h.check_bool(
            "advanced_wall_shooter_is_cheater",
            wall.shooter == "cheater",
        );
    }

    h.check_bool("advanced_teleport_detected", !report.teleports.is_empty());
    if let Some(tp) = report.teleports.first() {
        h.check_bool("advanced_teleport_is_cheater", tp.player == "cheater");
    }

    let advanced_count = report.identity_spoofs.len()
        + report.ghost_actions.len()
        + report.through_wall_shots.len()
        + report.teleports.len();
    h.check_bool(
        "advanced_total_spatial_violations_ge_4",
        advanced_count >= 4,
    );
}

// ===========================================================================
// 9. Honest Topology — spatial checks pass with clean play
// ===========================================================================

fn validate_honest_topology(h: &mut ValidationHarness) {
    let raid = scenarios::build_honest_topology_raid();
    let report = detection::analyze_raid(&raid);

    h.check_bool("topology_honest_is_clean", report.is_clean());
    h.check_bool("topology_zero_spoofs", report.identity_spoofs.is_empty());
    h.check_bool("topology_zero_ghosts", report.ghost_actions.is_empty());
    h.check_bool(
        "topology_zero_wallhacks",
        report.through_wall_shots.is_empty(),
    );
    h.check_bool("topology_zero_teleports", report.teleports.is_empty());
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp053_extraction_shooter_provenance");
    h.print_provenance(&[&PROVENANCE]);

    validate_honest_raid(&mut h);
    validate_fraudulent_raid(&mut h);
    validate_provenance_properties(&mut h);
    validate_isomorphism(&mut h);
    validate_fraud_report_detail(&mut h);
    validate_consumable_round_lifecycle(&mut h);
    validate_consumable_item_lifecycle(&mut h);
    validate_phantom_round_fraud(&mut h);
    validate_advanced_cheater(&mut h);
    validate_honest_topology(&mut h);

    h.finish();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") | None => cmd_validate(),
        Some(other) => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}
