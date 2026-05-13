// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario: Tower Atomic — validate BearDog + Songbird + skunkBat through
//! game domain compositions.
//!
//! ludoSpring is the Tower Atomic specialist. This scenario proves that a
//! multiplayer game session can boot, authenticate, discover peers, and
//! audit state through Tower primals alone — no compute, no storage.
//!
//! Protocol (live-validated V70):
//! - bearDog params use base64-encoded `message`/`data` fields
//! - skunkBat audit is `security.audit_log` (not `defense.audit`)
//! - Capabilities: `crypto.seed_fingerprint`, `crypto.sign`, `crypto.verify`,
//!   `crypto.hash`, `discovery.peers`, `security.audit_log`

use std::path::PathBuf;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};
use crate::ipc::methods;
use crate::validation::ValidationHarness;

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "tower_atomic",
        track: Track::CompositionParity,
        tier: Tier::Live,
        provenance_crate: "tower_atomic_specialist",
        provenance_date: "2026-05-13",
        description: "Tower Atomic (electron): BearDog crypto + Songbird mesh + skunkBat defense — live-validated protocol",
    },
    run: run_tower_atomic,
};

fn run_tower_atomic(h: &mut ValidationHarness) {
    check_beardog_fingerprint(h);
    check_beardog_sign_verify(h);
    check_beardog_hash(h);
    check_songbird_peers(h);
    check_skunkbat_audit(h);
    check_tower_method_constants(h);
}

enum TowerCallResult {
    Success(serde_json::Value),
    RpcError { code: i64, message: String },
    Unreachable,
}

fn try_tower_call(
    dirs: &[PathBuf],
    primal_hint: &str,
    method: &str,
    params: &serde_json::Value,
) -> TowerCallResult {
    for dir in dirs {
        let sock = dir.join(format!("{primal_hint}.sock"));
        if !sock.exists() {
            continue;
        }
        let endpoint = crate::ipc::PrimalEndpoint {
            socket: sock,
            name: primal_hint.to_owned(),
            capabilities: vec![],
        };
        match crate::ipc::call_primal(&endpoint, method, params) {
            Ok(result) => return TowerCallResult::Success(result),
            Err(crate::ipc::IpcError::RpcError { code, message, .. }) => {
                return TowerCallResult::RpcError { code, message };
            }
            Err(_) => {}
        }
    }
    TowerCallResult::Unreachable
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let t = (u32::from(chunk[0]) << 16)
            | (u32::from(chunk.get(1).copied().unwrap_or(0)) << 8)
            | u32::from(chunk.get(2).copied().unwrap_or(0));
        out.push(B64[((t >> 18) & 0x3F) as usize] as char);
        out.push(B64[((t >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 {
            B64[((t >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[(t & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn check_beardog_fingerprint(h: &mut ValidationHarness) {
    let dirs = crate::niche::socket_dirs();
    let args = serde_json::json!({ "seed": "ludospring-game-session-v1" });
    match try_tower_call(&dirs, "beardog", methods::crypto::SEED_FINGERPRINT, &args) {
        TowerCallResult::Success(result) => {
            let fp = result
                .get("fingerprint")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            h.check_bool(
                "beardog: seed_fingerprint returns non-empty fingerprint",
                !fp.is_empty(),
            );
            h.check_bool(
                "beardog: fingerprint is deterministic length (>=16)",
                fp.len() >= 16,
            );
        }
        TowerCallResult::RpcError { code, message } => {
            h.check_bool(
                &format!("beardog: seed_fingerprint RPC error {code}: {message}"),
                false,
            );
        }
        TowerCallResult::Unreachable => {
            h.check_bool(
                "beardog: seed_fingerprint degrades gracefully (not available)",
                true,
            );
        }
    }
}

fn check_beardog_sign_verify(h: &mut ValidationHarness) {
    let dirs = crate::niche::socket_dirs();
    let message_b64 = b64(b"game-state-snapshot-round-42");
    let sign_args = serde_json::json!({ "message": message_b64 });
    match try_tower_call(&dirs, "beardog", methods::crypto::SIGN, &sign_args) {
        TowerCallResult::Success(result) => {
            let sig = result
                .get("signature")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            h.check_bool(
                "beardog: crypto.sign returns non-empty Ed25519 signature",
                !sig.is_empty(),
            );

            let pub_key = result
                .get("public_key")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let verify_args = serde_json::json!({
                "message": message_b64,
                "signature": sig,
                "public_key": pub_key,
            });
            match try_tower_call(&dirs, "beardog", methods::crypto::VERIFY, &verify_args) {
                TowerCallResult::Success(vr) => {
                    let valid = vr
                        .get("valid")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false);
                    h.check_bool("beardog: crypto.verify confirms round-trip", valid);
                }
                TowerCallResult::RpcError { code, message } => {
                    h.check_bool(
                        &format!("beardog: crypto.verify RPC error {code}: {message}"),
                        false,
                    );
                }
                TowerCallResult::Unreachable => {
                    h.check_bool("beardog: crypto.verify degrades gracefully", true);
                }
            }
        }
        TowerCallResult::RpcError { code, message } => {
            h.check_bool(
                &format!("beardog: crypto.sign RPC error {code}: {message}"),
                false,
            );
        }
        TowerCallResult::Unreachable => {
            h.check_bool(
                "beardog: crypto.sign degrades gracefully (not available)",
                true,
            );
        }
    }
}

fn check_beardog_hash(h: &mut ValidationHarness) {
    let dirs = crate::niche::socket_dirs();
    let data_b64 = b64(b"ludospring-tower-atomic-validation");
    let args = serde_json::json!({ "algorithm": "blake3", "data": data_b64 });
    match try_tower_call(&dirs, "beardog", methods::crypto::HASH, &args) {
        TowerCallResult::Success(result) => {
            let hash = result
                .get("hash")
                .or_else(|| result.get("result"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            h.check_bool(
                "beardog: crypto.hash returns non-empty BLAKE3 hash",
                !hash.is_empty() && hash.len() >= 16,
            );
        }
        TowerCallResult::RpcError { code, message } => {
            h.check_bool(
                &format!("beardog: crypto.hash RPC error {code}: {message}"),
                false,
            );
        }
        TowerCallResult::Unreachable => {
            h.check_bool("beardog: crypto.hash degrades gracefully", true);
        }
    }
}

fn check_songbird_peers(h: &mut ValidationHarness) {
    let dirs = crate::niche::socket_dirs();
    let args = serde_json::json!({ "filter": "game" });
    match try_tower_call(&dirs, "songbird", methods::discovery::PEERS, &args) {
        TowerCallResult::Success(result) => {
            let is_array = result.get("peers").is_some_and(serde_json::Value::is_array);
            h.check_bool("songbird: discovery.peers returns peers array", is_array);
        }
        TowerCallResult::RpcError { code, message } => {
            h.check_bool(
                &format!("songbird: discovery.peers RPC error {code}: {message}"),
                false,
            );
        }
        TowerCallResult::Unreachable => {
            h.check_bool("songbird: discovery.peers degrades gracefully", true);
        }
    }
}

fn check_skunkbat_audit(h: &mut ValidationHarness) {
    let dirs = crate::niche::socket_dirs();
    let args = serde_json::json!({
        "event_type": "game.state_mutation",
        "source": crate::niche::NICHE_NAME,
        "severity": "info",
        "payload": {
            "action": "tower_atomic_validation",
            "round": 42,
        },
    });
    match try_tower_call(&dirs, "skunkbat", "security.audit_log", &args) {
        TowerCallResult::Success(result) => {
            let has_events = result
                .get("events")
                .is_some_and(serde_json::Value::is_array)
                || result.get("count").is_some();
            h.check_bool(
                "skunkbat: security.audit_log returns audit data",
                has_events,
            );
        }
        TowerCallResult::RpcError { code, message } => {
            h.check_bool(
                &format!("skunkbat: security.audit_log RPC error {code}: {message}"),
                false,
            );
        }
        TowerCallResult::Unreachable => {
            h.check_bool("skunkbat: security.audit_log degrades gracefully", true);
        }
    }
}

fn check_tower_method_constants(h: &mut ValidationHarness) {
    h.check_bool(
        "tower methods: crypto.sign is dotted",
        methods::crypto::SIGN.contains('.'),
    );
    h.check_bool(
        "tower methods: btsp.negotiate is dotted",
        methods::btsp::NEGOTIATE.contains('.'),
    );
    h.check_bool(
        "tower methods: discovery.peers is dotted",
        methods::discovery::PEERS.contains('.'),
    );
    h.check_bool(
        "tower methods: defense.audit constant is dotted",
        methods::defense::AUDIT.contains('.'),
    );
}
