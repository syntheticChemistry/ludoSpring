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

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — composable raid visualization)",
    commit: "N/A",
    date: "N/A",
    command: "N/A (pure Rust — IPC protocol)",
};

fn family_id() -> String {
    std::env::var("FAMILY_ID").unwrap_or_else(|_| "default".into())
}

fn primal_sock_name(primal: &str) -> String {
    format!("{primal}-{}.sock", family_id())
}

// ===========================================================================
// 1. Deployment graph topology
// ===========================================================================

#[expect(clippy::cast_precision_loss, reason = "validation counts fit in f64")]
fn validate_deployment_graph(h: &mut ValidationHarness) {
    let graph = coordination::build_raid_deployment_graph();

    h.check_abs("graph_5_nodes", graph.nodes.len() as f64, 5.0, 0.0);
    h.check_bool(
        "graph_coordination_continuous",
        graph.coordination == "continuous",
    );
    h.check_abs(
        "graph_tick_20hz",
        graph.tick.as_ref().map_or(0.0, |t| t.target_hz),
        20.0,
        0.0,
    );

    let errors = coordination::validate_graph_topology(&graph);
    h.check_bool("graph_topology_valid", errors.is_empty());

    let order = coordination::execution_order(&graph);
    h.check_abs("graph_order_5_nodes", order.len() as f64, 5.0, 0.0);

    let inputs_first = order.len() >= 2
        && order[..2].contains(&"input_p1".to_string())
        && order[..2].contains(&"input_p2".to_string());
    h.check_bool("graph_inputs_execute_first", inputs_first);

    let viz_last = order.last().is_some_and(|n| n == "viz_push");
    h.check_bool("graph_viz_executes_last", viz_last);

    let toml_str = coordination::graph_to_toml(&graph);
    h.check_bool(
        "graph_toml_nonempty",
        !toml_str.is_empty() && toml_str.contains("[graph]"),
    );

    let has_feedback = graph
        .nodes
        .iter()
        .any(|n| n.id == "raid_server" && n.feedback_to.as_deref() == Some("raid_server"));
    h.check_bool("graph_feedback_edge_exists", has_feedback);
}

// ===========================================================================
// 2. songbird registration/discovery
// ===========================================================================

#[expect(clippy::cast_precision_loss, reason = "validation counts fit in f64")]
fn validate_songbird_protocol(h: &mut ValidationHarness) {
    let base = std::env::temp_dir().join("biomeos");
    let p1_sock = base.join(primal_sock_name("ludospring-player1"));
    let p1_reg = coordination::player_register("player1", &p1_sock.to_string_lossy());
    let json = serde_json::to_value(&p1_reg).unwrap_or_default();
    h.check_bool(
        "songbird_register_has_primal_id",
        json.get("primal_id").is_some(),
    );
    h.check_bool(
        "songbird_register_has_capabilities",
        json.get("capabilities")
            .and_then(|c| c.as_array())
            .is_some_and(|a| !a.is_empty()),
    );

    let srv_sock = base.join(primal_sock_name("ludospring-raid-server"));
    let srv_reg = coordination::raid_server_register(&srv_sock.to_string_lossy());
    let srv_json = serde_json::to_value(&srv_reg).unwrap_or_default();
    let has_authority = srv_json
        .get("capabilities")
        .and_then(|c| c.as_array())
        .is_some_and(|a| a.iter().any(|v| v.as_str() == Some("game.raid_authority")));
    h.check_bool("songbird_server_has_authority_cap", has_authority);

    let disc = coordination::discover_players();
    h.check_abs(
        "songbird_discover_2_players",
        disc.providers.len() as f64,
        2.0,
        0.0,
    );

    let disc_json = serde_json::to_string(&disc).unwrap_or_default();
    let disc_rt: Result<protocol::SongbirdDiscoverResponse, _> = serde_json::from_str(&disc_json);
    h.check_bool("songbird_discover_json_roundtrip", disc_rt.is_ok());
}

// ===========================================================================
// 3. biomeOS lifecycle
// ===========================================================================

fn validate_biomeos_lifecycle(h: &mut ValidationHarness) {
    let base = std::env::temp_dir().join("biomeos");
    let p1_name = format!("ludospring-player1-{}", family_id());
    let p1_sock = base.join(primal_sock_name("ludospring-player1"));
    let lc = coordination::lifecycle_register(&p1_name, &p1_sock.to_string_lossy(), 12345);
    h.check_bool(
        "biomeos_lifecycle_method",
        lc.method == "lifecycle.register",
    );
    h.check_bool("biomeos_lifecycle_jsonrpc_2_0", lc.jsonrpc == "2.0");

    let cap = coordination::capability_register(
        "game.player_input",
        &p1_name,
        &p1_sock.to_string_lossy(),
    );
    h.check_bool(
        "biomeos_capability_method",
        cap.method == "capability.register",
    );

    let json = serde_json::to_string(&lc).unwrap_or_default();
    let rt: Result<protocol::JsonRpcRequest, _> = serde_json::from_str(&json);
    h.check_bool("biomeos_jsonrpc_roundtrip", rt.is_ok());
}

// ===========================================================================
// 4. 2-player simulation provenance
// ===========================================================================

#[expect(clippy::cast_precision_loss, reason = "validation counts fit in f64")]
fn validate_simulation(h: &mut ValidationHarness) {
    let raid = simulation::run_honest_2p_raid();

    h.check_bool("sim_rhizo_active", raid.rhizo_active());
    h.check_abs("sim_2_players", raid.players.len() as f64, 2.0, 0.0);
    h.check_bool("sim_vertices_gt_10", raid.vertex_count() > 10);
    h.check_bool("sim_certs_ge_8", raid.cert_count() >= 8);

    let both_extracted = raid
        .action_log
        .iter()
        .filter(|e| matches!(e.action, simulation::RaidAction::Extract { .. }))
        .count();
    h.check_abs("sim_both_extracted", both_extracted as f64, 2.0, 0.0);

    let monotonic = raid.action_log.windows(2).all(|w| w[1].tick >= w[0].tick);
    h.check_bool("sim_timestamps_monotonic", monotonic);

    let unique_vids: std::collections::HashSet<rhizo_crypt_core::VertexId> =
        raid.action_log.iter().map(|e| e.vertex_id).collect();
    h.check_bool(
        "sim_all_vertex_ids_unique",
        unique_vids.len() == raid.action_log.len(),
    );
}

// ===========================================================================
// 5. petalTongue visualization payloads
// ===========================================================================

#[expect(clippy::cast_precision_loss, reason = "validation counts fit in f64")]
fn validate_visualization(h: &mut ValidationHarness) {
    let raid = simulation::run_honest_2p_raid();
    let snapshot = raid.snapshot();
    let dashboard = visualization::build_raid_dashboard(&snapshot);

    h.check_bool(
        "viz_session_id_contains_raid",
        dashboard.session_id.contains("raid"),
    );
    h.check_bool("viz_bindings_ge_5", dashboard.bindings.len() >= 5);

    let has_heatmap = dashboard
        .bindings
        .iter()
        .any(|b| matches!(b, protocol::DataBinding::Heatmap { .. }));
    h.check_bool("viz_has_zone_heatmap", has_heatmap);

    let gauge_count = dashboard
        .bindings
        .iter()
        .filter(|b| matches!(b, protocol::DataBinding::Gauge { .. }))
        .count();
    h.check_abs("viz_has_2_health_gauges", gauge_count as f64, 2.0, 0.0);

    let has_fraud_bar = dashboard
        .bindings
        .iter()
        .any(|b| matches!(b, protocol::DataBinding::Bar { id, .. } if id == "fraud_report"));
    h.check_bool("viz_has_fraud_bar", has_fraud_bar);

    let json = serde_json::to_string(&dashboard).unwrap_or_default();
    let rt: Result<protocol::DashboardRenderRequest, _> = serde_json::from_str(&json);
    h.check_bool("viz_dashboard_json_roundtrip", rt.is_ok());

    let rpc = visualization::render_request(&dashboard, 1);
    h.check_bool(
        "viz_rpc_method_correct",
        rpc.method == "visualization.render",
    );

    let health_update = visualization::health_stream_update(&dashboard.session_id, "player1", 85.0);
    let stream_json = serde_json::to_string(&health_update).unwrap_or_default();
    let stream_rt: Result<protocol::StreamUpdateRequest, _> = serde_json::from_str(&stream_json);
    h.check_bool("viz_stream_health_roundtrip", stream_rt.is_ok());

    let action_update = visualization::action_stream_append(
        &dashboard.session_id,
        "player1",
        &[100.0, 200.0],
        &[1.0, 2.0],
    );
    let action_json = serde_json::to_string(&action_update).unwrap_or_default();
    let action_rt: Result<protocol::StreamUpdateRequest, _> = serde_json::from_str(&action_json);
    h.check_bool("viz_stream_actions_roundtrip", action_rt.is_ok());
}

// ===========================================================================
// 6. Cross-primal integration
// ===========================================================================

#[expect(clippy::cast_precision_loss, reason = "validation counts fit in f64")]
fn validate_cross_primal(h: &mut ValidationHarness) {
    let raid = simulation::run_honest_2p_raid();
    let snapshot = raid.snapshot();
    let dashboard = visualization::build_raid_dashboard(&snapshot);
    let json = serde_json::to_string(&dashboard).unwrap_or_default();
    let Ok(rt) = serde_json::from_str::<protocol::DashboardRenderRequest>(&json) else {
        eprintln!("FATAL: round-trip deserialization failed");
        std::process::exit(1);
    };

    h.check_abs(
        "e2e_roundtrip_preserves_bindings",
        rt.bindings.len() as f64,
        dashboard.bindings.len() as f64,
        0.0,
    );
    h.check_abs(
        "e2e_snapshot_vertices_match_log",
        snapshot.total_vertices as f64,
        raid.action_log.len() as f64,
        0.0,
    );
    h.check_abs(
        "e2e_snapshot_certs_match",
        snapshot.total_certs as f64,
        raid.cert_count() as f64,
        0.0,
    );

    let graph = coordination::build_raid_deployment_graph();
    let graph_has_viz = graph
        .nodes
        .iter()
        .any(|n| n.capability.as_deref() == Some("visualization.render"));
    h.check_bool("e2e_graph_has_viz_node", graph_has_viz);

    let graph_has_fraud = graph
        .nodes
        .iter()
        .any(|n| n.capability.as_deref() == Some("game.fraud_analysis"));
    h.check_bool("e2e_graph_has_fraud_node", graph_has_fraud);

    let viz_sock = std::env::temp_dir()
        .join("biomeos")
        .join(format!("viz-provider-{}.sock", family_id()));
    let viz_reg = coordination::viz_register("viz-provider", &viz_sock.to_string_lossy());
    let has_viz_cap = viz_reg
        .capabilities
        .contains(&"visualization.render".into());
    h.check_bool("e2e_songbird_finds_viz_capability", has_viz_cap);
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp054_composable_raid_visualization");
    h.print_provenance(&[&PROVENANCE]);

    validate_deployment_graph(&mut h);
    validate_songbird_protocol(&mut h);
    validate_biomeos_lifecycle(&mut h);
    validate_simulation(&mut h);
    validate_visualization(&mut h);
    validate_cross_primal(&mut h);

    h.finish();
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
