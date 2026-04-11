// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC integration test — validates full server lifecycle and method dispatch.
//!
//! Starts a real `IpcServer`, connects via Unix socket, exercises the local
//! capability methods (game science, not external primals), and verifies
//! structured responses.  The server runs in a background thread with
//! `run_until()` and shuts down cleanly via an `AtomicBool`.
//!
//! External-primal methods (Squirrel, NestGate, rhizoCrypt, etc.) are
//! intentionally skipped — those require live primals.  This test validates
//! the dispatch pipeline and wire format for everything ludoSpring can
//! evaluate locally.

#![cfg(feature = "ipc")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod ipc_test_util;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use ipc_test_util::IpcTestServer;

#[allow(clippy::needless_pass_by_value)]
fn send_rpc(stream: &mut UnixStream, method: &str, params: serde_json::Value) -> serde_json::Value {
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let mut msg = serde_json::to_string(&req).expect("serialize");
    msg.push('\n');
    stream.write_all(msg.as_bytes()).expect("write");
    stream.flush().expect("flush");

    let mut reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut response = String::new();
    reader.read_line(&mut response).expect("read");
    serde_json::from_str(&response).expect("parse response")
}

#[test]
fn lifecycle_status_returns_name_and_capabilities() {
    let server = IpcTestServer::start();

    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(&mut stream, "lifecycle.status", serde_json::json!({}));
    let result = resp.get("result").expect("result field");

    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some("ludospring")
    );
    let caps = result.get("capabilities").and_then(|v| v.as_array());
    assert!(caps.is_some(), "should have capabilities array");
    assert!(
        caps.expect("caps").len() >= 20,
        "should have 20+ capabilities"
    );

    drop(stream);
    server.shutdown();
}

#[test]
fn evaluate_flow_returns_structured_result() {
    let server = IpcTestServer::start();

    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(
        &mut stream,
        "game.evaluate_flow",
        serde_json::json!({"challenge": 0.5, "skill": 0.5}),
    );
    let result = resp.get("result").expect("result field");
    let state = result.get("state").and_then(|v| v.as_str());
    assert_eq!(state, Some("flow"));

    drop(stream);
    server.shutdown();
}

#[test]
fn fitts_cost_returns_movement_time() {
    let server = IpcTestServer::start();

    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(
        &mut stream,
        "game.fitts_cost",
        serde_json::json!({"distance": 100.0, "target_width": 10.0}),
    );
    let result = resp.get("result").expect("result field");
    let mt = result
        .get("movement_time_ms")
        .and_then(serde_json::Value::as_f64);
    assert!(mt.is_some(), "should return movement_time_ms");
    assert!(mt.expect("mt") > 0.0, "movement time should be positive");

    drop(stream);
    server.shutdown();
}

#[test]
fn capability_list_returns_all_capabilities() {
    let server = IpcTestServer::start();

    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(&mut stream, "capability.list", serde_json::json!({}));
    let result = resp.get("result").expect("result field");

    let total = result
        .get("total_capabilities")
        .and_then(serde_json::Value::as_u64);
    assert!(
        total.is_some_and(|n| n > 0),
        "should have total_capabilities > 0"
    );

    let domains = result.get("domains").and_then(serde_json::Value::as_array);
    assert!(
        domains.is_some_and(|d| !d.is_empty()),
        "should have domains from capability_domains"
    );

    drop(stream);
    server.shutdown();
}

#[test]
fn unknown_method_returns_error() {
    let server = IpcTestServer::start();

    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(&mut stream, "nonexistent.method", serde_json::json!({}));
    let error = resp.get("error");
    assert!(error.is_some(), "unknown method should return error");

    drop(stream);
    server.shutdown();
}

#[test]
fn health_check_responds() {
    let server = IpcTestServer::start();

    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(&mut stream, "health.check", serde_json::json!({}));
    assert!(
        resp.get("result").is_some(),
        "health check should return result"
    );

    drop(stream);
    server.shutdown();
}

#[test]
fn neural_bridge_rpc_round_trip_with_socket() {
    let server = IpcTestServer::start();
    let bridge = ludospring_barracuda::ipc::NeuralBridge::with_socket_and_timeout(
        server.socket_path.clone(),
        Duration::from_secs(5),
    );
    assert!(bridge.is_available(), "health.check should succeed");

    let reg_dir = ipc_test_util::ipc_test_socket_dir();
    std::fs::create_dir_all(&reg_dir).expect("registration socket dir");
    let reg_sock = reg_dir.join("register.sock");
    let reg = bridge.register(&reg_sock).expect("lifecycle.register");
    assert_eq!(
        reg.get("status").and_then(|v| v.as_str()),
        Some("registered")
    );

    let disc = bridge
        .discover_capability("game.evaluate_flow")
        .expect("capability.discover");
    assert!(disc.get("providers").is_some());

    let cap = bridge
        .capability_call(
            "game",
            "evaluate_flow",
            &serde_json::json!({"challenge": 0.5, "skill": 0.5}),
        )
        .expect("capability.call");
    assert_eq!(cap.get("state").and_then(|v| v.as_str()), Some("flow"));

    let dereg = bridge.deregister().expect("deregister");
    assert_eq!(
        dereg.get("status").and_then(|v| v.as_str()),
        Some("deregistered")
    );

    server.shutdown();
}

#[test]
fn discovery_probe_socket_finds_primal() {
    let server = IpcTestServer::start();
    let ep = ludospring_barracuda::ipc::discovery::probe_socket(&server.socket_path);
    assert!(ep.is_some(), "probe_socket should parse lifecycle.status");
    let ep = ep.expect("endpoint");
    assert_eq!(ep.name.as_str(), "ludospring");
    assert!(!ep.capabilities.is_empty());

    server.shutdown();
}

#[test]
fn discovery_discover_primals_scans_directory() {
    let server = IpcTestServer::start();
    let dir = server
        .socket_path
        .parent()
        .expect("socket parent")
        .to_path_buf();

    let reg = ludospring_barracuda::ipc::discover_primals_in_directories(&[dir]);
    assert!(
        reg.find("game.evaluate_flow").is_some(),
        "registry should list game.evaluate_flow from scanned socket"
    );

    server.shutdown();
}

#[test]
fn call_primal_evaluate_flow() {
    let server = IpcTestServer::start();
    let ep = ludospring_barracuda::ipc::discovery::probe_socket(&server.socket_path)
        .expect("probe_socket");
    let result = ludospring_barracuda::ipc::call_primal(
        &ep,
        "game.evaluate_flow",
        &serde_json::json!({"challenge": 0.5, "skill": 0.5}),
    )
    .expect("call_primal");
    assert_eq!(result.get("state").and_then(|v| v.as_str()), Some("flow"));

    server.shutdown();
}

#[test]
fn external_primal_degradation_squirrel_unavailable() {
    let server = IpcTestServer::start();

    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(
        &mut stream,
        "game.npc_dialogue",
        serde_json::json!({
            "npc_name": "Merchant",
            "personality_prompt": "You sell potions.",
            "player_input": "Hello"
        }),
    );
    let result = resp.get("result").expect("result field");
    assert_eq!(result["available"], false, "Squirrel unavailable");
    assert!(result.get("text").is_some());

    drop(stream);
    server.shutdown();
}

#[test]
fn external_primal_degradation_nestgate_unavailable() {
    let server = IpcTestServer::start();

    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(
        &mut stream,
        "game.storage_put",
        serde_json::json!({"key": "test-key", "data": {"v": 1}}),
    );
    let result = resp.get("result").expect("result field");
    assert_eq!(result["available"], false, "NestGate unavailable");

    let resp_get = send_rpc(
        &mut stream,
        "game.storage_get",
        serde_json::json!({"key": "test-key"}),
    );
    let result_get = resp_get.get("result").expect("result field");
    assert_eq!(result_get["available"], false, "NestGate unavailable");

    drop(stream);
    server.shutdown();
}

#[test]
fn external_primal_degradation_provenance_unavailable() {
    let server = IpcTestServer::start();

    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(
        &mut stream,
        "game.begin_session",
        serde_json::json!({"session_name": "degradation-test"}),
    );
    let result = resp.get("result").expect("result field");
    assert_eq!(result["provenance"], "unavailable");

    let resp_complete = send_rpc(
        &mut stream,
        "game.complete_session",
        serde_json::json!({"session_id": "test-session"}),
    );
    let result_complete = resp_complete.get("result").expect("result field");
    assert_eq!(result_complete["stage"], "unavailable");

    drop(stream);
    server.shutdown();
}

#[test]
fn external_primal_degradation_gpu_dispatch_unavailable() {
    let server = IpcTestServer::start();

    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(
        &mut stream,
        "game.gpu.batch_raycast",
        serde_json::json!({
            "grid_w": 4,
            "grid_h": 4,
            "origins_x": [1.5],
            "origins_y": [1.5],
            "angles": [0.0]
        }),
    );
    let result = resp.get("result").expect("result field");
    assert_eq!(result["available"], false, "toadStool unavailable");
    assert_eq!(result["fallback"], "cpu");

    drop(stream);
    server.shutdown();
}

#[test]
fn health_liveness_and_readiness_via_ipc() {
    let server = IpcTestServer::start();

    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp_live = send_rpc(&mut stream, "health.liveness", serde_json::json!({}));
    let result_live = resp_live.get("result").expect("result field");
    assert_eq!(result_live["status"], "alive");

    let resp_ready = send_rpc(&mut stream, "health.readiness", serde_json::json!({}));
    let result_ready = resp_ready.get("result").expect("result field");
    assert_eq!(result_ready["ready"], true);
    assert!(result_ready.get("subsystems").is_some());

    drop(stream);
    server.shutdown();
}

#[test]
fn visualization_push_client_uses_explicit_socket() {
    let server = IpcTestServer::start();
    let client = ludospring_barracuda::visualization::VisualizationPushClient::with_socket(
        server.socket_path.clone(),
    );
    client
        .push_render("s1", "t", &serde_json::json!({}))
        .expect("push_render");
    client.export("s1", "svg").expect("export send_with_result");

    server.shutdown();
}

// ── Composition validation: Rust library == IPC response ────────────

fn assert_f64_eq(name: &str, field: &str, got: f64, expected: f64, tol: f64) {
    let diff = (got - expected).abs();
    assert!(
        diff <= tol,
        "{name}.{field}: |{got} - {expected}| = {diff} > {tol}"
    );
}

#[test]
fn composition_flow_balanced_matches_library() {
    use ludospring_barracuda::interaction::flow::{evaluate_flow, flow_channel_metrics};
    use ludospring_barracuda::tolerances;

    let server = IpcTestServer::start();
    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let (challenge, skill) = (0.5, 0.5);
    let width = tolerances::FLOW_CHANNEL_WIDTH;
    let expected_state = evaluate_flow(challenge, skill, width);
    let (expected_score, expected_in_flow) = flow_channel_metrics(challenge, skill, width);

    let resp = send_rpc(
        &mut stream,
        "game.evaluate_flow",
        serde_json::json!({"challenge": challenge, "skill": skill}),
    );
    let result = resp.get("result").expect("result");

    assert_eq!(result["state"].as_str().unwrap(), expected_state.as_str());
    assert_f64_eq(
        "flow",
        "flow_score",
        result["flow_score"].as_f64().unwrap(),
        expected_score,
        tolerances::ANALYTICAL_TOL,
    );
    assert_eq!(result["in_flow"].as_bool().unwrap(), expected_in_flow);

    drop(stream);
    server.shutdown();
}

#[test]
fn composition_fitts_cost_matches_library() {
    use ludospring_barracuda::interaction::input_laws::{
        fitts_index_of_difficulty, fitts_movement_time,
    };
    use ludospring_barracuda::tolerances;

    let server = IpcTestServer::start();
    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let (distance, target_width) = (100.0, 10.0);
    let expected_mt = fitts_movement_time(
        distance,
        target_width,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
    );
    let expected_id = fitts_index_of_difficulty(distance, target_width);

    let resp = send_rpc(
        &mut stream,
        "game.fitts_cost",
        serde_json::json!({"distance": distance, "target_width": target_width}),
    );
    let result = resp.get("result").expect("result");

    assert_f64_eq(
        "fitts",
        "movement_time_ms",
        result["movement_time_ms"].as_f64().unwrap(),
        expected_mt,
        tolerances::ANALYTICAL_TOL,
    );
    assert_f64_eq(
        "fitts",
        "index_of_difficulty",
        result["index_of_difficulty"].as_f64().unwrap(),
        expected_id,
        tolerances::ANALYTICAL_TOL,
    );

    drop(stream);
    server.shutdown();
}

#[test]
fn composition_engagement_matches_library() {
    use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
    use ludospring_barracuda::tolerances;

    let server = IpcTestServer::start();
    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let snap = EngagementSnapshot {
        session_duration_s: 300.0,
        action_count: 100,
        exploration_breadth: 5,
        challenge_seeking: 3,
        retry_count: 10,
        deliberate_pauses: 5,
    };
    let expected = compute_engagement(&snap);

    let resp = send_rpc(
        &mut stream,
        "game.engagement",
        serde_json::json!({
            "session_duration_s": 300.0, "action_count": 100, "exploration_breadth": 5,
            "challenge_seeking": 3, "retry_count": 10, "deliberate_pauses": 5,
        }),
    );
    let result = resp.get("result").expect("result");

    assert_f64_eq(
        "engagement",
        "composite",
        result["composite"].as_f64().unwrap(),
        expected.composite,
        tolerances::ANALYTICAL_TOL,
    );
    assert_f64_eq(
        "engagement",
        "actions_per_minute",
        result["actions_per_minute"].as_f64().unwrap(),
        expected.actions_per_minute,
        tolerances::ANALYTICAL_TOL,
    );

    drop(stream);
    server.shutdown();
}

#[test]
fn composition_noise_matches_library() {
    use ludospring_barracuda::procedural::noise::fbm_2d;
    use ludospring_barracuda::tolerances;

    let server = IpcTestServer::start();
    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let expected = fbm_2d(1.23, 4.56, 4, 2.0, 0.5);

    let resp = send_rpc(
        &mut stream,
        "game.generate_noise",
        serde_json::json!({"x": 1.23, "y": 4.56}),
    );
    let result = resp.get("result").expect("result");

    assert_f64_eq(
        "noise",
        "value",
        result["value"].as_f64().unwrap(),
        expected,
        tolerances::ANALYTICAL_TOL,
    );

    drop(stream);
    server.shutdown();
}

#[test]
fn composition_dda_matches_library() {
    use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
    use ludospring_barracuda::tolerances;

    let server = IpcTestServer::start();
    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let outcomes = vec![1.0; 10];
    let target = tolerances::DDA_TARGET_SUCCESS_RATE;
    let mut window = PerformanceWindow::new(10);
    for &o in &outcomes {
        window.record(o);
    }
    let expected_adj = suggest_adjustment(&window, target);
    let expected_skill = window.estimated_skill();

    let resp = send_rpc(
        &mut stream,
        "game.difficulty_adjustment",
        serde_json::json!({"outcomes": outcomes}),
    );
    let result = resp.get("result").expect("result");

    assert_f64_eq(
        "dda",
        "adjustment",
        result["adjustment"].as_f64().unwrap(),
        expected_adj,
        tolerances::ANALYTICAL_TOL,
    );
    assert_f64_eq(
        "dda",
        "estimated_skill",
        result["estimated_skill"].as_f64().unwrap(),
        expected_skill,
        tolerances::ANALYTICAL_TOL,
    );

    drop(stream);
    server.shutdown();
}

#[test]
fn composition_accessibility_matches_library() {
    use ludospring_barracuda::interaction::accessibility::{
        VisualAccessibilityFeatures, score_visual_accessibility,
    };
    use ludospring_barracuda::tolerances;

    let server = IpcTestServer::start();
    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let features = VisualAccessibilityFeatures {
        audio_cues: true,
        descriptions: true,
        braille: true,
        haptic: true,
        color_independent: true,
        scalable_text: true,
    };
    let expected = score_visual_accessibility(&features);

    let resp = send_rpc(
        &mut stream,
        "game.accessibility",
        serde_json::json!({
            "audio_cues": true, "descriptions": true, "braille": true,
            "haptic": true, "color_independent": true, "scalable_text": true
        }),
    );
    let result = resp.get("result").expect("result");

    assert_f64_eq(
        "accessibility",
        "score",
        result["score"].as_f64().unwrap(),
        expected.score,
        tolerances::ANALYTICAL_TOL,
    );

    drop(stream);
    server.shutdown();
}

#[test]
fn composition_wfc_matches_library() {
    use ludospring_barracuda::procedural::wfc::{AdjacencyRules, WfcGrid};

    let server = IpcTestServer::start();
    let mut stream = UnixStream::connect(&server.socket_path).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let (w, h, n) = (4, 4, 3);
    let rules = AdjacencyRules::unconstrained(n);
    let mut grid = WfcGrid::new(w, h, n);
    grid.collapse(1, 1, 0);
    let expected_removed = grid.propagate(&rules);

    let resp = send_rpc(
        &mut stream,
        "game.wfc_step",
        serde_json::json!({
            "width": w, "height": h, "n_tiles": n, "collapse": [1, 1, 0]
        }),
    );
    let result = resp.get("result").expect("result");

    assert_eq!(
        usize::try_from(result["options_removed"].as_u64().unwrap()).unwrap(),
        expected_removed
    );

    drop(stream);
    server.shutdown();
}
