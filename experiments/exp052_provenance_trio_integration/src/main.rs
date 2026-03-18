// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp052 — Provenance Trio Integration
//!
//! First experiment to import and exercise the three provenance primals
//! directly from ludoSpring:
//!
//!   - **rhizoCrypt** (ephemeral DAG memory): game session as vertex graph
//!   - **loamSpine** (immutable ledger): ruleset & card certificates
//!   - **sweetGrass** (W3C PROV-O attribution): player action braids
//!
//! The trio lives among the biomeOS atomics. The rootpulse niche
//! coordinates them: rhizoCrypt provides ephemeral workspace, loamSpine
//! anchors permanent records, sweetGrass attributes creative contributions.
//! biomeOS deploys them as a continuous 60 Hz graph via `ContinuousExecutor`.
//!
//! This experiment validates:
//!   1. rhizoCrypt: game session DAG creation, vertex append, frontier tracking
//!   2. loamSpine: ruleset/card certificate minting, ownership, type fidelity
//!   3. sweetGrass: game action braids, PROV-O attribution, agent linking
//!   4. biomeOS graph model: trio topology, budget fit, dependency satisfaction
//!   5. Cross-primal round-trip: game action → vertex → certificate → braid
//!
//! Anti-pattern it replaces: exp045–047 modeled the trio types locally.
//! This experiment uses the real crate types. Same validation, real wiring.

mod trio;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — provenance trio integration)",
    commit: "N/A",
    date: "N/A",
    command: "N/A (pure Rust — crate types)",
};

// ===========================================================================
// 1. rhizoCrypt: Game Session DAG
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_rhizocrypt_session_dag(h: &mut ValidationHarness) {
    let alice = rhizo_crypt_core::Did::new("did:key:alice_game_player");
    let bob = rhizo_crypt_core::Did::new("did:key:bob_game_player");

    let mut dag = trio::GameSessionDag::new("MTG Session — Alice vs Bob");

    let genesis_id = dag.start_game(&alice, &bob);
    h.check_bool("rhizo_session_created", dag.session.is_active());
    h.check_bool(
        "rhizo_genesis_vertex_exists",
        genesis_id != rhizo_crypt_core::VertexId::ZERO,
    );

    let _play_land = dag.player_action(&alice, "play_land", Some("Forest"));
    let _tap_land = dag.player_action(&alice, "tap_land", Some("Forest"));
    let cast_bear = dag.player_action(&alice, "cast_spell", Some("Grizzly Bears"));
    let _bob_bolt = dag.player_action(&bob, "cast_spell", Some("Lightning Bolt"));
    let _bear_dies = dag.player_action(&alice, "creature_dies", Some("Grizzly Bears"));

    h.check_abs(
        "rhizo_dag_has_six_vertices",
        dag.vertices.len() as f64,
        6.0,
        0.0,
    );
    h.check_abs(
        "rhizo_session_vertex_count_matches",
        dag.session.vertex_count as f64,
        6.0,
        0.0,
    );
    h.check_abs(
        "rhizo_frontier_is_singleton",
        dag.frontier.len() as f64,
        1.0,
        0.0,
    );

    let all_unique = {
        let ids: Vec<_> = dag
            .vertices
            .iter()
            .filter_map(|v| v.compute_id().ok())
            .collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        unique.len() == ids.len()
    };
    h.check_bool("rhizo_all_vertex_ids_unique", all_unique);
    h.check_bool(
        "rhizo_cast_vertex_content_addressed",
        cast_bear != rhizo_crypt_core::VertexId::ZERO,
    );

    let two_agents = dag.session.agents.len();
    h.check_abs("rhizo_two_agents_registered", two_agents as f64, 2.0, 0.0);
}

// ===========================================================================
// 2. loamSpine: Ruleset & Card Certificates
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_loamspine_certificates(h: &mut ValidationHarness) {
    let owner = loam_spine_core::Did::new("did:key:alice_game_player");
    let spine_id = uuid::Uuid::now_v7();

    let pf2e = trio::RulesetCertificate::new(&owner, "Pathfinder 2e", "ORC", spine_id);
    h.check_bool("loam_pf2e_cert_active", pf2e.certificate.is_active());
    h.check_bool(
        "loam_pf2e_cert_owner_matches",
        pf2e.certificate.owner == owner,
    );

    let fate = trio::RulesetCertificate::new(&owner, "FATE Core", "CC-BY-3.0", spine_id);
    h.check_bool("loam_fate_cert_active", fate.certificate.is_active());

    let bear_cert = trio::CardCertificate::new(&owner, "Grizzly Bears", "10E", 268, spine_id);
    h.check_bool("loam_card_cert_active", bear_cert.certificate.is_active());
    h.check_bool(
        "loam_card_cert_is_game_item",
        matches!(
            bear_cert.certificate.cert_type,
            loam_spine_core::certificate::CertificateType::GameItem { .. }
        ),
    );

    let bolt_cert = trio::CardCertificate::new(&owner, "Lightning Bolt", "M10", 146, spine_id);
    h.check_bool(
        "loam_bolt_cert_different_id",
        bolt_cert.certificate.id != bear_cert.certificate.id,
    );

    let all_certs = [
        &pf2e.certificate,
        &fate.certificate,
        &bear_cert.certificate,
        &bolt_cert.certificate,
    ];
    let ids: std::collections::HashSet<_> = all_certs.iter().map(|c| c.id).collect();
    h.check_abs("loam_all_cert_ids_unique", ids.len() as f64, 4.0, 0.0);

    let holder_matches_owner = bear_cert.certificate.effective_holder() == &owner;
    h.check_bool("loam_card_holder_matches_owner", holder_matches_owner);
}

// ===========================================================================
// 3. sweetGrass: Game Action Attribution
// ===========================================================================

fn validate_sweetgrass_attribution(h: &mut ValidationHarness) {
    let alice = sweet_grass_core::Did::new("did:key:alice_game_player");
    let bob = sweet_grass_core::Did::new("did:key:bob_game_player");
    let vertex_hex = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4";

    let cast_braid =
        trio::GameActionBraid::new(&alice, "cast_spell", Some("Grizzly Bears"), vertex_hex);
    h.check_bool("sweet_cast_braid_created", cast_braid.is_ok());

    let Ok(braid) = cast_braid else {
        eprintln!("FATAL: cast_braid creation failed (validated above as Ok)");
        std::process::exit(1);
    };
    h.check_bool(
        "sweet_braid_attributed_to_alice",
        braid.braid.was_attributed_to == alice,
    );
    h.check_bool(
        "sweet_braid_has_activity",
        braid.braid.was_generated_by.is_some(),
    );
    h.check_bool(
        "sweet_braid_mime_type_correct",
        braid.braid.mime_type.as_ref() == "application/x-game-action",
    );
    h.check_bool(
        "sweet_braid_has_data_hash",
        !braid.braid.data_hash.as_str().is_empty(),
    );

    let Ok(bolt_braid) = trio::GameActionBraid::new(
        &bob,
        "cast_spell",
        Some("Lightning Bolt"),
        "f6e5d4c3b2a1f6e5d4c3b2a1f6e5d4c3",
    ) else {
        eprintln!("FATAL: bolt braid creation failed");
        std::process::exit(1);
    };

    h.check_bool(
        "sweet_bolt_attributed_to_bob",
        bolt_braid.braid.was_attributed_to == bob,
    );
    h.check_bool(
        "sweet_different_braids_different_ids",
        bolt_braid.braid.id != braid.braid.id,
    );

    let source_primal = &braid.braid.ecop.source_primal;
    h.check_bool(
        "sweet_braid_source_primal_is_ludospring",
        source_primal.as_deref() == Some("ludospring"),
    );
}

// ===========================================================================
// 4. biomeOS Graph: Trio Coordination Topology
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_biomeos_coordination(h: &mut ValidationHarness) {
    let graph = trio::TrioCoordinationGraph::game_session_graph();
    let topology = graph.verify_topology();

    h.check_abs(
        "biomeos_graph_has_four_nodes",
        topology.node_count as f64,
        4.0,
        0.0,
    );
    h.check_abs(
        "biomeos_no_unsatisfied_deps",
        topology.unsatisfied_deps.len() as f64,
        0.0,
        0.0,
    );
    h.check_abs(
        "biomeos_no_orphan_feedbacks",
        topology.orphan_feedbacks.len() as f64,
        0.0,
        0.0,
    );
    h.check_abs(
        "biomeos_total_budget_7ms",
        topology.total_budget_ms,
        7.0,
        0.0,
    );

    let tick_budget_16_67: f64 = (1000.0_f64 / 60.0 * 100.0).round() / 100.0;
    h.check_abs(
        "biomeos_tick_budget_16_67ms",
        (topology.tick_budget_ms * 100.0).round() / 100.0,
        tick_budget_16_67,
        0.01,
    );
    h.check_bool("biomeos_trio_fits_in_tick", topology.fits_in_tick);

    h.check_bool(
        "biomeos_pattern_is_continuous",
        graph.pattern == trio::CoordinationPattern::Continuous,
    );

    let rhizo_node = graph.nodes.iter().find(|n| n.id == "rhizocrypt");
    let has_feedback = rhizo_node
        .and_then(|n| n.feedback_to.as_ref())
        .is_some_and(|fb| fb == "ludospring");
    h.check_bool("biomeos_rhizo_feedback_to_ludospring", has_feedback);
}

// ===========================================================================
// 5. Cross-Primal Round-Trip
// ===========================================================================

fn validate_cross_primal_roundtrip(h: &mut ValidationHarness) {
    let alice_rhizo = rhizo_crypt_core::Did::new("did:key:alice_game_player");
    let alice_loam = loam_spine_core::Did::new("did:key:alice_game_player");
    let alice_sweet = sweet_grass_core::Did::new("did:key:alice_game_player");
    let spine_id = uuid::Uuid::now_v7();

    let mut dag = trio::GameSessionDag::new("Cross-Primal Round-Trip");
    let _genesis = dag.start_game(
        &alice_rhizo,
        &rhizo_crypt_core::Did::new("did:key:bob_game_player"),
    );
    let cast_id = dag.player_action(&alice_rhizo, "cast_spell", Some("Grizzly Bears"));

    let card_cert = trio::CardCertificate::new(&alice_loam, "Grizzly Bears", "10E", 268, spine_id);

    let cast_hex = cast_id.to_hex();
    let Ok(action_braid) =
        trio::GameActionBraid::new(&alice_sweet, "cast_spell", Some("Grizzly Bears"), &cast_hex)
    else {
        eprintln!("FATAL: action braid creation from vertex hex failed");
        std::process::exit(1);
    };

    h.check_bool(
        "roundtrip_dag_vertex_exists",
        cast_id != rhizo_crypt_core::VertexId::ZERO,
    );
    h.check_bool(
        "roundtrip_card_cert_active",
        card_cert.certificate.is_active(),
    );
    h.check_bool(
        "roundtrip_braid_links_to_vertex",
        action_braid
            .braid
            .data_hash
            .as_str()
            .contains(&cast_hex[..16]),
    );

    let same_did =
        alice_rhizo.as_str() == alice_loam.as_str() && alice_loam.as_str() == alice_sweet.as_str();
    h.check_bool("roundtrip_did_identity_across_primals", same_did);
    h.check_bool(
        "roundtrip_session_tracks_agent",
        dag.session.agents.contains(&alice_rhizo),
    );
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp052_provenance_trio_integration");
    h.print_provenance(&[&PROVENANCE]);

    validate_rhizocrypt_session_dag(&mut h);
    validate_loamspine_certificates(&mut h);
    validate_sweetgrass_attribution(&mut h);
    validate_biomeos_coordination(&mut h);
    validate_cross_primal_roundtrip(&mut h);

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
