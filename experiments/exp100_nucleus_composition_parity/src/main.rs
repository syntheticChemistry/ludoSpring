// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp100 — NUCLEUS Composition Parity: Python → Rust → IPC → Primal.
//!
//! Three-layer validation chain proving that scientific game models
//! produce identical results across all execution modes:
//!
//! ```text
//! Layer 1: Python baseline    → validates → Rust library   (python_parity.rs)
//! Layer 2: Rust library       → validates → IPC dispatch   (exp099)
//! Layer 3: IPC dispatch       → validates → NUCLEUS primal composition (THIS)
//! ```
//!
//! Layer 3 validates the primal composition story: capabilities discovered
//! via biomeOS, called by capability (not identity), health probes passing,
//! and golden-value parity maintained through the full NUCLEUS graph.
//!
//! Exit codes follow the ecosystem convention:
//! - 0: all checks pass
//! - 1: one or more checks fail
//! - 2: skip (required primal not available — not a failure)
//!
//! # References
//!
//! - Proto-nucleate: `primalSpring/graphs/downstream/ludospring_proto_nucleate.toml`
//! - Python baselines: `baselines/python/combined_baselines.json`
//! - Rust targets: `baselines/rust/composition_targets.json`
//! - Composition graph: `graphs/composition/nucleus_game_session.toml`

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ludospring_barracuda::interaction::flow::{evaluate_flow, flow_channel_metrics};
use ludospring_barracuda::interaction::input_laws::{
    fitts_index_of_difficulty, fitts_movement_time,
};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::niche;
use ludospring_barracuda::procedural::noise::fbm_2d;
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (NUCLEUS composition parity — primal graph validation)",
    commit: "exp100-v1",
    date: "2026-04-10",
    command: "cargo run -p ludospring-exp100",
};

fn main() {
    let mut h = ValidationHarness::new("exp100_nucleus_composition_parity");
    h.print_provenance(&[&PROVENANCE]);
    let tol = tolerances::ANALYTICAL_TOL;

    eprintln!("[exp100] Three-layer validation: Python → Rust → IPC → NUCLEUS");
    eprintln!(
        "[exp100] Niche: {} (domain: {})",
        niche::NICHE_NAME,
        niche::NICHE_DOMAIN
    );
    eprintln!("[exp100] Capabilities: {}", niche::CAPABILITIES.len());

    // ── Layer 0: Self-knowledge integrity ───────────────────────────
    validate_niche_integrity(&mut h);

    // ── Layer 1: Discover ludoSpring primal via ecosystem conventions ─
    let Some(socket) = discover_primal() else {
        eprintln!("[exp100] ludoSpring not running — skip (exit 2)");
        mark_skip(&mut h);
        std::process::exit(2);
    };
    eprintln!("[exp100] Discovered ludoSpring at {}", socket.display());

    // ── Layer 2: Health probes (biomeOS discovery prerequisite) ──────
    validate_health_probes(&mut h, &socket);

    // ── Layer 3: Capability discovery ───────────────────────────────
    validate_capability_discovery(&mut h, &socket);

    // ── Layer 4: Science parity through NUCLEUS composition ─────────
    validate_science_parity(&mut h, &socket, tol);

    // ── Layer 5: Python→Rust→IPC golden chain ───────────────────────
    validate_golden_chain(&mut h, &socket, tol);

    let code = h.summary();
    eprintln!(
        "\n[exp100] Three-layer chain: {}",
        if code == 0 { "ALL PASS" } else { "FAILURES" }
    );
    std::process::exit(code);
}

// ── Niche Self-Knowledge ────────────────────────────────────────────

fn validate_niche_integrity(h: &mut ValidationHarness) {
    h.check_bool("niche_name", niche::NICHE_NAME == "ludospring");
    h.check_bool("niche_domain", niche::NICHE_DOMAIN == "game");
    h.check_bool("capabilities_count", niche::CAPABILITIES.len() == 27);
    h.check_bool(
        "semantic_mappings_count",
        niche::SEMANTIC_MAPPINGS.len() == 27,
    );

    let all_mapped = niche::SEMANTIC_MAPPINGS
        .iter()
        .all(|(_, fqn)| niche::CAPABILITIES.contains(fqn));
    h.check_bool("mappings_consistent", all_mapped);

    let deps = niche::operation_dependencies();
    let all_deps = niche::CAPABILITIES
        .iter()
        .all(|cap| deps.get(cap).is_some());
    h.check_bool("dependencies_complete", all_deps);

    let costs = niche::cost_estimates();
    let all_costs = niche::CAPABILITIES
        .iter()
        .all(|cap| costs.get(cap).is_some());
    h.check_bool("costs_complete", all_costs);
}

// ── Discovery ───────────────────────────────────────────────────────

fn discover_primal() -> Option<PathBuf> {
    let fid = niche::family_id();
    let sock_name = format!("{}-{fid}.sock", niche::NICHE_NAME);

    for dir in niche::socket_dirs() {
        let candidate = dir.join(&sock_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    let legacy_names = ["ludospring.sock", "ludospring-server.sock", "game.sock"];
    for dir in niche::socket_dirs() {
        for name in &legacy_names {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

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
        .set_read_timeout(Some(Duration::from_secs(tolerances::RPC_TIMEOUT_SECS)))
        .map_err(|e| format!("timeout: {e}"))?;
    let mut writer = stream.try_clone().map_err(|e| format!("clone: {e}"))?;
    let mut payload = serde_json::to_string(&request).map_err(|e| format!("ser: {e}"))?;
    payload.push('\n');
    writer
        .write_all(payload.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    writer.flush().map_err(|e| format!("flush: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| format!("read: {e}"))?;
    let parsed: serde_json::Value =
        serde_json::from_str(&response).map_err(|e| format!("parse: {e}"))?;

    if let Some(err) = parsed.get("error") {
        return Err(format!("RPC error: {err}"));
    }
    parsed
        .get("result")
        .cloned()
        .ok_or_else(|| "no result field".into())
}

// ── Health Probes ───────────────────────────────────────────────────

fn validate_health_probes(h: &mut ValidationHarness, socket: &Path) {
    match rpc_call(socket, "health.liveness", &serde_json::json!({})) {
        Ok(result) => {
            let alive = result["status"].as_str() == Some("alive");
            h.check_bool("health_liveness", alive);
        }
        Err(e) => {
            eprintln!("[exp100] health.liveness: {e}");
            h.check_bool("health_liveness", false);
        }
    }

    match rpc_call(socket, "health.readiness", &serde_json::json!({})) {
        Ok(result) => {
            let ready = result["ready"].as_bool() == Some(true);
            h.check_bool("health_readiness", ready);
        }
        Err(e) => {
            eprintln!("[exp100] health.readiness: {e}");
            h.check_bool("health_readiness", false);
        }
    }
}

// ── Capability Discovery ────────────────────────────────────────────

fn validate_capability_discovery(h: &mut ValidationHarness, socket: &Path) {
    match rpc_call(socket, "capability.list", &serde_json::json!({})) {
        Ok(result) => {
            let caps = result["capabilities"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v["name"].as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let all_present = niche::CAPABILITIES
                .iter()
                .all(|expected| caps.iter().any(|c| c == expected));
            h.check_bool("capability_list_complete", all_present);

            let has_game_science = caps.iter().any(|c| c == "game.evaluate_flow");
            h.check_bool("has_game_science", has_game_science);

            let has_gpu = caps.iter().any(|c| c == "game.gpu.fog_of_war");
            h.check_bool("has_gpu_caps", has_gpu);

            let has_health = caps.iter().any(|c| c == "health.liveness");
            h.check_bool("has_health_caps", has_health);
        }
        Err(e) => {
            eprintln!("[exp100] capability.list: {e}");
            h.check_bool("capability_list_complete", false);
            h.check_bool("has_game_science", false);
            h.check_bool("has_gpu_caps", false);
            h.check_bool("has_health_caps", false);
        }
    }
}

// ── Science Parity ──────────────────────────────────────────────────

fn validate_science_parity(h: &mut ValidationHarness, socket: &Path, tol: f64) {
    validate_flow_parity(h, socket, tol);
    validate_fitts_parity(h, socket, tol);
    validate_engagement_parity(h, socket, tol);
    validate_noise_parity(h, socket, tol);
}

fn validate_flow_parity(h: &mut ValidationHarness, socket: &Path, tol: f64) {
    let width = tolerances::FLOW_CHANNEL_WIDTH;
    let cases = [
        ("nucleus_flow_balanced", 0.5, 0.5),
        ("nucleus_flow_anxiety", 0.9, 0.2),
        ("nucleus_flow_boredom", 0.2, 0.9),
    ];

    for (name, challenge, skill) in cases {
        let expected_state = evaluate_flow(challenge, skill, width);
        let (expected_score, expected_in_flow) = flow_channel_metrics(challenge, skill, width);

        match rpc_call(
            socket,
            "game.evaluate_flow",
            &serde_json::json!({"challenge": challenge, "skill": skill}),
        ) {
            Ok(result) => {
                let state_ok = result["state"].as_str() == Some(expected_state.as_str());
                let score_ok = result["flow_score"]
                    .as_f64()
                    .is_some_and(|v| (v - expected_score).abs() <= tol);
                let in_flow_ok = result["in_flow"].as_bool() == Some(expected_in_flow);
                h.check_bool(name, state_ok && score_ok && in_flow_ok);
            }
            Err(e) => {
                eprintln!("[exp100] {name}: {e}");
                h.check_bool(name, false);
            }
        }
    }
}

fn validate_fitts_parity(h: &mut ValidationHarness, socket: &Path, tol: f64) {
    let a = tolerances::FITTS_A_MOUSE_MS;
    let b = tolerances::FITTS_B_MOUSE_MS;

    let cases = [
        ("nucleus_fitts_100_10", 100.0, 10.0),
        ("nucleus_fitts_200_20", 200.0, 20.0),
        ("nucleus_fitts_500_50", 500.0, 50.0),
    ];

    for (name, distance, target_width) in cases {
        let expected_mt = fitts_movement_time(distance, target_width, a, b);
        let expected_id = fitts_index_of_difficulty(distance, target_width);

        match rpc_call(
            socket,
            "game.fitts_cost",
            &serde_json::json!({"distance": distance, "target_width": target_width}),
        ) {
            Ok(result) => {
                let mt_ok = result["movement_time_ms"]
                    .as_f64()
                    .is_some_and(|v| (v - expected_mt).abs() <= tol);
                let id_ok = result["index_of_difficulty"]
                    .as_f64()
                    .is_some_and(|v| (v - expected_id).abs() <= tol);
                h.check_bool(name, mt_ok && id_ok);
            }
            Err(e) => {
                eprintln!("[exp100] {name}: {e}");
                h.check_bool(name, false);
            }
        }
    }
}

fn validate_engagement_parity(h: &mut ValidationHarness, socket: &Path, tol: f64) {
    let snap = EngagementSnapshot {
        session_duration_s: 300.0,
        action_count: 100,
        exploration_breadth: 5,
        challenge_seeking: 3,
        retry_count: 10,
        deliberate_pauses: 5,
    };
    let expected = compute_engagement(&snap);

    match rpc_call(
        socket,
        "game.engagement",
        &serde_json::json!({
            "session_duration_s": snap.session_duration_s,
            "action_count": snap.action_count,
            "exploration_breadth": snap.exploration_breadth,
            "challenge_seeking": snap.challenge_seeking,
            "retry_count": snap.retry_count,
            "deliberate_pauses": snap.deliberate_pauses,
        }),
    ) {
        Ok(result) => {
            let composite_ok = result["composite"]
                .as_f64()
                .is_some_and(|v| (v - expected.composite).abs() <= tol);
            let apm_ok = result["actions_per_minute"]
                .as_f64()
                .is_some_and(|v| (v - expected.actions_per_minute).abs() <= tol);
            h.check_bool("nucleus_engagement", composite_ok && apm_ok);
        }
        Err(e) => {
            eprintln!("[exp100] nucleus_engagement: {e}");
            h.check_bool("nucleus_engagement", false);
        }
    }
}

fn validate_noise_parity(h: &mut ValidationHarness, socket: &Path, tol: f64) {
    let expected = fbm_2d(1.23, 4.56, 4, 2.0, 0.5);

    match rpc_call(
        socket,
        "game.generate_noise",
        &serde_json::json!({"x": 1.23, "y": 4.56}),
    ) {
        Ok(result) => {
            let ok = result["value"]
                .as_f64()
                .is_some_and(|v| (v - expected).abs() <= tol);
            h.check_bool("nucleus_noise", ok);
        }
        Err(e) => {
            eprintln!("[exp100] nucleus_noise: {e}");
            h.check_bool("nucleus_noise", false);
        }
    }
}

// ── Golden Chain (Python→Rust→IPC round-trip) ───────────────────────

fn validate_golden_chain(h: &mut ValidationHarness, socket: &Path, tol: f64) {
    let py_flow_balanced_score = 1.0_f64;
    let width = tolerances::FLOW_CHANNEL_WIDTH;
    let (rust_score, _) = flow_channel_metrics(0.5, 0.5, width);

    let py_rust_match = (py_flow_balanced_score - rust_score).abs() <= tol;
    h.check_bool("golden_py_rust_flow", py_rust_match);

    match rpc_call(
        socket,
        "game.evaluate_flow",
        &serde_json::json!({"challenge": 0.5, "skill": 0.5}),
    ) {
        Ok(result) => {
            let ipc_score = result["flow_score"].as_f64().unwrap_or(f64::NAN);
            let rust_ipc_match = (rust_score - ipc_score).abs() <= tol;
            h.check_bool("golden_rust_ipc_flow", rust_ipc_match);

            let py_ipc_match = (py_flow_balanced_score - ipc_score).abs() <= tol;
            h.check_bool("golden_py_ipc_flow", py_ipc_match);
        }
        Err(e) => {
            eprintln!("[exp100] golden chain flow: {e}");
            h.check_bool("golden_rust_ipc_flow", false);
            h.check_bool("golden_py_ipc_flow", false);
        }
    }

    let py_fitts_id = (100.0_f64 / 10.0 + 1.0).log2();
    let rust_fitts_id = fitts_index_of_difficulty(100.0, 10.0);

    let py_rust_fitts = (py_fitts_id - rust_fitts_id).abs() <= tol;
    h.check_bool("golden_py_rust_fitts", py_rust_fitts);

    match rpc_call(
        socket,
        "game.fitts_cost",
        &serde_json::json!({"distance": 100.0, "target_width": 10.0}),
    ) {
        Ok(result) => {
            let ipc_id = result["index_of_difficulty"].as_f64().unwrap_or(f64::NAN);
            h.check_bool(
                "golden_rust_ipc_fitts",
                (rust_fitts_id - ipc_id).abs() <= tol,
            );
            h.check_bool("golden_py_ipc_fitts", (py_fitts_id - ipc_id).abs() <= tol);
        }
        Err(e) => {
            eprintln!("[exp100] golden chain fitts: {e}");
            h.check_bool("golden_rust_ipc_fitts", false);
            h.check_bool("golden_py_ipc_fitts", false);
        }
    }
}

fn mark_skip(h: &mut ValidationHarness) {
    let checks = [
        "niche_name",
        "niche_domain",
        "capabilities_count",
        "semantic_mappings_count",
        "mappings_consistent",
        "dependencies_complete",
        "costs_complete",
        "health_liveness",
        "health_readiness",
        "capability_list_complete",
        "has_game_science",
        "has_gpu_caps",
        "has_health_caps",
        "nucleus_flow_balanced",
        "nucleus_flow_anxiety",
        "nucleus_flow_boredom",
        "nucleus_fitts_100_10",
        "nucleus_fitts_200_20",
        "nucleus_fitts_500_50",
        "nucleus_engagement",
        "nucleus_noise",
        "golden_py_rust_flow",
        "golden_rust_ipc_flow",
        "golden_py_ipc_flow",
        "golden_py_rust_fitts",
        "golden_rust_ipc_fitts",
        "golden_py_ipc_fitts",
    ];
    for name in checks {
        h.check_bool(name, false);
    }
}
