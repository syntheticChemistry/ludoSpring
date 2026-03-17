// SPDX-License-Identifier: AGPL-3.0-or-later
//! Object-level validation: cosmetics, certificate lifecycle, ownership, memory.

use loam_spine_core::certificate::LoanTerms;
use loam_spine_core::{Certificate, Did};
use ludospring_barracuda::validation::ValidationResult;

use crate::ferment::{CosmeticSchema, FermentEventType, FermentingSystem, Rarity};
use crate::{EXP, bool_f64};

// ===========================================================================
// 1. Cosmetic Schema
// ===========================================================================

pub fn validate_cosmetic_schema() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let cosmetics = CosmeticSchema {
        rarity: Rarity::Epic,
        skin: "dragon_scale".into(),
        color: "crimson".into(),
        material: "obsidian".into(),
        wear_level: 0.15,
    };

    let attrs = cosmetics.to_attributes();
    results.push(ValidationResult::check(
        EXP,
        "cosmetic_rarity_attribute",
        bool_f64(attrs.get("rarity") == Some(&"epic".to_string())),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "cosmetic_skin_attribute",
        bool_f64(attrs.get("skin") == Some(&"dragon_scale".to_string())),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "cosmetic_color_attribute",
        bool_f64(attrs.get("color") == Some(&"crimson".to_string())),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "cosmetic_material_attribute",
        bool_f64(attrs.get("material") == Some(&"obsidian".to_string())),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "cosmetic_wear_attribute",
        bool_f64(attrs.get("wear_level") == Some(&"0.15".to_string())),
        1.0,
        0.0,
    ));

    let round_tripped = CosmeticSchema::from_attributes(&attrs);
    results.push(ValidationResult::check(
        EXP,
        "cosmetic_roundtrip_succeeds",
        bool_f64(round_tripped.is_some()),
        1.0,
        0.0,
    ));

    let Some(rt) = round_tripped else {
        eprintln!("FATAL: cosmetic roundtrip returned None despite prior check");
        std::process::exit(1);
    };
    results.push(ValidationResult::check(
        EXP,
        "cosmetic_roundtrip_rarity",
        bool_f64(rt.rarity == Rarity::Epic),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "cosmetic_roundtrip_skin",
        bool_f64(rt.skin == "dragon_scale"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "cosmetic_roundtrip_wear",
        bool_f64((rt.wear_level - 0.15).abs() < f64::EPSILON),
        1.0,
        0.0,
    ));

    let all_rarities = [
        Rarity::Common,
        Rarity::Uncommon,
        Rarity::Rare,
        Rarity::Epic,
        Rarity::Legendary,
    ];
    let all_unique: std::collections::HashSet<&str> =
        all_rarities.iter().map(|r| r.as_str()).collect();
    results.push(ValidationResult::check(
        EXP,
        "cosmetic_five_rarity_tiers",
        bool_f64(all_unique.len() == 5),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 2. Certificate Lifecycle: mint → inspect → trade → loan → return → consume
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
#[expect(
    clippy::too_many_lines,
    reason = "validation section — sequential checks"
)]
pub fn validate_certificate_lifecycle() -> Vec<ValidationResult> {
    let mut results = Vec::new();

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
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_mint_cert_exists",
        bool_f64(cert.is_some()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_mint_cert_active",
        bool_f64(cert.is_some_and(Certificate::is_active)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_mint_owner_is_alice",
        bool_f64(cert.is_some_and(|c| c.owner == alice)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_mint_is_game_item",
        bool_f64(cert.is_some_and(|c| {
            matches!(
                c.cert_type,
                loam_spine_core::CertificateType::GameItem { .. }
            )
        })),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "lifecycle_object_registered",
        bool_f64(system.objects.contains_key(&sword_id)),
        1.0,
        0.0,
    ));

    system.advance_tick();
    system.inspect(sword_id, &alice);
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_inspect_event_recorded",
        system.object_timeline(sword_id).len() as f64,
        2.0,
        0.0,
    ));

    system.advance_tick();
    let trade_result = system.trade(sword_id, &alice, &bob);
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_trade_succeeds",
        bool_f64(trade_result.is_ok()),
        1.0,
        0.0,
    ));

    let cert_after_trade = system.cert_manager.get_certificate(&sword_id);
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_trade_owner_is_bob",
        bool_f64(cert_after_trade.is_some_and(|c| c.owner == bob)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_trade_transfer_count",
        bool_f64(cert_after_trade.is_some_and(|c| c.transfer_count == 1)),
        1.0,
        0.0,
    ));

    system.advance_tick();
    let loan_terms = LoanTerms::new()
        .with_duration(loam_spine_core::SECONDS_PER_DAY)
        .with_auto_return(true);
    let loan_result = system.loan(sword_id, &bob, &alice, loan_terms);
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_loan_succeeds",
        bool_f64(loan_result.is_ok()),
        1.0,
        0.0,
    ));

    let cert_loaned = system.cert_manager.get_certificate(&sword_id);
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_loan_holder_is_alice",
        bool_f64(cert_loaned.is_some_and(|c| c.holder.as_ref() == Some(&alice))),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_loan_owner_still_bob",
        bool_f64(cert_loaned.is_some_and(|c| c.owner == bob)),
        1.0,
        0.0,
    ));

    system.advance_tick();
    let return_result = system.return_loan(sword_id, &alice);
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_return_succeeds",
        bool_f64(return_result.is_ok()),
        1.0,
        0.0,
    ));

    let cert_returned = system.cert_manager.get_certificate(&sword_id);
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_return_holder_cleared",
        bool_f64(cert_returned.is_some_and(|c| c.holder.is_none())),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_return_state_active",
        bool_f64(cert_returned.is_some_and(Certificate::is_active)),
        1.0,
        0.0,
    ));

    system.advance_tick();
    system.consume(sword_id, &bob);
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_consume_event_recorded",
        bool_f64(
            system
                .object_timeline(sword_id)
                .iter()
                .any(|e| e.event_type == FermentEventType::Consume),
        ),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "lifecycle_full_timeline_length",
        system.object_timeline(sword_id).len() as f64,
        6.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 4. Object Memory — event timeline & cross-object queries
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
pub fn validate_object_memory() -> Vec<ValidationResult> {
    let mut results = Vec::new();

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
    results.push(ValidationResult::check(
        EXP,
        "memory_ring_timeline_length",
        ring_timeline.len() as f64,
        4.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "memory_ring_first_event_is_mint",
        bool_f64(
            ring_timeline
                .first()
                .is_some_and(|e| e.event_type == FermentEventType::Mint),
        ),
        1.0,
        0.0,
    ));

    let achievement_count = ring_timeline
        .iter()
        .filter(|e| e.event_type == FermentEventType::Achievement)
        .count();
    results.push(ValidationResult::check(
        EXP,
        "memory_ring_two_achievements",
        achievement_count as f64,
        2.0,
        0.0,
    ));

    let potion_timeline = system.object_timeline(potion_id);
    results.push(ValidationResult::check(
        EXP,
        "memory_potion_timeline_length",
        potion_timeline.len() as f64,
        1.0,
        0.0,
    ));

    let alice_objects = system.objects_owned_by(&alice);
    results.push(ValidationResult::check(
        EXP,
        "memory_alice_owns_two_objects",
        alice_objects.len() as f64,
        2.0,
        0.0,
    ));

    let ring_obj = system.objects.get(&ring_id);
    results.push(ValidationResult::check(
        EXP,
        "memory_ring_event_count",
        bool_f64(ring_obj.is_some_and(|o| o.event_count == 4)),
        1.0,
        0.0,
    ));

    system.advance_tick();
    system.consume(potion_id, &alice);
    let potion_timeline_after = system.object_timeline(potion_id);
    results.push(ValidationResult::check(
        EXP,
        "memory_potion_consumed",
        bool_f64(
            potion_timeline_after
                .iter()
                .any(|e| e.event_type == FermentEventType::Consume),
        ),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 6. Ownership Enforcement
// ===========================================================================

pub fn validate_ownership_enforcement() -> Vec<ValidationResult> {
    let mut results = Vec::new();

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
    results.push(ValidationResult::check(
        EXP,
        "ownership_non_owner_trade_fails",
        bool_f64(eve_trade.is_err()),
        1.0,
        0.0,
    ));

    let eve_loan = system.loan(amulet_id, &eve, &bob, LoanTerms::new());
    results.push(ValidationResult::check(
        EXP,
        "ownership_non_owner_loan_fails",
        bool_f64(eve_loan.is_err()),
        1.0,
        0.0,
    ));

    let Ok(()) = system.loan(amulet_id, &alice, &bob, LoanTerms::new()) else {
        eprintln!("FATAL: legitimate loan failed");
        std::process::exit(1);
    };

    let alice_return = system.return_loan(amulet_id, &alice);
    results.push(ValidationResult::check(
        EXP,
        "ownership_non_borrower_return_fails",
        bool_f64(alice_return.is_err()),
        1.0,
        0.0,
    ));

    let alice_trade_while_loaned = system.trade(amulet_id, &alice, &eve);
    results.push(ValidationResult::check(
        EXP,
        "ownership_trade_while_loaned_fails",
        bool_f64(alice_trade_while_loaned.is_err()),
        1.0,
        0.0,
    ));

    let bob_return = system.return_loan(amulet_id, &bob);
    results.push(ValidationResult::check(
        EXP,
        "ownership_legitimate_return_succeeds",
        bool_f64(bob_return.is_ok()),
        1.0,
        0.0,
    ));

    results
}
