// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)] // validation binary — no public API
//! Composition validation — Layer 3: IPC parity against golden targets.
//!
//! Validates that ludoSpring's `game.*` methods return identical results
//! when called over JSON-RPC IPC as when called via direct Rust library.
//! Golden targets in `baselines/rust/composition_targets.json` are produced
//! by `generate_composition_targets` (Layer 2 baselines from direct Rust calls).
//!
//! # Provenance
//!
//! - **Targets**: `baselines/rust/composition_targets.json`
//! - **Generator**: `baselines/rust/generate_composition_targets.rs`
//! - **Layer 1**: Python baselines → validates Rust
//! - **Layer 2**: Rust library → produces composition targets
//! - **Layer 3**: Composition targets → validates IPC (this binary)
//!
//! # Exit codes
//!
//! - 0: all checks pass
//! - 1: one or more checks fail
//! - 2: ludoSpring server not running (skip)

use ludospring_barracuda::ipc::{PrimalEndpoint, call_primal, probe_socket};
use ludospring_barracuda::niche;
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, EXIT_SKIPPED, ValidationHarness};

use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// Golden targets — same file consumed by `generate_composition_targets` output.
///
/// Path is relative to this source file (`barracuda/src/bin/` → repo root).
const GOLDEN_JSON: &str = include_str!("../../../baselines/rust/composition_targets.json");

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/rust/composition_targets.json",
    commit: "(see repo when regenerated)",
    date: "2026-04-18",
    command: "cargo run --example generate_composition_targets --features ipc",
};

const GAME_METHODS: &[&str] = &[
    "game.evaluate_flow",
    "game.fitts_cost",
    "game.engagement",
    "game.generate_noise",
    "game.difficulty_adjustment",
    "game.accessibility",
];

struct TolSet {
    analytical: f64,
    game_state: f64,
    noise_mean: f64,
}

fn main() {
    let root: serde_json::Value = serde_json::from_str(GOLDEN_JSON).unwrap_or_else(|e| {
        eprintln!("FATAL: parse composition_targets.json: {e}");
        std::process::exit(1);
    });

    let tol = load_tolerances(&root).unwrap_or(TolSet {
        analytical: tolerances::ANALYTICAL_TOL,
        game_state: tolerances::GAME_STATE_TOL,
        noise_mean: tolerances::NOISE_MEAN_TOL,
    });

    let Some(socket_path) = discover_socket() else {
        eprintln!(
            "SKIP: ludoSpring IPC socket not found (set LUDOSPRING_SOCK or start ludospring-server)"
        );
        std::process::exit(EXIT_SKIPPED);
    };

    if UnixStream::connect(&socket_path).is_err() {
        eprintln!(
            "SKIP: cannot connect to ludoSpring at {} (server not listening?)",
            socket_path.display()
        );
        std::process::exit(EXIT_SKIPPED);
    }

    let endpoint = probe_socket(&socket_path).unwrap_or_else(|| PrimalEndpoint {
        socket: socket_path,
        name: niche::NICHE_NAME.to_string(),
        capabilities: Vec::new(),
    });

    let mut h = ValidationHarness::new("validate_composition");
    h.print_provenance(&[&PROVENANCE]);

    validate_health(&mut h, &endpoint);
    validate_lifecycle_composition(&mut h, &endpoint);

    for &method in GAME_METHODS {
        validate_method_cases(&mut h, method, &root, &tol, &endpoint);
    }

    std::process::exit(h.summary());
}

fn load_tolerances(root: &serde_json::Value) -> Option<TolSet> {
    let t = root.get("_provenance")?.get("tolerances")?;
    Some(TolSet {
        analytical: t.get("analytical")?.as_f64()?,
        game_state: t.get("game_state")?.as_f64()?,
        noise_mean: t.get("noise_mean")?.as_f64()?,
    })
}

fn discover_socket() -> Option<PathBuf> {
    let resolved = niche::resolve_server_socket();
    if resolved.exists() {
        return Some(resolved);
    }
    let sock_name = format!("{}-{}.sock", niche::NICHE_NAME, niche::family_id());
    for dir in niche::socket_dirs() {
        let c = dir.join(&sock_name);
        if c.exists() {
            return Some(c);
        }
    }
    let fallback_names = [
        format!("{}.sock", niche::NICHE_NAME),
        format!("{}-server.sock", niche::NICHE_NAME),
        format!("{}.sock", niche::NICHE_DOMAIN),
    ];
    for dir in niche::socket_dirs() {
        for name in &fallback_names {
            let c = dir.join(name);
            if c.exists() {
                return Some(c);
            }
        }
    }
    None
}

fn validate_health(h: &mut ValidationHarness, ep: &PrimalEndpoint) {
    match call_primal(ep, "health.liveness", &serde_json::json!({})) {
        Ok(v) => {
            let ok = v.get("status").and_then(serde_json::Value::as_str) == Some("alive");
            h.check_bool("health.liveness status=alive", ok);
        }
        Err(e) => {
            eprintln!("health.liveness: {e}");
            h.check_bool("health.liveness status=alive", false);
        }
    }
    match call_primal(ep, "health.readiness", &serde_json::json!({})) {
        Ok(v) => {
            let ready = v.get("ready").and_then(serde_json::Value::as_bool) == Some(true);
            h.check_bool("health.readiness ready=true", ready);
        }
        Err(e) => {
            eprintln!("health.readiness: {e}");
            h.check_bool("health.readiness ready=true", false);
        }
    }
}

fn validate_lifecycle_composition(h: &mut ValidationHarness, ep: &PrimalEndpoint) {
    match call_primal(ep, "lifecycle.composition", &serde_json::json!({})) {
        Ok(report) => {
            h.check_bool(
                "lifecycle.composition composition_model=pure",
                report
                    .get("composition_model")
                    .and_then(serde_json::Value::as_str)
                    == Some("pure"),
            );
            let frags: Vec<&str> = report
                .get("fragments")
                .and_then(serde_json::Value::as_array)
                .map(|arr| arr.iter().filter_map(serde_json::Value::as_str).collect())
                .unwrap_or_default();
            for needle in ["tower_atomic", "node_atomic", "nest_atomic", "meta_tier"] {
                let label = format!("lifecycle.composition fragments include {needle}");
                h.check_bool(&label, frags.contains(&needle));
            }
        }
        Err(e) => {
            eprintln!("lifecycle.composition: {e}");
            h.check_bool("lifecycle.composition composition_model=pure", false);
        }
    }
}

fn validate_method_cases(
    h: &mut ValidationHarness,
    method: &str,
    root: &serde_json::Value,
    tol: &TolSet,
    ep: &PrimalEndpoint,
) {
    let Some(block) = root.get(method).and_then(serde_json::Value::as_object) else {
        h.check_bool(&format!("{method}: golden section present"), false);
        return;
    };

    for (case_id, case) in block {
        let Some(case_obj) = case.as_object() else {
            continue;
        };
        let Some(params) = case_obj.get("params").cloned() else {
            h.check_bool(&format!("{method}/{case_id}: params object"), false);
            continue;
        };
        let Some(expected) = case_obj.get("expected") else {
            h.check_bool(&format!("{method}/{case_id}: expected object"), false);
            continue;
        };

        let mut ipc_params = params;
        if let Some(m) = case_obj.get("method").and_then(serde_json::Value::as_str) {
            if let Some(obj) = ipc_params.as_object_mut() {
                obj.insert("method".to_owned(), serde_json::Value::String(m.to_owned()));
            }
        }

        let label_prefix = format!("{method}/{case_id}");
        match call_primal(ep, method, &ipc_params) {
            Ok(result) => compare_case(h, method, &label_prefix, expected, &result, tol),
            Err(e) => {
                eprintln!("{label_prefix}: IPC {e}");
                h.check_bool(&format!("{label_prefix}: IPC call succeeded"), false);
            }
        }
    }
}

fn compare_case(
    h: &mut ValidationHarness,
    method: &str,
    lp: &str,
    expected: &serde_json::Value,
    result: &serde_json::Value,
    tol: &TolSet,
) {
    match method {
        "game.evaluate_flow" => cmp_flow(h, lp, expected, result, tol.analytical),
        "game.fitts_cost" => cmp_fitts(h, lp, expected, result, tol.analytical),
        "game.engagement" => cmp_engagement(h, lp, expected, result, tol.game_state),
        "game.generate_noise" => {
            cmp_f64(h, &format!("{lp} value"), result, "value", expected, "value", tol.noise_mean);
        }
        "game.difficulty_adjustment" => cmp_dda(h, lp, expected, result, tol.analytical),
        "game.accessibility" => cmp_accessibility(h, lp, expected, result, tol.game_state),
        _ => {}
    }
}

fn cmp_flow(
    h: &mut ValidationHarness,
    lp: &str,
    expected: &serde_json::Value,
    result: &serde_json::Value,
    tol: f64,
) {
    cmp_str(h, &format!("{lp} state"), result, "state", expected, "state");
    cmp_bool(h, &format!("{lp} in_flow"), result, "in_flow", expected, "in_flow");
    cmp_f64(h, &format!("{lp} flow_score"), result, "flow_score", expected, "flow_score", tol);
}

fn cmp_fitts(
    h: &mut ValidationHarness,
    lp: &str,
    expected: &serde_json::Value,
    result: &serde_json::Value,
    tol: f64,
) {
    for key in ["movement_time_ms", "index_of_difficulty", "reaction_time_ms", "steering_time_ms"] {
        if expected.get(key).is_some() {
            cmp_f64(h, &format!("{lp} {key}"), result, key, expected, key, tol);
        }
    }
}

fn cmp_engagement(
    h: &mut ValidationHarness,
    lp: &str,
    expected: &serde_json::Value,
    result: &serde_json::Value,
    tol: f64,
) {
    for key in ["actions_per_minute", "challenge_appetite", "composite", "deliberation", "persistence"] {
        if expected.get(key).is_some() {
            cmp_f64(h, &format!("{lp} {key}"), result, key, expected, key, tol);
        }
    }
    if expected.get("exploration_rate").is_some() {
        let exp_v = expected.get("exploration_rate").and_then(serde_json::Value::as_f64);
        let obs = result
            .get("exploration_rate")
            .or_else(|| result.get("exploration_ratio"))
            .and_then(serde_json::Value::as_f64);
        match (obs, exp_v) {
            (Some(a), Some(b)) => h.check_abs(&format!("{lp} exploration_rate"), a, b, tol),
            _ => h.check_bool(&format!("{lp} exploration_rate"), false),
        }
    }
}

fn cmp_dda(
    h: &mut ValidationHarness,
    lp: &str,
    expected: &serde_json::Value,
    result: &serde_json::Value,
    tol: f64,
) {
    for key in ["adjustment", "estimated_skill", "trend"] {
        cmp_f64(h, &format!("{lp} {key}"), result, key, expected, key, tol);
    }
}

fn cmp_accessibility(
    h: &mut ValidationHarness,
    lp: &str,
    expected: &serde_json::Value,
    result: &serde_json::Value,
    tol: f64,
) {
    cmp_f64(h, &format!("{lp} score"), result, "score", expected, "score", tol);
    if let Some(exp_issues) = expected.get("issues_count").and_then(serde_json::Value::as_u64) {
        let got = result
            .get("issues")
            .and_then(serde_json::Value::as_array)
            .map_or(u64::MAX, |a| a.len() as u64);
        h.check_bool(&format!("{lp} issues_count"), got == exp_issues);
    }
    if let Some(exp_s) = expected.get("strengths_count").and_then(serde_json::Value::as_u64) {
        let got = result
            .get("strengths")
            .and_then(serde_json::Value::as_array)
            .map_or(u64::MAX, |a| a.len() as u64);
        h.check_bool(&format!("{lp} strengths_count"), got == exp_s);
    }
}

fn cmp_str(
    h: &mut ValidationHarness,
    label: &str,
    result: &serde_json::Value,
    rkey: &str,
    expected: &serde_json::Value,
    ekey: &str,
) {
    let ro = result.get(rkey).and_then(serde_json::Value::as_str);
    let eo = expected.get(ekey).and_then(serde_json::Value::as_str);
    h.check_bool(label, ro == eo);
}

fn cmp_bool(
    h: &mut ValidationHarness,
    label: &str,
    result: &serde_json::Value,
    rkey: &str,
    expected: &serde_json::Value,
    ekey: &str,
) {
    let ro = result.get(rkey).and_then(serde_json::Value::as_bool);
    let eo = expected.get(ekey).and_then(serde_json::Value::as_bool);
    h.check_bool(label, ro == eo);
}

fn cmp_f64(
    h: &mut ValidationHarness,
    label: &str,
    result: &serde_json::Value,
    rkey: &str,
    expected: &serde_json::Value,
    ekey: &str,
    tolerance: f64,
) {
    let ro = result.get(rkey).and_then(serde_json::Value::as_f64);
    let eo = expected.get(ekey).and_then(serde_json::Value::as_f64);
    match (ro, eo) {
        (Some(a), Some(b)) => h.check_abs(label, a, b, tolerance),
        _ => h.check_bool(label, false),
    }
}
