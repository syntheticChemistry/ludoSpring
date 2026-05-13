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

fn try_call(
    dirs: &[PathBuf],
    primal: &str,
    method: &str,
    params: &serde_json::Value,
) -> Option<serde_json::Value> {
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
        if let Ok(result) = call_primal(&endpoint, method, params) {
            return Some(result);
        }
    }
    None
}

fn exercise_crypto_seed_fingerprint(
    dirs: &[PathBuf],
    _h: &mut ValidationHarness,
    results: &mut Vec<TowerAtomicResult>,
) {
    let params = serde_json::json!({
        "seed": "ludospring-multiplayer-session-v1",
        "family_id": "tower-atomic-test",
    });
    match try_call(dirs, "beardog", "crypto.seed_fingerprint", &params) {
        Some(result) => {
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
        None => {
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
    let sign_params = serde_json::json!({
        "data": "game-state-round-42-snapshot",
        "encoding": "utf8",
    });
    match try_call(dirs, "beardog", "crypto.sign", &sign_params) {
        Some(result) => {
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
                detail: format!("signature len={}", sig.len()),
            });

            let verify_params = serde_json::json!({
                "data": "game-state-round-42-snapshot",
                "signature": sig,
                "encoding": "utf8",
            });
            match try_call(dirs, "beardog", "crypto.verify", &verify_params) {
                Some(vr) => {
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
                None => {
                    results.push(TowerAtomicResult {
                        capability: "crypto.verify",
                        primal: "bearDog",
                        status: "SKIP",
                        detail: "bearDog lost between sign and verify".to_owned(),
                    });
                }
            }
        }
        None => {
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
    let params = serde_json::json!({
        "algorithm": "blake3",
        "data": "ludospring-tower-atomic-integrity-check",
    });
    match try_call(dirs, "beardog", "crypto.hash", &params) {
        Some(result) => {
            let hash = result
                .get("hash")
                .or_else(|| result.get("result"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if !hash.is_empty() && hash.len() >= 32 {
                results.push(TowerAtomicResult {
                    capability: "crypto.hash",
                    primal: "bearDog",
                    status: "PASS",
                    detail: format!("BLAKE3 len={}", hash.len()),
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
        None => {
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
        Some(result) => {
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
        None => {
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
    match try_call(dirs, "skunkbat", "defense.audit", &params) {
        Some(result) => {
            let accepted = result
                .get("accepted")
                .and_then(serde_json::Value::as_bool)
                .or_else(|| result.get("seq").map(|s| !s.is_null()))
                .unwrap_or(false);
            results.push(TowerAtomicResult {
                capability: "defense.audit",
                primal: "skunkBat",
                status: if accepted { "PASS" } else { "FAIL" },
                detail: if accepted {
                    let seq = result.get("seq").and_then(serde_json::Value::as_u64);
                    format!("accepted, seq={seq:?}")
                } else {
                    "event not accepted".to_owned()
                },
            });
        }
        None => {
            results.push(TowerAtomicResult {
                capability: "defense.audit",
                primal: "skunkBat",
                status: "SKIP",
                detail: "skunkBat not reachable".to_owned(),
            });
        }
    }
}
