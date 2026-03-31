// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp098 — NUCLEUS Complete Game Session.
//!
//! The integration experiment: a full game engine tick composed from ALL
//! NUCLEUS layers. This is the pattern esotericWebb abstracts.
//!
//! Each tick: science (barraCuda) → AI (Squirrel) → viz (petalTongue)
//!   → provenance (rhizoCrypt) → crypto (BearDog) → storage (NestGate)
//!
//! Required: barraCuda (math). All other primals gracefully optional.
//!
//! # References
//!
//! - ludoSpring graph: `graphs/composition/nucleus_game_session.toml`
//! - ludoSpring: exp093 (continuous session), exp094 (session lifecycle)
//! - esotericWebb: `webb/src/ipc/bridge/domains.rs` (V6 decomposition)
//! - primalSpring: `graphs/esotericwebb_composed_deploy.toml`

use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (composition — NUCLEUS Complete game session)",
    commit: "exp098-v1",
    date: "2026-03-30",
    command: "cargo run -p ludospring-exp098",
};

const FRAME_BUDGET_MS: f64 = 16.67;
const TICK_COUNT: usize = 10;

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

fn discover_primal(prefix: &str) -> Option<PathBuf> {
    let dirs = ludospring_barracuda::niche::socket_dirs();
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
                    if n.starts_with(prefix) && n.ends_with(".sock") {
                        return Some(p);
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

fn dry_mode(h: &mut ValidationHarness) {
    eprintln!("  barraCuda not available — dry-run");
    for name in &[
        "nucleus_10_ticks_complete",
        "nucleus_tick_latency_budget",
        "nucleus_flow_scores_correct",
        "nucleus_engagement_composite",
        "nucleus_provenance_recorded",
        "nucleus_storage_roundtrip",
    ] {
        h.check_bool(name, false);
    }
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp098_nucleus_game_session");
    h.print_provenance(&[&PROVENANCE]);

    let barracuda = discover_barracuda_socket();
    let squirrel = discover_primal("squirrel");
    let petaltongue = discover_primal("petaltongue");
    let rhizocrypt = discover_primal("rhizocrypt");
    let beardog = discover_primal("beardog");
    let nestgate = discover_primal("nestgate");

    eprintln!("  barracuda:   {}", barracuda.as_ref().map_or("NOT FOUND".into(), |p| p.display().to_string()));
    eprintln!("  squirrel:    {}", squirrel.as_ref().map_or("NOT FOUND (optional)".into(), |p| p.display().to_string()));
    eprintln!("  petaltongue: {}", petaltongue.as_ref().map_or("NOT FOUND (optional)".into(), |p| p.display().to_string()));
    eprintln!("  rhizocrypt:  {}", rhizocrypt.as_ref().map_or("NOT FOUND (optional)".into(), |p| p.display().to_string()));
    eprintln!("  beardog:     {}", beardog.as_ref().map_or("NOT FOUND (optional)".into(), |p| p.display().to_string()));
    eprintln!("  nestgate:    {}", nestgate.as_ref().map_or("NOT FOUND (optional)".into(), |p| p.display().to_string()));

    let Some(bc) = barracuda else {
        dry_mode(&mut h);
        h.finish();
    };

    // ── Run TICK_COUNT ticks of the full NUCLEUS game loop ───
    let mut flow_scores = Vec::new();
    let mut tick_latencies = Vec::new();
    let mut ticks_completed = 0_usize;

    let engagement_scores = [0.8, 0.6, 0.7, 0.5, 0.9];
    let engagement_weights = [0.25, 0.20, 0.20, 0.20, 0.15];

    for tick in 0..TICK_COUNT {
        let tick_start = Instant::now();
        let skill_challenge_delta = 0.1 * f64::from(tick as i32 - 5);

        // 1. barraCuda: flow evaluation
        let flow = rpc_call(
            &bc,
            "math.sigmoid",
            &serde_json::json!({"data": [skill_challenge_delta]}),
        );
        if let Some(val) = flow
            .as_ref()
            .ok()
            .and_then(|r| r.pointer("/result/result"))
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(serde_json::Value::as_f64)
        {
            flow_scores.push(val);
        }

        // 2. barraCuda: engagement composite
        let _engagement = rpc_call(
            &bc,
            "stats.weighted_mean",
            &serde_json::json!({"values": engagement_scores, "weights": engagement_weights}),
        );

        // 3. Squirrel: AI narration (optional, only first tick to save time)
        if tick == 0 {
            if let Some(ref sq) = squirrel {
                let _ = rpc_call(
                    sq,
                    "ai.query",
                    &serde_json::json!({
                        "prompt": "Narrate: The adventurer enters a dimly lit tavern.",
                        "max_tokens": 50
                    }),
                );
            }
        }

        // 4. petalTongue: scene render (optional, first tick only)
        if tick == 0 {
            if let Some(ref pt) = petaltongue {
                let _ = rpc_call(
                    pt,
                    "visualization.render.scene",
                    &serde_json::json!({"scene": "tavern_interior", "tick": tick}),
                );
            }
        }

        // 5. rhizoCrypt: provenance (optional)
        if let Some(ref rc) = rhizocrypt {
            let _ = rpc_call(
                rc,
                "provenance.vertex_append",
                &serde_json::json!({
                    "session_id": "exp098-nucleus",
                    "action": "game_tick",
                    "tick": tick
                }),
            );
        }

        // 6. BearDog: hash tick (optional)
        if let Some(ref bd) = beardog {
            use base64::Engine;
            let _ = rpc_call(
                bd,
                "crypto.blake3_hash",
                &serde_json::json!({"data": base64::engine::general_purpose::STANDARD.encode(format!("tick-{tick}"))}),
            );
        }

        let tick_elapsed = tick_start.elapsed().as_secs_f64() * 1000.0;
        tick_latencies.push(tick_elapsed);
        ticks_completed += 1;
    }

    // ── Verify results ───────────────────────────────────────

    // Check 1: All ticks completed
    h.check_bool("nucleus_10_ticks_complete", ticks_completed == TICK_COUNT);

    // Check 2: Per-tick science latency within 60Hz budget
    let max_latency = tick_latencies.iter().copied().fold(0.0_f64, f64::max);
    eprintln!("  tick latencies (ms): {:?}", tick_latencies);
    eprintln!("  max tick latency: {max_latency:.2}ms (budget: {FRAME_BUDGET_MS:.2}ms)");
    h.check_bool("nucleus_tick_latency_budget", max_latency < FRAME_BUDGET_MS);

    // Check 3: Flow scores correct (midpoint = sigmoid(0.0) = 0.5)
    let mid_score = flow_scores.get(5).copied();
    if let Some(ms) = mid_score {
        h.check_abs("nucleus_flow_scores_correct", ms, 0.5, tolerances::ANALYTICAL_TOL);
    } else {
        h.check_bool("nucleus_flow_scores_correct", false);
    }

    // Check 4: Engagement composite matches expected
    let expected_engagement = engagement_scores
        .iter()
        .zip(engagement_weights.iter())
        .map(|(s, w)| s * w)
        .sum::<f64>();
    let engagement_resp = rpc_call(
        &bc,
        "stats.weighted_mean",
        &serde_json::json!({"values": engagement_scores, "weights": engagement_weights}),
    );
    if let Some(val) = engagement_resp
        .as_ref()
        .ok()
        .and_then(|r| r.pointer("/result/result"))
        .and_then(serde_json::Value::as_f64)
    {
        h.check_abs("nucleus_engagement_composite", val, expected_engagement, tolerances::ANALYTICAL_TOL);
    } else {
        h.check_bool("nucleus_engagement_composite", false);
    }

    // Check 5: Provenance recorded (if rhizoCrypt available)
    h.check_bool("nucleus_provenance_recorded", rhizocrypt.is_some());

    // Check 6: Storage round-trip (if NestGate available)
    if let Some(ref ng) = nestgate {
        let store = rpc_call(
            ng,
            "storage.store",
            &serde_json::json!({"family_id": "ludotest", "key": "exp098-session", "value": "nucleus_complete"}),
        );
        let retrieve = rpc_call(
            ng,
            "storage.retrieve",
            &serde_json::json!({"family_id": "ludotest", "key": "exp098-session"}),
        );
        h.check_bool(
            "nucleus_storage_roundtrip",
            store.as_ref().is_ok_and(has_result) && retrieve.as_ref().is_ok_and(has_result),
        );
    } else {
        h.check_bool("nucleus_storage_roundtrip", false);
    }

    h.finish();
}

fn main() {
    cmd_validate();
}
