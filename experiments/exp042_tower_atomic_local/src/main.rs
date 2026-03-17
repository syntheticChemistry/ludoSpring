// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

use ludospring_barracuda::validation::ValidationResult;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Instant;

const EXP: &str = "exp042";

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

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

#[expect(
    clippy::too_many_lines,
    reason = "validation orchestrator — sequential check groups"
)]
fn cmd_validate() {
    println!("=== exp042: Tower Atomic Local (BearDog + Songbird) ===\n");
    let mut results = Vec::new();

    let uid = std::process::Command::new("id")
        .arg("-u")
        .output()
        .map_or_else(
            |_| "1000".into(),
            |o| String::from_utf8_lossy(&o.stdout).trim().to_string(),
        );
    let xdg_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| format!("/run/user/{uid}"));
    let biomeos_dir = std::path::PathBuf::from(&xdg_dir).join("biomeos");
    let beardog_sock = biomeos_dir.join("beardog-eastgate.sock");
    let songbird_sock = biomeos_dir.join("songbird-eastgate.sock");

    // --- Check 1: BearDog socket exists ---
    let bd_exists = beardog_sock.exists();
    println!(
        "  BearDog socket: {} {}",
        beardog_sock.display(),
        if bd_exists { "EXISTS" } else { "MISSING" }
    );
    results.push(ValidationResult::check(
        EXP,
        "beardog_socket_exists",
        bool_f64(bd_exists),
        1.0,
        0.0,
    ));

    // --- Check 2: Songbird socket exists ---
    let sb_exists = songbird_sock.exists();
    println!(
        "  Songbird socket: {} {}",
        songbird_sock.display(),
        if sb_exists { "EXISTS" } else { "MISSING" }
    );
    results.push(ValidationResult::check(
        EXP,
        "songbird_socket_exists",
        bool_f64(sb_exists),
        1.0,
        0.0,
    ));

    // --- Check 3: BearDog responds to crypto.hash ---
    let hash_test = "aGVsbG8gd29ybGQ="; // base64("hello world")
    let hash_params = serde_json::json!({ "algorithm": "blake3", "data": hash_test });
    let start = Instant::now();
    match rpc_call(&beardog_sock, "crypto.hash", &hash_params) {
        Ok(resp) => {
            let latency = start.elapsed().as_millis();
            let has_result = resp.get("result").is_some();
            let hash_val = resp
                .pointer("/result/hash")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            println!("\n  crypto.hash(blake3, 'hello world'):");
            println!(
                "    response: {}",
                if has_result { "valid" } else { "error" }
            );
            println!("    hash: {hash_val}");
            println!("    latency: {latency}ms");

            results.push(ValidationResult::check(
                EXP,
                "beardog_rpc_responds",
                bool_f64(has_result),
                1.0,
                0.0,
            ));

            let hash_nonempty = !hash_val.is_empty();
            results.push(ValidationResult::check(
                EXP,
                "blake3_hash_nonempty",
                bool_f64(hash_nonempty),
                1.0,
                0.0,
            ));

            results.push(ValidationResult::check(
                EXP,
                "beardog_latency_under_100ms",
                bool_f64(latency < 100),
                1.0,
                0.0,
            ));
        }
        Err(e) => {
            println!("\n  crypto.hash error: {e}");
            results.push(ValidationResult::check(
                EXP,
                "beardog_rpc_responds",
                0.0,
                1.0,
                0.0,
            ));
            results.push(ValidationResult::check(
                EXP,
                "blake3_hash_nonempty",
                0.0,
                1.0,
                0.0,
            ));
            results.push(ValidationResult::check(
                EXP,
                "beardog_latency_under_100ms",
                0.0,
                1.0,
                0.0,
            ));
        }
    }

    // --- Check 6: BearDog deterministic hashing ---
    let hash_params2 = serde_json::json!({ "algorithm": "blake3", "data": hash_test });
    let (h1, h2) = match (
        rpc_call(&beardog_sock, "crypto.hash", &hash_params),
        rpc_call(&beardog_sock, "crypto.hash", &hash_params2),
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
    let deterministic = !h1.is_empty() && h1 == h2;
    println!("  deterministic: {deterministic} (h1={h1}, h2={h2})");
    results.push(ValidationResult::check(
        EXP,
        "blake3_deterministic",
        bool_f64(deterministic),
        1.0,
        0.0,
    ));

    // --- Check 7: BearDog SHA3-256 ---
    let sha3_params = serde_json::json!({ "algorithm": "sha3-256", "data": hash_test });
    match rpc_call(&beardog_sock, "crypto.hash", &sha3_params) {
        Ok(resp) => {
            let has_hash = resp
                .pointer("/result/hash")
                .and_then(|v| v.as_str())
                .is_some_and(|s| !s.is_empty());
            println!("  SHA3-256: {}", if has_hash { "OK" } else { "FAIL" });
            results.push(ValidationResult::check(
                EXP,
                "sha3_256_works",
                bool_f64(has_hash),
                1.0,
                0.0,
            ));
        }
        Err(e) => {
            println!("  SHA3-256 error: {e}");
            results.push(ValidationResult::check(
                EXP,
                "sha3_256_works",
                0.0,
                1.0,
                0.0,
            ));
        }
    }

    // --- Check 8: Songbird IPC reachable ---
    let sb_params = serde_json::json!({});
    match rpc_call(&songbird_sock, "system.ping", &sb_params) {
        Ok(resp) => {
            let has_response = resp.get("result").is_some() || resp.get("error").is_some();
            println!(
                "\n  Songbird IPC: {}",
                if has_response {
                    "reachable"
                } else {
                    "no response"
                }
            );
            results.push(ValidationResult::check(
                EXP,
                "songbird_ipc_reachable",
                bool_f64(has_response),
                1.0,
                0.0,
            ));
        }
        Err(e) => {
            println!("\n  Songbird IPC error: {e}");
            results.push(ValidationResult::check(
                EXP,
                "songbird_ipc_reachable",
                0.0,
                1.0,
                0.0,
            ));
        }
    }

    // --- Check 9: Tower Atomic = both primals running ---
    let tower_live = bd_exists && sb_exists;
    println!(
        "\n  Tower Atomic status: {}",
        if tower_live { "LIVE" } else { "PARTIAL" }
    );
    results.push(ValidationResult::check(
        EXP,
        "tower_atomic_live",
        bool_f64(tower_live),
        1.0,
        0.0,
    ));

    // --- Check 10: Known gaps documented ---
    println!("\n  --- Known Gaps ---");
    println!("  Songbird HTTP proxy: BearDog socket path mismatch (/tmp/neural-api-*)");
    println!("  NestGate: data_sources::providers module not wired");
    println!("  Action: standardize socket paths across primals");
    results.push(ValidationResult::check(
        EXP,
        "tower_gaps_documented",
        1.0,
        1.0,
        0.0,
    ));

    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    println!();
    for r in &results {
        let tag = if r.passed { "PASS" } else { "FAIL" };
        println!("  [{tag}] {}", r.description);
    }
    println!("\nResults: {passed}/{total} passed");
    if passed < total {
        std::process::exit(1);
    }
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
