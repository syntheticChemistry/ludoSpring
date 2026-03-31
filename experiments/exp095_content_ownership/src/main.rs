// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp095 — Content Ownership via Provenance Trio Composition.
//!
//! Validates the Fermenting pattern (exp061/064 locally) via trio composition:
//! loamSpine (certificates) + rhizoCrypt (DAG) + sweetGrass (attribution) + BearDog (crypto).
//!
//! This is the "bounded game" foundation: every creative asset has ownership,
//! attribution, and a provenance chain. No ludoSpring binary — just primals.
//!
//! # Composition chain
//!
//! loamSpine:  certificate.mint({type: "ferment", owner, data})
//! rhizoCrypt: dag.event.append({cert_id, action: "trade"})
//! sweetGrass: attribution.record({cert_id, contributor, share}) x2
//! BearDog:    crypto.sign_ed25519(ownership_chain_digest)
//! sweetGrass: attribution.query({cert_id}) → verify shares sum to 1.0
//!
//! # References
//!
//! - ludoSpring: exp061 (Fermenting), exp064 (BearDog signing), exp066 (radiating attribution)
//! - ludoSpring capability: `game.mint_certificate`
//! - ludoSpring graph: `graphs/composition/session_provenance.toml`

use base64::Engine;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (composition — Provenance Trio content ownership)",
    commit: "exp095-v1",
    date: "2026-03-30",
    command: "cargo run -p ludospring-exp095",
};

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

fn dry_mode(h: &mut ValidationHarness) {
    eprintln!("  Required primals not available — dry-run");
    for name in &[
        "loamspine_mint_certificate",
        "rhizocrypt_trade_event",
        "sweetgrass_attribution_creator_a",
        "sweetgrass_attribution_creator_b",
        "beardog_sign_ownership",
        "sweetgrass_query_shares",
        "attribution_shares_sum_to_one",
        "loamspine_query_certificate",
    ] {
        h.check_bool(name, false);
    }
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp095_content_ownership");
    h.print_provenance(&[&PROVENANCE]);

    let beardog = discover_primal("beardog");
    let loamspine = discover_primal("loamspine");
    let rhizocrypt = discover_primal("rhizocrypt");
    let sweetgrass = discover_primal("sweetgrass");

    eprintln!("  beardog:    {}", beardog.as_ref().map_or("NOT FOUND".into(), |p| p.display().to_string()));
    eprintln!("  loamspine:  {}", loamspine.as_ref().map_or("NOT FOUND".into(), |p| p.display().to_string()));
    eprintln!("  rhizocrypt: {}", rhizocrypt.as_ref().map_or("NOT FOUND".into(), |p| p.display().to_string()));
    eprintln!("  sweetgrass: {}", sweetgrass.as_ref().map_or("NOT FOUND".into(), |p| p.display().to_string()));

    let (Some(bd), Some(ls), Some(rc), Some(sg)) = (beardog, loamspine, rhizocrypt, sweetgrass) else {
        dry_mode(&mut h);
        h.finish();
    };

    // ── Step 1: Mint a ferment certificate via loamSpine ─────
    let mint = rpc_call(
        &ls,
        "certificate.mint",
        &serde_json::json!({
            "cert_type": "ferment",
            "owner": "creator_a",
            "data": {"name": "Enchanted Sword", "rarity": "legendary"}
        }),
    );
    let mint_ok = mint.as_ref().is_ok_and(has_result);
    h.check_bool("loamspine_mint_certificate", mint_ok);

    let cert_id = mint
        .as_ref()
        .ok()
        .and_then(|r| r.pointer("/result/cert_id"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("fallback-cert-id")
        .to_string();

    // ── Step 2: Record trade event in rhizoCrypt DAG ─────────
    let trade = rpc_call(
        &rc,
        "dag.event.append",
        &serde_json::json!({
            "cert_id": cert_id,
            "action": "trade",
            "from": "creator_a",
            "to": "creator_b"
        }),
    );
    h.check_bool("rhizocrypt_trade_event", trade.as_ref().is_ok_and(has_result));

    // ── Step 3: Record attribution via sweetGrass ────────────
    let attr_a = rpc_call(
        &sg,
        "attribution.record",
        &serde_json::json!({
            "cert_id": cert_id,
            "contributor": "creator_a",
            "share": 0.7,
            "role": "original_creator"
        }),
    );
    h.check_bool("sweetgrass_attribution_creator_a", attr_a.as_ref().is_ok_and(has_result));

    let attr_b = rpc_call(
        &sg,
        "attribution.record",
        &serde_json::json!({
            "cert_id": cert_id,
            "contributor": "creator_b",
            "share": 0.3,
            "role": "modifier"
        }),
    );
    h.check_bool("sweetgrass_attribution_creator_b", attr_b.as_ref().is_ok_and(has_result));

    // ── Step 4: Sign ownership chain via BearDog ─────────────
    let sign = rpc_call(
        &bd,
        "crypto.sign_ed25519",
        &serde_json::json!({"message": base64::engine::general_purpose::STANDARD.encode(format!("{cert_id}:creator_a:0.7:creator_b:0.3"))}),
    );
    h.check_bool("beardog_sign_ownership", sign.as_ref().is_ok_and(has_result));

    // ── Step 5: Query attribution → verify shares sum to 1.0 ─
    let query = rpc_call(
        &sg,
        "attribution.query",
        &serde_json::json!({"cert_id": cert_id}),
    );
    h.check_bool("sweetgrass_query_shares", query.as_ref().is_ok_and(has_result));

    // Verify conservation: shares must sum to 1.0
    let shares_ok = query
        .as_ref()
        .ok()
        .and_then(|r| r.pointer("/result/shares"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            let sum: f64 = arr
                .iter()
                .filter_map(|s| s.get("share").and_then(serde_json::Value::as_f64))
                .sum();
            (sum - 1.0).abs() < 1e-10
        })
        .unwrap_or(false);
    h.check_bool("attribution_shares_sum_to_one", shares_ok);

    // ── Step 6: Query certificate still exists ───────────────
    let cert_query = rpc_call(
        &ls,
        "certificate.query",
        &serde_json::json!({"cert_id": cert_id}),
    );
    h.check_bool("loamspine_query_certificate", cert_query.as_ref().is_ok_and(has_result));

    h.finish();
}

fn main() {
    cmd_validate();
}
