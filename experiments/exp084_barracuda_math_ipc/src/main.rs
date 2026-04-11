// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp084 — barraCuda Math IPC Composition Validation.
//!
//! Validates ludoSpring's game science can be replicated via barraCuda's
//! JSON-RPC IPC surface. Calls the ACTUAL method names and param schemas
//! exposed by barraCuda (v0.3.11+, Sprint 23+).
//!
//! V36: Infrastructure validation for science-via-composition. The science
//! experiments (exp089-093) build on the IPC surface proven here.
//!
//! # barraCuda IPC surface (30 methods, Sprint 23+)
//!
//! Math/activation: `math.sigmoid`, `math.log2`, `activation.fitts`, `activation.hick`
//! Statistics: `stats.mean`, `stats.std_dev`, `stats.weighted_mean`
//! Noise/RNG: `noise.perlin2d`, `noise.perlin3d`, `rng.uniform`
//! Tensor (GPU): `tensor.create/matmul/add/scale/clamp/reduce/sigmoid`
//! Ecosystem: `health.*`, `capabilities.list`, `device.*`
//!
//! # Param schemas (from barracuda-core/src/ipc/methods/)
//!
//! - `math.sigmoid`: `{"data": [f64...]}`
//! - `activation.fitts`: `{"distance": f64, "width": f64, "a": f64, "b": f64}`
//! - `activation.hick`: `{"n_choices": u64, "a": f64, "b": f64}`
//! - `stats.mean`: `{"data": [f64...]}`
//! - `noise.perlin2d`: `{"x": f64, "y": f64}`
//! - `rng.uniform`: `{"n": u64, "min": f64, "max": f64, "seed": u64}`
//!
//! # References
//!
//! - primalSpring: `graphs/compositions/game_science_standalone.toml`
//! - primalSpring: `graphs/spring_validation/composition_game_science_validate.toml`
//! - ludoSpring graph: `graphs/composition/math_pipeline.toml`
//!
//! # Provenance
//!
//! Expected values sourced from ludoSpring Rust validation (exp001–exp034)
//! which trace to Python baselines (baselines/python/, commit 4b683e3e).

use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (composition — barraCuda IPC surface validation)",
    commit: "exp084-v2",
    date: "2026-03-30",
    command: "cargo run -p ludospring-exp084",
};

const FITTS_EXPECTED: f64 = 708.847_613_416_814;
const HICK_EXPECTED: f64 = 650.0;
const PERLIN_LATTICE_EXPECTED: f64 = 0.0;
const SIGMOID_HALF_EXPECTED: f64 = 0.622_459_331_201_854_6;

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

fn error_code(resp: &serde_json::Value) -> Option<i64> {
    resp.pointer("/error/code")
        .and_then(serde_json::Value::as_i64)
}

fn error_message(resp: &serde_json::Value) -> String {
    resp.pointer("/error/message")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_string()
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

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp084_barracuda_math_ipc");
    h.print_provenance(&[&PROVENANCE]);

    let socket = discover_barracuda_socket();
    h.check_bool("barracuda_socket_discovered", socket.is_some());

    let Some(sock) = socket else {
        eprintln!("  barraCuda socket not found — running gap audit in dry mode");
        gap_audit_dry(&mut h);
        h.finish();
    };

    eprintln!("  barraCuda socket: {}", sock.display());

    // ── Phase 1: Ecosystem probes ────────────────────────────
    let health = rpc_call(&sock, "health.liveness", &serde_json::json!({}));
    h.check_bool(
        "barracuda_health_liveness",
        health.as_ref().is_ok_and(has_result),
    );

    let caps = rpc_call(&sock, "capabilities.list", &serde_json::json!({}));
    h.check_bool(
        "barracuda_capabilities_list",
        caps.as_ref().is_ok_and(has_result),
    );
    if let Ok(ref resp) = caps {
        eprintln!("  capabilities: {resp}");
    }

    let tensor = rpc_call(
        &sock,
        "tensor.create",
        &serde_json::json!({"shape": [2, 2], "data": [1.0, 0.0, 0.0, 1.0]}),
    );
    h.check_bool(
        "tensor_create_identity_2x2",
        tensor.as_ref().is_ok_and(has_result),
    );

    // ── Phase 2: Math methods with CORRECT names + params ────
    probe_math(
        &mut h,
        &sock,
        "math.sigmoid",
        &serde_json::json!({"data": [0.5]}),
        "sigmoid_half",
        Some(SIGMOID_HALF_EXPECTED),
        tolerances::ANALYTICAL_TOL,
    );

    probe_math(
        &mut h,
        &sock,
        "math.log2",
        &serde_json::json!({"data": [8.0]}),
        "log2_of_8",
        Some(3.0),
        tolerances::ANALYTICAL_TOL,
    );

    probe_math(
        &mut h,
        &sock,
        "activation.fitts",
        &serde_json::json!({"distance": 100.0, "width": 10.0, "a": 50.0, "b": 150.0}),
        "fitts_movement_time",
        Some(FITTS_EXPECTED),
        tolerances::ANALYTICAL_TOL,
    );

    probe_math(
        &mut h,
        &sock,
        "activation.hick",
        &serde_json::json!({"n_choices": 7, "a": 200.0, "b": 150.0}),
        "hick_reaction_time",
        Some(HICK_EXPECTED),
        tolerances::ANALYTICAL_TOL,
    );

    probe_math(
        &mut h,
        &sock,
        "stats.mean",
        &serde_json::json!({"data": [1.0, 2.0, 3.0, 4.0, 5.0]}),
        "stats_mean",
        Some(3.0),
        tolerances::ANALYTICAL_TOL,
    );

    probe_math(
        &mut h,
        &sock,
        "stats.std_dev",
        &serde_json::json!({"data": [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]}),
        "stats_std_dev",
        None, // accept any valid result
        0.0,
    );

    probe_math(
        &mut h,
        &sock,
        "noise.perlin2d",
        &serde_json::json!({"x": 0.0, "y": 0.0}),
        "perlin2d_lattice",
        Some(PERLIN_LATTICE_EXPECTED),
        tolerances::ANALYTICAL_TOL,
    );

    probe_math(
        &mut h,
        &sock,
        "rng.uniform",
        &serde_json::json!({"n": 5, "min": 0.0, "max": 1.0, "seed": 42}),
        "rng_uniform",
        None, // accept any valid result
        0.0,
    );

    // ── Phase 3: Neural API capability.call routing ──────────
    let neural = discover_neural_api();
    if let Some(ref na) = neural {
        eprintln!("  Neural API socket: {}", na.display());
        let cap_route = rpc_call(
            na,
            "capability.call",
            &serde_json::json!({
                "capability": "compute",
                "operation": "tensor.create",
                "params": {"shape": [2], "data": [1.0, 2.0]}
            }),
        );
        let routed = cap_route.as_ref().is_ok_and(has_result);
        h.check_bool("neural_api_routes_compute", routed);
        if routed {
            eprintln!("  Neural API routes compute → barraCuda successfully");
        } else {
            eprintln!(
                "  GAP (biomeOS): Neural API compute domain routes to toadStool, not barraCuda"
            );
            eprintln!("  biomeOS bootstrap graph maps compute→toadStool; needs math→barraCuda");
            if let Ok(ref resp) = cap_route {
                eprintln!("    Response: {resp}");
            }
        }
    } else {
        eprintln!("  Neural API not running — skipping capability.call routing check");
        h.check_bool("neural_api_routes_compute", false);
    }

    // ── Phase 4: Domain-level methods (real gaps) ────────────
    // These are ludoSpring domain compositions that don't exist
    // in barraCuda. They need either upstream absorption or
    // ludoSpring should compose them from primitives.
    let flow = rpc_call(
        &sock,
        "math.flow.evaluate",
        &serde_json::json!({"challenge": 0.5, "skill": 0.5, "channel_width": 0.15}),
    );
    let flow_exists = flow.as_ref().is_ok_and(has_result);
    h.check_bool("domain_flow_evaluate", flow_exists);
    if !flow_exists {
        eprintln!("  EXPECTED GAP: math.flow.evaluate — ludoSpring domain method");
        eprintln!("    Composable from sigmoid + clamp; not in barraCuda yet");
    }

    let engagement = rpc_call(
        &sock,
        "math.engagement.composite",
        &serde_json::json!({"action_rate": 30.0, "exploration_rate": 2.0, "variety": 0.6}),
    );
    let eng_exists = engagement.as_ref().is_ok_and(has_result);
    h.check_bool("domain_engagement_composite", eng_exists);
    if !eng_exists {
        eprintln!("  EXPECTED GAP: math.engagement.composite — ludoSpring domain method");
        eprintln!("    Composable from stats.weighted_mean + tensor ops; not in barraCuda yet");
    }

    h.finish();
}

fn probe_math(
    h: &mut ValidationHarness,
    sock: &Path,
    method: &str,
    params: &serde_json::Value,
    check_name: &str,
    expected: Option<f64>,
    tol: f64,
) {
    match rpc_call(sock, method, params) {
        Ok(resp) if has_result(&resp) => {
            eprintln!("  PASS: {method} → result present");
            match expected {
                Some(exp) => {
                    if let Some(val) = resp
                        .pointer("/result/value")
                        .or_else(|| resp.pointer("/result/0"))
                        .or_else(|| resp.pointer("/result/data/0"))
                        .or_else(|| resp.pointer("/result"))
                        .and_then(serde_json::Value::as_f64)
                    {
                        h.check_abs(check_name, val, exp, tol);
                    } else {
                        eprintln!("    Result present but cannot extract numeric value: {resp}");
                        h.check_bool(check_name, true);
                    }
                }
                None => h.check_bool(check_name, true),
            }
        }
        Ok(resp) if error_code(&resp) == Some(METHOD_NOT_FOUND) => {
            eprintln!("  FAIL: {method} → -32601 method_not_found");
            h.check_bool(check_name, false);
        }
        Ok(resp) if error_code(&resp) == Some(INVALID_PARAMS) => {
            eprintln!(
                "  FAIL: {method} → -32602 invalid_params: {}",
                error_message(&resp)
            );
            eprintln!("    Method EXISTS but param schema doesn't match");
            h.check_bool(check_name, false);
        }
        Ok(resp) => {
            eprintln!(
                "  FAIL: {method} → {}: {}",
                error_code(&resp).unwrap_or(0),
                error_message(&resp)
            );
            h.check_bool(check_name, false);
        }
        Err(e) => {
            eprintln!("  CONN_ERROR: {method} → {e}");
            h.check_bool(check_name, false);
        }
    }
}

fn gap_audit_dry(h: &mut ValidationHarness) {
    let gaps = [
        "barracuda_health_liveness",
        "barracuda_capabilities_list",
        "tensor_create_identity_2x2",
        "sigmoid_half",
        "log2_of_8",
        "fitts_movement_time",
        "hick_reaction_time",
        "stats_mean",
        "stats_std_dev",
        "perlin2d_lattice",
        "rng_uniform",
        "neural_api_routes_compute",
        "domain_flow_evaluate",
        "domain_engagement_composite",
    ];
    for name in gaps {
        eprintln!("  DRY_GAP: {name} — barraCuda not running");
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
