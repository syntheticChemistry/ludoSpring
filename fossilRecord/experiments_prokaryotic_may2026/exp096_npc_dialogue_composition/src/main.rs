// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp096 — NPC Dialogue via NUCLEUS Composition.
//!
//! Validates the RPGPT dialogue engine patterns (exp067-075 locally) via
//! NUCLEUS composition: Squirrel (AI) + barraCuda (math) + rhizoCrypt (DAG).
//!
//! # Composition chain
//!
//! Squirrel:    ai.query({prompt, constraints}) → NPC response
//! barraCuda:   math.sigmoid(skill_challenge_delta) → flow_state
//! barraCuda:   stats.weighted_mean(trust_actions) → trust_level
//! rhizoCrypt:  dag.event.append({session, action: "dialogue_exchange"})
//! Squirrel:    ai.query({prompt, temperature: 0.3}) → internal voice
//! petalTongue: visualization.render.scene({dialogue_tree}) (optional)
//!
//! # References
//!
//! - ludoSpring: exp067 (knowledge bounds), exp072 (trust), exp074 (dialogue flow)
//! - ludoSpring graph: `graphs/rpgpt_dialogue_engine.toml`
//! - ludoSpring capabilities: `game.npc_dialogue`, `game.voice_check`, `game.evaluate_flow`

use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (composition — RPGPT dialogue via NUCLEUS primals)",
    commit: "exp096-v1",
    date: "2026-03-30",
    command: "cargo run -p ludospring-exp096",
};

// Trust: weighted_mean([+1, +1, -5, +1, +1], equal weights) = -0.2
// Betrayal (-5) outweighs 4 positive (+1) interactions
const TRUST_ACTIONS: [f64; 5] = [1.0, 1.0, -5.0, 1.0, 1.0];
const TRUST_WEIGHTS: [f64; 5] = [0.2, 0.2, 0.2, 0.2, 0.2];
const TRUST_EXPECTED: f64 = -0.2;

// Flow: sigmoid(0.0) = 0.5 (balanced skill-challenge)
const FLOW_BALANCED_EXPECTED: f64 = 0.5;

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

fn discover_primal(prefix: &str) -> Option<PathBuf> {
    let dirs = ludospring_barracuda::niche::socket_dirs();
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
                    if n.starts_with(prefix)
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
    eprintln!("  Required primals not available — dry-run");
    for name in &[
        "squirrel_ai_reachable",
        "squirrel_npc_query",
        "barracuda_flow_sigmoid",
        "flow_balanced_tolerance",
        "barracuda_trust_weighted_mean",
        "trust_tolerance",
        "rhizocrypt_dialogue_event",
        "squirrel_voice_query",
        "dialogue_flow_hick_check",
        "petaltongue_scene_push",
    ] {
        h.check_bool(name, false);
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "validation harness with many sequential checks"
)]
fn cmd_validate() {
    let mut h = ValidationHarness::new("exp096_npc_dialogue_composition");
    h.print_provenance(&[&PROVENANCE]);

    let squirrel = discover_primal("squirrel");
    let barracuda = discover_barracuda_socket();
    let rhizocrypt = discover_primal("rhizocrypt");
    let petaltongue = discover_primal("petaltongue");

    eprintln!(
        "  squirrel:    {}",
        squirrel
            .as_ref()
            .map_or_else(|| "NOT FOUND".to_string(), |p| p.display().to_string(),)
    );
    eprintln!(
        "  barracuda:   {}",
        barracuda
            .as_ref()
            .map_or_else(|| "NOT FOUND".to_string(), |p| p.display().to_string(),)
    );
    eprintln!(
        "  rhizocrypt:  {}",
        rhizocrypt
            .as_ref()
            .map_or_else(|| "NOT FOUND".to_string(), |p| p.display().to_string(),)
    );
    eprintln!(
        "  petaltongue: {}",
        petaltongue
            .as_ref()
            .map_or_else(|| "NOT FOUND".to_string(), |p| p.display().to_string(),)
    );

    // barraCuda is required for science checks
    let Some(bc) = barracuda else {
        dry_mode(&mut h);
        h.finish();
    };

    // ── Squirrel: AI reachability + NPC query ────────────────
    if let Some(ref sq) = squirrel {
        let health = rpc_call(sq, "health.liveness", &serde_json::json!({}));
        h.check_bool(
            "squirrel_ai_reachable",
            health.as_ref().is_ok_and(has_result),
        );

        let npc_query = rpc_call(
            sq,
            "ai.query",
            &serde_json::json!({
                "prompt": "You are Maren, an innkeeper. A traveler asks about the old ruins. You know the ruins are dangerous but not why.",
                "max_tokens": 100
            }),
        );
        h.check_bool(
            "squirrel_npc_query",
            npc_query.as_ref().is_ok_and(has_result),
        );

        // Internal voice with temperature constraint
        let voice = rpc_call(
            sq,
            "ai.query",
            &serde_json::json!({
                "prompt": "Internal voice: Composure. Evaluate the situation calmly.",
                "temperature": 0.3,
                "max_tokens": 50
            }),
        );
        h.check_bool("squirrel_voice_query", voice.as_ref().is_ok_and(has_result));
    } else {
        h.check_bool("squirrel_ai_reachable", false);
        h.check_bool("squirrel_npc_query", false);
        h.check_bool("squirrel_voice_query", false);
    }

    // ── barraCuda: Flow evaluation via sigmoid ───────────────
    let flow = rpc_call(&bc, "math.sigmoid", &serde_json::json!({"data": [0.0]}));
    h.check_bool(
        "barracuda_flow_sigmoid",
        flow.as_ref().is_ok_and(has_result),
    );

    if let Some(val) = flow
        .as_ref()
        .ok()
        .and_then(|r| r.pointer("/result/result"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(serde_json::Value::as_f64)
    {
        h.check_abs(
            "flow_balanced_tolerance",
            val,
            FLOW_BALANCED_EXPECTED,
            tolerances::ANALYTICAL_TOL,
        );
    }

    // ── barraCuda: Trust accumulation via weighted_mean ──────
    let trust = rpc_call(
        &bc,
        "stats.weighted_mean",
        &serde_json::json!({"values": TRUST_ACTIONS, "weights": TRUST_WEIGHTS}),
    );
    h.check_bool(
        "barracuda_trust_weighted_mean",
        trust.as_ref().is_ok_and(has_result),
    );

    if let Some(val) = trust
        .as_ref()
        .ok()
        .and_then(|r| r.pointer("/result/result"))
        .and_then(serde_json::Value::as_f64)
    {
        h.check_abs(
            "trust_tolerance",
            val,
            TRUST_EXPECTED,
            tolerances::ANALYTICAL_TOL,
        );
    }

    // ── rhizoCrypt: Record dialogue event ────────────────────
    if let Some(ref rc) = rhizocrypt {
        let event = rpc_call(
            rc,
            "dag.event.append",
            &serde_json::json!({
                "session_id": "exp096-dialogue",
                "action": "dialogue_exchange",
                "npc": "Maren",
                "topic": "old_ruins"
            }),
        );
        h.check_bool(
            "rhizocrypt_dialogue_event",
            event.as_ref().is_ok_and(has_result),
        );
    } else {
        h.check_bool("rhizocrypt_dialogue_event", false);
    }

    // ── Dialogue flow: Hick's law check on option count ──────
    // With 4 dialogue options: Hick = a + b*log2(4+1) = a + b*2.32
    // This is a structural check — we verify math.log2 works for option count
    let hick_options = 4.0_f64 + 1.0;
    let log_resp = rpc_call(
        &bc,
        "math.log2",
        &serde_json::json!({"data": [hick_options]}),
    );
    h.check_bool(
        "dialogue_flow_hick_check",
        log_resp.as_ref().is_ok_and(|r| {
            r.pointer("/result/result")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(serde_json::Value::as_f64)
                .is_some_and(|v| (v - hick_options.log2()).abs() < tolerances::ANALYTICAL_TOL)
        }),
    );

    // ── petalTongue: Scene push (optional) ───────────────────
    if let Some(ref pt) = petaltongue {
        let scene = rpc_call(
            pt,
            "visualization.render.scene",
            &serde_json::json!({
                "scene_type": "dialogue_tree",
                "npc": "Maren",
                "options": ["Ask about ruins", "Ask about town", "Leave", "Bribe"]
            }),
        );
        h.check_bool(
            "petaltongue_scene_push",
            scene.as_ref().is_ok_and(has_result),
        );
    } else {
        h.check_bool("petaltongue_scene_push", false);
    }

    h.finish();
}

fn main() {
    cmd_validate();
}
