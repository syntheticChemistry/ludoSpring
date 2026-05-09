// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pre-built raid scenarios for validation.
//!
//! Each function constructs a [`RaidSession`] with a specific narrative:
//! honest play, fraud injection, or consumable lifecycle.

use crate::raid::{self, BulletEvent, EntityKind, ItemCategory, LootItem, LootSource, RaidSession};

fn make_item(id: &str, name: &'static str, cat: ItemCategory, value: u32) -> LootItem {
    LootItem {
        id: id.into(),
        name,
        category: cat,
        value_roubles: value,
    }
}

fn make_round(id: &str, caliber: &'static str) -> LootItem {
    LootItem {
        id: id.into(),
        name: caliber,
        category: ItemCategory::Ammo,
        value_roubles: 50,
    }
}

// ============================================================================
// Honest Raid — clean play, no fraud detected
// ============================================================================

/// Two PMCs, one scav, clean firefight, proper looting.
pub fn build_honest_raid() -> RaidSession {
    let mut raid = RaidSession::new("Customs");

    let pmc1_gear = vec![
        make_item("pmc1_ak74", "AK-74N", ItemCategory::Weapon, 45_000),
        make_item("pmc1_armor", "6B3TM", ItemCategory::Armor, 38_000),
        make_item("pmc1_ifak", "IFAK", ItemCategory::Medical, 12_000),
    ];
    let pmc2_gear = vec![
        make_item("pmc2_m4", "M4A1", ItemCategory::Weapon, 62_000),
        make_item("pmc2_armor", "Korund-VM", ItemCategory::Armor, 95_000),
    ];
    let scav_gear = vec![make_item(
        "scav1_shotgun",
        "MP-153",
        ItemCategory::Weapon,
        18_000,
    )];

    raid.spawn_entity("pmc1", EntityKind::Pmc, "big_red", &pmc1_gear);
    raid.advance_time(100);
    raid.spawn_entity("pmc2", EntityKind::Pmc, "crossroads", &pmc2_gear);
    raid.advance_time(100);
    raid.spawn_entity("scav1", EntityKind::Scav, "gas_station", &scav_gear);
    raid.advance_time(500);

    raid.spawn_ground_loot(make_item(
        "ground_gpu",
        "Graphics Card",
        ItemCategory::Barter,
        320_000,
    ));
    raid.move_entity("pmc1", "gas_station");
    raid.advance_time(3000);

    for i in 0..3 {
        raid.advance_time(200);
        raid.fire_bullet(BulletEvent {
            shooter: "pmc1",
            target: "scav1",
            weapon: "AK-74N",
            distance_m: 45.0,
            headshot: i == 2,
            damage: 35,
            round_id: None,
        });
    }
    raid.kill("pmc1", "scav1");
    raid.advance_time(2000);

    raid.pickup_loot("pmc1", "scav1_shotgun", LootSource::Corpse("scav1"));
    raid.advance_time(1000);

    raid.open_container("pmc1", "gas_station_crate");
    raid.advance_time(500);
    raid.pickup_loot(
        "pmc1",
        "ground_gpu",
        LootSource::Container("gas_station_crate"),
    );
    raid.advance_time(5000);

    raid.move_entity("pmc1", "crossroads");
    raid.advance_time(2000);
    raid.extract("pmc1", "crossroads_extract");

    raid
}

// ============================================================================
// Fraudulent Raid — multiple fraud patterns injected
// ============================================================================

/// Cheater with orphan items, duplicate certs, speed hack, impossible kill,
/// unattributed loot, and aimbot headshot ratio.
pub fn build_fraudulent_raid() -> RaidSession {
    let mut raid = RaidSession::new("Customs (Compromised)");

    let cheater_gear = vec![make_item(
        "cheat_ak",
        "AK-74N",
        ItemCategory::Weapon,
        45_000,
    )];
    let victim_gear = vec![
        make_item("victim_m4", "M4A1", ItemCategory::Weapon, 62_000),
        make_item("victim_ledx", "LEDX", ItemCategory::Barter, 1_200_000),
    ];

    raid.spawn_entity("cheater", EntityKind::Pmc, "big_red", &cheater_gear);
    raid.advance_time(100);
    raid.spawn_entity("victim", EntityKind::Pmc, "crossroads", &victim_gear);
    raid.advance_time(100);

    // FRAUD 1: Orphan item — item appears without loot vertex
    raid.inject_orphan_item(
        "cheater",
        &make_item("duped_gpu", "Graphics Card", ItemCategory::Barter, 320_000),
    );

    // FRAUD 2: Duplicate certificate — same item type, two certs
    raid.duplicate_cert("cheat_ak", "duped_ak");
    raid.items.insert(
        "duped_ak".into(),
        make_item("duped_ak", "AK-74N", ItemCategory::Weapon, 45_000),
    );

    // FRAUD 3: Speed hack — 20 actions in 500ms
    for i in 0..20 {
        raid.advance_time(25);
        raid.fire_bullet(BulletEvent {
            shooter: "cheater",
            target: "victim",
            weapon: "AK-74N",
            distance_m: 30.0,
            headshot: true,
            damage: 5,
            round_id: None,
        });
        if i == 19 {
            raid.kill("cheater", "victim");
        }
    }

    // FRAUD 4: Impossible kill — shot at 800m with AK (max effective ~300m)
    raid.advance_time(1000);
    raid.spawn_entity(
        "scav_far",
        EntityKind::Scav,
        "sniper_rock",
        &[make_item(
            "scav_far_toz",
            "TOZ-106",
            ItemCategory::Weapon,
            8_000,
        )],
    );
    raid.fire_bullet(BulletEvent {
        shooter: "cheater",
        target: "scav_far",
        weapon: "AK-74N",
        distance_m: 800.0,
        headshot: true,
        damage: 100,
        round_id: None,
    });
    raid.kill("cheater", "scav_far");

    // FRAUD 5: Unattributed container loot — loots crate without opening it
    raid.spawn_ground_loot(make_item(
        "hidden_keycard",
        "Lab Keycard",
        ItemCategory::Key,
        180_000,
    ));
    raid.pickup_loot(
        "cheater",
        "hidden_keycard",
        LootSource::Container("locked_room_crate"),
    );

    raid.advance_time(1000);
    raid.extract("cheater", "big_red_extract");

    raid
}

// ============================================================================
// Consumable Lifecycle — rounds, meds, food with provenance
// ============================================================================

/// Honest raid where every round is tracked, meds consumed, food eaten.
/// All consumable provenance chains are clean.
pub fn build_consumable_raid() -> RaidSession {
    let mut raid = RaidSession::new("Customs (Consumables)");

    let pmc_gear = vec![
        make_item("pmc_ak", "AK-74N", ItemCategory::Weapon, 45_000),
        make_item("pmc_armor", "6B3TM", ItemCategory::Armor, 38_000),
        make_item("pmc_ifak", "IFAK", ItemCategory::Medical, 12_000),
        make_item("pmc_crackers", "Army Crackers", ItemCategory::Food, 3_000),
    ];

    // Spawn with 6 rounds in a magazine
    let spawn_rounds: Vec<LootItem> = (0..6)
        .map(|i| make_round(&format!("pmc_rnd_{i}"), "5.45x39"))
        .collect();

    raid.spawn_entity("pmc", EntityKind::Pmc, "big_red", &pmc_gear);
    for rnd in &spawn_rounds {
        raid.spawn_item_in_inventory("pmc", rnd);
    }
    raid.load_magazine("pmc", "pmc_mag_0", "5.45x39", "pmc_ak", &spawn_rounds);
    raid.advance_time(500);

    // Fire 3 rounds — each consumed with provenance
    for i in 0..3 {
        raid.advance_time(150);
        raid.fire_round(
            BulletEvent {
                shooter: "pmc",
                target: "scav_dummy",
                weapon: "AK-74N",
                distance_m: 30.0,
                headshot: i == 2,
                damage: 35,
                round_id: Some(format!("pmc_rnd_{i}")),
            },
            &format!("pmc_rnd_{i}"),
        );
    }
    raid.advance_time(2000);

    // Find loose ammo in raid and add to magazine
    let found_rounds: Vec<LootItem> = (0..4)
        .map(|i| make_round(&format!("found_rnd_{i}"), "5.45x39"))
        .collect();
    for rnd in &found_rounds {
        raid.spawn_ground_loot(rnd.clone());
        raid.pickup_loot("pmc", &rnd.id, LootSource::Ground);
        raid.advance_time(100);
    }
    raid.top_up_magazine("pmc", "pmc_mag_0", &found_rounds);

    // Use medical — consumed
    raid.advance_time(1000);
    raid.consume_item("pmc", "pmc_ifak", 1);

    // Eat food — consumed
    raid.advance_time(500);
    raid.consume_item("pmc", "pmc_crackers", 1);

    raid.advance_time(2000);
    raid.extract("pmc", "crossroads_extract");

    raid
}

/// Fraud: fires rounds that were never in inventory (phantom rounds),
/// and consumes a medkit twice (overconsumption).
pub fn build_phantom_round_raid() -> RaidSession {
    let mut raid = RaidSession::new("Customs (Phantom Rounds)");

    let cheater_gear = vec![make_item(
        "cheat_ak",
        "AK-74N",
        ItemCategory::Weapon,
        45_000,
    )];
    let victim_gear = vec![make_item("victim_m4", "M4A1", ItemCategory::Weapon, 62_000)];

    raid.spawn_entity("cheater", EntityKind::Pmc, "big_red", &cheater_gear);
    raid.advance_time(50);
    raid.spawn_entity("victim", EntityKind::Pmc, "gas_station", &victim_gear);
    raid.advance_time(50);

    // Give cheater 2 real rounds
    let real_rounds: Vec<LootItem> = (0..2)
        .map(|i| make_round(&format!("real_rnd_{i}"), "5.45x39"))
        .collect();
    for rnd in &real_rounds {
        raid.spawn_item_in_inventory("cheater", rnd);
    }
    raid.load_magazine(
        "cheater",
        "cheat_mag_0",
        "5.45x39",
        "cheat_ak",
        &real_rounds,
    );

    // Fire the 2 real rounds — legitimate
    for (i, rnd) in real_rounds.iter().enumerate() {
        raid.advance_time(200);
        raid.fire_round(
            BulletEvent {
                shooter: "cheater",
                target: "victim",
                weapon: "AK-74N",
                distance_m: 25.0,
                headshot: false,
                damage: 30,
                round_id: Some(rnd.id.clone()),
            },
            &rnd.id,
        );
        if i == 1 {
            // after 2nd shot
        }
    }

    // FRAUD: Fire 3 phantom rounds — never in inventory
    for i in 0..3 {
        raid.advance_time(200);
        raid.fire_round(
            BulletEvent {
                shooter: "cheater",
                target: "victim",
                weapon: "AK-74N",
                distance_m: 25.0,
                headshot: true,
                damage: 40,
                round_id: Some(format!("phantom_rnd_{i}")),
            },
            &format!("phantom_rnd_{i}"),
        );
    }
    raid.kill("cheater", "victim");

    // FRAUD: Consume same medkit twice (cheater never had one, but let's
    // give them one and consume it twice)
    let medkit = make_item("cheat_ifak", "IFAK", ItemCategory::Medical, 12_000);
    raid.spawn_item_in_inventory("cheater", &medkit);
    raid.advance_time(500);
    raid.consume_item("cheater", "cheat_ifak", 1);
    raid.advance_time(500);
    raid.consume_item("cheater", "cheat_ifak", 1);

    raid.advance_time(1000);
    raid.extract("cheater", "big_red_extract");

    raid
}

// ============================================================================
// Advanced Cheater — spatial fraud: spoof, ghost, wallhack, teleport
// ============================================================================

/// Advanced cheater who spoofs identity, acts from zones they never entered,
/// shoots through walls, and teleports between non-adjacent zones.
pub fn build_advanced_cheater_raid() -> RaidSession {
    let mut raid = RaidSession::new("Customs (Advanced Cheats)");
    raid.set_topology(raid::customs_topology());

    let cheater_gear = vec![make_item("adv_ak", "AK-74N", ItemCategory::Weapon, 45_000)];
    let honest_gear = vec![make_item("honest_m4", "M4A1", ItemCategory::Weapon, 62_000)];
    let bystander_gear = vec![make_item(
        "bystander_ump",
        "UMP-45",
        ItemCategory::Weapon,
        28_000,
    )];

    raid.spawn_entity("cheater", EntityKind::Pmc, "big_red", &cheater_gear);
    raid.advance_time(100);
    raid.spawn_entity("honest", EntityKind::Pmc, "gas_station", &honest_gear);
    raid.advance_time(100);
    raid.spawn_entity("bystander", EntityKind::Pmc, "crossroads", &bystander_gear);
    raid.advance_time(500);

    // FRAUD 1: Identity spoof — cheater fires but attributes the shot to
    // "honest". The real honest is at gas_station, but the fire vertex
    // claims honest shot from big_red.
    raid.mark_spoof("honest", "big_red");
    raid.fire_bullet(BulletEvent {
        shooter: "honest",
        target: "bystander",
        weapon: "M4A1",
        distance_m: 50.0,
        headshot: false,
        damage: 25,
        round_id: None,
    });
    raid.advance_time(200);

    // FRAUD 2: Ghost action — cheater kills in sniper_rock but never
    // moved there (no Spawn or Move vertex for sniper_rock).
    raid.set_entity_zone("cheater", "sniper_rock");
    raid.kill("cheater", "bystander");
    raid.set_entity_zone("cheater", "big_red");
    raid.advance_time(500);

    // FRAUD 3: Through-wall shot — cheater at big_red, target at
    // sniper_rock. No line-of-sight between these zones.
    raid.spawn_entity(
        "scav_hidden",
        EntityKind::Scav,
        "sniper_rock",
        &[make_item(
            "scav_hidden_toz",
            "TOZ-106",
            ItemCategory::Weapon,
            8_000,
        )],
    );
    raid.fire_bullet(BulletEvent {
        shooter: "cheater",
        target: "scav_hidden",
        weapon: "AK-74N",
        distance_m: 120.0,
        headshot: true,
        damage: 100,
        round_id: None,
    });
    raid.kill("cheater", "scav_hidden");
    raid.advance_time(500);

    // FRAUD 4: Teleport — big_red directly to sniper_rock, skipping
    // crossroads and gas_station.
    raid.move_entity("cheater", "sniper_rock");
    raid.advance_time(500);

    raid.extract("cheater", "big_red_extract");

    raid
}

/// Honest raid with topology — all moves adjacent, all shots within `LoS`.
pub fn build_honest_topology_raid() -> RaidSession {
    let mut raid = RaidSession::new("Customs (Honest + Topology)");
    raid.set_topology(raid::customs_topology());

    let pmc_gear = vec![make_item("topo_ak", "AK-74N", ItemCategory::Weapon, 45_000)];
    let scav_gear = vec![make_item(
        "topo_scav_shotgun",
        "MP-153",
        ItemCategory::Weapon,
        18_000,
    )];

    raid.spawn_entity("pmc_topo", EntityKind::Pmc, "big_red", &pmc_gear);
    raid.advance_time(100);
    raid.spawn_entity("scav_topo", EntityKind::Scav, "crossroads", &scav_gear);
    raid.advance_time(500);

    // Legal move: big_red → crossroads (adjacent)
    raid.move_entity("pmc_topo", "crossroads");
    raid.advance_time(200);

    // Legal shot: both in crossroads (same zone)
    raid.fire_bullet(BulletEvent {
        shooter: "pmc_topo",
        target: "scav_topo",
        weapon: "AK-74N",
        distance_m: 20.0,
        headshot: false,
        damage: 40,
        round_id: None,
    });
    raid.advance_time(150);
    raid.fire_bullet(BulletEvent {
        shooter: "pmc_topo",
        target: "scav_topo",
        weapon: "AK-74N",
        distance_m: 18.0,
        headshot: true,
        damage: 45,
        round_id: None,
    });
    raid.kill("pmc_topo", "scav_topo");
    raid.advance_time(1000);

    // Legal move chain: crossroads → gas_station → sniper_rock → back
    raid.move_entity("pmc_topo", "gas_station");
    raid.advance_time(3000);
    raid.move_entity("pmc_topo", "sniper_rock");
    raid.advance_time(2000);
    raid.move_entity("pmc_topo", "gas_station");
    raid.advance_time(2000);
    raid.move_entity("pmc_topo", "crossroads");
    raid.advance_time(2000);
    raid.move_entity("pmc_topo", "checkpoint");
    raid.advance_time(1000);
    raid.move_entity("pmc_topo", "extract_zone");
    raid.advance_time(500);
    raid.extract("pmc_topo", "extract_zone");

    raid
}
