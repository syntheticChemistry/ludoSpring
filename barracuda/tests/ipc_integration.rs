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

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

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

fn start_server() -> (
    std::path::PathBuf,
    Arc<AtomicBool>,
    std::thread::JoinHandle<()>,
) {
    use std::sync::atomic::AtomicU64;
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("ludospring_ipc_test_{}_{id}", std::process::id()));
    std::fs::create_dir_all(&dir).ok();
    let sock = dir.join("test.sock");

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);
    let sock_clone = sock.clone();

    let handle = std::thread::spawn(move || {
        let server = ludospring_barracuda::ipc::IpcServer::with_path(&sock_clone);
        let _ = server.run_until(&shutdown_clone);
    });

    for _ in 0..100 {
        if sock.exists() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(sock.exists(), "server did not start in time");

    (sock, shutdown, handle)
}

fn cleanup(sock: &std::path::Path, shutdown: &AtomicBool, handle: std::thread::JoinHandle<()>) {
    shutdown.store(true, Ordering::Relaxed);
    std::fs::remove_file(sock).ok();
    std::fs::remove_dir(sock.parent().unwrap_or(sock)).ok();
    let _ = handle.join();
}

#[test]
fn lifecycle_status_returns_name_and_capabilities() {
    let (sock, shutdown, handle) = start_server();

    let mut stream = UnixStream::connect(&sock).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(&mut stream, "lifecycle.status", serde_json::json!({}));
    let result = resp.get("result").expect("result field");

    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some("ludospring")
    );
    let caps = result.get("capabilities").and_then(|v| v.as_array());
    assert!(caps.is_some(), "should have capabilities array");
    assert!(caps.unwrap().len() >= 20, "should have 20+ capabilities");

    drop(stream);
    cleanup(&sock, &shutdown, handle);
}

#[test]
fn evaluate_flow_returns_structured_result() {
    let (sock, shutdown, handle) = start_server();

    let mut stream = UnixStream::connect(&sock).expect("connect");
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
    cleanup(&sock, &shutdown, handle);
}

#[test]
fn fitts_cost_returns_movement_time() {
    let (sock, shutdown, handle) = start_server();

    let mut stream = UnixStream::connect(&sock).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(
        &mut stream,
        "game.fitts_cost",
        serde_json::json!({"distance": 100.0, "target_width": 10.0}),
    );
    let result = resp.get("result").expect("result field");
    let mt = result.get("movement_time_ms").and_then(|v| v.as_f64());
    assert!(mt.is_some(), "should return movement_time_ms");
    assert!(mt.unwrap() > 0.0, "movement time should be positive");

    drop(stream);
    cleanup(&sock, &shutdown, handle);
}

#[test]
fn capability_list_returns_all_capabilities() {
    let (sock, shutdown, handle) = start_server();

    let mut stream = UnixStream::connect(&sock).expect("connect");
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
    cleanup(&sock, &shutdown, handle);
}

#[test]
fn unknown_method_returns_error() {
    let (sock, shutdown, handle) = start_server();

    let mut stream = UnixStream::connect(&sock).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(&mut stream, "nonexistent.method", serde_json::json!({}));
    let error = resp.get("error");
    assert!(error.is_some(), "unknown method should return error");

    drop(stream);
    cleanup(&sock, &shutdown, handle);
}

#[test]
fn health_check_responds() {
    let (sock, shutdown, handle) = start_server();

    let mut stream = UnixStream::connect(&sock).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let resp = send_rpc(&mut stream, "health.check", serde_json::json!({}));
    assert!(
        resp.get("result").is_some(),
        "health check should return result"
    );

    drop(stream);
    cleanup(&sock, &shutdown, handle);
}
