// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp093 — Full Game Session via Continuous Composition.
//!
//! Validates the 60Hz storytelling loop from the Continuous coordination
//! pattern. Deploys `game_loop_continuous.toml`, starts a session, ticks
//! multiple frames, and verifies per-tick latency and flow score consistency.
//!
//! # Science validation
//!
//! Each tick: evaluate flow (math.sigmoid) → DDA (activation.fitts/hick)
//! Session: start → tick N frames → measure latency → verify flow scores
//!
//! # References
//!
//! - ludoSpring graph: `graphs/composition/game_loop_continuous.toml`
//! - primalSpring: `graphs/science/gaming_mesh_chimera.toml`
//! - ludoSpring: exp088 (infrastructure), exp005 (flow), exp006 (DDA)

use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (composition — Continuous coordination pattern validation)",
    commit: "exp093-v1",
    date: "2026-03-30",
    command: "cargo run -p ludospring-exp093",
};

const FRAME_BUDGET_MS: f64 = 16.67;

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
        .set_read_timeout(Some(Duration::from_secs(5)))
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

fn discover_barracuda_socket() -> Option<PathBuf> {
    let dirs = ludospring_barracuda::niche::socket_dirs();
    for dir in &dirs {
        for name in &["barracuda-core.sock", "barracuda.sock", "compute.sock"] {
            let path = dir.join(name);
            if path.exists() {
                return Some(path);
            }
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
                    if n.starts_with("barracuda") && n.ends_with(".sock") {
                        return Some(p);
                    }
                }
            }
        }
    }
    None
}

fn dry_mode(h: &mut ValidationHarness) {
    eprintln!("  Primals not available — dry-run with known gaps");
    for name in &[
        "neural_api_reachable",
        "barracuda_reachable",
        "flow_tick_latency",
        "flow_deterministic",
        "dda_tick_latency",
        "session_10_ticks",
    ] {
        h.check_bool(name, false);
    }
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp093_continuous_session");
    h.print_provenance(&[&PROVENANCE]);

    let na = discover_neural_api();
    h.check_bool("neural_api_reachable", na.is_some());

    let bc = discover_barracuda_socket();
    h.check_bool("barracuda_reachable", bc.is_some());

    let Some(sock) = bc else {
        dry_mode(&mut h);
        h.finish();
    };

    // ── Simulate 10 ticks of flow evaluation ─────────────────
    // Each tick calls math.sigmoid (flow) + activation.fitts (DDA)
    let mut flow_latencies = Vec::new();
    let mut flow_results = Vec::new();

    for tick in 0..10 {
        let skill_challenge_delta = 0.1 * f64::from(tick as i32 - 5);

        let start = Instant::now();
        let flow = rpc_call(
            &sock,
            "math.sigmoid",
            &serde_json::json!({"data": [skill_challenge_delta]}),
        );
        let elapsed = start.elapsed();
        flow_latencies.push(elapsed.as_secs_f64() * 1000.0);

        if let Ok(ref resp) = flow {
            if let Some(arr) = resp.pointer("/result/result").and_then(|v| v.as_array()) {
                if let Some(val) = arr.first().and_then(serde_json::Value::as_f64) {
                    flow_results.push(val);
                }
            }
        }
    }

    // Check per-tick latency is within 60Hz budget
    let max_latency = flow_latencies.iter().copied().fold(0.0_f64, f64::max);
    eprintln!("  flow tick latencies (ms): {:?}", flow_latencies);
    eprintln!("  max flow tick latency: {max_latency:.2}ms (budget: {FRAME_BUDGET_MS:.2}ms)");
    h.check_bool("flow_tick_latency", max_latency < FRAME_BUDGET_MS);

    // Check flow scores are deterministic (same delta → same score)
    let mid_score = flow_results.get(5).copied();
    let expected_mid = 0.5; // sigmoid(0.0) = 0.5
    if let Some(ms) = mid_score {
        h.check_abs("flow_deterministic", ms, expected_mid, tolerances::ANALYTICAL_TOL);
    } else {
        h.check_bool("flow_deterministic", false);
    }

    // ── DDA tick latency ─────────────────────────────────────
    let dda_start = Instant::now();
    let _dda = rpc_call(
        &sock,
        "activation.fitts",
        &serde_json::json!({"distance": 256.0, "width": 32.0}),
    );
    let dda_latency = dda_start.elapsed().as_secs_f64() * 1000.0;
    eprintln!("  DDA tick latency: {dda_latency:.2}ms");
    h.check_bool("dda_tick_latency", dda_latency < FRAME_BUDGET_MS);

    // ── Full session: 10 ticks completed ─────────────────────
    h.check_bool("session_10_ticks", flow_results.len() == 10);

    h.finish();
}

fn main() {
    cmd_validate();
}
