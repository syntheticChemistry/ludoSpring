// SPDX-License-Identifier: AGPL-3.0-or-later
//! Object-level validation: cosmetics, certificate lifecycle, ownership, memory.

use loam_spine_core::certificate::LoanTerms;
use loam_spine_core::{Certificate, Did};
use ludospring_barracuda::validation::{OrExit, ValidationHarness};

use crate::ferment::{CosmeticSchema, FermentEventType, FermentingSystem, Rarity};

// ===========================================================================
// 1. Cosmetic Schema
// ===========================================================================

pub fn validate_cosmetic_schema(h: &mut ValidationHarness) {
    let cosmetics = CosmeticSchema {
        rarity: Rarity::Epic,
        skin: "dragon_scale".into(),
        color: "crimson".into(),
        material: "obsidian".into(),
        wear_level: 0.15,
    };

    let attrs = cosmetics.to_attributes();
    h.check_bool(
        "cosmetic_rarity_attribute",
        attrs.get("rarity") == Some(&"epic".to_string()),
    );
    h.check_bool(
        "cosmetic_skin_attribute",
        attrs.get("skin") == Some(&"dragon_scale".to_string()),
    );
    h.check_bool(
        "cosmetic_color_attribute",
        attrs.get("color") == Some(&"crimson".to_string()),
    );
    h.check_bool(
        "cosmetic_material_attribute",
        attrs.get("material") == Some(&"obsidian".to_string()),
    );
    h.check_bool(
        "cosmetic_wear_attribute",
        attrs.get("wear_level") == Some(&"0.15".to_string()),
    );

    let round_tripped = CosmeticSchema::from_attributes(&attrs);
    h.check_bool("cosmetic_roundtrip_succeeds", round_tripped.is_some());

    let rt = round_tripped.or_exit("cosmetic roundtrip");
    h.check_bool("cosmetic_roundtrip_rarity", rt.rarity == Rarity::Epic);
    h.check_bool("cosmetic_roundtrip_skin", rt.skin == "dragon_scale");
    h.check_bool(
        "cosmetic_roundtrip_wear",
        (rt.wear_level - 0.15).abs() < f64::EPSILON,
    );

    let all_rarities = [
        Rarity::Common,
        Rarity::Uncommon,
        Rarity::Rare,
        Rarity::Epic,
        Rarity::Legendary,
    ];
    let all_unique: std::collections::HashSet<&str> =
        all_rarities.iter().map(|r| r.as_str()).collect();
    h.check_bool("cosmetic_five_rarity_tiers", all_unique.len() == 5);
}

// ===========================================================================
// 2. Certificate Lifecycle: mint → inspect → trade → loan → return → consume
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
pub fn validate_certificate_lifecycle(h: &mut ValidationHarness) {
    let alice = Did::new("did:key:alice_ferment");
    let bob = Did::new("did:key:bob_ferment");

    let mut system = FermentingSystem::new(&alice);

    let sword_cosmetics = CosmeticSchema {
        rarity: Rarity::Rare,
        skin: "flame_blade".into(),
        color: "orange".into(),
        material: "mithril".into(),
        wear_level: 0.0,
    };

    let sword_id = system.mint(&alice, "Flame Sword", "weapon", sword_cosmetics);

    let cert = system.cert_manager.get_certificate(&sword_id);
    h.check_bool("lifecycle_mint_cert_exists", cert.is_some());
    h.check_bool(
        "lifecycle_mint_cert_active",
        cert.is_some_and(Certificate::is_active),
    );
    h.check_bool(
        "lifecycle_mint_owner_is_alice",
        cert.is_some_and(|c| c.owner == alice),
    );
    h.check_bool(
        "lifecycle_mint_is_game_item",
        cert.is_some_and(|c| {
            matches!(
                c.cert_type,
                loam_spine_core::CertificateType::GameItem { .. }
            )
        }),
    );

    h.check_bool(
        "lifecycle_object_registered",
        system.objects.contains_key(&sword_id),
    );

    system.advance_tick();
    system.inspect(sword_id, &alice);
    h.check_abs(
        "lifecycle_inspect_event_recorded",
        system.object_timeline(sword_id).len() as f64,
        2.0,
        0.0,
    );

    system.advance_tick();
    let trade_result = system.trade(sword_id, &alice, &bob);
    h.check_bool("lifecycle_trade_succeeds", trade_result.is_ok());

    let cert_after_trade = system.cert_manager.get_certificate(&sword_id);
    h.check_bool(
        "lifecycle_trade_owner_is_bob",
        cert_after_trade.is_some_and(|c| c.owner == bob),
    );
    h.check_bool(
        "lifecycle_trade_transfer_count",
        cert_after_trade.is_some_and(|c| c.transfer_count == 1),
    );

    system.advance_tick();
    let loan_terms = LoanTerms::new()
        .with_duration(loam_spine_core::SECONDS_PER_DAY)
        .with_auto_return(true);
    let loan_result = system.loan(sword_id, &bob, &alice, loan_terms);
    h.check_bool("lifecycle_loan_succeeds", loan_result.is_ok());

    let cert_loaned = system.cert_manager.get_certificate(&sword_id);
    h.check_bool(
        "lifecycle_loan_holder_is_alice",
        cert_loaned.is_some_and(|c| c.holder.as_ref() == Some(&alice)),
    );
    h.check_bool(
        "lifecycle_loan_owner_still_bob",
        cert_loaned.is_some_and(|c| c.owner == bob),
    );

    system.advance_tick();
    let return_result = system.return_loan(sword_id, &alice);
    h.check_bool("lifecycle_return_succeeds", return_result.is_ok());

    let cert_returned = system.cert_manager.get_certificate(&sword_id);
    h.check_bool(
        "lifecycle_return_holder_cleared",
        cert_returned.is_some_and(|c| c.holder.is_none()),
    );
    h.check_bool(
        "lifecycle_return_state_active",
        cert_returned.is_some_and(Certificate::is_active),
    );

    system.advance_tick();
    system.consume(sword_id, &bob);
    h.check_bool(
        "lifecycle_consume_event_recorded",
        system
            .object_timeline(sword_id)
            .iter()
            .any(|e| e.event_type == FermentEventType::Consume),
    );

    h.check_abs(
        "lifecycle_full_timeline_length",
        system.object_timeline(sword_id).len() as f64,
        6.0,
        0.0,
    );
}

// ===========================================================================
// 4. Object Memory — event timeline & cross-object queries
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
pub fn validate_object_memory(h: &mut ValidationHarness) {
    let alice = Did::new("did:key:alice_memory");

    let mut system = FermentingSystem::new(&alice);

    let potion_cosmetics = CosmeticSchema {
        rarity: Rarity::Common,
        skin: "red_vial".into(),
        color: "red".into(),
        material: "glass".into(),
        wear_level: 0.0,
    };
    let ring_cosmetics = CosmeticSchema {
        rarity: Rarity::Legendary,
        skin: "starfall_band".into(),
        color: "silver".into(),
        material: "starmetal".into(),
        wear_level: 0.0,
    };

    let potion_id = system.mint(&alice, "Health Potion", "consumable", potion_cosmetics);
    let ring_id = system.mint(&alice, "Ring of Stars", "accessory", ring_cosmetics);

    system.advance_tick();
    system.inspect(ring_id, &alice);
    system.advance_tick();
    system.record_achievement(ring_id, &alice, "equipped_during_boss_kill");
    system.advance_tick();
    system.record_achievement(ring_id, &alice, "survived_100_raids");

    let ring_timeline = system.object_timeline(ring_id);
    h.check_abs(
        "memory_ring_timeline_length",
        ring_timeline.len() as f64,
        4.0,
        0.0,
    );

    h.check_bool(
        "memory_ring_first_event_is_mint",
        ring_timeline
            .first()
            .is_some_and(|e| e.event_type == FermentEventType::Mint),
    );

    let achievement_count = ring_timeline
        .iter()
        .filter(|e| e.event_type == FermentEventType::Achievement)
        .count();
    h.check_abs(
        "memory_ring_two_achievements",
        achievement_count as f64,
        2.0,
        0.0,
    );

    let potion_timeline = system.object_timeline(potion_id);
    h.check_abs(
        "memory_potion_timeline_length",
        potion_timeline.len() as f64,
        1.0,
        0.0,
    );

    let alice_objects = system.objects_owned_by(&alice);
    h.check_abs(
        "memory_alice_owns_two_objects",
        alice_objects.len() as f64,
        2.0,
        0.0,
    );

    let ring_obj = system.objects.get(&ring_id);
    h.check_bool(
        "memory_ring_event_count",
        ring_obj.is_some_and(|o| o.event_count == 4),
    );

    system.advance_tick();
    system.consume(potion_id, &alice);
    let potion_timeline_after = system.object_timeline(potion_id);
    h.check_bool(
        "memory_potion_consumed",
        potion_timeline_after
            .iter()
            .any(|e| e.event_type == FermentEventType::Consume),
    );
}

// ===========================================================================
// 6. Ownership Enforcement
// ===========================================================================

pub fn validate_ownership_enforcement(h: &mut ValidationHarness) {
    let alice = Did::new("did:key:alice_owner");
    let bob = Did::new("did:key:bob_owner");
    let eve = Did::new("did:key:eve_attacker");

    let mut system = FermentingSystem::new(&alice);

    let amulet_cosmetics = CosmeticSchema {
        rarity: Rarity::Rare,
        skin: "jade_pendant".into(),
        color: "green".into(),
        material: "jade".into(),
        wear_level: 0.1,
    };

    let amulet_id = system.mint(&alice, "Jade Amulet", "accessory", amulet_cosmetics);

    let eve_trade = system.trade(amulet_id, &eve, &bob);
    h.check_bool("ownership_non_owner_trade_fails", eve_trade.is_err());

    let eve_loan = system.loan(amulet_id, &eve, &bob, LoanTerms::new());
    h.check_bool("ownership_non_owner_loan_fails", eve_loan.is_err());

    system
        .loan(amulet_id, &alice, &bob, LoanTerms::new())
        .or_exit("legitimate loan");

    let alice_return = system.return_loan(amulet_id, &alice);
    h.check_bool("ownership_non_borrower_return_fails", alice_return.is_err());

    let alice_trade_while_loaned = system.trade(amulet_id, &alice, &eve);
    h.check_bool(
        "ownership_trade_while_loaned_fails",
        alice_trade_while_loaned.is_err(),
    );

    let bob_return = system.return_loan(amulet_id, &bob);
    h.check_bool("ownership_legitimate_return_succeeds", bob_return.is_ok());
}
