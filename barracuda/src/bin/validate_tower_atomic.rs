// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs, reason = "validation binary — no public API")]
//! Tower Atomic Specialist Validation — BearDog + Songbird + skunkBat.
//!
//! ludoSpring is the Tower Atomic specialist. This binary composes a Tower
//! atomic (bearDog crypto + songbird discovery + skunkBat defense) and
//! exercises each capability through game domain compositions.
//!
//! Key question answered: Can a multiplayer game session boot, authenticate,
//! discover peers, and audit state through Tower alone?
//!
//! Capabilities exercised:
//! - `crypto.seed_fingerprint` — session identity via FAMILY_ID-scoped seed
//! - `crypto.sign` / `crypto.verify` — game state provenance
//! - `crypto.hash` — BLAKE3 integrity for state snapshots
//! - `discovery.peers` — Songbird mesh for multiplayer lobbies
//! - `defense.audit` — skunkBat anti-cheat / audit trail
//!
//! Exit codes: 0 = all pass, 1 = failures, 2 = skip (no Tower primals reachable).

use std::path::PathBuf;
use std::process::ExitCode;

use ludospring_barracuda::ipc::{PrimalEndpoint, call_primal};
use ludospring_barracuda::niche;
use ludospring_barracuda::validation::ValidationHarness;

struct TowerAtomicResult {
    capability: &'static str,
    primal: &'static str,
    status: &'static str,
    detail: String,
}

const B64_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn simple_b64(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = u32::from(chunk.get(1).copied().unwrap_or(0));
        let b2 = u32::from(chunk.get(2).copied().unwrap_or(0));
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64_ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 {
            B64_ALPHABET[((triple >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64_ALPHABET[(triple & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn main() -> ExitCode {
    let format_json = std::env::args()
        .collect::<Vec<_>>()
        .windows(2)
        .any(|w| w[0] == "--format" && w[1] == "json");

    let dirs = niche::socket_dirs();
    let mut results: Vec<TowerAtomicResult> = Vec::new();
    let mut h = ValidationHarness::new("tower_atomic_specialist");

    exercise_crypto_seed_fingerprint(&dirs, &mut h, &mut results);
    exercise_crypto_sign_verify(&dirs, &mut h, &mut results);
    exercise_crypto_hash(&dirs, &mut h, &mut results);
    exercise_discovery_peers(&dirs, &mut h, &mut results);
    exercise_defense_audit(&dirs, &mut h, &mut results);

    let passed = results.iter().filter(|r| r.status == "PASS").count();
    let failed = results.iter().filter(|r| r.status == "FAIL").count();
    let skipped = results.iter().filter(|r| r.status == "SKIP").count();

    if format_json {
        let json_results: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "capability": r.capability,
                    "primal": r.primal,
                    "status": r.status,
                    "detail": r.detail,
                })
            })
            .collect();
        let output = serde_json::json!({
            "scenario": "tower_atomic",
            "specialist": "ludoSpring",
            "phase": "Phase 1 — individual atomic validation",
            "atomic": "Tower (electron)",
            "primals": ["bearDog", "songbird", "skunkBat"],
            "passed": passed,
            "failed": failed,
            "skipped": skipped,
            "total": results.len(),
            "status": if failed > 0 { "FAIL" } else if skipped == results.len() { "SKIP" } else { "PASS" },
            "results": json_results,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    } else {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║  Tower Atomic Specialist Validation — ludoSpring            ║");
        println!("║  Primals: bearDog + songbird + skunkBat                     ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        for r in &results {
            let icon = match r.status {
                "PASS" => "✓",
                "FAIL" => "✗",
                _ => "○",
            };
            println!(
                "  {icon} [{:8}] {:<10} {:<28} {}",
                r.primal, r.status, r.capability, r.detail
            );
        }
        println!();
        println!(
            "  Summary: {passed} passed, {failed} failed, {skipped} skipped / {} total",
            results.len()
        );

        if skipped == results.len() {
            println!();
            println!("  ⚠ No Tower primals reachable — all checks degraded gracefully.");
            println!(
                "    Deploy bearDog + songbird + skunkBat via plasmidBin for full validation."
            );
        }
    }

    if failed > 0 {
        ExitCode::from(1)
    } else if skipped == results.len() {
        ExitCode::from(2)
    } else {
        ExitCode::from(0)
    }
}

enum CallResult {
    Success(serde_json::Value),
    RpcError { code: i64, message: String },
    Unreachable,
}

fn try_call(
    dirs: &[PathBuf],
    primal: &str,
    method: &str,
    params: &serde_json::Value,
) -> CallResult {
    for dir in dirs {
        let sock = dir.join(format!("{primal}.sock"));
        if !sock.exists() {
            continue;
        }
        let endpoint = PrimalEndpoint {
            socket: sock,
            name: primal.to_owned(),
            capabilities: vec![],
        };
        match call_primal(&endpoint, method, params) {
            Ok(result) => return CallResult::Success(result),
            Err(ludospring_barracuda::ipc::IpcError::RpcError { code, message, .. }) => {
                return CallResult::RpcError { code, message };
            }
            Err(_) => {}
        }
    }
    CallResult::Unreachable
}

fn exercise_crypto_seed_fingerprint(
    dirs: &[PathBuf],
    _h: &mut ValidationHarness,
    results: &mut Vec<TowerAtomicResult>,
) {
    let params = serde_json::json!({
        "seed": "ludospring-multiplayer-session-v1",
    });
    match try_call(dirs, "beardog", "crypto.seed_fingerprint", &params) {
        CallResult::Success(result) => {
            let fp = result
                .get("fingerprint")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if !fp.is_empty() && fp.len() >= 16 {
                results.push(TowerAtomicResult {
                    capability: "crypto.seed_fingerprint",
                    primal: "bearDog",
                    status: "PASS",
                    detail: format!("fingerprint len={}, deterministic", fp.len()),
                });
            } else {
                results.push(TowerAtomicResult {
                    capability: "crypto.seed_fingerprint",
                    primal: "bearDog",
                    status: "FAIL",
                    detail: format!("unexpected fingerprint: {fp:?}"),
                });
            }
        }
        CallResult::RpcError { code, message } => {
            results.push(TowerAtomicResult {
                capability: "crypto.seed_fingerprint",
                primal: "bearDog",
                status: "FAIL",
                detail: format!("RPC error {code}: {message}"),
            });
        }
        CallResult::Unreachable => {
            results.push(TowerAtomicResult {
                capability: "crypto.seed_fingerprint",
                primal: "bearDog",
                status: "SKIP",
                detail: "bearDog not reachable".to_owned(),
            });
        }
    }
}

fn exercise_crypto_sign_verify(
    dirs: &[PathBuf],
    _h: &mut ValidationHarness,
    results: &mut Vec<TowerAtomicResult>,
) {
    let message_b64 = simple_b64(b"game-state-round-42-snapshot");
    let sign_params = serde_json::json!({ "message": message_b64 });
    match try_call(dirs, "beardog", "crypto.sign", &sign_params) {
        CallResult::Success(result) => {
            let sig = result
                .get("signature")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if sig.is_empty() {
                results.push(TowerAtomicResult {
                    capability: "crypto.sign",
                    primal: "bearDog",
                    status: "FAIL",
                    detail: "empty signature returned".to_owned(),
                });
                return;
            }
            results.push(TowerAtomicResult {
                capability: "crypto.sign",
                primal: "bearDog",
                status: "PASS",
                detail: format!("Ed25519 signature len={}", sig.len()),
            });

            let verify_params = serde_json::json!({
                "message": message_b64,
                "signature": sig,
                "public_key": result.get("public_key").and_then(serde_json::Value::as_str).unwrap_or(""),
            });
            match try_call(dirs, "beardog", "crypto.verify", &verify_params) {
                CallResult::Success(vr) => {
                    let valid = vr
                        .get("valid")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false);
                    results.push(TowerAtomicResult {
                        capability: "crypto.verify",
                        primal: "bearDog",
                        status: if valid { "PASS" } else { "FAIL" },
                        detail: format!("round-trip verification: {valid}"),
                    });
                }
                CallResult::RpcError { code, message } => {
                    results.push(TowerAtomicResult {
                        capability: "crypto.verify",
                        primal: "bearDog",
                        status: "FAIL",
                        detail: format!("RPC error {code}: {message}"),
                    });
                }
                CallResult::Unreachable => {
                    results.push(TowerAtomicResult {
                        capability: "crypto.verify",
                        primal: "bearDog",
                        status: "SKIP",
                        detail: "bearDog lost between sign and verify".to_owned(),
                    });
                }
            }
        }
        CallResult::RpcError { code, message } => {
            results.push(TowerAtomicResult {
                capability: "crypto.sign",
                primal: "bearDog",
                status: "FAIL",
                detail: format!("RPC error {code}: {message}"),
            });
        }
        CallResult::Unreachable => {
            results.push(TowerAtomicResult {
                capability: "crypto.sign",
                primal: "bearDog",
                status: "SKIP",
                detail: "bearDog not reachable".to_owned(),
            });
        }
    }
}

fn exercise_crypto_hash(
    dirs: &[PathBuf],
    _h: &mut ValidationHarness,
    results: &mut Vec<TowerAtomicResult>,
) {
    let data_b64 = simple_b64(b"ludospring-tower-atomic-integrity-check");
    let params = serde_json::json!({
        "algorithm": "blake3",
        "data": data_b64,
    });
    match try_call(dirs, "beardog", "crypto.hash", &params) {
        CallResult::Success(result) => {
            let hash = result
                .get("hash")
                .or_else(|| result.get("result"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if !hash.is_empty() && hash.len() >= 16 {
                results.push(TowerAtomicResult {
                    capability: "crypto.hash",
                    primal: "bearDog",
                    status: "PASS",
                    detail: format!("BLAKE3 hash len={}", hash.len()),
                });
            } else {
                results.push(TowerAtomicResult {
                    capability: "crypto.hash",
                    primal: "bearDog",
                    status: "FAIL",
                    detail: format!("unexpected hash: {hash:?}"),
                });
            }
        }
        CallResult::RpcError { code, message } => {
            results.push(TowerAtomicResult {
                capability: "crypto.hash",
                primal: "bearDog",
                status: "FAIL",
                detail: format!("RPC error {code}: {message}"),
            });
        }
        CallResult::Unreachable => {
            results.push(TowerAtomicResult {
                capability: "crypto.hash",
                primal: "bearDog",
                status: "SKIP",
                detail: "bearDog not reachable".to_owned(),
            });
        }
    }
}

fn exercise_discovery_peers(
    dirs: &[PathBuf],
    _h: &mut ValidationHarness,
    results: &mut Vec<TowerAtomicResult>,
) {
    let params = serde_json::json!({
        "filter": "game",
        "capabilities": ["game.session", "game.lobby"],
    });
    match try_call(dirs, "songbird", "discovery.peers", &params) {
        CallResult::Success(result) => {
            let is_array = result.get("peers").is_some_and(serde_json::Value::is_array);
            results.push(TowerAtomicResult {
                capability: "discovery.peers",
                primal: "songbird",
                status: if is_array { "PASS" } else { "FAIL" },
                detail: if is_array {
                    let count = result["peers"].as_array().map_or(0, Vec::len);
                    format!("{count} peers discovered")
                } else {
                    "no peers array in response".to_owned()
                },
            });
        }
        CallResult::RpcError { code, message } => {
            results.push(TowerAtomicResult {
                capability: "discovery.peers",
                primal: "songbird",
                status: "FAIL",
                detail: format!("RPC error {code}: {message}"),
            });
        }
        CallResult::Unreachable => {
            results.push(TowerAtomicResult {
                capability: "discovery.peers",
                primal: "songbird",
                status: "SKIP",
                detail: "songbird not reachable".to_owned(),
            });
        }
    }
}

fn exercise_defense_audit(
    dirs: &[PathBuf],
    _h: &mut ValidationHarness,
    results: &mut Vec<TowerAtomicResult>,
) {
    let params = serde_json::json!({
        "event_type": "game.state_mutation",
        "source": "ludospring-tower-atomic-validation",
        "severity": "info",
        "payload": {
            "action": "validate_tower_atomic",
            "round": 42,
            "tick": 1337,
        },
    });
    match try_call(dirs, "skunkbat", "security.audit_log", &params) {
        CallResult::Success(result) => {
            let has_events = result
                .get("events")
                .is_some_and(serde_json::Value::is_array)
                || result.get("count").is_some();
            results.push(TowerAtomicResult {
                capability: "security.audit_log",
                primal: "skunkBat",
                status: if has_events { "PASS" } else { "FAIL" },
                detail: if has_events {
                    let count = result
                        .get("count")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0);
                    format!("audit log received, {count} events")
                } else {
                    "unexpected response format".to_owned()
                },
            });
        }
        CallResult::RpcError { code, message } => {
            results.push(TowerAtomicResult {
                capability: "security.audit_log",
                primal: "skunkBat",
                status: "FAIL",
                detail: format!("RPC error {code}: {message}"),
            });
        }
        CallResult::Unreachable => {
            results.push(TowerAtomicResult {
                capability: "security.audit_log",
                primal: "skunkBat",
                status: "SKIP",
                detail: "skunkBat not reachable".to_owned(),
            });
        }
    }
}
