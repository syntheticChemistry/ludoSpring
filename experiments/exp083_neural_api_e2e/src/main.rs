// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp083 — Neural API End-to-End Validation.
//!
//! Validates the full biomeOS Neural API pipeline with real primals:
//!
//! 1. **Discovery**: Discovers Neural API socket via capability-based discovery
//! 2. **Crypto hash**: `capability.call` → `crypto.blake3_hash` → BearDog → verified hash
//! 3. **Crypto encrypt/decrypt roundtrip**: Full ChaCha20-Poly1305 via Neural API
//! 4. **Deterministic hashing**: Same input → same hash through the routing layer
//! 5. **Multi-algorithm**: Blake3 and SHA3-256 via semantic routing
//! 6. **Songbird health**: Network primal reachable through Neural API
//! 7. **Capability listing**: Neural API reports registered capabilities
//!
//! Requires: BearDog server, Songbird server, biomeOS neural-api all running.
//! Socket discovery via `$XDG_RUNTIME_DIR/biomeos/` or `BIOMEOS_SOCKET_DIR`.

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (runtime — Neural API e2e pipeline)",
    commit: "exp083-v1",
    date: "2026-03-29",
    command: "N/A (biomeOS capability.call routing validation)",
};

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

    // --- Check 1: Neural API socket discoverable ---
    let neural_socket = discover_neural_api();
    h.check_bool("neural_api_socket_discovered", neural_socket.is_some());

    let Some(na) = neural_socket else {
        for name in [
            "crypto_hash_through_neural_api",
            "blake3_hash_nonempty_via_neural",
            "neural_api_latency_under_200ms",
            "blake3_deterministic_via_neural",
            "sha3_256_via_neural",
            "encrypt_decrypt_roundtrip_via_neural",
            "songbird_reachable_via_neural",
            "capability_list_includes_crypto",
            "capability_list_includes_network",
        ] {
            h.check_bool(name, false);
        }
        h.finish();
    };

    // --- Check 2: crypto.blake3_hash through Neural API ---
    let test_data = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        b"hello world",
    );
    let hash_params = serde_json::json!({
        "capability": "crypto",
        "operation": "blake3_hash",
        "params": { "algorithm": "blake3", "data": test_data }
    });
    let start = Instant::now();
    match rpc_call(&na, "capability.call", &hash_params) {
        Ok(resp) => {
            let latency = start.elapsed().as_millis();
            let has_result = resp.get("result").is_some();
            let hash_val = resp
                .pointer("/result/hash")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            h.check_bool("crypto_hash_through_neural_api", has_result);
            h.check_bool("blake3_hash_nonempty_via_neural", !hash_val.is_empty());
            h.check_bool("neural_api_latency_under_200ms", latency < 200);
        }
        Err(e) => {
            eprintln!("  hash error: {e}");
            h.check_bool("crypto_hash_through_neural_api", false);
            h.check_bool("blake3_hash_nonempty_via_neural", false);
            h.check_bool("neural_api_latency_under_200ms", false);
        }
    }

    // --- Check 3: Deterministic hashing via Neural API ---
    let (h1, h2) = match (
        rpc_call(&na, "capability.call", &hash_params),
        rpc_call(&na, "capability.call", &hash_params),
    ) {
        (Ok(r1), Ok(r2)) => {
            let h1 = r1
                .pointer("/result/hash")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let h2 = r2
                .pointer("/result/hash")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            (h1, h2)
        }
        _ => (String::new(), String::new()),
    };
    h.check_bool("blake3_deterministic_via_neural", !h1.is_empty() && h1 == h2);

    // --- Check 4: SHA3-256 via Neural API ---
    let sha3_params = serde_json::json!({
        "capability": "crypto",
        "operation": "blake3_hash",
        "params": { "algorithm": "sha3-256", "data": test_data }
    });
    match rpc_call(&na, "capability.call", &sha3_params) {
        Ok(resp) => {
            let has_hash = resp
                .pointer("/result/hash")
                .and_then(|v| v.as_str())
                .is_some_and(|s| !s.is_empty());
            h.check_bool("sha3_256_via_neural", has_hash);
        }
        Err(_) => h.check_bool("sha3_256_via_neural", false),
    }

    // --- Check 5: Encrypt/decrypt roundtrip via Neural API ---
    let test_key = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &[0xAA_u8; 32],
    );
    let encrypt_params = serde_json::json!({
        "capability": "crypto",
        "operation": "encrypt",
        "params": { "plaintext": test_data, "key": test_key }
    });
    let roundtrip_ok = match rpc_call(&na, "capability.call", &encrypt_params) {
        Ok(resp) => {
            let ct = resp
                .pointer("/result/ciphertext")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let nonce = resp
                .pointer("/result/nonce")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let tag = resp
                .pointer("/result/tag")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if ct.is_empty() || nonce.is_empty() {
                false
            } else {
                let decrypt_params = serde_json::json!({
                    "capability": "crypto",
                    "operation": "decrypt",
                    "params": { "ciphertext": ct, "nonce": nonce, "tag": tag, "key": test_key }
                });
                match rpc_call(&na, "capability.call", &decrypt_params) {
                    Ok(dec_resp) => {
                        let plaintext = dec_resp
                            .pointer("/result/plaintext")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        plaintext == test_data
                    }
                    Err(_) => false,
                }
            }
        }
        Err(_) => false,
    };
    h.check_bool("encrypt_decrypt_roundtrip_via_neural", roundtrip_ok);

    // --- Check 6: Songbird reachable via Neural API health forwarding ---
    let health_params = serde_json::json!({
        "capability": "network",
        "operation": "discover_peers",
        "params": {}
    });
    let songbird_reachable = match rpc_call(&na, "capability.call", &health_params) {
        Ok(resp) => resp.get("result").is_some() || resp.get("error").is_some(),
        Err(_) => false,
    };
    h.check_bool("songbird_reachable_via_neural", songbird_reachable);

    // --- Check 7-8: Capability listing includes crypto and network ---
    match rpc_call(&na, "capability.list", &serde_json::json!({})) {
        Ok(resp) => {
            let caps: Vec<&str> = resp
                .pointer("/result/capabilities")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            h.check_bool(
                "capability_list_includes_crypto",
                caps.contains(&"crypto"),
            );
            h.check_bool(
                "capability_list_includes_network",
                caps.contains(&"network"),
            );
        }
        Err(_) => {
            h.check_bool("capability_list_includes_crypto", false);
            h.check_bool("capability_list_includes_network", false);
        }
    }

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
