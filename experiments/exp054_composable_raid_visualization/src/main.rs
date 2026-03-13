// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp054: Composable Raid Visualization
//!
//! Demonstrates the composable primal architecture:
//! - **petalTongue** receives `DataBinding` JSON via `visualization.render` IPC
//! - **biomeOS** coordinates 2 players via `DeploymentGraph` (Continuous, 20 Hz)
//! - **songbird** provides discovery (`ipc.register` / `ipc.discover`)
//!
//! No chimeric dependencies on infrastructure primals. All interaction is
//! protocol-level (JSON-RPC 2.0 over Unix sockets). Data primals (rhizoCrypt,
//! loamSpine, sweetGrass) remain direct dependencies — they ARE the data layer.
//!
//! ## Architecture
//!
//! ```text
//!   biomeOS NUCLEUS
//!       │ lifecycle.register / graph.execute
//!       ├─► player1 (ludoSpring bin) ──┐
//!       ├─► player2 (ludoSpring bin)   ├─► raid_server ─► fraud_detect
//!       ├─► petalTongue (viz bin)      │        ▲                │
//!       └─► songbird (discovery)       │        └── feedback ────┘
//!                                      └─► viz_push ─► petalTongue
//! ```
//!
//! ## Fraud types validated
//!
//! 12 types across 3 tiers: basic (6), consumable (2), spatial (4).
//!
//! ## Checks
//!
//! 1. Deployment graph topology (acyclic, valid refs, execution order)
//! 2. songbird registration/discovery protocol well-formedness
//! 3. biomeOS lifecycle messages well-formedness
//! 4. 2-player simulation provenance (DAG vertices, certificates)
//! 5. petalTongue `DataBinding` JSON well-formedness (all 4 channel types)
//! 6. Dashboard composition (correct panel count, session ID)
//! 7. Streaming protocol (append, `set_value` operations)
//! 8. Cross-primal round-trip: simulation → viz payload → JSON → deserialize

#![forbid(unsafe_code)]

mod coordination;
mod protocol;
mod simulation;
mod visualization;

use ludospring_barracuda::validation::ValidationResult;

const EXP: &str = "exp054";

const fn bool_f64(b: bool) -> f64 {
    if b {
        1.0
    } else {
        0.0
    }
}

// ===========================================================================
// 1. Deployment graph topology
// ===========================================================================

#[expect(clippy::cast_precision_loss, reason = "validation counts fit in f64")]
fn validate_deployment_graph() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let graph = coordination::build_raid_deployment_graph();

    // Graph has 5 nodes
    results.push(ValidationResult::check(
        EXP,
        "graph_5_nodes",
        graph.nodes.len() as f64,
        5.0,
        0.0,
    ));

    // Coordination is continuous
    results.push(ValidationResult::check(
        EXP,
        "graph_coordination_continuous",
        bool_f64(graph.coordination == "continuous"),
        1.0,
        0.0,
    ));

    // Tick rate is 20 Hz
    results.push(ValidationResult::check(
        EXP,
        "graph_tick_20hz",
        graph.tick.as_ref().map_or(0.0, |t| t.target_hz),
        20.0,
        0.0,
    ));

    // Topology is valid (no errors)
    let errors = coordination::validate_graph_topology(&graph);
    results.push(ValidationResult::check(
        EXP,
        "graph_topology_valid",
        bool_f64(errors.is_empty()),
        1.0,
        0.0,
    ));

    // Execution order starts with inputs, ends with viz
    let order = coordination::execution_order(&graph);
    results.push(ValidationResult::check(
        EXP,
        "graph_order_5_nodes",
        order.len() as f64,
        5.0,
        0.0,
    ));

    let inputs_first = order.len() >= 2
        && order[..2].contains(&"input_p1".to_string())
        && order[..2].contains(&"input_p2".to_string());
    results.push(ValidationResult::check(
        EXP,
        "graph_inputs_execute_first",
        bool_f64(inputs_first),
        1.0,
        0.0,
    ));

    let viz_last = order.last().is_some_and(|n| n == "viz_push");
    results.push(ValidationResult::check(
        EXP,
        "graph_viz_executes_last",
        bool_f64(viz_last),
        1.0,
        0.0,
    ));

    // TOML serialization round-trip
    let toml_str = coordination::graph_to_toml(&graph);
    results.push(ValidationResult::check(
        EXP,
        "graph_toml_nonempty",
        bool_f64(!toml_str.is_empty() && toml_str.contains("[graph]")),
        1.0,
        0.0,
    ));

    // Feedback edge exists (raid_server → raid_server)
    let has_feedback = graph
        .nodes
        .iter()
        .any(|n| n.id == "raid_server" && n.feedback_to.as_deref() == Some("raid_server"));
    results.push(ValidationResult::check(
        EXP,
        "graph_feedback_edge_exists",
        bool_f64(has_feedback),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 2. songbird registration/discovery
// ===========================================================================

#[expect(clippy::cast_precision_loss, reason = "validation counts fit in f64")]
fn validate_songbird_protocol() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Player registration
    let p1_reg = coordination::player_register("player1", "/tmp/biomeos/ludospring-player1.sock");
    let json = serde_json::to_value(&p1_reg).unwrap_or_default();
    results.push(ValidationResult::check(
        EXP,
        "songbird_register_has_primal_id",
        bool_f64(json.get("primal_id").is_some()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "songbird_register_has_capabilities",
        bool_f64(
            json.get("capabilities")
                .and_then(|c| c.as_array())
                .is_some_and(|a| !a.is_empty()),
        ),
        1.0,
        0.0,
    ));

    // Server registration
    let srv_reg = coordination::raid_server_register("/tmp/biomeos/ludospring-raid-server.sock");
    let srv_json = serde_json::to_value(&srv_reg).unwrap_or_default();
    let has_authority = srv_json
        .get("capabilities")
        .and_then(|c| c.as_array())
        .is_some_and(|a| a.iter().any(|v| v.as_str() == Some("game.raid_authority")));
    results.push(ValidationResult::check(
        EXP,
        "songbird_server_has_authority_cap",
        bool_f64(has_authority),
        1.0,
        0.0,
    ));

    // Discovery response
    let disc = coordination::discover_players();
    results.push(ValidationResult::check(
        EXP,
        "songbird_discover_2_players",
        disc.providers.len() as f64,
        2.0,
        0.0,
    ));

    // Discovery JSON round-trip
    let disc_json = serde_json::to_string(&disc).unwrap_or_default();
    let disc_rt: Result<protocol::SongbirdDiscoverResponse, _> = serde_json::from_str(&disc_json);
    results.push(ValidationResult::check(
        EXP,
        "songbird_discover_json_roundtrip",
        bool_f64(disc_rt.is_ok()),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 3. biomeOS lifecycle
// ===========================================================================

fn validate_biomeos_lifecycle() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let lc = coordination::lifecycle_register(
        "ludospring-player1",
        "/tmp/biomeos/ludospring-player1.sock",
        12345,
    );
    results.push(ValidationResult::check(
        EXP,
        "biomeos_lifecycle_method",
        bool_f64(lc.method == "lifecycle.register"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "biomeos_lifecycle_jsonrpc_2_0",
        bool_f64(lc.jsonrpc == "2.0"),
        1.0,
        0.0,
    ));

    let cap = coordination::capability_register(
        "game.player_input",
        "ludospring-player1",
        "/tmp/biomeos/ludospring-player1.sock",
    );
    results.push(ValidationResult::check(
        EXP,
        "biomeos_capability_method",
        bool_f64(cap.method == "capability.register"),
        1.0,
        0.0,
    ));

    // JSON-RPC envelope round-trip
    let json = serde_json::to_string(&lc).unwrap_or_default();
    let rt: Result<protocol::JsonRpcRequest, _> = serde_json::from_str(&json);
    results.push(ValidationResult::check(
        EXP,
        "biomeos_jsonrpc_roundtrip",
        bool_f64(rt.is_ok()),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 4. 2-player simulation provenance
// ===========================================================================

#[expect(clippy::cast_precision_loss, reason = "validation counts fit in f64")]
fn validate_simulation() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let raid = simulation::run_honest_2p_raid();

    results.push(ValidationResult::check(
        EXP,
        "sim_rhizo_active",
        bool_f64(raid.rhizo_active()),
        1.0,
        0.0,
    ));

    // Both players exist
    results.push(ValidationResult::check(
        EXP,
        "sim_2_players",
        raid.players.len() as f64,
        2.0,
        0.0,
    ));

    // Vertex count > 0 (every action creates a vertex)
    results.push(ValidationResult::check(
        EXP,
        "sim_vertices_gt_10",
        bool_f64(raid.vertex_count() > 10),
        1.0,
        0.0,
    ));

    // Certs minted for all items
    results.push(ValidationResult::check(
        EXP,
        "sim_certs_ge_8",
        bool_f64(raid.cert_count() >= 8),
        1.0,
        0.0,
    ));

    // Both players extracted
    let both_extracted = raid
        .action_log
        .iter()
        .filter(|e| matches!(e.action, simulation::RaidAction::Extract { .. }))
        .count();
    results.push(ValidationResult::check(
        EXP,
        "sim_both_extracted",
        both_extracted as f64,
        2.0,
        0.0,
    ));

    // Action log timestamps are monotonic
    let monotonic = raid.action_log.windows(2).all(|w| w[1].tick >= w[0].tick);
    results.push(ValidationResult::check(
        EXP,
        "sim_timestamps_monotonic",
        bool_f64(monotonic),
        1.0,
        0.0,
    ));

    // All vertex IDs are unique
    let unique_vids: std::collections::HashSet<rhizo_crypt_core::VertexId> =
        raid.action_log.iter().map(|e| e.vertex_id).collect();
    results.push(ValidationResult::check(
        EXP,
        "sim_all_vertex_ids_unique",
        bool_f64(unique_vids.len() == raid.action_log.len()),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 5. petalTongue visualization payloads
// ===========================================================================

#[expect(clippy::cast_precision_loss, reason = "validation counts fit in f64")]
fn validate_visualization() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let raid = simulation::run_honest_2p_raid();
    let snapshot = raid.snapshot();
    let dashboard = visualization::build_raid_dashboard(&snapshot);

    // Dashboard has correct session ID
    results.push(ValidationResult::check(
        EXP,
        "viz_session_id_contains_raid",
        bool_f64(dashboard.session_id.contains("raid")),
        1.0,
        0.0,
    ));

    // Dashboard has multiple bindings
    results.push(ValidationResult::check(
        EXP,
        "viz_bindings_ge_5",
        bool_f64(dashboard.bindings.len() >= 5),
        1.0,
        0.0,
    ));

    // Has a heatmap (zone topology)
    let has_heatmap = dashboard
        .bindings
        .iter()
        .any(|b| matches!(b, protocol::DataBinding::Heatmap { .. }));
    results.push(ValidationResult::check(
        EXP,
        "viz_has_zone_heatmap",
        bool_f64(has_heatmap),
        1.0,
        0.0,
    ));

    // Has gauges (player health)
    let gauge_count = dashboard
        .bindings
        .iter()
        .filter(|b| matches!(b, protocol::DataBinding::Gauge { .. }))
        .count();
    results.push(ValidationResult::check(
        EXP,
        "viz_has_2_health_gauges",
        gauge_count as f64,
        2.0,
        0.0,
    ));

    // Has bar chart (fraud report)
    let has_fraud_bar = dashboard
        .bindings
        .iter()
        .any(|b| matches!(b, protocol::DataBinding::Bar { id, .. } if id == "fraud_report"));
    results.push(ValidationResult::check(
        EXP,
        "viz_has_fraud_bar",
        bool_f64(has_fraud_bar),
        1.0,
        0.0,
    ));

    // JSON serialization round-trip
    let json = serde_json::to_string(&dashboard).unwrap_or_default();
    let rt: Result<protocol::DashboardRenderRequest, _> = serde_json::from_str(&json);
    results.push(ValidationResult::check(
        EXP,
        "viz_dashboard_json_roundtrip",
        bool_f64(rt.is_ok()),
        1.0,
        0.0,
    ));

    // Build full JSON-RPC render request
    let rpc = visualization::render_request(&dashboard, 1);
    results.push(ValidationResult::check(
        EXP,
        "viz_rpc_method_correct",
        bool_f64(rpc.method == "visualization.render"),
        1.0,
        0.0,
    ));

    // Streaming: health gauge update
    let health_update = visualization::health_stream_update(&dashboard.session_id, "player1", 85.0);
    let stream_json = serde_json::to_string(&health_update).unwrap_or_default();
    let stream_rt: Result<protocol::StreamUpdateRequest, _> = serde_json::from_str(&stream_json);
    results.push(ValidationResult::check(
        EXP,
        "viz_stream_health_roundtrip",
        bool_f64(stream_rt.is_ok()),
        1.0,
        0.0,
    ));

    // Streaming: action timeline append
    let action_update = visualization::action_stream_append(
        &dashboard.session_id,
        "player1",
        &[100.0, 200.0],
        &[1.0, 2.0],
    );
    let action_json = serde_json::to_string(&action_update).unwrap_or_default();
    let action_rt: Result<protocol::StreamUpdateRequest, _> = serde_json::from_str(&action_json);
    results.push(ValidationResult::check(
        EXP,
        "viz_stream_actions_roundtrip",
        bool_f64(action_rt.is_ok()),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 6. Cross-primal integration
// ===========================================================================

#[expect(clippy::cast_precision_loss, reason = "validation counts fit in f64")]
fn validate_cross_primal() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // End-to-end: simulation → snapshot → viz → JSON → deserialize → binding count
    let raid = simulation::run_honest_2p_raid();
    let snapshot = raid.snapshot();
    let dashboard = visualization::build_raid_dashboard(&snapshot);
    let json = serde_json::to_string(&dashboard).unwrap_or_default();
    let rt: protocol::DashboardRenderRequest = serde_json::from_str(&json).unwrap();

    results.push(ValidationResult::check(
        EXP,
        "e2e_roundtrip_preserves_bindings",
        rt.bindings.len() as f64,
        dashboard.bindings.len() as f64,
        0.0,
    ));

    // Snapshot has correct vertex/cert counts
    results.push(ValidationResult::check(
        EXP,
        "e2e_snapshot_vertices_match_log",
        snapshot.total_vertices as f64,
        raid.action_log.len() as f64,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "e2e_snapshot_certs_match",
        snapshot.total_certs as f64,
        raid.cert_count() as f64,
        0.0,
    ));

    // Graph + simulation + viz all reference same domain
    let graph = coordination::build_raid_deployment_graph();
    let graph_has_viz = graph
        .nodes
        .iter()
        .any(|n| n.capability.as_deref() == Some("visualization.render"));
    results.push(ValidationResult::check(
        EXP,
        "e2e_graph_has_viz_node",
        bool_f64(graph_has_viz),
        1.0,
        0.0,
    ));

    let graph_has_fraud = graph
        .nodes
        .iter()
        .any(|n| n.capability.as_deref() == Some("game.fraud_analysis"));
    results.push(ValidationResult::check(
        EXP,
        "e2e_graph_has_fraud_node",
        bool_f64(graph_has_fraud),
        1.0,
        0.0,
    ));

    // Songbird discovers viz provider
    let viz_reg = coordination::viz_register("/tmp/petaltongue/petaltongue-nat0-default.sock");
    let has_viz_cap = viz_reg
        .capabilities
        .contains(&"visualization.render".into());
    results.push(ValidationResult::check(
        EXP,
        "e2e_songbird_finds_petaltongue",
        bool_f64(has_viz_cap),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    println!("=== exp054: Composable Raid Visualization ===\n");

    let mut all_results = Vec::new();

    let sections: Vec<(&str, Vec<ValidationResult>)> = vec![
        ("Deployment Graph Topology", validate_deployment_graph()),
        (
            "songbird Registration/Discovery",
            validate_songbird_protocol(),
        ),
        ("biomeOS Lifecycle", validate_biomeos_lifecycle()),
        ("2-Player Simulation Provenance", validate_simulation()),
        (
            "petalTongue Visualization Payloads",
            validate_visualization(),
        ),
        ("Cross-Primal Integration", validate_cross_primal()),
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
            eprintln!("unknown command: {other}");
            std::process::exit(1);
        }
    }
}
