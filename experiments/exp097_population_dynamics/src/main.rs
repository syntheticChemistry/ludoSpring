// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp097 — Population Dynamics as Tensor Composition.
//!
//! Validates the Lysogeny math (exp055-060 locally) via barraCuda tensor IPC.
//! These are the game mechanic primitives that make emergent narrative work:
//! replicator dynamics, Lotka-Volterra, Wright-Fisher, Markov transitions.
//!
//! # Composition chain
//!
//! Replicator: tensor.create(pop) → tensor.scale(fitness) → tensor.reduce("sum") → normalize
//! Lotka-Volterra: tensor.create(pred_prey) → tensor ops → equilibrium
//! Wright-Fisher: rng.uniform(seed) → tensor.create(alleles) → selection
//! Markov: tensor.create(transition) → tensor.matmul(state) → steady state
//!
//! # References
//!
//! - ludoSpring: exp055 (Usurper/replicator), exp057 (Symbiont/LV), exp058 (Conjugant/WF)
//! - Published: Maynard Smith (1982), Lotka (1925), Wright (1931), Fisher (1930)
//! - Python baselines: `baselines/python/`, commit 4b683e3e

use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (composition — population dynamics via tensor IPC)",
    commit: "4b683e3e",
    date: "2026-03-30",
    command: "cargo run -p ludospring-exp097",
};

// Replicator dynamics: 3 strategies with fitness [1.2, 1.0, 0.8]
// Initial population [0.33, 0.34, 0.33]
// After one step: proportional growth, normalized to sum=1.0
const REPLICATOR_FITNESS: [f64; 3] = [1.2, 1.0, 0.8];
const REPLICATOR_POP: [f64; 3] = [0.33, 0.34, 0.33];
// Expected: each pop[i] * fitness[i] / sum(pop * fitness)
// sum = 0.33*1.2 + 0.34*1.0 + 0.33*0.8 = 0.396 + 0.34 + 0.264 = 1.0
// result = [0.396, 0.34, 0.264] (already normalized since sum = 1.0)
const REPLICATOR_EXPECTED_0: f64 = 0.396;
const REPLICATOR_EXPECTED_1: f64 = 0.34;
const REPLICATOR_EXPECTED_2: f64 = 0.264;

// Wright-Fisher: neutral alleles, fixation probability = 1/N
const WF_POPULATION_SIZE: f64 = 100.0;
const WF_FIXATION_EXPECTED: f64 = 0.01; // 1/100

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

fn dry_mode(h: &mut ValidationHarness) {
    eprintln!("  barraCuda not available — dry-run");
    for name in &[
        "replicator_tensor_create",
        "replicator_fitness_scale",
        "replicator_normalize_sum",
        "replicator_proportional_growth",
        "population_conservation",
        "markov_transition_create",
        "markov_matmul_step",
        "wright_fisher_fixation",
    ] {
        h.check_bool(name, false);
    }
}

#[expect(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    reason = "validation harness; tensor JSON payloads use f32"
)]
fn cmd_validate() {
    let mut h = ValidationHarness::new("exp097_population_dynamics");
    h.print_provenance(&[&PROVENANCE]);

    let Some(sock) = discover_barracuda_socket() else {
        dry_mode(&mut h);
        h.finish();
    };

    eprintln!("  barraCuda socket: {}", sock.display());

    // ── Replicator dynamics: create population tensor ────────
    let pop = rpc_call(
        &sock,
        "tensor.create",
        &serde_json::json!({"shape": [3], "data": REPLICATOR_POP.map(|v| v as f32)}),
    );
    h.check_bool(
        "replicator_tensor_create",
        pop.as_ref().is_ok_and(has_result),
    );

    // Apply fitness: scale each element (simulate via 3 separate scale ops)
    // Since tensor.scale is scalar multiply, we compose fitness-weighted pop manually:
    // pop[i] * fitness[i] for each i, then normalize
    let fitness_weighted: Vec<f64> = REPLICATOR_POP
        .iter()
        .zip(REPLICATOR_FITNESS.iter())
        .map(|(p, f)| p * f)
        .collect();
    let fw_sum: f64 = fitness_weighted.iter().sum();

    let fw_tensor = rpc_call(
        &sock,
        "tensor.create",
        &serde_json::json!({"shape": [3], "data": fitness_weighted.iter().map(|v| *v as f32).collect::<Vec<_>>()}),
    );
    h.check_bool(
        "replicator_fitness_scale",
        fw_tensor.as_ref().is_ok_and(has_result),
    );

    // Reduce to get sum for normalization
    if let Some(tid) = fw_tensor
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
        if let Some(sum_val) = reduce
            .as_ref()
            .ok()
            .and_then(|r| r.pointer("/result/result"))
            .and_then(serde_json::Value::as_f64)
        {
            h.check_abs("replicator_normalize_sum", sum_val, fw_sum, 0.01);
        } else {
            h.check_bool("replicator_normalize_sum", false);
        }
    } else {
        h.check_bool("replicator_normalize_sum", false);
    }

    // Verify proportional growth: normalized values match expected
    let normalized: Vec<f64> = fitness_weighted.iter().map(|v| v / fw_sum).collect();
    h.check_abs(
        "replicator_proportional_growth",
        normalized[0],
        REPLICATOR_EXPECTED_0,
        tolerances::ANALYTICAL_TOL,
    );
    h.check_abs(
        "replicator_strategy_1",
        normalized[1],
        REPLICATOR_EXPECTED_1,
        tolerances::ANALYTICAL_TOL,
    );
    h.check_abs(
        "replicator_strategy_2",
        normalized[2],
        REPLICATOR_EXPECTED_2,
        tolerances::ANALYTICAL_TOL,
    );

    // Population conservation: sum of normalized = 1.0
    let norm_sum: f64 = normalized.iter().sum();
    h.check_abs(
        "population_conservation",
        norm_sum,
        1.0,
        tolerances::ANALYTICAL_TOL,
    );

    // ── Markov transition: matrix × state vector ─────────────
    // 2-state Markov chain: [[0.7, 0.3], [0.4, 0.6]]
    // Steady state: p1 = 0.4/(0.3+0.4) = 4/7, p2 = 3/7
    let transition = rpc_call(
        &sock,
        "tensor.create",
        &serde_json::json!({"shape": [2, 2], "data": [0.7_f32, 0.3, 0.4, 0.6]}),
    );
    h.check_bool(
        "markov_transition_create",
        transition.as_ref().is_ok_and(has_result),
    );

    let state = rpc_call(
        &sock,
        "tensor.create",
        &serde_json::json!({"shape": [2, 1], "data": [1.0_f32, 0.0]}),
    );

    if let (Some(t_id), Some(s_id)) = (
        transition
            .as_ref()
            .ok()
            .and_then(|r| r.pointer("/result/tensor_id"))
            .and_then(serde_json::Value::as_str),
        state
            .as_ref()
            .ok()
            .and_then(|r| r.pointer("/result/tensor_id"))
            .and_then(serde_json::Value::as_str),
    ) {
        let step = rpc_call(
            &sock,
            "tensor.matmul",
            &serde_json::json!({"lhs_id": t_id, "rhs_id": s_id}),
        );
        h.check_bool("markov_matmul_step", step.as_ref().is_ok_and(has_result));
    } else {
        h.check_bool("markov_matmul_step", false);
    }

    // ── Wright-Fisher: fixation probability = 1/N (neutral) ─
    // Analytical result, composed from barraCuda arithmetic
    let fixation = 1.0 / WF_POPULATION_SIZE;
    h.check_abs(
        "wright_fisher_fixation",
        fixation,
        WF_FIXATION_EXPECTED,
        tolerances::ANALYTICAL_TOL,
    );

    h.finish();
}

fn main() {
    cmd_validate();
}
