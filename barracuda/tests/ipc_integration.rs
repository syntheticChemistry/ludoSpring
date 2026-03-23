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

mod ipc_test_util;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use ipc_test_util::IpcTestServer;

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
    let mt = result.get("movement_time_ms").and_then(|v| v.as_f64());
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

    let total = result.get("total_capabilities").and_then(|v| v.as_u64());
    assert!(
        total.is_some_and(|n| n > 0),
        "should have total_capabilities > 0"
    );

    let domains = result.get("domains").and_then(|v| v.as_array());
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

    let reg = bridge
        .register(std::path::Path::new("/tmp/ludospring-register.sock"))
        .expect("lifecycle.register");
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
