// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp083 — Neural API End-to-End NUCLEUS Validation.
//!
//! Validates the full biomeOS Neural API pipeline with real primals
//! in a complete Nest Atomic composition (5 primals):
//!
//! **BearDog** (Crypto Tower):
//!   1. Blake3 hash via capability routing
//!   2. SHA3-256 multi-algorithm routing
//!   3. ChaCha20-Poly1305 authenticated encrypt/decrypt roundtrip
//!   4. Ed25519 digital signature (crypto.sign)
//!   5. Deterministic hashing verification
//!
//! **Songbird** (Discovery):
//!   6. Peer discovery via Neural API
//!
//! **ToadStool** (Compute):
//!   7. GPU/compute dispatch capabilities via Neural API
//!
//! **NestGate** (Storage):
//!   8. Store/retrieve roundtrip via Neural API
//!
//! **Squirrel** (AI/MCP):
//!   9. AI provider listing via Neural API
//!  10. Tool listing via Neural API
//!
//! **Cross-Domain**:
//!  11. Provenance chain: hash(BearDog) → sign(BearDog) → store(NestGate) → verify
//!  12. Capability registry completeness
//!
//! Requires: All 5 primals + biomeOS neural-api running.

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (runtime — NUCLEUS Nest Atomic e2e pipeline)",
    commit: "exp083-v2",
    date: "2026-03-29",
    command: "N/A (biomeOS capability.call — 5 primals x Neural API)",
};

const SKIP_CHECKS: &[&str] = &[
    "blake3_hash_via_neural",
    "sha3_256_via_neural",
    "chacha20_encrypt_decrypt_roundtrip",
    "ed25519_sign_via_neural",
    "blake3_deterministic_via_neural",
    "songbird_discovery_peers_via_neural",
    "toadstool_compute_caps_via_neural",
    "nestgate_store_retrieve_roundtrip",
    "squirrel_ai_providers_via_neural",
    "squirrel_tool_list_via_neural",
    "provenance_chain_integrity",
    "capability_registry_has_5_domains",
];

fn rpc_call(
    socket_path: &Path,
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let stream = UnixStream::connect(socket_path)
        .map_err(|e| format!("connect {}: {e}", socket_path.display()))?;
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

fn cap_call(
    na: &Path,
    capability: &str,
    operation: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    rpc_call(
        na,
        "capability.call",
        &serde_json::json!({
            "capability": capability,
            "operation": operation,
            "params": params
        }),
    )
}

fn result_field(resp: &serde_json::Value, field: &str) -> String {
    resp.pointer(&format!("/result/{field}"))
        .or_else(|| resp.pointer(&format!("/result/result/{field}")))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn discover_neural_api() -> Option<PathBuf> {
    let dirs = ludospring_barracuda::niche::socket_dirs();
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("neural-api") && name.ends_with(".sock") {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp083_neural_api_e2e");
    h.print_provenance(&[&PROVENANCE]);

    let neural_socket = discover_neural_api();
    h.check_bool("neural_api_socket_discovered", neural_socket.is_some());

    let Some(na) = neural_socket else {
        for name in SKIP_CHECKS {
            h.check_bool(name, false);
        }
        h.finish();
    };

    let test_data = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        b"NUCLEUS Nest Atomic validation",
    );
    let test_key =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &[0xAA_u8; 32]);

    // ── BearDog: Blake3 hash ───────────────────────────────────────────
    let start = Instant::now();
    let blake3_hash = match cap_call(
        &na,
        "crypto",
        "blake3_hash",
        serde_json::json!({ "data": test_data }),
    ) {
        Ok(resp) => {
            let _latency = start.elapsed();
            result_field(&resp, "hash")
        }
        Err(e) => {
            eprintln!("  blake3 error: {e}");
            String::new()
        }
    };
    h.check_bool("blake3_hash_via_neural", !blake3_hash.is_empty());

    // ── BearDog: SHA3-256 hash ─────────────────────────────────────────
    let sha3_ok = cap_call(
        &na,
        "crypto",
        "sha3_256",
        serde_json::json!({ "data": test_data }),
    )
    .map(|r| !result_field(&r, "hash").is_empty())
    .unwrap_or(false);
    h.check_bool("sha3_256_via_neural", sha3_ok);

    // ── BearDog: ChaCha20-Poly1305 encrypt/decrypt ─────────────────────
    let roundtrip_ok =
        (|| {
            let enc = cap_call(
                &na,
                "crypto",
                "chacha20_poly1305_encrypt",
                serde_json::json!({ "plaintext": test_data, "key": test_key }),
            )
            .ok()?;
            let ct = result_field(&enc, "ciphertext");
            let nonce = result_field(&enc, "nonce");
            let tag = result_field(&enc, "tag");
            if ct.is_empty() || nonce.is_empty() {
                return Some(false);
            }
            let dec = cap_call(
            &na, "crypto", "chacha20_poly1305_decrypt",
            serde_json::json!({ "ciphertext": ct, "nonce": nonce, "tag": tag, "key": test_key }),
        ).ok()?;
            Some(result_field(&dec, "plaintext") == test_data)
        })()
        .unwrap_or(false);
    h.check_bool("chacha20_encrypt_decrypt_roundtrip", roundtrip_ok);

    // ── BearDog: Ed25519 sign ──────────────────────────────────────────
    let signature = cap_call(
        &na,
        "crypto",
        "sign",
        serde_json::json!({ "message": test_data }),
    )
    .map(|r| result_field(&r, "signature"))
    .unwrap_or_default();
    h.check_bool("ed25519_sign_via_neural", !signature.is_empty());

    // ── BearDog: Deterministic hashing ─────────────────────────────────
    let h2 = cap_call(
        &na,
        "crypto",
        "blake3_hash",
        serde_json::json!({ "data": test_data }),
    )
    .map(|r| result_field(&r, "hash"))
    .unwrap_or_default();
    h.check_bool(
        "blake3_deterministic_via_neural",
        !blake3_hash.is_empty() && blake3_hash == h2,
    );

    // ── Songbird: Discovery peers ──────────────────────────────────────
    let peers_ok = cap_call(&na, "discovery", "peers", serde_json::json!({}))
        .map(|r| r.get("result").is_some())
        .unwrap_or(false);
    h.check_bool("songbird_discovery_peers_via_neural", peers_ok);

    // ── ToadStool: Compute dispatch capabilities ───────────────────────
    let compute_ok = cap_call(
        &na,
        "compute",
        "dispatch.capabilities",
        serde_json::json!({}),
    )
    .map(|r| r.get("result").is_some())
    .unwrap_or(false);
    h.check_bool("toadstool_compute_caps_via_neural", compute_ok);

    // ── NestGate: Store/retrieve roundtrip ─────────────────────────────
    let store_retrieve_ok = (|| {
        let val = serde_json::json!({"experiment": "exp083", "composition": "NUCLEUS"});
        cap_call(
            &na,
            "storage",
            "store",
            serde_json::json!({ "family_id": "dev0", "key": "exp083_test", "value": val }),
        )
        .ok()?;
        let ret = cap_call(
            &na,
            "storage",
            "retrieve",
            serde_json::json!({ "family_id": "dev0", "key": "exp083_test" }),
        )
        .ok()?;
        let stored_val = ret
            .pointer("/result/value")
            .or_else(|| ret.pointer("/result/result/value"));
        Some(stored_val.is_some())
    })()
    .unwrap_or(false);
    h.check_bool("nestgate_store_retrieve_roundtrip", store_retrieve_ok);

    // ── Squirrel: AI providers ─────────────────────────────────────────
    let ai_ok = cap_call(&na, "ai", "list_providers", serde_json::json!({}))
        .map(|r| r.get("result").is_some())
        .unwrap_or(false);
    h.check_bool("squirrel_ai_providers_via_neural", ai_ok);

    // ── Squirrel: Tool list ────────────────────────────────────────────
    let tools_ok = cap_call(&na, "tool", "list", serde_json::json!({}))
        .map(|r| r.get("result").is_some())
        .unwrap_or(false);
    h.check_bool("squirrel_tool_list_via_neural", tools_ok);

    // ── Cross-Domain: Provenance chain ─────────────────────────────────
    let provenance_ok =
        (|| {
            let hash = cap_call(
                &na,
                "crypto",
                "blake3_hash",
                serde_json::json!({ "data": test_data }),
            )
            .ok()
            .map(|r| result_field(&r, "hash"))?;
            if hash.is_empty() {
                return Some(false);
            }

            let sig = cap_call(
                &na,
                "crypto",
                "sign",
                serde_json::json!({ "message": hash }),
            )
            .ok()
            .map(|r| result_field(&r, "signature"))?;
            if sig.is_empty() {
                return Some(false);
            }

            let record =
                serde_json::json!({"hash": hash, "signature": sig, "algo": "blake3+ed25519"});
            cap_call(
            &na, "storage", "store",
            serde_json::json!({ "family_id": "dev0", "key": "provenance_exp083", "value": record }),
        ).ok()?;

            let ret = cap_call(
                &na,
                "storage",
                "retrieve",
                serde_json::json!({ "family_id": "dev0", "key": "provenance_exp083" }),
            )
            .ok()?;

            let val = ret
                .pointer("/result/value")
                .or_else(|| ret.pointer("/result/result/value"))?;
            let stored_hash = val.get("hash").and_then(|v| v.as_str()).unwrap_or("");
            let stored_sig = val.get("signature").and_then(|v| v.as_str()).unwrap_or("");
            Some(stored_hash == hash && stored_sig == sig)
        })()
        .unwrap_or(false);
    h.check_bool("provenance_chain_integrity", provenance_ok);

    // ── Capability registry completeness ───────────────────────────────
    let registry_ok = rpc_call(&na, "capability.list", &serde_json::json!({}))
        .map(|resp| {
            let caps_text = serde_json::to_string(&resp).unwrap_or_default();
            ["crypto", "storage", "compute", "ai", "discovery"]
                .iter()
                .all(|domain| caps_text.contains(domain))
        })
        .unwrap_or(false);
    h.check_bool("capability_registry_has_5_domains", registry_ok);

    h.finish();
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
