// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp099 — Composition Validation: Rust → IPC Parity.
//!
//! Validates that ludoSpring's science methods produce identical results
//! whether called as direct Rust library calls or via JSON-RPC IPC.
//!
//! This is the middle link of the three-layer validation chain:
//!
//! ```text
//! Python baseline → validates → Rust library code
//! Rust library    → validates → IPC composition   (THIS EXPERIMENT)
//! IPC composition → validates → NUCLEUS deployment (biomeOS graph)
//! ```
//!
//! Requires: ludoSpring server running on UDS (auto-discovered via niche).
//! Golden targets: `baselines/rust/composition_targets.json`
//!
//! # References
//!
//! - Rust targets generator: `baselines/rust/generate_composition_targets.rs`
//! - Python baselines: `baselines/python/combined_baselines.json`
//! - Proto-nucleate graph: `primalSpring:graphs/downstream/ludospring_proto_nucleate.toml`
//! - Composition graph: `graphs/composition/science_validation.toml`

use ludospring_barracuda::interaction::accessibility::{
    VisualAccessibilityFeatures, score_visual_accessibility,
};
use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::{evaluate_flow, flow_channel_metrics};
use ludospring_barracuda::interaction::input_laws::{
    fitts_index_of_difficulty, fitts_movement_time,
};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::procedural::noise::fbm_2d;
use ludospring_barracuda::procedural::wfc::{AdjacencyRules, WfcGrid};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (composition validation — Rust library vs IPC parity)",
    commit: "exp099-v1",
    date: "2026-04-10",
    command: "cargo run -p ludospring-exp099",
};

fn discover_ludospring_socket() -> Option<PathBuf> {
    let dirs = ludospring_barracuda::niche::socket_dirs();
    for dir in &dirs {
        for name in &["ludospring.sock", "ludospring-server.sock", "game.sock"] {
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

fn main() {
    let tol = tolerances::ANALYTICAL_TOL;
    let mut h = ValidationHarness::new("exp099_composition_validation");
    h.print_provenance(&[&PROVENANCE]);

    let Some(socket) = discover_ludospring_socket() else {
        eprintln!("[exp099] ludoSpring socket not found — dry_mode (all checks forced false).");
        mark_dry_mode(&mut h);
        std::process::exit(h.summary());
    };

    eprintln!("[exp099] Connected to ludoSpring at {}", socket.display());

    validate_flow(&mut h, &socket, tol);
    validate_fitts(&mut h, &socket, tol);
    validate_engagement(&mut h, &socket, tol);
    validate_noise(&mut h, &socket, tol);
    validate_dda(&mut h, &socket, tol);
    validate_accessibility(&mut h, &socket, tol);
    validate_wfc(&mut h, &socket);

    std::process::exit(h.summary());
}

fn mark_dry_mode(h: &mut ValidationHarness) {
    let names = [
        "flow_balanced",
        "flow_anxiety",
        "fitts_d100_w10",
        "fitts_d200_w20",
        "engagement_active",
        "engagement_idle",
        "noise_fbm_4oct",
        "dda_all_wins",
        "dda_all_losses",
        "dda_balanced",
        "accessibility_full",
        "accessibility_none",
        "wfc_4x4x3",
    ];
    for name in names {
        h.check_bool(name, false);
    }
}

// ── Flow ────────────────────────────────────────────────────────────

fn validate_flow(h: &mut ValidationHarness, socket: &Path, tol: f64) {
    let cases = [("flow_balanced", 0.5, 0.5), ("flow_anxiety", 0.9, 0.2)];

    for (name, challenge, skill) in cases {
        let width = tolerances::FLOW_CHANNEL_WIDTH;
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
                eprintln!("[exp099] {name}: {e}");
                h.check_bool(name, false);
            }
        }
    }
}

// ── Fitts ───────────────────────────────────────────────────────────

fn validate_fitts(h: &mut ValidationHarness, socket: &Path, tol: f64) {
    let a = tolerances::FITTS_A_MOUSE_MS;
    let b = tolerances::FITTS_B_MOUSE_MS;
    let cases = [
        ("fitts_d100_w10", 100.0, 10.0),
        ("fitts_d200_w20", 200.0, 20.0),
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
                eprintln!("[exp099] {name}: {e}");
                h.check_bool(name, false);
            }
        }
    }
}

// ── Engagement ──────────────────────────────────────────────────────

fn validate_engagement(h: &mut ValidationHarness, socket: &Path, tol: f64) {
    let cases: [(&str, EngagementSnapshot); 2] = [
        (
            "engagement_active",
            EngagementSnapshot {
                session_duration_s: 300.0,
                action_count: 100,
                exploration_breadth: 5,
                challenge_seeking: 3,
                retry_count: 10,
                deliberate_pauses: 5,
            },
        ),
        (
            "engagement_idle",
            EngagementSnapshot {
                session_duration_s: 600.0,
                action_count: 5,
                exploration_breadth: 1,
                challenge_seeking: 0,
                retry_count: 0,
                deliberate_pauses: 0,
            },
        ),
    ];

    for (name, snap) in &cases {
        let expected = compute_engagement(snap);

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
                h.check_bool(name, composite_ok && apm_ok);
            }
            Err(e) => {
                eprintln!("[exp099] {name}: {e}");
                h.check_bool(name, false);
            }
        }
    }
}

// ── Noise ───────────────────────────────────────────────────────────

fn validate_noise(h: &mut ValidationHarness, socket: &Path, tol: f64) {
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
            h.check_bool("noise_fbm_4oct", ok);
        }
        Err(e) => {
            eprintln!("[exp099] noise_fbm_4oct: {e}");
            h.check_bool("noise_fbm_4oct", false);
        }
    }
}

// ── DDA ─────────────────────────────────────────────────────────────

fn validate_dda(h: &mut ValidationHarness, socket: &Path, tol: f64) {
    let target = tolerances::DDA_TARGET_SUCCESS_RATE;

    let cases: [(&str, Vec<f64>); 3] = [
        ("dda_all_wins", vec![1.0; 10]),
        ("dda_all_losses", vec![0.0; 10]),
        (
            "dda_balanced",
            vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
        ),
    ];

    for (name, outcomes) in &cases {
        let mut window = PerformanceWindow::new(outcomes.len().max(1));
        for &o in outcomes {
            window.record(o);
        }
        let expected_adj = suggest_adjustment(&window, target);
        let expected_skill = window.estimated_skill();

        match rpc_call(
            socket,
            "game.difficulty_adjustment",
            &serde_json::json!({"outcomes": outcomes}),
        ) {
            Ok(result) => {
                let adj_ok = result["adjustment"]
                    .as_f64()
                    .is_some_and(|v| (v - expected_adj).abs() <= tol);
                let skill_ok = result["estimated_skill"]
                    .as_f64()
                    .is_some_and(|v| (v - expected_skill).abs() <= tol);
                h.check_bool(name, adj_ok && skill_ok);
            }
            Err(e) => {
                eprintln!("[exp099] {name}: {e}");
                h.check_bool(name, false);
            }
        }
    }
}

// ── Accessibility ───────────────────────────────────────────────────

fn validate_accessibility(h: &mut ValidationHarness, socket: &Path, tol: f64) {
    let cases: [(&str, VisualAccessibilityFeatures); 2] = [
        (
            "accessibility_full",
            VisualAccessibilityFeatures {
                audio_cues: true,
                descriptions: true,
                braille: true,
                haptic: true,
                color_independent: true,
                scalable_text: true,
            },
        ),
        (
            "accessibility_none",
            VisualAccessibilityFeatures {
                audio_cues: false,
                descriptions: false,
                braille: false,
                haptic: false,
                color_independent: false,
                scalable_text: false,
            },
        ),
    ];

    for (name, features) in &cases {
        let expected = score_visual_accessibility(features);

        match rpc_call(
            socket,
            "game.accessibility",
            &serde_json::json!({
                "audio_cues": features.audio_cues,
                "descriptions": features.descriptions,
                "braille": features.braille,
                "haptic": features.haptic,
                "color_independent": features.color_independent,
                "scalable_text": features.scalable_text,
            }),
        ) {
            Ok(result) => {
                let score_ok = result["score"]
                    .as_f64()
                    .is_some_and(|v| (v - expected.score).abs() <= tol);
                h.check_bool(name, score_ok);
            }
            Err(e) => {
                eprintln!("[exp099] {name}: {e}");
                h.check_bool(name, false);
            }
        }
    }
}

// ── WFC ─────────────────────────────────────────────────────────────

#[expect(
    clippy::cast_possible_truncation,
    reason = "WFC options_removed fits in usize on validation hosts"
)]
fn validate_wfc(h: &mut ValidationHarness, socket: &Path) {
    let (w, ht, n) = (4, 4, 3);
    let rules = AdjacencyRules::unconstrained(n);
    let mut grid = WfcGrid::new(w, ht, n);
    grid.collapse(1, 1, 0);
    let expected_removed = grid.propagate(&rules);

    match rpc_call(
        socket,
        "game.wfc_step",
        &serde_json::json!({"width": w, "height": ht, "n_tiles": n, "collapse": [1, 1, 0]}),
    ) {
        Ok(result) => {
            let ok = result["options_removed"]
                .as_u64()
                .is_some_and(|v| v as usize == expected_removed);
            h.check_bool("wfc_4x4x3", ok);
        }
        Err(e) => {
            eprintln!("[exp099] wfc_4x4x3: {e}");
            h.check_bool("wfc_4x4x3", false);
        }
    }
}
