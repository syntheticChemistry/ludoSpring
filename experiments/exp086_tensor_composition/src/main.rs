// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp086 — Tensor API Composition Stress Test.
//!
//! Pushes barraCuda's tensor IPC surface (`tensor.create`, `tensor.matmul`)
//! to discover what composite science is expressible today. We attempt to
//! build engagement-like and flow-like computations from tensor primitives.
//!
//! # What we test
//!
//! 1. `tensor.create` with real game-science data (engagement weights, scores)
//! 2. `tensor.matmul` to compute weighted sums (engagement composite)
//! 3. Probe `tensor.add`, `tensor.scale`, `tensor.clamp` (likely missing)
//! 4. Attempt to chain: create → matmul → read — the full IPC round-trip
//!
//! # Gap discovery
//!
//! Engagement composite needs: create, scale, clamp, weighted sum.
//! barraCuda tensor API has: create, matmul.
//! Gap: no element-wise ops, no clamp, no reduce, no activation over IPC.
//!
//! # Provenance
//!
//! Expected engagement values from exp010 (engagement curves) and
//! `barracuda/tests/python_parity.rs` (engagement composite validation).

use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (composition — tensor API stress test)",
    commit: "exp086-v1",
    date: "2026-03-29",
    command: "cargo run -p ludospring-exp086",
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

fn extract_tensor_id(resp: &serde_json::Value) -> Option<String> {
    resp.pointer("/result/tensor_id")
        .or_else(|| resp.pointer("/result/id"))
        .and_then(serde_json::Value::as_str)
        .map(String::from)
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
    }
    None
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp086_tensor_composition");
    h.print_provenance(&[&PROVENANCE]);

    let socket = discover_barracuda_socket();
    h.check_bool("barracuda_socket_discovered", socket.is_some());

    let Some(sock) = socket else {
        eprintln!("  barraCuda not running — skipping tensor composition checks");
        for name in [
            "create_weights_vector",
            "create_scores_vector",
            "matmul_weighted_sum",
            "round_trip_create_read",
            "gap_tensor_add",
            "gap_tensor_scale",
            "gap_tensor_clamp",
            "gap_tensor_reduce_sum",
            "gap_tensor_sigmoid",
        ] {
            h.check_bool(name, false);
        }
        h.finish();
    };

    eprintln!("  barraCuda socket: {}", sock.display());

    // ── Engagement composite via tensor ops ────────────────────
    // Engagement = 0.2*apm + 0.2*exploration + 0.2*variety + 0.2*completion + 0.2*social
    // Using matmul: weights (1x5) × scores (5x1) = composite (1x1)
    let weights = rpc_call(
        &sock,
        "tensor.create",
        &serde_json::json!({
            "shape": [1, 5],
            "data": [
                tolerances::ENGAGEMENT_WEIGHT,
                tolerances::ENGAGEMENT_WEIGHT,
                tolerances::ENGAGEMENT_WEIGHT,
                tolerances::ENGAGEMENT_WEIGHT,
                tolerances::ENGAGEMENT_WEIGHT
            ]
        }),
    );
    let weights_ok = weights.as_ref().is_ok_and(has_result);
    h.check_bool("create_weights_vector", weights_ok);

    let scores = rpc_call(
        &sock,
        "tensor.create",
        &serde_json::json!({
            "shape": [5, 1],
            "data": [0.5, 0.4, 0.6, 0.4, 0.3]
        }),
    );
    let scores_ok = scores.as_ref().is_ok_and(has_result);
    h.check_bool("create_scores_vector", scores_ok);

    if weights_ok && scores_ok {
        let w_id = weights.as_ref().ok().and_then(extract_tensor_id);
        let s_id = scores.as_ref().ok().and_then(extract_tensor_id);

        if let (Some(wid), Some(sid)) = (w_id, s_id) {
            let matmul = rpc_call(
                &sock,
                "tensor.matmul",
                &serde_json::json!({"lhs_id": wid, "rhs_id": sid}),
            );
            let matmul_ok = matmul.as_ref().is_ok_and(has_result);
            h.check_bool("matmul_weighted_sum", matmul_ok);

            if matmul_ok {
                let expected = 0.2 * (0.5 + 0.4 + 0.6 + 0.4 + 0.3);
                eprintln!("  Engagement composite expected: {expected}");
                if let Ok(ref resp) = matmul {
                    eprintln!("  matmul response: {resp}");
                }
            }
        } else {
            eprintln!("  GAP: tensor.create returns no tensor_id — cannot chain ops");
            h.check_bool("matmul_weighted_sum", false);
        }
    } else {
        h.check_bool("matmul_weighted_sum", false);
    }

    // ── Round-trip: create → compute.dispatch read ────────────
    let rt = rpc_call(
        &sock,
        "tensor.create",
        &serde_json::json!({"shape": [3], "data": [1.0, 2.0, 3.0]}),
    );
    let rt_ok = rt.as_ref().is_ok_and(has_result);
    if rt_ok {
        if let Some(tid) = rt.as_ref().ok().and_then(extract_tensor_id) {
            let read = rpc_call(
                &sock,
                "compute.dispatch",
                &serde_json::json!({"op": "read", "tensor_id": tid}),
            );
            let read_ok = read.as_ref().is_ok_and(has_result);
            h.check_bool("round_trip_create_read", read_ok);
            if !read_ok {
                eprintln!("  GAP: compute.dispatch(read) failed for tensor {tid}");
            }
        } else {
            h.check_bool("round_trip_create_read", false);
        }
    } else {
        h.check_bool("round_trip_create_read", false);
    }

    // ── Probe missing tensor ops ──────────────────────────────
    let missing_ops: &[(&str, &str, serde_json::Value)] = &[
        (
            "tensor.add",
            "gap_tensor_add",
            serde_json::json!({"a": "t0", "b": "t1"}),
        ),
        (
            "tensor.scale",
            "gap_tensor_scale",
            serde_json::json!({"tensor_id": "t0", "scalar": 2.0}),
        ),
        (
            "tensor.clamp",
            "gap_tensor_clamp",
            serde_json::json!({"tensor_id": "t0", "min": 0.0, "max": 1.0}),
        ),
        (
            "tensor.reduce_sum",
            "gap_tensor_reduce_sum",
            serde_json::json!({"tensor_id": "t0"}),
        ),
        (
            "tensor.sigmoid",
            "gap_tensor_sigmoid",
            serde_json::json!({"tensor_id": "t0"}),
        ),
    ];

    for (method, check, params) in missing_ops {
        let resp = rpc_call(&sock, method, params);
        let found = resp.as_ref().is_ok_and(has_result);
        if found {
            eprintln!("  AVAILABLE: {method}");
        } else {
            eprintln!("  GAP: {method} — not available over IPC");
            eprintln!("    Needed for composition-tier engagement/flow computation");
        }
        h.check_bool(check, found);
    }

    eprintln!();
    eprintln!("  ══════════════════════════════════════════════════════");
    eprintln!("  TENSOR SUMMARY: engagement composite needs create +");
    eprintln!("  scale + clamp + weighted_sum. Only create + matmul");
    eprintln!("  exist today. Each gap = barraCuda evolution pressure.");
    eprintln!("  ══════════════════════════════════════════════════════");

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
