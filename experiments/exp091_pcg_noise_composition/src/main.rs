// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp091 — Perlin Noise + WFC via Composition.
//!
//! Validates procedural content generation models by composing barraCuda
//! noise and tensor IPC primitives against Python baselines.
//!
//! # Science validation
//!
//! Perlin: `noise.perlin2d` / `noise.perlin3d` point queries match Python
//! WFC: Constraint propagation via tensor ops (create → reduce for entropy)
//!
//! # References
//!
//! - ludoSpring: exp009 (Perlin), exp011 (WFC)
//! - Python baselines: `baselines/python/`, commit 4b683e3e
//! - barraCuda IPC: `noise.perlin2d`, `noise.perlin3d`, `tensor.*`

use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/perlin_noise.py (WFC: analytical)",
    commit: "4b683e3e",
    date: "2026-03-30",
    command: "cargo run -p ludospring-exp091",
};

// Perlin at lattice points is always 0.0 (mathematical property)
const PERLIN_LATTICE_EXPECTED: f64 = 0.0;
// Perlin range is [-1, 1] for any input
const PERLIN_RANGE_MIN: f64 = -1.0;
const PERLIN_RANGE_MAX: f64 = 1.0;

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
                        && std::path::Path::new(n)
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

fn extract_result_f64(resp: &serde_json::Value) -> Option<f64> {
    resp.pointer("/result/result")
        .and_then(serde_json::Value::as_f64)
}

fn dry_mode(h: &mut ValidationHarness) {
    eprintln!("  barraCuda not available — dry-run with known gaps");
    for name in &[
        "perlin2d_lattice_zero",
        "perlin2d_nonlattice_range",
        "perlin2d_deterministic",
        "perlin3d_lattice_zero",
        "perlin3d_nonlattice_range",
        "wfc_tensor_create_adjacency",
        "wfc_tensor_reduce_min_entropy",
        "rng_seeded_deterministic",
    ] {
        h.check_bool(name, false);
    }
}

#[expect(clippy::too_many_lines, reason = "validation harness")]
fn cmd_validate() {
    let mut h = ValidationHarness::new("exp091_pcg_noise_composition");
    h.print_provenance(&[&PROVENANCE]);

    let Some(sock) = discover_barracuda_socket() else {
        dry_mode(&mut h);
        h.finish();
    };

    eprintln!("  barraCuda socket: {}", sock.display());

    // ── Perlin 2D: lattice point = 0.0 ──────────────────────
    let p2d_lattice = rpc_call(
        &sock,
        "noise.perlin2d",
        &serde_json::json!({"x": 1.0, "y": 1.0}),
    );
    if let Some(val) = p2d_lattice.as_ref().ok().and_then(extract_result_f64) {
        h.check_abs(
            "perlin2d_lattice_zero",
            val,
            PERLIN_LATTICE_EXPECTED,
            tolerances::ANALYTICAL_TOL,
        );
    } else {
        h.check_bool("perlin2d_lattice_zero", false);
    }

    // ── Perlin 2D: non-lattice is in [-1, 1] ────────────────
    let p2d_mid = rpc_call(
        &sock,
        "noise.perlin2d",
        &serde_json::json!({"x": 0.5, "y": 0.5}),
    );
    if let Some(val) = p2d_mid.as_ref().ok().and_then(extract_result_f64) {
        h.check_bool(
            "perlin2d_nonlattice_range",
            (PERLIN_RANGE_MIN..=PERLIN_RANGE_MAX).contains(&val),
        );
    } else {
        h.check_bool("perlin2d_nonlattice_range", false);
    }

    // ── Perlin 2D: deterministic (same input → same output) ─
    let p2d_again = rpc_call(
        &sock,
        "noise.perlin2d",
        &serde_json::json!({"x": 0.5, "y": 0.5}),
    );
    let first = p2d_mid.as_ref().ok().and_then(extract_result_f64);
    let second = p2d_again.as_ref().ok().and_then(extract_result_f64);
    h.check_bool("perlin2d_deterministic", first.is_some() && first == second);

    // ── Perlin 3D: lattice point = 0.0 ──────────────────────
    let p3d_lattice = rpc_call(
        &sock,
        "noise.perlin3d",
        &serde_json::json!({"x": 1.0, "y": 1.0, "z": 1.0}),
    );
    if let Some(val) = p3d_lattice.as_ref().ok().and_then(extract_result_f64) {
        h.check_abs(
            "perlin3d_lattice_zero",
            val,
            PERLIN_LATTICE_EXPECTED,
            tolerances::ANALYTICAL_TOL,
        );
    } else {
        h.check_bool("perlin3d_lattice_zero", false);
    }

    // ── Perlin 3D: non-lattice is in [-1, 1] ────────────────
    let p3d_mid = rpc_call(
        &sock,
        "noise.perlin3d",
        &serde_json::json!({"x": 0.3, "y": 0.7, "z": 0.1}),
    );
    if let Some(val) = p3d_mid.as_ref().ok().and_then(extract_result_f64) {
        h.check_bool(
            "perlin3d_nonlattice_range",
            (PERLIN_RANGE_MIN..=PERLIN_RANGE_MAX).contains(&val),
        );
    } else {
        h.check_bool("perlin3d_nonlattice_range", false);
    }

    // ── WFC: tensor adjacency matrix for constraint propagation
    let adj = rpc_call(
        &sock,
        "tensor.create",
        &serde_json::json!({"shape": [3, 3], "data": [
            1.0_f32, 1.0, 0.0,
            1.0, 1.0, 1.0,
            0.0, 1.0, 1.0
        ]}),
    );
    h.check_bool(
        "wfc_tensor_create_adjacency",
        adj.as_ref().is_ok_and(has_result),
    );

    if let Some(tid) = adj
        .as_ref()
        .ok()
        .and_then(|r| r.pointer("/result/tensor_id"))
        .and_then(serde_json::Value::as_str)
    {
        let reduce = rpc_call(
            &sock,
            "tensor.reduce",
            &serde_json::json!({"tensor_id": tid, "op": "min"}),
        );
        h.check_bool(
            "wfc_tensor_reduce_min_entropy",
            reduce.as_ref().is_ok_and(|r| {
                r.pointer("/result/result")
                    .and_then(serde_json::Value::as_f64)
                    .is_some_and(|v| (v - 0.0).abs() < tolerances::ANALYTICAL_TOL)
            }),
        );
    }

    // ── RNG: seeded deterministic output ─────────────────────
    let rng1 = rpc_call(
        &sock,
        "rng.uniform",
        &serde_json::json!({"n": 5, "min": 0.0, "max": 1.0, "seed": 42}),
    );
    let rng2 = rpc_call(
        &sock,
        "rng.uniform",
        &serde_json::json!({"n": 5, "min": 0.0, "max": 1.0, "seed": 42}),
    );
    let r1 = rng1
        .as_ref()
        .ok()
        .and_then(|r| r.pointer("/result/result").cloned());
    let r2 = rng2
        .as_ref()
        .ok()
        .and_then(|r| r.pointer("/result/result").cloned());
    h.check_bool("rng_seeded_deterministic", r1.is_some() && r1 == r2);

    h.finish();
}

fn main() {
    cmd_validate();
}
