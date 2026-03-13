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

use ludospring_barracuda::validation::ValidationResult;

const EXP: &str = "exp052_provenance_trio_integration";

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

// ===========================================================================
// 1. rhizoCrypt: Game Session DAG
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_rhizocrypt_session_dag() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let alice = rhizo_crypt_core::Did::new("did:key:alice_game_player");
    let bob = rhizo_crypt_core::Did::new("did:key:bob_game_player");

    let mut dag = trio::GameSessionDag::new("MTG Session — Alice vs Bob");

    let genesis_id = dag.start_game(&alice, &bob);
    results.push(ValidationResult::check(
        EXP,
        "rhizo_session_created",
        bool_f64(dag.session.is_active()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "rhizo_genesis_vertex_exists",
        bool_f64(genesis_id != rhizo_crypt_core::VertexId::ZERO),
        1.0,
        0.0,
    ));

    let _play_land = dag.player_action(&alice, "play_land", Some("Forest"));
    let _tap_land = dag.player_action(&alice, "tap_land", Some("Forest"));
    let cast_bear = dag.player_action(&alice, "cast_spell", Some("Grizzly Bears"));
    let _bob_bolt = dag.player_action(&bob, "cast_spell", Some("Lightning Bolt"));
    let _bear_dies = dag.player_action(&alice, "creature_dies", Some("Grizzly Bears"));

    results.push(ValidationResult::check(
        EXP,
        "rhizo_dag_has_six_vertices",
        dag.vertices.len() as f64,
        6.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "rhizo_session_vertex_count_matches",
        dag.session.vertex_count as f64,
        6.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "rhizo_frontier_is_singleton",
        dag.frontier.len() as f64,
        1.0,
        0.0,
    ));

    let all_unique = {
        let ids: Vec<_> = dag
            .vertices
            .iter()
            .filter_map(|v| v.compute_id().ok())
            .collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        unique.len() == ids.len()
    };
    results.push(ValidationResult::check(
        EXP,
        "rhizo_all_vertex_ids_unique",
        bool_f64(all_unique),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "rhizo_cast_vertex_content_addressed",
        bool_f64(cast_bear != rhizo_crypt_core::VertexId::ZERO),
        1.0,
        0.0,
    ));

    let two_agents = dag.session.agents.len();
    results.push(ValidationResult::check(
        EXP,
        "rhizo_two_agents_registered",
        two_agents as f64,
        2.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 2. loamSpine: Ruleset & Card Certificates
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_loamspine_certificates() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let owner = loam_spine_core::Did::new("did:key:alice_game_player");
    let spine_id = uuid::Uuid::now_v7();

    let pf2e = trio::RulesetCertificate::new(&owner, "Pathfinder 2e", "ORC", spine_id);
    results.push(ValidationResult::check(
        EXP,
        "loam_pf2e_cert_active",
        bool_f64(pf2e.certificate.is_active()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "loam_pf2e_cert_owner_matches",
        bool_f64(pf2e.certificate.owner == owner),
        1.0,
        0.0,
    ));

    let fate = trio::RulesetCertificate::new(&owner, "FATE Core", "CC-BY-3.0", spine_id);
    results.push(ValidationResult::check(
        EXP,
        "loam_fate_cert_active",
        bool_f64(fate.certificate.is_active()),
        1.0,
        0.0,
    ));

    let bear_cert = trio::CardCertificate::new(&owner, "Grizzly Bears", "10E", 268, spine_id);
    results.push(ValidationResult::check(
        EXP,
        "loam_card_cert_active",
        bool_f64(bear_cert.certificate.is_active()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "loam_card_cert_is_game_item",
        bool_f64(matches!(
            bear_cert.certificate.cert_type,
            loam_spine_core::certificate::CertificateType::GameItem { .. }
        )),
        1.0,
        0.0,
    ));

    let bolt_cert = trio::CardCertificate::new(&owner, "Lightning Bolt", "M10", 146, spine_id);
    results.push(ValidationResult::check(
        EXP,
        "loam_bolt_cert_different_id",
        bool_f64(bolt_cert.certificate.id != bear_cert.certificate.id),
        1.0,
        0.0,
    ));

    let all_certs = [
        &pf2e.certificate,
        &fate.certificate,
        &bear_cert.certificate,
        &bolt_cert.certificate,
    ];
    let ids: std::collections::HashSet<_> = all_certs.iter().map(|c| c.id).collect();
    results.push(ValidationResult::check(
        EXP,
        "loam_all_cert_ids_unique",
        ids.len() as f64,
        4.0,
        0.0,
    ));

    let holder_matches_owner = bear_cert.certificate.effective_holder() == &owner;
    results.push(ValidationResult::check(
        EXP,
        "loam_card_holder_matches_owner",
        bool_f64(holder_matches_owner),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 3. sweetGrass: Game Action Attribution
// ===========================================================================

fn validate_sweetgrass_attribution() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let alice = sweet_grass_core::Did::new("did:key:alice_game_player");
    let bob = sweet_grass_core::Did::new("did:key:bob_game_player");
    let vertex_hex = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4";

    let cast_braid =
        trio::GameActionBraid::new(&alice, "cast_spell", Some("Grizzly Bears"), vertex_hex);
    results.push(ValidationResult::check(
        EXP,
        "sweet_cast_braid_created",
        bool_f64(cast_braid.is_ok()),
        1.0,
        0.0,
    ));

    let braid = cast_braid.expect("validated above");
    results.push(ValidationResult::check(
        EXP,
        "sweet_braid_attributed_to_alice",
        bool_f64(braid.braid.was_attributed_to == alice),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "sweet_braid_has_activity",
        bool_f64(braid.braid.was_generated_by.is_some()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "sweet_braid_mime_type_correct",
        bool_f64(braid.braid.mime_type == "application/x-game-action"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "sweet_braid_has_data_hash",
        bool_f64(!braid.braid.data_hash.as_str().is_empty()),
        1.0,
        0.0,
    ));

    let bolt_braid = trio::GameActionBraid::new(
        &bob,
        "cast_spell",
        Some("Lightning Bolt"),
        "f6e5d4c3b2a1f6e5d4c3b2a1f6e5d4c3",
    )
    .expect("braid creation should succeed");

    results.push(ValidationResult::check(
        EXP,
        "sweet_bolt_attributed_to_bob",
        bool_f64(bolt_braid.braid.was_attributed_to == bob),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "sweet_different_braids_different_ids",
        bool_f64(bolt_braid.braid.id != braid.braid.id),
        1.0,
        0.0,
    ));

    let source_primal = &braid.braid.ecop.source_primal;
    results.push(ValidationResult::check(
        EXP,
        "sweet_braid_source_primal_is_ludospring",
        bool_f64(source_primal.as_deref() == Some("ludospring")),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 4. biomeOS Graph: Trio Coordination Topology
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_biomeos_coordination() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let graph = trio::TrioCoordinationGraph::game_session_graph();
    let topology = graph.verify_topology();

    results.push(ValidationResult::check(
        EXP,
        "biomeos_graph_has_four_nodes",
        topology.node_count as f64,
        4.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "biomeos_no_unsatisfied_deps",
        topology.unsatisfied_deps.len() as f64,
        0.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "biomeos_no_orphan_feedbacks",
        topology.orphan_feedbacks.len() as f64,
        0.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "biomeos_total_budget_7ms",
        topology.total_budget_ms,
        7.0,
        0.0,
    ));

    let tick_budget_16_67: f64 = (1000.0_f64 / 60.0 * 100.0).round() / 100.0;
    results.push(ValidationResult::check(
        EXP,
        "biomeos_tick_budget_16_67ms",
        (topology.tick_budget_ms * 100.0).round() / 100.0,
        tick_budget_16_67,
        0.01,
    ));
    results.push(ValidationResult::check(
        EXP,
        "biomeos_trio_fits_in_tick",
        bool_f64(topology.fits_in_tick),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "biomeos_pattern_is_continuous",
        bool_f64(graph.pattern == trio::CoordinationPattern::Continuous),
        1.0,
        0.0,
    ));

    let rhizo_node = graph.nodes.iter().find(|n| n.id == "rhizocrypt");
    let has_feedback = rhizo_node
        .and_then(|n| n.feedback_to.as_ref())
        .is_some_and(|fb| fb == "ludospring");
    results.push(ValidationResult::check(
        EXP,
        "biomeos_rhizo_feedback_to_ludospring",
        bool_f64(has_feedback),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 5. Cross-Primal Round-Trip
// ===========================================================================

fn validate_cross_primal_roundtrip() -> Vec<ValidationResult> {
    let mut results = Vec::new();

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
    let action_braid =
        trio::GameActionBraid::new(&alice_sweet, "cast_spell", Some("Grizzly Bears"), &cast_hex)
            .expect("braid should be created from vertex hex");

    results.push(ValidationResult::check(
        EXP,
        "roundtrip_dag_vertex_exists",
        bool_f64(cast_id != rhizo_crypt_core::VertexId::ZERO),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "roundtrip_card_cert_active",
        bool_f64(card_cert.certificate.is_active()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "roundtrip_braid_links_to_vertex",
        bool_f64(action_braid.braid.data_hash.as_str().contains(&cast_hex[..16])),
        1.0,
        0.0,
    ));

    let same_did =
        alice_rhizo.as_str() == alice_loam.as_str() && alice_loam.as_str() == alice_sweet.as_str();
    results.push(ValidationResult::check(
        EXP,
        "roundtrip_did_identity_across_primals",
        bool_f64(same_did),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "roundtrip_session_tracks_agent",
        bool_f64(dag.session.agents.contains(&alice_rhizo)),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    println!("=== exp052: Provenance Trio Integration ===\n");

    let mut all_results = Vec::new();

    let sections: Vec<(&str, Vec<ValidationResult>)> = vec![
        (
            "rhizoCrypt — Game Session DAG",
            validate_rhizocrypt_session_dag(),
        ),
        (
            "loamSpine — Certificates",
            validate_loamspine_certificates(),
        ),
        (
            "sweetGrass — Attribution",
            validate_sweetgrass_attribution(),
        ),
        (
            "biomeOS — Coordination Graph",
            validate_biomeos_coordination(),
        ),
        ("Cross-Primal Round-Trip", validate_cross_primal_roundtrip()),
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
