// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp042 — Tower Atomic Local (crypto + discovery primals).
//!
//! Validates crypto primitives and IPC via capability-based discovery.
//! Discovers primals at runtime by probing socket directories for
//! `crypto.hash` and `system.ping` capabilities — never by hardcoded name.

use ludospring_barracuda::ipc::discovery;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Instant;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (runtime — crypto + discovery Tower Atomic)",
    commit: "19e402c0",
    date: "2026-04-10",
    command: "N/A (Unix socket RPC validation)",
};

fn rpc_call(
    socket_path: &std::path::Path,
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let mut stream = UnixStream::connect(socket_path)
        .map_err(|e| format!("connect {}: {e}", socket_path.display()))?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .map_err(|e| format!("timeout: {e}"))?;
    let payload = request.to_string();
    stream
        .write_all(payload.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|e| format!("shutdown write: {e}"))?;
    let mut buf = String::new();
    stream
        .read_to_string(&mut buf)
        .map_err(|e| format!("read: {e}"))?;
    serde_json::from_str(&buf).map_err(|e| format!("parse: {e}"))
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp042_tower_atomic_local");
    h.print_provenance(&[&PROVENANCE]);

    let registry = discovery::discover_primals();

    // --- Check 1: Discovered a primal with crypto.hash capability ---
    let crypto_endpoint = registry.find("crypto.hash");
    h.check_bool("crypto_capability_discovered", crypto_endpoint.is_some());

    // --- Check 2: Discovered a primal with system.ping capability ---
    let ping_endpoint = registry.find("system.ping");
    h.check_bool("ping_capability_discovered", ping_endpoint.is_some());

    // --- Check 3: Crypto primal responds to crypto.hash ---
    if let Some(ep) = crypto_endpoint {
        let hash_test = "aGVsbG8gd29ybGQ=";
        let hash_params = serde_json::json!({ "algorithm": "blake3", "data": hash_test });
        let start = Instant::now();
        if let Ok(resp) = rpc_call(&ep.socket, "crypto.hash", &hash_params) {
            let latency = start.elapsed().as_millis();
            let has_result = resp.get("result").is_some();
            let hash_val = resp
                .pointer("/result/hash")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            h.check_bool("crypto_rpc_responds", has_result);
            h.check_bool("blake3_hash_nonempty", !hash_val.is_empty());
            h.check_bool("crypto_latency_under_100ms", latency < 100);
        } else {
            h.check_bool("crypto_rpc_responds", false);
            h.check_bool("blake3_hash_nonempty", false);
            h.check_bool("crypto_latency_under_100ms", false);
        }

        // --- Check 4: Deterministic hashing ---
        let hash_params2 = serde_json::json!({ "algorithm": "blake3", "data": hash_test });
        let (h1, h2) = match (
            rpc_call(&ep.socket, "crypto.hash", &hash_params),
            rpc_call(&ep.socket, "crypto.hash", &hash_params2),
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
        h.check_bool("blake3_deterministic", !h1.is_empty() && h1 == h2);

        // --- Check 5: SHA3-256 ---
        let sha3_params = serde_json::json!({ "algorithm": "sha3-256", "data": hash_test });
        match rpc_call(&ep.socket, "crypto.hash", &sha3_params) {
            Ok(resp) => {
                let has_hash = resp
                    .pointer("/result/hash")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| !s.is_empty());
                h.check_bool("sha3_256_works", has_hash);
            }
            Err(_) => {
                h.check_bool("sha3_256_works", false);
            }
        }
    } else {
        h.check_bool("crypto_rpc_responds", false);
        h.check_bool("blake3_hash_nonempty", false);
        h.check_bool("crypto_latency_under_100ms", false);
        h.check_bool("blake3_deterministic", false);
        h.check_bool("sha3_256_works", false);
    }

    // --- Check 6: Discovery primal reachable ---
    if let Some(ep) = ping_endpoint {
        let sb_params = serde_json::json!({});
        match rpc_call(&ep.socket, "system.ping", &sb_params) {
            Ok(resp) => {
                let has_response = resp.get("result").is_some() || resp.get("error").is_some();
                h.check_bool("discovery_ipc_reachable", has_response);
            }
            Err(_) => {
                h.check_bool("discovery_ipc_reachable", false);
            }
        }
    } else {
        h.check_bool("discovery_ipc_reachable", false);
    }

    // --- Check 7: Tower Atomic = both capabilities discoverable ---
    h.check_bool(
        "tower_atomic_live",
        crypto_endpoint.is_some() && ping_endpoint.is_some(),
    );

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
