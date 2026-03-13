// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
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

use ludospring_barracuda::validation::ValidationResult;
use raid::ConsumableState;

const EXP: &str = "exp053_extraction_shooter_provenance";

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

// ===========================================================================
// 1. Honest Raid — clean play, no fraud detected
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_honest_raid() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let raid = scenarios::build_honest_raid();
    let report = detection::analyze_raid(&raid);

    results.push(ValidationResult::check(
        EXP,
        "honest_raid_is_clean",
        bool_f64(report.is_clean()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "honest_zero_violations",
        report.total_violations() as f64,
        0.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "honest_dag_vertex_count",
        raid.vertex_ids.len() as f64,
        13.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "honest_rhizo_session_active",
        bool_f64(raid.rhizo_session_active()),
        1.0,
        0.0,
    ));

    let pmc1_inv = raid.inventories.get("pmc1").map_or(0, Vec::len);
    results.push(ValidationResult::check(
        EXP,
        "honest_pmc1_has_5_items",
        pmc1_inv as f64,
        5.0,
        0.0,
    ));

    let cert_count = raid.certificates.len();
    results.push(ValidationResult::check(
        EXP,
        "honest_all_items_have_certs",
        cert_count as f64,
        raid.items.len() as f64,
        0.0,
    ));

    let scav_dead = raid.entities.get("scav1").is_some_and(|e| !e.alive);
    results.push(ValidationResult::check(
        EXP,
        "honest_scav1_dead",
        bool_f64(scav_dead),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 2. Fraudulent Raid — every fraud type detected
// ===========================================================================

fn validate_fraudulent_raid() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let raid = scenarios::build_fraudulent_raid();
    let report = detection::analyze_raid(&raid);

    results.push(ValidationResult::check(
        EXP,
        "fraud_raid_not_clean",
        bool_f64(!report.is_clean()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "fraud_orphan_items_detected",
        bool_f64(!report.orphan_items.is_empty()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "fraud_duplicate_certs_detected",
        bool_f64(!report.duplicate_certs.is_empty()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "fraud_speed_violation_detected",
        bool_f64(!report.speed_violations.is_empty()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "fraud_impossible_kill_detected",
        bool_f64(!report.impossible_kills.is_empty()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "fraud_unattributed_loot_detected",
        bool_f64(!report.unattributed_loots.is_empty()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "fraud_headshot_anomaly_detected",
        bool_f64(!report.headshot_anomalies.is_empty()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "fraud_total_violations_ge_6",
        bool_f64(report.total_violations() >= 6),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 3. Provenance Properties — structural invariants
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_provenance_properties() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let raid = scenarios::build_honest_raid();

    let all_vertex_ids_unique = {
        let unique: std::collections::HashSet<_> = raid.vertex_ids.iter().collect();
        unique.len() == raid.vertex_ids.len()
    };
    results.push(ValidationResult::check(
        EXP,
        "provenance_all_vertex_ids_unique",
        bool_f64(all_vertex_ids_unique),
        1.0,
        0.0,
    ));

    let all_certs_unique = {
        let ids: std::collections::HashSet<_> = raid.certificates.values().map(|c| c.id).collect();
        ids.len() == raid.certificates.len()
    };
    results.push(ValidationResult::check(
        EXP,
        "provenance_all_cert_ids_unique",
        bool_f64(all_certs_unique),
        1.0,
        0.0,
    ));

    let all_certs_active = raid
        .certificates
        .values()
        .all(loam_spine_core::Certificate::is_active);
    results.push(ValidationResult::check(
        EXP,
        "provenance_all_certs_active",
        bool_f64(all_certs_active),
        1.0,
        0.0,
    ));

    let timestamps_monotonic = raid
        .action_log
        .windows(2)
        .all(|w| w[1].tick_ms >= w[0].tick_ms);
    results.push(ValidationResult::check(
        EXP,
        "provenance_timestamps_monotonic",
        bool_f64(timestamps_monotonic),
        1.0,
        0.0,
    ));

    let bullet_count = raid
        .action_log
        .iter()
        .filter(|a| matches!(a.action, raid::RaidAction::Fire(_)))
        .count();
    results.push(ValidationResult::check(
        EXP,
        "provenance_three_bullets_tracked",
        bullet_count as f64,
        3.0,
        0.0,
    ));

    let kill_count = raid
        .action_log
        .iter()
        .filter(|a| matches!(a.action, raid::RaidAction::Kill { .. }))
        .count();
    results.push(ValidationResult::check(
        EXP,
        "provenance_one_kill_tracked",
        kill_count as f64,
        1.0,
        0.0,
    ));

    let loot_count = raid
        .action_log
        .iter()
        .filter(|a| matches!(a.action, raid::RaidAction::LootPickup { .. }))
        .count();
    results.push(ValidationResult::check(
        EXP,
        "provenance_two_loots_tracked",
        loot_count as f64,
        2.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 4. Chain of Custody Isomorphism
// ===========================================================================

fn validate_isomorphism() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let raid = scenarios::build_honest_raid();

    let spawn_exists = raid
        .action_log
        .iter()
        .any(|a| matches!(a.action, raid::RaidAction::Spawn { entity: "pmc1", .. }));
    results.push(ValidationResult::check(
        EXP,
        "isomorphism_spawn_vertex_exists",
        bool_f64(spawn_exists),
        1.0,
        0.0,
    ));

    let extract_exists = raid
        .action_log
        .iter()
        .any(|a| matches!(a.action, raid::RaidAction::Extract { entity: "pmc1", .. }));
    results.push(ValidationResult::check(
        EXP,
        "isomorphism_extract_vertex_exists",
        bool_f64(extract_exists),
        1.0,
        0.0,
    ));

    let item_has_cert = raid.certificates.contains_key("scav1_shotgun");
    results.push(ValidationResult::check(
        EXP,
        "isomorphism_looted_item_has_cert",
        bool_f64(item_has_cert),
        1.0,
        0.0,
    ));

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
    results.push(ValidationResult::check(
        EXP,
        "isomorphism_kill_before_loot",
        bool_f64(kill_before_loot),
        1.0,
        0.0,
    ));

    let zone_transition_count = raid
        .action_log
        .iter()
        .filter(|a| matches!(a.action, raid::RaidAction::Move { .. }))
        .count();
    results.push(ValidationResult::check(
        EXP,
        "isomorphism_zone_transitions_tracked",
        bool_f64(zone_transition_count > 0),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 5. Fraud Report Detail Validation
// ===========================================================================

fn validate_fraud_report_detail() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let raid = scenarios::build_fraudulent_raid();
    let report = detection::analyze_raid(&raid);

    if let Some(speed) = report.speed_violations.first() {
        results.push(ValidationResult::check(
            EXP,
            "detail_speed_hack_is_cheater",
            bool_f64(speed.player == "cheater"),
            1.0,
            0.0,
        ));
        results.push(ValidationResult::check(
            EXP,
            "detail_speed_hack_window_under_1s",
            bool_f64(speed.window_ms < 1000),
            1.0,
            0.0,
        ));
    }

    if let Some(hs) = report.headshot_anomalies.first() {
        results.push(ValidationResult::check(
            EXP,
            "detail_aimbot_is_cheater",
            bool_f64(hs.player == "cheater"),
            1.0,
            0.0,
        ));
        results.push(ValidationResult::check(
            EXP,
            "detail_aimbot_ratio_above_90pct",
            bool_f64(hs.headshot_ratio > 0.9),
            1.0,
            0.0,
        ));
    }

    if let Some(impossible) = report.impossible_kills.first() {
        results.push(ValidationResult::check(
            EXP,
            "detail_impossible_kill_is_cheater",
            bool_f64(impossible.killer == "cheater"),
            1.0,
            0.0,
        ));
    }

    if let Some(orphan) = report.orphan_items.first() {
        results.push(ValidationResult::check(
            EXP,
            "detail_orphan_item_is_cheater",
            bool_f64(orphan.player == "cheater"),
            1.0,
            0.0,
        ));
    }

    results
}

// ===========================================================================
// 6. Consumable Lifecycle — rounds, meds, food
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_consumable_round_lifecycle() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let raid = scenarios::build_consumable_raid();
    let report = detection::analyze_raid(&raid);

    results.push(ValidationResult::check(
        EXP,
        "consumable_raid_is_clean",
        bool_f64(report.is_clean()),
        1.0,
        0.0,
    ));

    let consumed_rounds = raid
        .consumable_states
        .iter()
        .filter(|(id, state)| {
            matches!(state, ConsumableState::Consumed { .. }) && id.starts_with("pmc_rnd_")
        })
        .count();
    results.push(ValidationResult::check(
        EXP,
        "consumable_3_rounds_consumed",
        consumed_rounds as f64,
        3.0,
        0.0,
    ));

    let intact_rounds = raid
        .consumable_states
        .iter()
        .filter(|(id, state)| {
            matches!(state, ConsumableState::Intact) && id.starts_with("pmc_rnd_")
        })
        .count();
    results.push(ValidationResult::check(
        EXP,
        "consumable_3_rounds_intact",
        intact_rounds as f64,
        3.0,
        0.0,
    ));

    let found_intact = raid
        .consumable_states
        .iter()
        .filter(|(id, state)| {
            matches!(state, ConsumableState::Intact) && id.starts_with("found_rnd_")
        })
        .count();
    results.push(ValidationResult::check(
        EXP,
        "consumable_4_found_rounds_intact",
        found_intact as f64,
        4.0,
        0.0,
    ));

    // Started with 6, fired 3 = 3 remaining, added 4 = 7 in magazine
    let mag_round_count = raid
        .magazines
        .get("pmc_mag_0")
        .map_or(0, |m| m.round_ids.len());
    results.push(ValidationResult::check(
        EXP,
        "consumable_magazine_7_rounds_after_cycle",
        mag_round_count as f64,
        7.0,
        0.0,
    ));

    let all_rounds_have_certs = (0..6)
        .map(|i| format!("pmc_rnd_{i}"))
        .all(|id| raid.certificates.contains_key(&id));
    results.push(ValidationResult::check(
        EXP,
        "consumable_all_rounds_retain_certs",
        bool_f64(all_rounds_have_certs),
        1.0,
        0.0,
    ));

    let found_have_certs = (0..4)
        .map(|i| format!("found_rnd_{i}"))
        .all(|id| raid.certificates.contains_key(&id));
    results.push(ValidationResult::check(
        EXP,
        "consumable_found_rounds_have_certs",
        bool_f64(found_have_certs),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "consumable_zero_phantom_rounds",
        bool_f64(report.phantom_rounds.is_empty()),
        1.0,
        0.0,
    ));

    results
}

fn validate_consumable_item_lifecycle() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let raid = scenarios::build_consumable_raid();
    let report = detection::analyze_raid(&raid);

    let ifak_consumed = raid
        .consumable_states
        .get("pmc_ifak")
        .is_some_and(|s| matches!(s, ConsumableState::Consumed { .. }));
    results.push(ValidationResult::check(
        EXP,
        "consumable_ifak_consumed",
        bool_f64(ifak_consumed),
        1.0,
        0.0,
    ));

    let food_consumed = raid
        .consumable_states
        .get("pmc_crackers")
        .is_some_and(|s| matches!(s, ConsumableState::Consumed { .. }));
    results.push(ValidationResult::check(
        EXP,
        "consumable_food_consumed",
        bool_f64(food_consumed),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "consumable_zero_overconsumption",
        bool_f64(report.overconsumptions.is_empty()),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 7. Phantom Rounds + Overconsumption Fraud
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_phantom_round_fraud() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let raid = scenarios::build_phantom_round_raid();
    let report = detection::analyze_raid(&raid);

    results.push(ValidationResult::check(
        EXP,
        "phantom_raid_not_clean",
        bool_f64(!report.is_clean()),
        1.0,
        0.0,
    ));

    // 3 phantom rounds detected
    results.push(ValidationResult::check(
        EXP,
        "phantom_3_rounds_detected",
        report.phantom_rounds.len() as f64,
        3.0,
        0.0,
    ));

    // All phantoms attributed to cheater
    let all_cheater = report.phantom_rounds.iter().all(|p| p.shooter == "cheater");
    results.push(ValidationResult::check(
        EXP,
        "phantom_all_attributed_to_cheater",
        bool_f64(all_cheater),
        1.0,
        0.0,
    ));

    // Overconsumption: medkit consumed twice
    results.push(ValidationResult::check(
        EXP,
        "phantom_overconsumption_detected",
        bool_f64(!report.overconsumptions.is_empty()),
        1.0,
        0.0,
    ));

    // The overconsumption is on cheat_ifak
    let ifak_overconsumed = report
        .overconsumptions
        .iter()
        .any(|o| o.item_id == "cheat_ifak" && o.consume_count == 2);
    results.push(ValidationResult::check(
        EXP,
        "phantom_ifak_consumed_twice",
        bool_f64(ifak_overconsumed),
        1.0,
        0.0,
    ));

    // Real rounds were fine — no phantom flag for real_rnd_0, real_rnd_1
    let real_rounds_clean = !report
        .phantom_rounds
        .iter()
        .any(|p| p.claimed_round_id.starts_with("real_rnd_"));
    results.push(ValidationResult::check(
        EXP,
        "phantom_real_rounds_not_flagged",
        bool_f64(real_rounds_clean),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 8. Advanced Cheater — spatial fraud detection
// ===========================================================================

fn validate_advanced_cheater() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let raid = scenarios::build_advanced_cheater_raid();
    let report = detection::analyze_raid(&raid);

    results.push(ValidationResult::check(
        EXP,
        "advanced_raid_not_clean",
        bool_f64(!report.is_clean()),
        1.0,
        0.0,
    ));

    // Identity spoof detected
    results.push(ValidationResult::check(
        EXP,
        "advanced_identity_spoof_detected",
        bool_f64(!report.identity_spoofs.is_empty()),
        1.0,
        0.0,
    ));
    if let Some(spoof) = report.identity_spoofs.first() {
        results.push(ValidationResult::check(
            EXP,
            "advanced_spoof_claims_honest",
            bool_f64(spoof.claimed_shooter == "honest"),
            1.0,
            0.0,
        ));
    }

    // Ghost action detected
    results.push(ValidationResult::check(
        EXP,
        "advanced_ghost_action_detected",
        bool_f64(!report.ghost_actions.is_empty()),
        1.0,
        0.0,
    ));
    if let Some(ghost) = report.ghost_actions.first() {
        results.push(ValidationResult::check(
            EXP,
            "advanced_ghost_is_cheater",
            bool_f64(ghost.player == "cheater"),
            1.0,
            0.0,
        ));
    }

    // Through-wall shot detected
    results.push(ValidationResult::check(
        EXP,
        "advanced_through_wall_detected",
        bool_f64(!report.through_wall_shots.is_empty()),
        1.0,
        0.0,
    ));
    if let Some(wall) = report.through_wall_shots.first() {
        results.push(ValidationResult::check(
            EXP,
            "advanced_wall_shooter_is_cheater",
            bool_f64(wall.shooter == "cheater"),
            1.0,
            0.0,
        ));
    }

    // Teleport detected
    results.push(ValidationResult::check(
        EXP,
        "advanced_teleport_detected",
        bool_f64(!report.teleports.is_empty()),
        1.0,
        0.0,
    ));
    if let Some(tp) = report.teleports.first() {
        results.push(ValidationResult::check(
            EXP,
            "advanced_teleport_is_cheater",
            bool_f64(tp.player == "cheater"),
            1.0,
            0.0,
        ));
    }

    // Total advanced violations >= 4
    let advanced_count = report.identity_spoofs.len()
        + report.ghost_actions.len()
        + report.through_wall_shots.len()
        + report.teleports.len();
    results.push(ValidationResult::check(
        EXP,
        "advanced_total_spatial_violations_ge_4",
        bool_f64(advanced_count >= 4),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 9. Honest Topology — spatial checks pass with clean play
// ===========================================================================

fn validate_honest_topology() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let raid = scenarios::build_honest_topology_raid();
    let report = detection::analyze_raid(&raid);

    results.push(ValidationResult::check(
        EXP,
        "topology_honest_is_clean",
        bool_f64(report.is_clean()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "topology_zero_spoofs",
        bool_f64(report.identity_spoofs.is_empty()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "topology_zero_ghosts",
        bool_f64(report.ghost_actions.is_empty()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "topology_zero_wallhacks",
        bool_f64(report.through_wall_shots.is_empty()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "topology_zero_teleports",
        bool_f64(report.teleports.is_empty()),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    println!("=== exp053: Extraction Shooter Provenance + Fraud Detection ===\n");

    let mut all_results = Vec::new();

    let sections: Vec<(&str, Vec<ValidationResult>)> = vec![
        ("Honest Raid — Clean Play", validate_honest_raid()),
        ("Fraudulent Raid — Detection", validate_fraudulent_raid()),
        ("Provenance Properties", validate_provenance_properties()),
        ("Chain-of-Custody Isomorphism", validate_isomorphism()),
        ("Fraud Report Detail", validate_fraud_report_detail()),
        (
            "Consumable Lifecycle — Round Tracking",
            validate_consumable_round_lifecycle(),
        ),
        (
            "Consumable Lifecycle — Meds + Food",
            validate_consumable_item_lifecycle(),
        ),
        (
            "Phantom Rounds + Overconsumption Fraud",
            validate_phantom_round_fraud(),
        ),
        (
            "Advanced Cheater — Spatial Fraud",
            validate_advanced_cheater(),
        ),
        (
            "Honest Topology — Clean Spatial Play",
            validate_honest_topology(),
        ),
    ];

    for (name, results) in sections {
        println!("--- {name} ---");
        for v in &results {
            println!(
                "  [{}] {}",
                if v.passed { "PASS" } else { "FAIL" },
                v.description
            );
        }
        all_results.extend(results);
        println!();
    }

    let passed = all_results.iter().filter(|r| r.passed).count();
    let total = all_results.len();
    println!("=== SUMMARY: {passed}/{total} checks passed ===");

    if passed != total {
        println!("\nFAILED:");
        for r in all_results.iter().filter(|r| !r.passed) {
            println!(
                "  {} — measured={}, expected={}",
                r.description, r.measured, r.expected
            );
        }
        std::process::exit(1);
    }
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
