// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp084 — barraCuda Math IPC Gap Discovery.
//!
//! Attempts to replicate ludoSpring's validated game science (Fitts, Hick,
//! steering, Perlin noise, engagement, flow) by composing barraCuda's
//! JSON-RPC IPC surface. Every method that doesn't exist over IPC is a
//! **gap** — evolution pressure for barraCuda to expose its math library.
//!
//! The goal is NOT to pass. The goal is to discover what's missing so
//! that primal composition can eventually replace in-process delegation.
//!
//! # Known barraCuda IPC surface (as of 2026-03-29)
//!
//! - `tensor.create` — allocate f32 tensor from shape + data
//! - `tensor.matmul` — matrix multiply two stored tensors
//! - `compute.dispatch` — only `zeros`, `ones`, `read` ops
//! - `fhe.ntt`, `fhe.pointwise_mul` — FHE domain
//! - `health.*`, `device.*`, `capabilities.*` — discovery
//!
//! # What we NEED callable over IPC
//!
//! - `math.activation.sigmoid`, `.relu`, `.gelu`, `.softmax`
//! - `math.stats.mean`, `.dot`, `.variance`, `.std_dev`, `.pearson`
//! - `math.noise.perlin_2d`, `.fbm_2d`
//! - `math.rng.lcg_step`
//! - `math.formula.fitts_mt`, `.hick_rt`, `.steering_time`
//! - `math.flow.evaluate`, `math.engagement.composite`
//!
//! # Also probe via Neural API capability.call
//!
//! biomeOS `capability.call` IS implemented — it routes `compute` domain
//! to barraCuda if registered. We test both direct UDS and Neural API
//! routing to discover which path works.
//!
//! # Reference
//!
//! primalSpring exp088: uses `tcp::neural_api_capability_call` pattern
//! primalSpring STORYTELLING_EVOLUTION.md: documents ludoSpring's 8 game.* methods
//! wateringHole CAPABILITY_BASED_DISCOVERY_STANDARD.md: tiered fallback
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
    script: "N/A (composition gap discovery — barraCuda IPC surface audit)",
    commit: "exp084-v1",
    date: "2026-03-29",
    command: "cargo run -p ludospring-exp084",
};

/// Known Fitts result: MT for D=100, W=10, a=50, b=150.
/// Validated in exp001, python_parity.rs, baselines/python/interaction_laws.py.
const FITTS_EXPECTED: f64 = 708.847_613_416_814;

/// Known Hick result: RT for N=7, a=200, b=150.
const HICK_EXPECTED: f64 = 650.0;

/// Known Perlin 2D at integer lattice: always 0.0.
const PERLIN_LATTICE_EXPECTED: f64 = 0.0;

/// Known sigmoid(0.5) from Python/Rust parity.
const SIGMOID_HALF_EXPECTED: f64 = 0.622_459_331_201_854_6;

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
        eprintln!("  GAP: barraCuda process must be running for composition tests");

        gap_audit_dry(&mut h);
        h.finish();
    };

    eprintln!("  barraCuda socket: {}", sock.display());

    // ── Phase 1: Confirm known IPC surface works ──────────────
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

    // ── Phase 2: Probe for math methods that SHOULD exist ─────
    // Each probe tests a method ludoSpring needs. Method-not-found
    // is the expected gap — we document it, not fail on it.

    probe_method(
        &mut h,
        &sock,
        "math.activation.sigmoid",
        &serde_json::json!({"x": 0.5}),
        "gap_sigmoid",
        SIGMOID_HALF_EXPECTED,
        tolerances::ANALYTICAL_TOL,
    );

    probe_method(
        &mut h,
        &sock,
        "math.activation.relu",
        &serde_json::json!({"x": -0.5}),
        "gap_relu",
        0.0,
        tolerances::ANALYTICAL_TOL,
    );

    probe_method(
        &mut h,
        &sock,
        "math.stats.mean",
        &serde_json::json!({"values": [1.0, 2.0, 3.0, 4.0, 5.0]}),
        "gap_mean",
        3.0,
        tolerances::ANALYTICAL_TOL,
    );

    probe_method(
        &mut h,
        &sock,
        "math.stats.dot",
        &serde_json::json!({"a": [1.0, 2.0, 3.0], "b": [4.0, 5.0, 6.0]}),
        "gap_dot_product",
        32.0,
        tolerances::ANALYTICAL_TOL,
    );

    probe_method(
        &mut h,
        &sock,
        "math.noise.perlin_2d",
        &serde_json::json!({"x": 0.0, "y": 0.0}),
        "gap_perlin_2d_lattice",
        PERLIN_LATTICE_EXPECTED,
        tolerances::ANALYTICAL_TOL,
    );

    probe_method(
        &mut h,
        &sock,
        "math.rng.lcg_step",
        &serde_json::json!({"state": 42}),
        "gap_lcg_rng",
        f64::NAN, // any valid state is fine
        f64::INFINITY,
    );

    probe_formula(
        &mut h,
        &sock,
        "math.formula.fitts_mt",
        &serde_json::json!({"d": 100.0, "w": 10.0, "a": 50.0, "b": 150.0}),
        "gap_fitts_movement_time",
        FITTS_EXPECTED,
        tolerances::ANALYTICAL_TOL,
    );

    probe_formula(
        &mut h,
        &sock,
        "math.formula.hick_rt",
        &serde_json::json!({"n": 7, "a": 200.0, "b": 150.0}),
        "gap_hick_reaction_time",
        HICK_EXPECTED,
        tolerances::ANALYTICAL_TOL,
    );

    probe_method(
        &mut h,
        &sock,
        "math.flow.evaluate",
        &serde_json::json!({"challenge": 0.5, "skill": 0.5, "channel_width": 0.15}),
        "gap_flow_evaluate",
        f64::NAN,
        f64::INFINITY,
    );

    probe_method(
        &mut h,
        &sock,
        "math.engagement.composite",
        &serde_json::json!({
            "action_rate": 30.0, "exploration_rate": 2.0,
            "variety": 0.6, "completion": 0.4, "social": 0.3,
            "session_minutes": 10.0
        }),
        "gap_engagement_composite",
        f64::NAN,
        f64::INFINITY,
    );

    // ── Phase 3: Neural API capability.call routing ──────────
    // biomeOS capability.call IS implemented — test whether it can
    // route `compute` domain to barraCuda for tensor ops.
    let neural = discover_neural_api();
    if let Some(ref na) = neural {
        eprintln!("  Neural API socket: {}", na.display());
        let cap_route = rpc_call(
            na,
            "capability.call",
            &serde_json::json!({
                "capability": "compute",
                "operation": "tensor.create",
                "args": {"shape": [2], "data": [1.0, 2.0]}
            }),
        );
        let routed = cap_route.as_ref().is_ok_and(has_result);
        h.check_bool("neural_api_routes_compute", routed);
        if routed {
            eprintln!("  Neural API routes compute → barraCuda successfully");
        } else {
            eprintln!("  GAP: Neural API cannot route compute domain to barraCuda");
            if let Ok(ref resp) = cap_route {
                eprintln!("    Response: {resp}");
            }
        }
    } else {
        eprintln!("  Neural API not running — skipping capability.call routing check");
        h.check_bool("neural_api_routes_compute", false);
    }

    // ── Phase 4: Summary ──────────────────────────────────────
    eprintln!();
    eprintln!("  ══════════════════════════════════════════════════════");
    eprintln!("  GAP SUMMARY: Methods probed that returned method-not-found");
    eprintln!("  are evolution pressure for barraCuda to expose its math");
    eprintln!("  library over JSON-RPC. Each gap blocks composition-tier");
    eprintln!("  validation of ludoSpring's science.");
    eprintln!("  ══════════════════════════════════════════════════════");

    h.finish();
}

fn probe_method(
    h: &mut ValidationHarness,
    sock: &Path,
    method: &str,
    params: &serde_json::Value,
    check_name: &str,
    expected: f64,
    tol: f64,
) {
    match rpc_call(sock, method, params) {
        Ok(resp) if has_result(&resp) => {
            eprintln!("  AVAILABLE: {method} → result present");
            if expected.is_nan() {
                h.check_bool(check_name, true);
            } else if let Some(val) = resp
                .pointer("/result/value")
                .or_else(|| resp.pointer("/result"))
                .and_then(serde_json::Value::as_f64)
            {
                h.check_abs(check_name, val, expected, tol);
            } else {
                h.check_bool(check_name, true);
            }
        }
        Ok(resp) if has_error_code(&resp, METHOD_NOT_FOUND) => {
            eprintln!("  GAP: {method} → method_not_found (-32601)");
            eprintln!("    Evolution pressure: barraCuda should expose this over IPC");
            eprintln!("    ludoSpring expected: {expected} (tol {tol})");
            h.check_bool(check_name, false);
        }
        Ok(resp) => {
            let code = resp
                .pointer("/error/code")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);
            let msg = resp
                .pointer("/error/message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            eprintln!("  ERROR: {method} → {code}: {msg}");
            h.check_bool(check_name, false);
        }
        Err(e) => {
            eprintln!("  CONN_ERROR: {method} → {e}");
            h.check_bool(check_name, false);
        }
    }
}

fn probe_formula(
    h: &mut ValidationHarness,
    sock: &Path,
    method: &str,
    params: &serde_json::Value,
    check_name: &str,
    expected: f64,
    tol: f64,
) {
    probe_method(h, sock, method, params, check_name, expected, tol);
}

fn gap_audit_dry(h: &mut ValidationHarness) {
    let gaps = [
        "gap_sigmoid",
        "gap_relu",
        "gap_mean",
        "gap_dot_product",
        "gap_perlin_2d_lattice",
        "gap_lcg_rng",
        "gap_fitts_movement_time",
        "gap_hick_reaction_time",
        "gap_flow_evaluate",
        "gap_engagement_composite",
        "neural_api_routes_compute",
    ];
    for name in gaps {
        eprintln!("  DRY_GAP: {name} — barraCuda not running, cannot probe");
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
