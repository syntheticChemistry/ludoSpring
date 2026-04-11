// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp090 — Flow + Engagement + DDA via Tensor Composition.
//!
//! Validates the three game-feel models by composing barraCuda tensor ops
//! into the same metrics ludoSpring computes locally. Compares results
//! against Python baselines.
//!
//! # Science validation
//!
//! Flow: tensor.create([skill, challenge]) → tensor.sigmoid → tensor.reduce
//! Engagement: tensor.create(5 metrics) → tensor.scale(weights) → tensor.reduce
//! DDA: math.sigmoid(performance_delta) → difficulty recommendation
//!
//! # References
//!
//! - ludoSpring: exp005 (Flow), exp010 (Engagement), exp006 (DDA)
//! - primalSpring: `graphs/compositions/game_science_standalone.toml`
//! - Python baselines: `baselines/python/`, commit 4b683e3e

use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/exp005_flow.py + exp010_engagement.py + exp006_dda.py",
    commit: "4b683e3e",
    date: "2026-03-30",
    command: "cargo run -p ludospring-exp090",
};

// Python baseline: sigmoid(0.0) = 0.5 (flow at skill=challenge boundary)
const FLOW_BOUNDARY_EXPECTED: f64 = 0.5;
// Python baseline: sigmoid(0.5) = 0.6225 (flow with slight challenge advantage)
const FLOW_SLIGHT_CHALLENGE: f64 = 0.622_459_331_201_854_6;
// Engagement composite: weighted sum of 5 normalized metrics
// weights = [0.25, 0.20, 0.20, 0.20, 0.15], scores = [0.8, 0.6, 0.7, 0.5, 0.9]
const ENGAGEMENT_EXPECTED: f64 = 0.25 * 0.8 + 0.20 * 0.6 + 0.20 * 0.7 + 0.20 * 0.5 + 0.15 * 0.9;
// DDA: sigmoid(-1.0) for "too hard" → lower difficulty
const DDA_TOO_HARD_EXPECTED: f64 = 0.268_941_421_369_995_1;
// DDA: sigmoid(1.0) for "too easy" → raise difficulty
const DDA_TOO_EASY_EXPECTED: f64 = 0.731_058_578_630_005;

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
    let stream =
        UnixStream::connect(socket).map_err(|e| format!("connect {}: {e}", socket.display()))?;
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

fn has_result(resp: &serde_json::Value) -> bool {
    resp.get("result").is_some() && resp.get("error").is_none()
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
                    if n.starts_with("barracuda")
                        && Path::new(n)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
                    {
                        return Some(p);
                    }
                }
            }
        }
    }
    None
}

fn dry_mode(h: &mut ValidationHarness) {
    eprintln!("  barraCuda not available — dry-run with known gaps");
    for name in &[
        "flow_boundary_sigmoid",
        "flow_slight_challenge",
        "flow_tolerance",
        "engagement_weighted_mean",
        "engagement_tolerance",
        "dda_too_hard",
        "dda_too_easy",
        "dda_symmetric",
        "tensor_create_scores",
        "tensor_reduce_sum",
    ] {
        h.check_bool(name, false);
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "flow, engagement, DDA, and tensor IPC validation"
)]
fn cmd_validate() {
    let mut h = ValidationHarness::new("exp090_gameflow_tensor_composition");
    h.print_provenance(&[&PROVENANCE]);

    let Some(sock) = discover_barracuda_socket() else {
        dry_mode(&mut h);
        h.finish();
    };

    eprintln!("  barraCuda socket: {}", sock.display());

    // ── Flow: sigmoid composition ────────────────────────────
    let sig_boundary = rpc_call(&sock, "math.sigmoid", &serde_json::json!({"data": [0.0]}));
    h.check_bool(
        "flow_boundary_sigmoid",
        sig_boundary.as_ref().is_ok_and(has_result),
    );

    let sig_challenge = rpc_call(&sock, "math.sigmoid", &serde_json::json!({"data": [0.5]}));
    h.check_bool(
        "flow_slight_challenge",
        sig_challenge.as_ref().is_ok_and(has_result),
    );

    if let Ok(ref resp) = sig_challenge {
        if let Some(arr) = resp.pointer("/result/result").and_then(|v| v.as_array()) {
            if let Some(val) = arr.first().and_then(serde_json::Value::as_f64) {
                h.check_abs(
                    "flow_challenge_tolerance",
                    val,
                    FLOW_SLIGHT_CHALLENGE,
                    tolerances::ANALYTICAL_TOL,
                );
            }
        }
    }

    if let Ok(ref resp) = sig_boundary {
        if let Some(arr) = resp.pointer("/result/result").and_then(|v| v.as_array()) {
            if let Some(val) = arr.first().and_then(serde_json::Value::as_f64) {
                h.check_abs(
                    "flow_tolerance",
                    val,
                    FLOW_BOUNDARY_EXPECTED,
                    tolerances::ANALYTICAL_TOL,
                );
            }
        }
    }

    // ── Engagement: weighted mean via stats IPC ──────────────
    let scores = [0.8, 0.6, 0.7, 0.5, 0.9];
    let weights = [0.25, 0.20, 0.20, 0.20, 0.15];
    let wm = rpc_call(
        &sock,
        "stats.weighted_mean",
        &serde_json::json!({"values": scores, "weights": weights}),
    );
    let wm_ok = wm.as_ref().is_ok_and(has_result);
    h.check_bool("engagement_weighted_mean", wm_ok);

    if wm_ok {
        if let Some(val) = wm
            .as_ref()
            .ok()
            .and_then(|r| r.pointer("/result/result"))
            .and_then(serde_json::Value::as_f64)
        {
            h.check_abs(
                "engagement_tolerance",
                val,
                ENGAGEMENT_EXPECTED,
                tolerances::ANALYTICAL_TOL,
            );
        }
    }

    // ── DDA: sigmoid for performance-to-difficulty mapping ───
    let dda_hard = rpc_call(&sock, "math.sigmoid", &serde_json::json!({"data": [-1.0]}));
    h.check_bool("dda_too_hard", dda_hard.as_ref().is_ok_and(has_result));

    let dda_easy = rpc_call(&sock, "math.sigmoid", &serde_json::json!({"data": [1.0]}));
    h.check_bool("dda_too_easy", dda_easy.as_ref().is_ok_and(has_result));

    // Verify DDA values match expected and symmetry holds
    let hard_val = dda_hard
        .as_ref()
        .ok()
        .and_then(|r| r.pointer("/result/result"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(serde_json::Value::as_f64);
    let easy_val = dda_easy
        .as_ref()
        .ok()
        .and_then(|r| r.pointer("/result/result"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(serde_json::Value::as_f64);
    if let Some(hv) = hard_val {
        h.check_abs(
            "dda_hard_tolerance",
            hv,
            DDA_TOO_HARD_EXPECTED,
            tolerances::ANALYTICAL_TOL,
        );
    }
    if let Some(ev) = easy_val {
        h.check_abs(
            "dda_easy_tolerance",
            ev,
            DDA_TOO_EASY_EXPECTED,
            tolerances::ANALYTICAL_TOL,
        );
    }
    if let (Some(hv), Some(ev)) = (hard_val, easy_val) {
        h.check_abs("dda_symmetric", hv + ev, 1.0, tolerances::ANALYTICAL_TOL);
    }

    // ── Tensor path: create + reduce for composite scoring ──
    let tc = rpc_call(
        &sock,
        "tensor.create",
        &serde_json::json!({"shape": [5], "data": [0.8_f32, 0.6, 0.7, 0.5, 0.9]}),
    );
    h.check_bool("tensor_create_scores", tc.as_ref().is_ok_and(has_result));

    if let Some(tid) = tc
        .as_ref()
        .ok()
        .and_then(|r| r.pointer("/result/tensor_id"))
        .and_then(serde_json::Value::as_str)
    {
        let reduce = rpc_call(
            &sock,
            "tensor.reduce",
            &serde_json::json!({"tensor_id": tid, "op": "sum"}),
        );
        h.check_bool(
            "tensor_reduce_sum",
            reduce.as_ref().is_ok_and(|r| {
                r.pointer("/result/result")
                    .and_then(serde_json::Value::as_f64)
                    .is_some_and(|v| (v - 3.5).abs() < 0.01)
            }),
        );
    }

    h.finish();
}

fn main() {
    cmd_validate();
}
