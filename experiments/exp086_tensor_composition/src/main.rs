// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp086 — Tensor API Composition Stress Test.
//!
//! Validates barraCuda's tensor IPC surface with REAL tensor IDs from prior
//! creates. Tests the full chain: create → element-wise ops → reduce → read.
//!
//! # barraCuda tensor param schemas (v0.3.11)
//!
//! - `tensor.create`: `{"shape": [...], "data": [...]}`
//! - `tensor.matmul`: `{"lhs_id": str, "rhs_id": str}`
//! - `tensor.add`: `{"tensor_id": str, "scalar": f64}` or `{"tensor_id": str, "other_id": str}`
//! - `tensor.scale`: `{"tensor_id": str, "scalar": f64}`
//! - `tensor.clamp`: `{"tensor_id": str, "min": f64, "max": f64}`
//! - `tensor.reduce`: `{"tensor_id": str, "op": "sum"|"mean"|"max"|"min"}`
//! - `tensor.sigmoid`: `{"tensor_id": str}`
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
    commit: "exp086-v2",
    date: "2026-03-30",
    command: "cargo run -p ludospring-exp086",
};

const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

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

fn error_code(resp: &serde_json::Value) -> Option<i64> {
    resp.pointer("/error/code").and_then(serde_json::Value::as_i64)
}

fn error_message(resp: &serde_json::Value) -> String {
    resp.pointer("/error/message")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn extract_tensor_id(resp: &serde_json::Value) -> Option<String> {
    resp.pointer("/result/tensor_id")
        .or_else(|| resp.pointer("/result/id"))
        .or_else(|| resp.pointer("/result/result_id"))
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

fn report_error(method: &str, resp: &serde_json::Value) {
    match error_code(resp) {
        Some(c) if c == METHOD_NOT_FOUND => {
            eprintln!("  FAIL: {method} → -32601 method_not_found");
        }
        Some(c) if c == INVALID_PARAMS => {
            eprintln!("  FAIL: {method} → -32602 invalid_params: {}", error_message(resp));
        }
        _ => {
            eprintln!("  FAIL: {method} → {}", error_message(resp));
        }
    }
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
            "tensor_add_scalar",
            "tensor_scale",
            "tensor_clamp",
            "tensor_reduce_sum",
            "tensor_sigmoid",
        ] {
            h.check_bool(name, false);
        }
        h.finish();
    };

    eprintln!("  barraCuda socket: {}", sock.display());

    // ── Engagement composite via tensor ops ────────────────────
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
            eprintln!("  tensor.create returns no tensor_id — cannot chain ops");
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
                if let Ok(ref resp) = read {
                    report_error("compute.dispatch(read)", resp);
                }
            }
        } else {
            h.check_bool("round_trip_create_read", false);
        }
    } else {
        h.check_bool("round_trip_create_read", false);
    }

    // ── Element-wise tensor ops with REAL tensor IDs ──────────
    // Create a test tensor, then apply ops to it using its actual ID.
    let test_tensor = rpc_call(
        &sock,
        "tensor.create",
        &serde_json::json!({"shape": [4], "data": [0.1, 0.5, -0.3, 0.9]}),
    );
    let test_id = test_tensor.as_ref().ok().and_then(extract_tensor_id);

    if let Some(ref tid) = test_id {
        // tensor.add (scalar mode)
        let add = rpc_call(
            &sock,
            "tensor.add",
            &serde_json::json!({"tensor_id": tid, "scalar": 1.0}),
        );
        let add_ok = add.as_ref().is_ok_and(has_result);
        h.check_bool("tensor_add_scalar", add_ok);
        if add_ok {
            eprintln!("  PASS: tensor.add (scalar) → result");
        } else if let Ok(ref resp) = add {
            report_error("tensor.add", resp);
        }

        // tensor.scale
        let scale = rpc_call(
            &sock,
            "tensor.scale",
            &serde_json::json!({"tensor_id": tid, "scalar": 2.0}),
        );
        let scale_ok = scale.as_ref().is_ok_and(has_result);
        h.check_bool("tensor_scale", scale_ok);
        if scale_ok {
            eprintln!("  PASS: tensor.scale → result");
        } else if let Ok(ref resp) = scale {
            report_error("tensor.scale", resp);
        }

        // tensor.clamp
        let clamp = rpc_call(
            &sock,
            "tensor.clamp",
            &serde_json::json!({"tensor_id": tid, "min": 0.0, "max": 1.0}),
        );
        let clamp_ok = clamp.as_ref().is_ok_and(has_result);
        h.check_bool("tensor_clamp", clamp_ok);
        if clamp_ok {
            eprintln!("  PASS: tensor.clamp → result");
        } else if let Ok(ref resp) = clamp {
            report_error("tensor.clamp", resp);
        }

        // tensor.reduce (correct name, not tensor.reduce_sum)
        let reduce = rpc_call(
            &sock,
            "tensor.reduce",
            &serde_json::json!({"tensor_id": tid, "op": "sum"}),
        );
        let reduce_ok = reduce.as_ref().is_ok_and(has_result);
        h.check_bool("tensor_reduce_sum", reduce_ok);
        if reduce_ok {
            eprintln!("  PASS: tensor.reduce(sum) → result");
            if let Ok(ref resp) = reduce {
                eprintln!("    reduce response: {resp}");
            }
        } else if let Ok(ref resp) = reduce {
            report_error("tensor.reduce", resp);
        }

        // tensor.sigmoid
        let sigmoid = rpc_call(
            &sock,
            "tensor.sigmoid",
            &serde_json::json!({"tensor_id": tid}),
        );
        let sigmoid_ok = sigmoid.as_ref().is_ok_and(has_result);
        h.check_bool("tensor_sigmoid", sigmoid_ok);
        if sigmoid_ok {
            eprintln!("  PASS: tensor.sigmoid → result");
        } else if let Ok(ref resp) = sigmoid {
            report_error("tensor.sigmoid", resp);
        }
    } else {
        eprintln!("  Cannot create test tensor — skipping element-wise ops");
        for name in [
            "tensor_add_scalar",
            "tensor_scale",
            "tensor_clamp",
            "tensor_reduce_sum",
            "tensor_sigmoid",
        ] {
            h.check_bool(name, false);
        }
    }

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
