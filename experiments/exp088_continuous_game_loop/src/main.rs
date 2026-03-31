// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp088 — Continuous Game Loop via Primal Composition.
//!
//! The storytelling loop at 60Hz: evaluate flow → DDA recommend →
//! AI narrate → render scene → provenance stamp.
//!
//! V36: Infrastructure validation for Continuous coordination pattern.
//! Science experiment exp093 (full game session) builds on this.
//!
//! Reference: primalSpring `graphs/science/gaming_mesh_chimera.toml`,
//!            ludoSpring `graphs/composition/game_loop_continuous.toml`
//!
//! # biomeOS v2.80 status
//!
//! `graph.start_continuous` IS implemented in biomeOS:
//!   - Session lifecycle works: start/pause/resume/stop are wired
//!   - Sessions are tracked with IDs and can be queried via graph.status
//!   - Node execution callback routes through NeuralRouter in v2.80
//!   - Needs <16ms per capability.call hop for 60Hz budget
//!
//! This means: session management SHOULD pass, but actual per-tick
//! primal routing will not produce real science results until biomeOS
//! wires the continuous executor callback to `capability.call`.
//!
//! # Composition graph
//!
//! `graphs/composition/game_loop_continuous.toml`:
//!   coordination = "Continuous", tick_hz = 60
//!   6 nodes: compute → compute → ai → visualization → dag → security
//!
//! Modeled after primalSpring `gen4_storytelling_full.toml` (12 nodes)
//! but decomposed to pure capability routing (no esotericWebb/ludoSpring
//! processes in the loop).
//!
//! # Gap summary
//!
//! - biomeOS continuous executor stub (session works, routing doesn't)
//! - barraCuda math.flow.evaluate / math.dda.recommend not on IPC
//! - IPC round-trip latency vs 16ms frame budget unknown
//!
//! # Reference
//!
//! primalSpring gen4_storytelling_full.toml: the 12-node aspirational graph
//! biomeOS handlers/graph.rs: ContinuousExecutor spawn + stub callback

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (composition — continuous game loop orchestration)",
    commit: "exp088-v1",
    date: "2026-03-29",
    command: "cargo run -p ludospring-exp088",
};

fn rpc_call(
    socket: &Path,
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let stream = UnixStream::connect(socket)
        .map_err(|e| format!("connect {}: {e}", socket.display()))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| format!("timeout: {e}"))?;
    let mut writer = stream.try_clone().map_err(|e| format!("clone: {e}"))?;
    let mut payload = serde_json::to_string(&request).map_err(|e| format!("ser: {e}"))?;
    payload.push('\n');
    writer
        .write_all(payload.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    writer.flush().map_err(|e| format!("flush: {e}"))?;
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    reader
        .read_line(&mut buf)
        .map_err(|e| format!("read: {e}"))?;
    serde_json::from_str(&buf).map_err(|e| format!("parse: {e}"))
}

fn has_result(resp: &serde_json::Value) -> bool {
    resp.get("result").is_some() && resp.get("error").is_none()
}

fn has_error_code(resp: &serde_json::Value, code: i64) -> bool {
    resp.pointer("/error/code")
        .and_then(serde_json::Value::as_i64)
        == Some(code)
}

const METHOD_NOT_FOUND: i64 = -32601;

fn discover_neural_api() -> Option<PathBuf> {
    let dirs = ludospring_barracuda::niche::socket_dirs();
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("neural-api") && name.ends_with(".sock") {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

fn probe_primal_via_capability(
    h: &mut ValidationHarness,
    na: &Path,
    domain: &str,
    operation: &str,
    params: serde_json::Value,
    check_name: &str,
) {
    let resp = rpc_call(
        na,
        "capability.call",
        &serde_json::json!({
            "capability": domain,
            "operation": operation,
            "params": params
        }),
    );

    match resp {
        Ok(ref r) if has_result(r) => {
            eprintln!("  ROUTED: {domain}.{operation} → result");
            h.check_bool(check_name, true);
        }
        Ok(ref r) if has_error_code(r, METHOD_NOT_FOUND) => {
            eprintln!("  GAP: {domain}.{operation} → method not found");
            eprintln!("    Primal '{domain}' may not expose '{operation}' over IPC");
            h.check_bool(check_name, false);
        }
        Ok(ref r) => {
            let msg = r
                .pointer("/error/message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            eprintln!("  ERROR: {domain}.{operation} → {msg}");
            h.check_bool(check_name, false);
        }
        Err(e) => {
            eprintln!("  CONN: {domain}.{operation} → {e}");
            h.check_bool(check_name, false);
        }
    }
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp088_continuous_game_loop");
    h.print_provenance(&[&PROVENANCE]);

    let neural_socket = discover_neural_api();
    h.check_bool("neural_api_discovered", neural_socket.is_some());

    let Some(na) = neural_socket else {
        eprintln!("  Neural API not running — documenting composition gaps");
        dry_mode(&mut h);
        h.finish();
    };

    eprintln!("  Neural API socket: {}", na.display());

    // ── Phase 0: Deploy our continuous graph ──────────────────
    deploy_composition_graph(&na);

    // ── Phase 1: Probe each primal needed in the game loop ────
    // Each check validates that the Neural API can route to the
    // primal domain needed for one stage of the 60Hz loop.

    probe_primal_via_capability(
        &mut h,
        &na,
        "tensor",
        "tensor.create",
        serde_json::json!({"shape": [2], "data": [0.5, 0.5]}),
        "route_tensor_create",
    );

    probe_primal_via_capability(
        &mut h,
        &na,
        "ai",
        "query",
        serde_json::json!({"messages": [{"role": "user", "content": "test"}]}),
        "route_ai_narration",
    );

    probe_primal_via_capability(
        &mut h,
        &na,
        "visualization",
        "render.scene",
        serde_json::json!({"scene": {"type": "test"}}),
        "route_visualization_render",
    );

    probe_primal_via_capability(
        &mut h,
        &na,
        "dag",
        "session.create",
        serde_json::json!({"agent": "exp088", "metadata": {}}),
        "route_dag_provenance",
    );

    probe_primal_via_capability(
        &mut h,
        &na,
        "crypto",
        "blake3_hash",
        serde_json::json!({"data": "dGVzdA=="}),
        "route_crypto_integrity",
    );

    // ── Phase 2: Continuous graph submission ───────────────────
    let cont = rpc_call(
        &na,
        "graph.start_continuous",
        &serde_json::json!({"graph_id": "ludospring_game_loop_continuous"}),
    );
    let cont_ok = cont.as_ref().is_ok_and(has_result);
    h.check_bool("continuous_graph_start", cont_ok);

    if cont_ok {
        eprintln!("  Continuous graph started — checking session control");

        if let Ok(ref resp) = cont {
            if let Some(sid) = resp
                .pointer("/result/session_id")
                .and_then(serde_json::Value::as_str)
            {
                eprintln!("  Session ID: {sid}");

                std::thread::sleep(Duration::from_millis(200));

                let status = rpc_call(
                    &na,
                    "graph.status",
                    &serde_json::json!({"session_id": sid}),
                );
                h.check_bool(
                    "continuous_graph_status",
                    status.as_ref().is_ok_and(has_result),
                );

                let stop = rpc_call(
                    &na,
                    "graph.stop_continuous",
                    &serde_json::json!({"session_id": sid}),
                );
                h.check_bool(
                    "continuous_graph_stop",
                    stop.as_ref().is_ok_and(has_result),
                );
            } else {
                h.check_bool("continuous_graph_status", false);
                h.check_bool("continuous_graph_stop", false);
            }
        }
    } else {
        if cont
            .as_ref()
            .is_ok_and(|r| has_error_code(r, METHOD_NOT_FOUND))
        {
            eprintln!("  GAP: graph.start_continuous not exposed (GAP-018)");
            eprintln!("    60Hz game loop composition requires Continuous executor");
            eprintln!("    This is the primary biomeOS evolution target for gaming");
        } else {
            eprintln!("  Continuous graph submission failed (graph may not be deployed)");
        }
        h.check_bool("continuous_graph_status", false);
        h.check_bool("continuous_graph_stop", false);
    }

    // ── Phase 3: Latency budget estimate ──────────────────────
    let start = std::time::Instant::now();
    let _ = rpc_call(
        &na,
        "capability.call",
        &serde_json::json!({
            "capability": "math",
            "operation": "math.sigmoid",
            "params": {"data": [0.5]}
        }),
    );
    let latency = start.elapsed();
    let within_budget = latency.as_millis() < 16;
    eprintln!(
        "  Single capability.call latency: {}ms (budget: <16ms for 60Hz)",
        latency.as_millis()
    );
    h.check_bool("single_hop_within_16ms", within_budget);
    if !within_budget {
        eprintln!(
            "  GAP: IPC round-trip {}ms exceeds 16ms frame budget",
            latency.as_millis()
        );
        eprintln!("    Multi-node pipeline at 60Hz requires <2-3ms per hop");
    }

    eprintln!();
    eprintln!("  ══════════════════════════════════════════════════════");
    eprintln!("  GAME LOOP COMPOSITION SUMMARY:");
    eprintln!("  Continuous coordination = the 60Hz storytelling loop");
    eprintln!("  Each stage routed through Neural API capability.call");
    eprintln!("  Gaps drive biomeOS executor + barraCuda math evolution");
    eprintln!("  ══════════════════════════════════════════════════════");

    h.finish();
}

fn deploy_composition_graph(na: &Path) {
    let graph_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|root| root.join("graphs/composition"));

    let Some(ref dir) = graph_dir else { return };
    let path = dir.join("game_loop_continuous.toml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        eprintln!("  WARN: cannot read game_loop_continuous.toml — graph not deployed");
        return;
    };
    let resp = rpc_call(
        na,
        "graph.save",
        &serde_json::json!({"toml": content}),
    );
    match resp {
        Ok(ref r) if has_result(r) => {
            eprintln!("  Deployed graph: game_loop_continuous.toml");
        }
        Ok(ref r) => {
            let msg = r.pointer("/error/message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            eprintln!("  WARN: graph.save → {msg}");
        }
        Err(e) => {
            eprintln!("  WARN: graph.save → {e}");
        }
    }
}

fn dry_mode(h: &mut ValidationHarness) {
    for name in [
        "route_tensor_create",
        "route_ai_narration",
        "route_visualization_render",
        "route_dag_provenance",
        "route_crypto_integrity",
        "continuous_graph_start",
        "continuous_graph_status",
        "continuous_graph_stop",
        "single_hop_within_16ms",
    ] {
        eprintln!("  DRY: {name} — Neural API not running");
        h.check_bool(name, false);
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
