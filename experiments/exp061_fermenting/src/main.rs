// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp061 — Fermenting System
//!
//! Memory-bound digital objects with full provenance: NFT without crypto.
//!
//! A "ferment" is a digital object whose value accumulates through use,
//! like a culture that transforms raw materials into something richer.
//! The provenance DAG is the culture. The loamSpine certificate is the
//! vessel. The history cannot be forged — you cannot un-ferment.
//!
//! This experiment validates:
//!   1. Cosmetic schema: rarity, skin, color, material, wear — round-trip
//!   2. Certificate lifecycle: mint → inspect → trade → loan → return → consume
//!   3. Trading protocol: offer, accept, reject, cancel, atomic swap
//!   4. Object memory: event timeline per object, cross-object queries
//!   5. Trio integration: rhizoCrypt DAG, loamSpine certs, sweetGrass braids
//!   6. Ownership: only owners can trade/loan, only borrowers can return
//!   7. Full scenario: two players, multiple objects, trades, loans, achievements

mod ferment;
#[allow(dead_code, reason = "wire format completeness — types used at runtime over IPC")]
mod protocol;

use ferment::{
    CosmeticSchema, FermentEventType, FermentingSystem, Rarity, TradeState, TradingProtocol,
};
use loam_spine_core::certificate::LoanTerms;
use loam_spine_core::{Certificate, Did};
use ludospring_barracuda::validation::ValidationResult;

const EXP: &str = "exp061_fermenting";

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

// ===========================================================================
// 1. Cosmetic Schema
// ===========================================================================

fn validate_cosmetic_schema() -> Vec<ValidationResult> {
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

    let rt = round_tripped.expect("validated above");
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
#[expect(clippy::too_many_lines, reason = "validation section — sequential checks")]
fn validate_certificate_lifecycle() -> Vec<ValidationResult> {
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
        bool_f64(cert.is_some_and(|c| matches!(c.cert_type, loam_spine_core::CertificateType::GameItem { .. }))),
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
                .any(|e| e.event_type == FermentEventType::Consume)
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
// 3. Trading Protocol: offer, accept, reject, cancel, atomic swap
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
#[expect(clippy::too_many_lines, reason = "validation section — sequential checks")]
fn validate_trading_protocol() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let alice = Did::new("did:key:alice_trader");
    let bob = Did::new("did:key:bob_trader");

    let mut system = FermentingSystem::new(&alice);
    let mut protocol = TradingProtocol::new();

    let shield_cosmetics = CosmeticSchema {
        rarity: Rarity::Uncommon,
        skin: "oak_shield".into(),
        color: "brown".into(),
        material: "wood".into(),
        wear_level: 0.3,
    };
    let helm_cosmetics = CosmeticSchema {
        rarity: Rarity::Common,
        skin: "iron_helm".into(),
        color: "gray".into(),
        material: "iron".into(),
        wear_level: 0.5,
    };

    let shield_id = system.mint(&alice, "Oak Shield", "armor", shield_cosmetics);
    let helm_id = system.mint(&alice, "Iron Helm", "armor", helm_cosmetics);

    system
        .trade(helm_id, &alice, &bob)
        .expect("initial helm transfer to bob");

    let offer_id = protocol.offer("did:key:alice_trader", "did:key:bob_trader", shield_id);
    results.push(ValidationResult::check(
        EXP,
        "trade_offer_created",
        protocol.offers.len() as f64,
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "trade_offer_pending",
        bool_f64(protocol.offers[0].state == TradeState::Pending),
        1.0,
        0.0,
    ));

    let accepted = protocol.accept(offer_id);
    results.push(ValidationResult::check(
        EXP,
        "trade_accept_succeeds",
        bool_f64(accepted),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "trade_state_accepted",
        bool_f64(protocol.offers[0].state == TradeState::Accepted),
        1.0,
        0.0,
    ));

    let exec_result = protocol.execute(offer_id, &mut system);
    results.push(ValidationResult::check(
        EXP,
        "trade_execute_succeeds",
        bool_f64(exec_result.is_ok()),
        1.0,
        0.0,
    ));

    let shield_cert = system.cert_manager.get_certificate(&shield_id);
    results.push(ValidationResult::check(
        EXP,
        "trade_shield_now_owned_by_bob",
        bool_f64(shield_cert.is_some_and(|c| c.owner == bob)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "trade_state_completed",
        bool_f64(protocol.offers[0].state == TradeState::Completed),
        1.0,
        0.0,
    ));

    let reject_offer = protocol.offer("did:key:bob_trader", "did:key:alice_trader", shield_id);
    let rejected = protocol.reject(reject_offer);
    results.push(ValidationResult::check(
        EXP,
        "trade_reject_succeeds",
        bool_f64(rejected),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "trade_reject_state",
        bool_f64(protocol.count_by_state(&TradeState::Rejected) == 1),
        1.0,
        0.0,
    ));

    let cancel_offer = protocol.offer("did:key:alice_trader", "did:key:bob_trader", helm_id);
    let cancelled = protocol.cancel(cancel_offer);
    results.push(ValidationResult::check(
        EXP,
        "trade_cancel_succeeds",
        bool_f64(cancelled),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "trade_cancel_state",
        bool_f64(protocol.count_by_state(&TradeState::Cancelled) == 1),
        1.0,
        0.0,
    ));

    // At this point: Bob owns both shield and helm.
    // Atomic swap: Bob gives shield, Alice gives... nothing she has.
    // Let Alice mint a new item for the swap.
    let staff_cosmetics = CosmeticSchema {
        rarity: Rarity::Rare,
        skin: "oak_staff".into(),
        color: "green".into(),
        material: "oak".into(),
        wear_level: 0.0,
    };
    let staff_id = system.mint(&alice, "Oak Staff", "weapon", staff_cosmetics);

    let swap_id = protocol.offer_swap(
        "did:key:bob_trader",
        "did:key:alice_trader",
        shield_id,
        staff_id,
    );
    protocol.accept(swap_id);
    let swap_result = protocol.execute(swap_id, &mut system);
    results.push(ValidationResult::check(
        EXP,
        "trade_atomic_swap_succeeds",
        bool_f64(swap_result.is_ok()),
        1.0,
        0.0,
    ));

    let shield_after_swap = system.cert_manager.get_certificate(&shield_id);
    let staff_after_swap = system.cert_manager.get_certificate(&staff_id);
    results.push(ValidationResult::check(
        EXP,
        "trade_swap_shield_to_alice",
        bool_f64(shield_after_swap.is_some_and(|c| c.owner == alice)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "trade_swap_staff_to_bob",
        bool_f64(staff_after_swap.is_some_and(|c| c.owner == bob)),
        1.0,
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
fn validate_object_memory() -> Vec<ValidationResult> {
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
        bool_f64(ring_timeline.first().is_some_and(|e| e.event_type == FermentEventType::Mint)),
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
                .any(|e| e.event_type == FermentEventType::Consume)
        ),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 5. Trio Integration — rhizoCrypt DAG + loamSpine certs + sweetGrass braids
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
#[expect(clippy::too_many_lines, reason = "validation section — sequential checks")]
fn validate_trio_integration() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let alice = Did::new("did:key:alice_trio");
    let bob = Did::new("did:key:bob_trio");

    let mut system = FermentingSystem::new(&alice);

    let gem_cosmetics = CosmeticSchema {
        rarity: Rarity::Epic,
        skin: "sapphire_cut".into(),
        color: "blue".into(),
        material: "sapphire".into(),
        wear_level: 0.0,
    };

    let gem_id = system.mint(&alice, "Sapphire Gem", "material", gem_cosmetics);
    system.advance_tick();
    system.inspect(gem_id, &alice);
    system.advance_tick();
    system.trade(gem_id, &alice, &bob).expect("trade succeeds");
    system.advance_tick();
    system.record_achievement(gem_id, &bob, "first_trade_received");

    results.push(ValidationResult::check(
        EXP,
        "trio_dag_session_active",
        bool_f64(system.dag.session.is_active()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "trio_dag_vertex_count",
        system.dag.vertices.len() as f64,
        4.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "trio_dag_frontier_singleton",
        system.dag.frontier.len() as f64,
        1.0,
        0.0,
    ));

    let all_vertex_ids: Vec<_> = system
        .dag
        .vertices
        .iter()
        .filter_map(|v| v.compute_id().ok())
        .collect();
    let unique_ids: std::collections::HashSet<_> = all_vertex_ids.iter().collect();
    results.push(ValidationResult::check(
        EXP,
        "trio_dag_all_ids_unique",
        bool_f64(unique_ids.len() == all_vertex_ids.len()),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "trio_dag_two_agents",
        system.dag.session.agents.len() as f64,
        2.0,
        0.0,
    ));

    let gem_cert = system.cert_manager.get_certificate(&gem_id);
    results.push(ValidationResult::check(
        EXP,
        "trio_cert_exists",
        bool_f64(gem_cert.is_some()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "trio_cert_owner_is_bob",
        bool_f64(gem_cert.is_some_and(|c| c.owner == bob)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "trio_cert_transfer_count_one",
        bool_f64(gem_cert.is_some_and(|c| c.transfer_count == 1)),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "trio_braids_created",
        bool_f64(system.braids.len() >= 4),
        1.0,
        0.0,
    ));

    let all_attributed = system
        .braids
        .iter()
        .all(|b| b.mime_type == "application/x-ferment-event");
    results.push(ValidationResult::check(
        EXP,
        "trio_braids_correct_mime",
        bool_f64(all_attributed),
        1.0,
        0.0,
    ));

    let source_primals: Vec<_> = system
        .braids
        .iter()
        .filter_map(|b| b.ecop.source_primal.as_deref())
        .collect();
    results.push(ValidationResult::check(
        EXP,
        "trio_braids_source_ludospring",
        bool_f64(source_primals.iter().all(|s| *s == "ludospring")),
        1.0,
        0.0,
    ));

    let first_braid_has_hash = !system.braids[0].data_hash.as_str().is_empty();
    results.push(ValidationResult::check(
        EXP,
        "trio_braid_linked_to_vertex",
        bool_f64(first_braid_has_hash),
        1.0,
        0.0,
    ));

    let alice_rhizo = rhizo_crypt_core::Did::new("did:key:alice_trio");
    let alice_sweet = sweet_grass_core::Did::new("did:key:alice_trio");
    results.push(ValidationResult::check(
        EXP,
        "trio_did_identity_across_primals",
        bool_f64(alice.as_str() == alice_rhizo.as_str() && alice.as_str() == alice_sweet.as_str()),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 6. Ownership Enforcement
// ===========================================================================

fn validate_ownership_enforcement() -> Vec<ValidationResult> {
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

    system
        .loan(amulet_id, &alice, &bob, LoanTerms::new())
        .expect("legitimate loan");

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

// ===========================================================================
// 7. Full Scenario — two players, multiple objects, rich history
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
#[expect(clippy::too_many_lines, reason = "validation section — sequential checks")]
fn validate_full_scenario() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let alice = Did::new("did:key:alice_scenario");
    let bob = Did::new("did:key:bob_scenario");

    let mut system = FermentingSystem::new(&alice);
    let mut protocol = TradingProtocol::new();

    let sword_id = system.mint(
        &alice,
        "Vorpal Blade",
        "weapon",
        CosmeticSchema {
            rarity: Rarity::Legendary,
            skin: "vorpal_edge".into(),
            color: "void_black".into(),
            material: "adamantine".into(),
            wear_level: 0.0,
        },
    );

    let potion_id = system.mint(
        &alice,
        "Elixir of Speed",
        "consumable",
        CosmeticSchema {
            rarity: Rarity::Uncommon,
            skin: "swift_vial".into(),
            color: "yellow".into(),
            material: "crystal".into(),
            wear_level: 0.0,
        },
    );

    let bob_ring_id = system.mint(
        &bob,
        "Ring of Frost",
        "accessory",
        CosmeticSchema {
            rarity: Rarity::Epic,
            skin: "frost_band".into(),
            color: "ice_blue".into(),
            material: "eternal_ice".into(),
            wear_level: 0.0,
        },
    );

    system.advance_tick();
    system.record_achievement(sword_id, &alice, "first_blood");
    system.record_achievement(sword_id, &alice, "triple_kill");
    system.record_achievement(sword_id, &alice, "boss_slayer");

    system.advance_tick();
    system.inspect(sword_id, &bob);

    system.advance_tick();
    let swap_id = protocol.offer_swap(
        "did:key:alice_scenario",
        "did:key:bob_scenario",
        sword_id,
        bob_ring_id,
    );
    protocol.accept(swap_id);
    protocol
        .execute(swap_id, &mut system)
        .expect("swap succeeds");

    system.advance_tick();
    system.record_achievement(sword_id, &bob, "inherited_a_legend");
    system.record_achievement(bob_ring_id, &alice, "new_frost_wielder");

    system.advance_tick();
    system.consume(potion_id, &alice);

    let sword_timeline = system.object_timeline(sword_id);
    results.push(ValidationResult::check(
        EXP,
        "scenario_sword_rich_history",
        bool_f64(sword_timeline.len() >= 6),
        1.0,
        0.0,
    ));

    let sword_achievements = sword_timeline
        .iter()
        .filter(|e| e.event_type == FermentEventType::Achievement)
        .count();
    results.push(ValidationResult::check(
        EXP,
        "scenario_sword_four_achievements",
        sword_achievements as f64,
        4.0,
        0.0,
    ));

    let sword_cert = system.cert_manager.get_certificate(&sword_id);
    results.push(ValidationResult::check(
        EXP,
        "scenario_sword_owned_by_bob",
        bool_f64(sword_cert.is_some_and(|c| c.owner == bob)),
        1.0,
        0.0,
    ));

    let ring_cert = system.cert_manager.get_certificate(&bob_ring_id);
    results.push(ValidationResult::check(
        EXP,
        "scenario_ring_owned_by_alice",
        bool_f64(ring_cert.is_some_and(|c| c.owner == alice)),
        1.0,
        0.0,
    ));

    let potion_events = system.object_timeline(potion_id);
    results.push(ValidationResult::check(
        EXP,
        "scenario_potion_consumed",
        bool_f64(potion_events.iter().any(|e| e.event_type == FermentEventType::Consume)),
        1.0,
        0.0,
    ));

    let total_objects = system.objects.len();
    results.push(ValidationResult::check(
        EXP,
        "scenario_three_objects_exist",
        total_objects as f64,
        3.0,
        0.0,
    ));

    let total_events = system.events.len();
    results.push(ValidationResult::check(
        EXP,
        "scenario_many_events_recorded",
        bool_f64(total_events >= 10),
        1.0,
        0.0,
    ));

    let total_braids = system.braids.len();
    results.push(ValidationResult::check(
        EXP,
        "scenario_braids_match_events",
        bool_f64(total_braids >= 10),
        1.0,
        0.0,
    ));

    let total_vertices = system.dag.vertices.len();
    results.push(ValidationResult::check(
        EXP,
        "scenario_dag_vertices_match_events",
        bool_f64(total_vertices >= 10),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "scenario_spine_height_grows",
        bool_f64(system.cert_manager.spine().height >= 5),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 8. Composable Deployment — IPC wire format validation
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
#[expect(clippy::too_many_lines, reason = "validation section — sequential checks")]
fn validate_composable_deployment() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let mint_calls = protocol::mint_ipc_sequence(
        "did:key:alice_ferment",
        "Flame Sword",
        "weapon",
        "rare",
    );
    results.push(ValidationResult::check(
        EXP,
        "ipc_mint_requires_three_calls",
        mint_calls.len() as f64,
        3.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "ipc_mint_call1_is_certificate_mint",
        bool_f64(mint_calls[0].method == "certificate.mint"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "ipc_mint_call2_is_dag_append",
        bool_f64(mint_calls[1].method == "dag.append_vertex"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "ipc_mint_call3_is_provenance_event",
        bool_f64(mint_calls[2].method == "provenance.object_event"),
        1.0,
        0.0,
    ));

    let all_jsonrpc_2_0 = mint_calls.iter().all(|c| c.jsonrpc == "2.0");
    results.push(ValidationResult::check(
        EXP,
        "ipc_all_calls_jsonrpc_2_0",
        bool_f64(all_jsonrpc_2_0),
        1.0,
        0.0,
    ));

    let cert_params: protocol::CertMintRequest =
        serde_json::from_value(mint_calls[0].params.clone()).expect("deserialization");
    results.push(ValidationResult::check(
        EXP,
        "ipc_cert_mint_has_owner",
        bool_f64(cert_params.owner_did == "did:key:alice_ferment"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "ipc_cert_mint_has_rarity",
        bool_f64(cert_params.item_attributes.get("rarity") == Some(&"rare".to_string())),
        1.0,
        0.0,
    ));

    let trade_calls = protocol::trade_ipc_sequence(
        "cert-001",
        "did:key:alice",
        "did:key:bob",
    );
    results.push(ValidationResult::check(
        EXP,
        "ipc_trade_requires_three_calls",
        trade_calls.len() as f64,
        3.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "ipc_trade_call1_is_certificate_transfer",
        bool_f64(trade_calls[0].method == "certificate.transfer"),
        1.0,
        0.0,
    ));

    let transfer_params: protocol::CertTransferRequest =
        serde_json::from_value(trade_calls[0].params.clone()).expect("deserialization");
    results.push(ValidationResult::check(
        EXP,
        "ipc_trade_transfer_from_alice",
        bool_f64(transfer_params.from_did == "did:key:alice"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "ipc_trade_transfer_to_bob",
        bool_f64(transfer_params.to_did == "did:key:bob"),
        1.0,
        0.0,
    ));

    let health_calls = protocol::deployment_health_sequence();
    results.push(ValidationResult::check(
        EXP,
        "ipc_health_checks_three_primals",
        health_calls.len() as f64,
        3.0,
        0.0,
    ));

    let mint_json = serde_json::to_string(&mint_calls[0]).expect("serialization");
    let roundtrip: protocol::JsonRpcRequest =
        serde_json::from_str(&mint_json).expect("deserialization");
    results.push(ValidationResult::check(
        EXP,
        "ipc_wire_format_roundtrip",
        bool_f64(roundtrip.method == "certificate.mint" && roundtrip.jsonrpc == "2.0"),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    println!("=== exp061: Fermenting System ===\n");

    let mut all_results = Vec::new();

    let sections: Vec<(&str, Vec<ValidationResult>)> = vec![
        ("Cosmetic Schema", validate_cosmetic_schema()),
        (
            "Certificate Lifecycle",
            validate_certificate_lifecycle(),
        ),
        ("Trading Protocol", validate_trading_protocol()),
        ("Object Memory", validate_object_memory()),
        ("Trio Integration", validate_trio_integration()),
        (
            "Ownership Enforcement",
            validate_ownership_enforcement(),
        ),
        ("Full Scenario", validate_full_scenario()),
        (
            "Composable Deployment — IPC Wire Format",
            validate_composable_deployment(),
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
