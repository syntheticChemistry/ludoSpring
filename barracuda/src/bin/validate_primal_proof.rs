// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)] // validation binary — no public API
//! Level 5 — Primal proof: domain science via barraCuda IPC.
//!
//! Proves that ludoSpring's peer-reviewed game science produces correct
//! results when the underlying math primitives are called through
//! barraCuda's JSON-RPC UDS socket (32 methods) rather than via Rust
//! library imports.
//!
//! Golden values sourced from Python baselines (`baselines/python/`),
//! validated in Rust by `validate_interaction` / `validate_procedural`
//! (Level 2). This binary re-validates those same values but through
//! the primal IPC boundary — the "primal proof."
//!
//! Exit codes: 0 = pass, 1 = fail, 2 = skip (barraCuda socket absent).

use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

use ludospring_barracuda::ipc::{IpcError, RpcClient};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, EXIT_SKIPPED, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/run_all_baselines.py",
    commit: "231928a",
    date: "2026-04-17",
    command: "cargo run --bin validate_primal_proof --features ipc",
};

const FITTS_MT_D100_W10: f64 = 708.847_613_416_814;
const HICK_RT_N7: f64 = 650.0;
const PERLIN_ORIGIN: f64 = 0.0;
const SIGMOID_HALF: f64 = 0.622_459_331_201_854_6;
const STATS_MEAN_1_5: f64 = 3.0;
const LOG2_OF_8: f64 = 3.0;

fn rpc_call(
    client: &RpcClient,
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, IpcError> {
    client.send_raw(&serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    }))
}

fn has_result(resp: &serde_json::Value) -> bool {
    resp.get("result").is_some() && resp.get("error").is_none()
}

fn extract_scalar(resp: &serde_json::Value) -> Option<f64> {
    resp.pointer("/result/value")
        .or_else(|| resp.pointer("/result/0"))
        .or_else(|| resp.pointer("/result/data/0"))
        .or_else(|| resp.pointer("/result"))
        .and_then(serde_json::Value::as_f64)
}

/// Discover a socket providing the `compute` capability.
///
/// Priority:
/// 1. `BARRACUDA_SOCK` env override (explicit)
/// 2. Capability-based scan: any `.sock` file whose name contains `compute`
///    or matches the known dependency for `compute` in `niche::DEPENDENCIES`
/// 3. Pattern fallback: scan for any `.sock` in ecosystem dirs
fn discover_barracuda_socket() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("BARRACUDA_SOCK") {
        let p = PathBuf::from(&explicit);
        if p.exists() {
            return Some(p);
        }
    }

    let dep = ludospring_barracuda::niche::DEPENDENCIES
        .iter()
        .find(|d| d.capability == "compute" || d.capability == "tensor");
    let dep_name = dep.and_then(|d| d.hint_name);
    let capability = dep.map_or("compute", |d| d.capability);

    let dirs = ludospring_barracuda::niche::socket_dirs();
    for dir in &dirs {
        let cap_sock = dir.join(format!("{capability}.sock"));
        if cap_sock.exists() {
            return Some(cap_sock);
        }
        if let Some(name) = dep_name {
            let named = dir.join(format!("{name}.sock"));
            if named.exists() {
                return Some(named);
            }
            let named_core = dir.join(format!("{name}-core.sock"));
            if named_core.exists() {
                return Some(named_core);
            }
        }
    }
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
                    let is_sock = p
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"));
                    if is_sock
                        && (n.contains(capability) || dep_name.is_some_and(|dn| n.starts_with(dn)))
                    {
                        return Some(p);
                    }
                }
            }
        }
    }
    None
}

fn check_ipc_scalar(
    h: &mut ValidationHarness,
    client: &RpcClient,
    method: &str,
    params: &serde_json::Value,
    label: &str,
    expected: f64,
    tol: f64,
) {
    match rpc_call(client, method, params) {
        Ok(resp) if has_result(&resp) => {
            if let Some(val) = extract_scalar(&resp) {
                h.check_abs(label, val, expected, tol);
            } else {
                eprintln!("  {method}: result present but no scalar — {resp}");
                h.check_bool(label, false);
            }
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
            eprintln!("  {method}: error {code} — {msg}");
            h.check_bool(label, false);
        }
        Err(e) => {
            eprintln!("  {method}: connection error — {e}");
            h.check_bool(label, false);
        }
    }
}

fn check_ipc_exists(h: &mut ValidationHarness, client: &RpcClient, method: &str, label: &str) {
    let resp = rpc_call(client, method, &serde_json::json!({}));
    h.check_bool(label, resp.as_ref().is_ok_and(has_result));
}

fn phase_health(h: &mut ValidationHarness, client: &RpcClient) {
    use ludospring_barracuda::ipc::methods;
    check_ipc_exists(
        h,
        client,
        methods::health::LIVENESS,
        "barracuda_health_liveness",
    );
    check_ipc_exists(
        h,
        client,
        methods::capability::LIST_ALT,
        "barracuda_capabilities_list",
    );
}

fn phase_interaction(h: &mut ValidationHarness, client: &RpcClient) {
    check_ipc_scalar(
        h,
        client,
        "activation.fitts",
        &serde_json::json!({"distance": 100.0, "width": 10.0, "a": 50.0, "b": 150.0}),
        "primal_fitts_mt_D100_W10",
        FITTS_MT_D100_W10,
        tolerances::ANALYTICAL_TOL,
    );

    check_ipc_scalar(
        h,
        client,
        "activation.hick",
        &serde_json::json!({"n_choices": 7, "a": 200.0, "b": 150.0}),
        "primal_hick_rt_N7",
        HICK_RT_N7,
        tolerances::ANALYTICAL_TOL,
    );
}

fn phase_math(h: &mut ValidationHarness, client: &RpcClient) {
    check_ipc_scalar(
        h,
        client,
        "math.sigmoid",
        &serde_json::json!({"data": [0.5]}),
        "primal_sigmoid_half",
        SIGMOID_HALF,
        tolerances::ANALYTICAL_TOL,
    );

    check_ipc_scalar(
        h,
        client,
        "math.log2",
        &serde_json::json!({"data": [8.0]}),
        "primal_log2_of_8",
        LOG2_OF_8,
        tolerances::ANALYTICAL_TOL,
    );

    check_ipc_scalar(
        h,
        client,
        "stats.mean",
        &serde_json::json!({"data": [1.0, 2.0, 3.0, 4.0, 5.0]}),
        "primal_stats_mean",
        STATS_MEAN_1_5,
        tolerances::ANALYTICAL_TOL,
    );
}

fn phase_procedural(h: &mut ValidationHarness, client: &RpcClient) {
    check_ipc_scalar(
        h,
        client,
        "noise.perlin2d",
        &serde_json::json!({"x": 0.0, "y": 0.0}),
        "primal_perlin_origin",
        PERLIN_ORIGIN,
        tolerances::ANALYTICAL_TOL,
    );

    let std_dev = rpc_call(
        client,
        "stats.std_dev",
        &serde_json::json!({"data": [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]}),
    );
    h.check_bool(
        "primal_stats_std_dev",
        std_dev.as_ref().is_ok_and(has_result),
    );

    let rng = rpc_call(
        client,
        "rng.uniform",
        &serde_json::json!({"n": 5, "min": 0.0, "max": 1.0, "seed": 42}),
    );
    h.check_bool("primal_rng_uniform", rng.as_ref().is_ok_and(has_result));
}

fn phase_tensor(h: &mut ValidationHarness, client: &RpcClient) {
    let tensor = rpc_call(
        client,
        "tensor.create",
        &serde_json::json!({"shape": [2, 2], "data": [1.0, 0.0, 0.0, 1.0]}),
    );
    h.check_bool(
        "primal_tensor_create",
        tensor.as_ref().is_ok_and(has_result),
    );
}

fn main() {
    let mut h = ValidationHarness::new("validate_primal_proof (Level 5)");
    h.print_provenance(&[&PROVENANCE]);

    let Some(sock) = discover_barracuda_socket() else {
        eprintln!("SKIP: barraCuda socket not found (set BARRACUDA_SOCK or start barracuda-core)");
        std::process::exit(EXIT_SKIPPED);
    };

    if UnixStream::connect(&sock).is_err() {
        eprintln!(
            "SKIP: cannot connect to barraCuda at {} (server not listening?)",
            sock.display()
        );
        std::process::exit(EXIT_SKIPPED);
    }

    eprintln!("  barraCuda socket: {}", sock.display());

    let client = RpcClient::new(&sock, Duration::from_secs(5));
    phase_health(&mut h, &client);
    phase_interaction(&mut h, &client);
    phase_math(&mut h, &client);
    phase_procedural(&mut h, &client);
    phase_tensor(&mut h, &client);

    h.finish();
}
