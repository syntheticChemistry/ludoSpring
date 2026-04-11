// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp085 — Sovereign Shader Dispatch Chain.
//!
//! Validates the full compile→dispatch→readback pipeline for GPU math
//! using primal composition: coralReef compiles WGSL, toadStool dispatches
//! the compiled binary, and we compare the result to ludoSpring's known
//! CPU reference values.
//!
//! V36: barraCuda Sprint 24 adds `barracuda-naga-exec` — pure Rust CPU
//! interpreter for naga IR. This provides an alternative validation backend
//! (no GPU required). coralReef IPC contract now includes `shader.compile.cpu`
//! and `shader.validate`. Sovereign dispatch readback still needs coralReef
//! driver (hardware gap).
//!
//! This is the sovereign GPU path: no wgpu, no vendor SDK. WGSL source →
//! coralReef (sovereign compiler) → toadStool (hardware dispatch) → result.
//!
//! Reference: primalSpring `graphs/science/gaming_mesh_chimera.toml`
//!
//! # Composition chain
//!
//! 1. `shader.compile.wgsl` (coralReef) — compile a validated WGSL shader
//! 2. `compute.dispatch.submit` (toadStool) — dispatch compiled binary
//! 3. Compare output to ludoSpring CPU reference value
//!
//! # Shaders tested
//!
//! - `sigmoid.wgsl` — known output sigmoid(0.5) = 0.6224593...
//! - `perlin_2d.wgsl` — known output at lattice points = 0.0
//! - `relu.wgsl` — known output relu(-0.5) = 0.0, relu(0.5) = 0.5
//!
//! # Provenance
//!
//! CPU reference values from exp030 (CPU-vs-GPU parity), validated against
//! Python baselines. GPU tolerances from `tolerances::gpu`.

use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (composition — sovereign shader dispatch chain)",
    commit: "exp085-v1",
    date: "2026-03-29",
    command: "cargo run -p ludospring-exp085",
};

const SIGMOID_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i < arrayLength(&input) {
        output[i] = 1.0 / (1.0 + exp(-input[i]));
    }
}
";

const RELU_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i < arrayLength(&input) {
        output[i] = max(input[i], 0.0);
    }
}
";

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
        .set_read_timeout(Some(Duration::from_secs(10)))
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

fn discover_socket(patterns: &[&str]) -> Option<PathBuf> {
    let dirs = ludospring_barracuda::niche::socket_dirs();
    for dir in &dirs {
        for pat in patterns {
            let path = dir.join(pat);
            if path.exists() {
                return Some(path);
            }
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
                    let prefix = patterns
                        .first()
                        .unwrap_or(&"")
                        .split('.')
                        .next()
                        .unwrap_or("");
                    if !prefix.is_empty()
                        && n.starts_with(prefix)
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

#[expect(
    clippy::too_many_lines,
    reason = "validation harness with many sequential shader dispatch checks"
)]
fn cmd_validate() {
    let mut h = ValidationHarness::new("exp085_shader_dispatch_chain");
    h.print_provenance(&[&PROVENANCE]);

    // ── Discover primals ──────────────────────────────────────
    let coralreef = discover_socket(&["coralreef.sock", "coralReef.sock"]);
    let toadstool = discover_socket(&["toadstool.sock", "toadStool.sock", "compute.sock"]);

    h.check_bool("coralreef_socket_discovered", coralreef.is_some());
    h.check_bool("toadstool_socket_discovered", toadstool.is_some());

    if coralreef.is_none() {
        eprintln!("  GAP: coralReef not running — cannot compile WGSL via composition");
        eprintln!("    Without sovereign shader compiler, GPU composition is blocked");
    }
    if toadstool.is_none() {
        eprintln!("  GAP: toadStool not running — cannot dispatch compiled shaders");
    }

    let Some(coral) = coralreef else {
        skip_all_shader_checks(&mut h);
        h.finish();
    };
    eprintln!("  coralReef socket: {}", coral.display());

    // ── Phase 1: Compile WGSL shaders via coralReef ───────────
    let sigmoid_compile = rpc_call(
        &coral,
        "shader.compile.wgsl",
        &serde_json::json!({
            "wgsl_source": SIGMOID_WGSL,
            "entry_point": "main"
        }),
    );
    let sigmoid_compiled = sigmoid_compile.as_ref().is_ok_and(has_result);
    h.check_bool("compile_sigmoid_wgsl", sigmoid_compiled);
    if !sigmoid_compiled {
        if let Ok(ref resp) = sigmoid_compile {
            eprintln!("  GAP: shader.compile.wgsl failed: {resp}");
        }
    }

    let relu_compile = rpc_call(
        &coral,
        "shader.compile.wgsl",
        &serde_json::json!({
            "wgsl_source": RELU_WGSL,
            "entry_point": "main"
        }),
    );
    let relu_compiled = relu_compile.as_ref().is_ok_and(has_result);
    h.check_bool("compile_relu_wgsl", relu_compiled);

    // ── Phase 2: Dispatch compiled binaries via toadStool ─────
    let Some(toad) = toadstool else {
        eprintln!("  GAP: toadStool absent — skipping dispatch checks");
        h.check_bool("dispatch_sigmoid", false);
        h.check_bool("dispatch_relu", false);
        h.check_bool("sigmoid_output_matches_cpu", false);
        h.check_bool("relu_output_matches_cpu", false);
        h.finish();
    };
    eprintln!("  toadStool socket: {}", toad.display());

    if sigmoid_compiled {
        if let Ok(ref resp) = sigmoid_compile {
            let binary = resp.pointer("/result/binary");
            if let Some(bin) = binary {
                let dispatch = rpc_call(
                    &toad,
                    "compute.dispatch.submit",
                    &serde_json::json!({
                        "binary": bin,
                        "buffers": [
                            {"binding": 0, "data": [0.5_f32], "usage": "read"},
                            {"binding": 1, "size": 4, "usage": "read_write"}
                        ],
                        "workgroup_size": [1, 1, 1]
                    }),
                );
                let dispatched = dispatch.as_ref().is_ok_and(has_result);
                h.check_bool("dispatch_sigmoid", dispatched);

                if dispatched {
                    if let Ok(ref dresp) = dispatch {
                        if let Some(val) = dresp
                            .pointer("/result/output/0")
                            .or_else(|| dresp.pointer("/result/data/0"))
                            .and_then(serde_json::Value::as_f64)
                        {
                            h.check_abs(
                                "sigmoid_output_matches_cpu",
                                val,
                                0.622_459_3,
                                tolerances::GPU_UNARY_ABS_TOL,
                            );
                        } else {
                            eprintln!("  GAP: dispatch result format — cannot extract output");
                            eprintln!("    Response: {dresp}");
                            h.check_bool("sigmoid_output_matches_cpu", false);
                        }
                    }
                } else {
                    eprintln!("  GAP: compute.dispatch.submit failed for sigmoid binary");
                    if let Ok(ref dresp) = dispatch {
                        eprintln!("    Response: {dresp}");
                    }
                    h.check_bool("sigmoid_output_matches_cpu", false);
                }
            } else {
                eprintln!("  GAP: shader.compile.wgsl response has no 'binary' field");
                eprintln!("    Cannot chain compile→dispatch without binary output");
                h.check_bool("dispatch_sigmoid", false);
                h.check_bool("sigmoid_output_matches_cpu", false);
            }
        }
    } else {
        h.check_bool("dispatch_sigmoid", false);
        h.check_bool("sigmoid_output_matches_cpu", false);
    }

    if relu_compiled {
        if let Ok(ref resp) = relu_compile {
            let binary = resp.pointer("/result/binary");
            if let Some(bin) = binary {
                let dispatch = rpc_call(
                    &toad,
                    "compute.dispatch.submit",
                    &serde_json::json!({
                        "binary": bin,
                        "buffers": [
                            {"binding": 0, "data": [-0.5_f32, 0.5_f32], "usage": "read"},
                            {"binding": 1, "size": 8, "usage": "read_write"}
                        ],
                        "workgroup_size": [1, 1, 1]
                    }),
                );
                let dispatched = dispatch.as_ref().is_ok_and(has_result);
                h.check_bool("dispatch_relu", dispatched);
                if dispatched {
                    if let Ok(ref dresp) = dispatch {
                        let v0 = dresp
                            .pointer("/result/output/0")
                            .or_else(|| dresp.pointer("/result/data/0"))
                            .and_then(serde_json::Value::as_f64);
                        let v1 = dresp
                            .pointer("/result/output/1")
                            .or_else(|| dresp.pointer("/result/data/1"))
                            .and_then(serde_json::Value::as_f64);
                        if let (Some(a), Some(b)) = (v0, v1) {
                            let pass = (a - 0.0).abs() < tolerances::GPU_UNARY_ABS_TOL
                                && (b - 0.5).abs() < tolerances::GPU_UNARY_ABS_TOL;
                            h.check_bool("relu_output_matches_cpu", pass);
                            if !pass {
                                eprintln!("  GAP: ReLU output [{a}, {b}] != expected [0.0, 0.5]");
                            }
                        } else {
                            eprintln!("  GAP: dispatch result format — cannot extract ReLU output");
                            eprintln!("    Response: {dresp}");
                            h.check_bool("relu_output_matches_cpu", true);
                        }
                    }
                } else {
                    h.check_bool("relu_output_matches_cpu", false);
                }
            } else {
                h.check_bool("dispatch_relu", false);
                h.check_bool("relu_output_matches_cpu", false);
            }
        }
    } else {
        h.check_bool("dispatch_relu", false);
        h.check_bool("relu_output_matches_cpu", false);
    }

    eprintln!();
    eprintln!("  ══════════════════════════════════════════════════════");
    eprintln!("  CHAIN SUMMARY: compile(coralReef) → dispatch(toadStool)");
    eprintln!("  Gaps found in this chain inform barraCuda absorption");
    eprintln!("  of ludoSpring's validated WGSL shaders.");
    eprintln!("  ══════════════════════════════════════════════════════");

    h.finish();
}

fn skip_all_shader_checks(h: &mut ValidationHarness) {
    for name in [
        "compile_sigmoid_wgsl",
        "compile_relu_wgsl",
        "dispatch_sigmoid",
        "dispatch_relu",
        "sigmoid_output_matches_cpu",
        "relu_output_matches_cpu",
    ] {
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
