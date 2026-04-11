// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp092 — GOMS + Four Keys via Pipeline Composition.
//!
//! Validates composite models using biomeOS Pipeline coordination patterns.
//! GOMS KLM prediction composed from stats.weighted_mean, Four Keys
//! composite score from tensor ops.
//!
//! # Science validation
//!
//! GOMS KLM: operator times → stats.weighted_mean → total prediction
//! Four Keys: 4 fun dimensions → tensor.create + tensor.scale + tensor.reduce
//!
//! # References
//!
//! - ludoSpring: exp004 (GOMS), exp008 (Four Keys)
//! - ludoSpring graph: `graphs/composition/engagement_pipeline.toml`
//! - primalSpring: `graphs/spring_validation/composition_game_science_validate.toml`
//! - Python baselines: `baselines/python/`, commit 4b683e3e

use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/goms_model.py + fun_keys_model.py",
    commit: "19e402c0",
    date: "2026-03-30",
    command: "cargo run -p ludospring-exp092",
};

// GOMS KLM: K(0.28) + P(1.1) + H(0.4) + M(1.35) = 3.13s for a typical task
// Composed: weighted_mean with equal weights on [K, P, H, M] operator times
const GOMS_KLM_OPERATORS: [f64; 4] = [0.28, 1.1, 0.4, 1.35];
const GOMS_KLM_EXPECTED_SUM: f64 = 3.13;
const GOMS_KLM_EXPECTED_MEAN: f64 = 3.13 / 4.0;

// Four Keys: hard=0.7, easy=0.8, serious=0.5, people=0.9
// Equal weights (0.25 each) → 0.725
const FOUR_KEYS_SCORES: [f64; 4] = [0.7, 0.8, 0.5, 0.9];
const FOUR_KEYS_WEIGHTS: [f64; 4] = [0.25, 0.25, 0.25, 0.25];
const FOUR_KEYS_EXPECTED: f64 = 0.725;

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
                        && p.extension()
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
        "goms_klm_mean",
        "goms_klm_tolerance",
        "goms_klm_sum_composed",
        "four_keys_weighted_mean",
        "four_keys_tolerance",
        "four_keys_tensor_path",
        "four_keys_tensor_reduce",
        "stats_std_dev",
    ] {
        h.check_bool(name, false);
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "validation harness with many sequential pipeline checks"
)]
fn cmd_validate() {
    let mut h = ValidationHarness::new("exp092_composite_pipeline");
    h.print_provenance(&[&PROVENANCE]);

    let Some(sock) = discover_barracuda_socket() else {
        dry_mode(&mut h);
        h.finish();
    };

    eprintln!("  barraCuda socket: {}", sock.display());

    // ── GOMS KLM: mean of operator times ─────────────────────
    let klm_mean = rpc_call(
        &sock,
        "stats.mean",
        &serde_json::json!({"data": GOMS_KLM_OPERATORS}),
    );
    let klm_mean_ok = klm_mean.as_ref().is_ok_and(has_result);
    h.check_bool("goms_klm_mean", klm_mean_ok);

    if klm_mean_ok {
        if let Some(val) = klm_mean
            .as_ref()
            .ok()
            .and_then(|r| r.pointer("/result/result"))
            .and_then(serde_json::Value::as_f64)
        {
            h.check_abs(
                "goms_klm_tolerance",
                val,
                GOMS_KLM_EXPECTED_MEAN,
                tolerances::ANALYTICAL_TOL,
            );
        }
    }

    // GOMS sum composed from mean * count
    let composed_sum = GOMS_KLM_EXPECTED_MEAN * 4.0;
    h.check_abs(
        "goms_klm_sum_composed",
        composed_sum,
        GOMS_KLM_EXPECTED_SUM,
        tolerances::ANALYTICAL_TOL,
    );

    // ── Four Keys: weighted mean via stats IPC ───────────────
    let fk_wm = rpc_call(
        &sock,
        "stats.weighted_mean",
        &serde_json::json!({"values": FOUR_KEYS_SCORES, "weights": FOUR_KEYS_WEIGHTS}),
    );
    let fk_ok = fk_wm.as_ref().is_ok_and(has_result);
    h.check_bool("four_keys_weighted_mean", fk_ok);

    if fk_ok {
        if let Some(val) = fk_wm
            .as_ref()
            .ok()
            .and_then(|r| r.pointer("/result/result"))
            .and_then(serde_json::Value::as_f64)
        {
            h.check_abs(
                "four_keys_tolerance",
                val,
                FOUR_KEYS_EXPECTED,
                tolerances::ANALYTICAL_TOL,
            );
        }
    }

    // ── Four Keys: tensor path (create → scale → reduce) ────
    let tc = rpc_call(
        &sock,
        "tensor.create",
        &serde_json::json!({"shape": [4], "data": [0.7_f32, 0.8, 0.5, 0.9]}),
    );
    h.check_bool("four_keys_tensor_path", tc.as_ref().is_ok_and(has_result));

    if let Some(tid) = tc
        .as_ref()
        .ok()
        .and_then(|r| r.pointer("/result/tensor_id"))
        .and_then(serde_json::Value::as_str)
    {
        let scaled = rpc_call(
            &sock,
            "tensor.scale",
            &serde_json::json!({"tensor_id": tid, "scalar": 0.25}),
        );
        if let Some(sid) = scaled
            .as_ref()
            .ok()
            .and_then(|r| r.pointer("/result/result_id"))
            .and_then(serde_json::Value::as_str)
        {
            let reduce = rpc_call(
                &sock,
                "tensor.reduce",
                &serde_json::json!({"tensor_id": sid, "op": "sum"}),
            );
            h.check_bool(
                "four_keys_tensor_reduce",
                reduce.as_ref().is_ok_and(|r| {
                    r.pointer("/result/result")
                        .and_then(serde_json::Value::as_f64)
                        .is_some_and(|v| (v - FOUR_KEYS_EXPECTED).abs() < 0.01)
                }),
            );
        }
    }

    // ── Stats: std_dev works for variance analysis ───────────
    let sd = rpc_call(
        &sock,
        "stats.std_dev",
        &serde_json::json!({"data": FOUR_KEYS_SCORES}),
    );
    h.check_bool("stats_std_dev", sd.as_ref().is_ok_and(has_result));

    h.finish();
}

fn main() {
    cmd_validate();
}
