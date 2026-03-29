// SPDX-License-Identifier: AGPL-3.0-or-later
//! System-level validation: trading protocol, trio integration, full scenario, IPC wire format.

use loam_spine_core::Did;
use ludospring_barracuda::validation::{OrExit, ValidationHarness};

use crate::ferment::{
    CosmeticSchema, FermentEventType, FermentingSystem, Rarity, TradeState, TradingProtocol,
};
use crate::protocol;

// ===========================================================================
// 3. Trading Protocol: offer, accept, reject, cancel, atomic swap
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
pub fn validate_trading_protocol(h: &mut ValidationHarness) {
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
        .or_exit("initial helm transfer to bob");

    let offer_id = protocol.offer("did:key:alice_trader", "did:key:bob_trader", shield_id);
    h.check_abs(
        "trade_offer_created",
        protocol.offers.len() as f64,
        1.0,
        0.0,
    );
    h.check_bool(
        "trade_offer_pending",
        protocol.offers[0].state == TradeState::Pending,
    );

    let accepted = protocol.accept(offer_id);
    h.check_bool("trade_accept_succeeds", accepted);
    h.check_bool(
        "trade_state_accepted",
        protocol.offers[0].state == TradeState::Accepted,
    );

    let exec_result = protocol.execute(offer_id, &mut system);
    h.check_bool("trade_execute_succeeds", exec_result.is_ok());

    let shield_cert = system.cert_manager.get_certificate(&shield_id);
    h.check_bool(
        "trade_shield_now_owned_by_bob",
        shield_cert.is_some_and(|c| c.owner == bob),
    );
    h.check_bool(
        "trade_state_completed",
        protocol.offers[0].state == TradeState::Completed,
    );

    let reject_offer = protocol.offer("did:key:bob_trader", "did:key:alice_trader", shield_id);
    let rejected = protocol.reject(reject_offer);
    h.check_bool("trade_reject_succeeds", rejected);
    h.check_bool(
        "trade_reject_state",
        protocol.count_by_state(&TradeState::Rejected) == 1,
    );

    let cancel_offer = protocol.offer("did:key:alice_trader", "did:key:bob_trader", helm_id);
    let cancelled = protocol.cancel(cancel_offer);
    h.check_bool("trade_cancel_succeeds", cancelled);
    h.check_bool(
        "trade_cancel_state",
        protocol.count_by_state(&TradeState::Cancelled) == 1,
    );

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
    h.check_bool("trade_atomic_swap_succeeds", swap_result.is_ok());

    let shield_after_swap = system.cert_manager.get_certificate(&shield_id);
    let staff_after_swap = system.cert_manager.get_certificate(&staff_id);
    h.check_bool(
        "trade_swap_shield_to_alice",
        shield_after_swap.is_some_and(|c| c.owner == alice),
    );
    h.check_bool(
        "trade_swap_staff_to_bob",
        staff_after_swap.is_some_and(|c| c.owner == bob),
    );
}

// ===========================================================================
// 5. Trio Integration — rhizoCrypt DAG + loamSpine certs + sweetGrass braids
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
pub fn validate_trio_integration(h: &mut ValidationHarness) {
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
    system
        .trade(gem_id, &alice, &bob)
        .or_exit("gem trade to bob");
    system.advance_tick();
    system.record_achievement(gem_id, &bob, "first_trade_received");

    h.check_bool("trio_dag_session_active", system.dag.session.is_active());
    h.check_abs(
        "trio_dag_vertex_count",
        system.dag.vertices.len() as f64,
        4.0,
        0.0,
    );
    h.check_abs(
        "trio_dag_frontier_singleton",
        system.dag.frontier.len() as f64,
        1.0,
        0.0,
    );

    let all_vertex_ids: Vec<_> = system
        .dag
        .vertices
        .iter()
        .filter_map(|v| v.compute_id().ok())
        .collect();
    let unique_ids: std::collections::HashSet<_> = all_vertex_ids.iter().collect();
    h.check_bool(
        "trio_dag_all_ids_unique",
        unique_ids.len() == all_vertex_ids.len(),
    );

    h.check_abs(
        "trio_dag_two_agents",
        system.dag.session.agents.len() as f64,
        2.0,
        0.0,
    );

    let gem_cert = system.cert_manager.get_certificate(&gem_id);
    h.check_bool("trio_cert_exists", gem_cert.is_some());
    h.check_bool(
        "trio_cert_owner_is_bob",
        gem_cert.is_some_and(|c| c.owner == bob),
    );
    h.check_bool(
        "trio_cert_transfer_count_one",
        gem_cert.is_some_and(|c| c.transfer_count == 1),
    );

    h.check_bool("trio_braids_created", system.braids.len() >= 4);

    let all_attributed = system
        .braids
        .iter()
        .all(|b| b.mime_type.as_ref() == "application/x-ferment-event");
    h.check_bool("trio_braids_correct_mime", all_attributed);

    let source_primals: Vec<_> = system
        .braids
        .iter()
        .filter_map(|b| b.ecop.source_primal.as_deref())
        .collect();
    h.check_bool(
        "trio_braids_source_ludospring",
        source_primals.iter().all(|s| *s == "ludospring"),
    );

    let first_braid_has_hash = !system.braids[0].data_hash.as_str().is_empty();
    h.check_bool("trio_braid_linked_to_vertex", first_braid_has_hash);

    let alice_rhizo = rhizo_crypt_core::Did::new("did:key:alice_trio");
    let alice_sweet = sweet_grass_core::Did::new("did:key:alice_trio");
    h.check_bool(
        "trio_did_identity_across_primals",
        alice.as_str() == alice_rhizo.as_str() && alice.as_str() == alice_sweet.as_str(),
    );
}

// ===========================================================================
// 7. Full Scenario — two players, multiple objects, rich history
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
pub fn validate_full_scenario(h: &mut ValidationHarness) {
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
        .or_exit("swap execution");

    system.advance_tick();
    system.record_achievement(sword_id, &bob, "inherited_a_legend");
    system.record_achievement(bob_ring_id, &alice, "new_frost_wielder");

    system.advance_tick();
    system.consume(potion_id, &alice);

    let sword_timeline = system.object_timeline(sword_id);
    h.check_bool("scenario_sword_rich_history", sword_timeline.len() >= 6);

    let sword_achievements = sword_timeline
        .iter()
        .filter(|e| e.event_type == FermentEventType::Achievement)
        .count();
    h.check_abs(
        "scenario_sword_four_achievements",
        sword_achievements as f64,
        4.0,
        0.0,
    );

    let sword_cert = system.cert_manager.get_certificate(&sword_id);
    h.check_bool(
        "scenario_sword_owned_by_bob",
        sword_cert.is_some_and(|c| c.owner == bob),
    );

    let ring_cert = system.cert_manager.get_certificate(&bob_ring_id);
    h.check_bool(
        "scenario_ring_owned_by_alice",
        ring_cert.is_some_and(|c| c.owner == alice),
    );

    let potion_events = system.object_timeline(potion_id);
    h.check_bool(
        "scenario_potion_consumed",
        potion_events
            .iter()
            .any(|e| e.event_type == FermentEventType::Consume),
    );

    let total_objects = system.objects.len();
    h.check_abs(
        "scenario_three_objects_exist",
        total_objects as f64,
        3.0,
        0.0,
    );

    let total_events = system.events.len();
    h.check_bool("scenario_many_events_recorded", total_events >= 10);

    let total_braids = system.braids.len();
    h.check_bool("scenario_braids_match_events", total_braids >= 10);

    let total_vertices = system.dag.vertices.len();
    h.check_bool("scenario_dag_vertices_match_events", total_vertices >= 10);

    h.check_bool(
        "scenario_spine_height_grows",
        system.cert_manager.spine().height >= 5,
    );
}

// ===========================================================================
// 8. Composable Deployment — IPC wire format validation
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
pub fn validate_composable_deployment(h: &mut ValidationHarness) {
    let mint_calls = protocol::mint_ipc_sequence(
        "did:key:alice_ferment",
        "Flame Sword",
        "weapon",
        "rare",
    )
    .or_exit("mint IPC sequence serialization");
    h.check_abs(
        "ipc_mint_requires_three_calls",
        mint_calls.len() as f64,
        3.0,
        0.0,
    );
    h.check_bool(
        "ipc_mint_call1_is_certificate_mint",
        mint_calls[0].method == "certificate.mint",
    );
    h.check_bool(
        "ipc_mint_call2_is_dag_append",
        mint_calls[1].method == "dag.append_vertex",
    );
    h.check_bool(
        "ipc_mint_call3_is_provenance_event",
        mint_calls[2].method == "provenance.object_event",
    );

    let all_jsonrpc_2_0 = mint_calls.iter().all(|c| c.jsonrpc == "2.0");
    h.check_bool("ipc_all_calls_jsonrpc_2_0", all_jsonrpc_2_0);

    let cert_params = serde_json::from_value::<protocol::CertMintRequest>(
        mint_calls[0].params.clone(),
    )
    .or_exit("CertMintRequest deserialization");
    h.check_bool(
        "ipc_cert_mint_has_owner",
        cert_params.owner_did == "did:key:alice_ferment",
    );
    h.check_bool(
        "ipc_cert_mint_has_rarity",
        cert_params.item_attributes.get("rarity") == Some(&"rare".to_string()),
    );

    let trade_calls =
        protocol::trade_ipc_sequence("cert-001", "did:key:alice", "did:key:bob")
            .or_exit("trade IPC sequence serialization");
    h.check_abs(
        "ipc_trade_requires_three_calls",
        trade_calls.len() as f64,
        3.0,
        0.0,
    );
    h.check_bool(
        "ipc_trade_call1_is_certificate_transfer",
        trade_calls[0].method == "certificate.transfer",
    );

    let transfer_params = serde_json::from_value::<protocol::CertTransferRequest>(
        trade_calls[0].params.clone(),
    )
    .or_exit("CertTransferRequest deserialization");
    h.check_bool(
        "ipc_trade_transfer_from_alice",
        transfer_params.from_did == "did:key:alice",
    );
    h.check_bool(
        "ipc_trade_transfer_to_bob",
        transfer_params.to_did == "did:key:bob",
    );

    let health_calls = protocol::deployment_health_sequence();
    h.check_abs(
        "ipc_health_checks_three_primals",
        health_calls.len() as f64,
        3.0,
        0.0,
    );

    let mint_json =
        serde_json::to_string(&mint_calls[0]).or_exit("mint call serialization");
    let roundtrip = serde_json::from_str::<protocol::JsonRpcRequest>(&mint_json)
        .or_exit("mint call roundtrip deserialization");
    h.check_bool(
        "ipc_wire_format_roundtrip",
        roundtrip.method == "certificate.mint" && roundtrip.jsonrpc == "2.0",
    );
}
