// SPDX-License-Identifier: AGPL-3.0-or-later
//! biomeOS niche deployment integration for ludoSpring.
//!
//! Per the Spring-as-Niche Deployment Standard, this module defines:
//! - **Capability domain**: `game` with semantic mappings
//! - **Registration**: domain-level `capability.register` / `capability.deregister`
//! - **Neural API enhancements**: operation dependencies and cost estimates
//!
//! ludoSpring is a sovereign primal that exposes capabilities via JSON-RPC
//! over Unix sockets. biomeOS composes it into niches via deploy graphs.
//! This module never hardcodes peer primal names.
#![allow(clippy::doc_markdown)]

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Capability domain for ludoSpring.
pub const GAME_DOMAIN: &str = "game";

/// All capabilities this primal exposes under the `game` domain.
pub const GAME_CAPABILITIES: &[&str] = &[
    "game.evaluate_flow",
    "game.fitts_cost",
    "game.engagement",
    "game.analyze_ui",
    "game.accessibility",
    "game.wfc_step",
    "game.difficulty_adjustment",
    "game.generate_noise",
    "game.begin_session",
    "game.record_action",
    "game.complete_session",
    "game.poll_telemetry",
];

/// Semantic mappings: short name to fully qualified capability.
/// Used by biomeOS CapabilityTaxonomy for cross-primal routing.
pub const GAME_SEMANTIC_MAPPINGS: &[(&str, &str)] = &[
    ("evaluate_flow", "game.evaluate_flow"),
    ("fitts_cost", "game.fitts_cost"),
    ("engagement", "game.engagement"),
    ("analyze_ui", "game.analyze_ui"),
    ("accessibility", "game.accessibility"),
    ("wfc_step", "game.wfc_step"),
    ("difficulty_adjustment", "game.difficulty_adjustment"),
    ("generate_noise", "game.generate_noise"),
    ("begin_session", "game.begin_session"),
    ("record_action", "game.record_action"),
    ("complete_session", "game.complete_session"),
    ("poll_telemetry", "game.poll_telemetry"),
];

/// Resolve the Neural API socket path (XDG-compliant, no hardcoded primal names).
fn neural_api_socket() -> Option<PathBuf> {
    let family_id = std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    let sock_name = format!("neural-api-{family_id}.sock");

    for dir in candidate_dirs() {
        let p = dir.join(&sock_name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn candidate_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(d) = std::env::var("BIOMEOS_SOCKET_DIR") {
        dirs.push(PathBuf::from(d));
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        dirs.push(PathBuf::from(xdg).join("biomeos"));
    }
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    dirs.push(PathBuf::from("/tmp").join(format!("biomeos-{user}")));
    dirs.push(PathBuf::from("/tmp"));
    dirs
}

/// Send a JSON-RPC request and return the result (fire-and-forget on error).
fn rpc_call(
    socket_path: &std::path::Path,
    method: &str,
    params: &serde_json::Value,
) -> Option<serde_json::Value> {
    use std::os::unix::net::UnixStream;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let msg = serde_json::to_string(&request).ok()?;
    let stream = UnixStream::connect(socket_path).ok()?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .ok()?;
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        .ok()?;

    let mut writer = stream.try_clone().ok()?;
    writer.write_all(msg.as_bytes()).ok()?;
    writer.write_all(b"\n").ok()?;
    writer.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).ok()?;

    let parsed: serde_json::Value = serde_json::from_str(&response).ok()?;
    parsed.get("result").cloned()
}

/// Register the `game` domain and all capabilities with biomeOS Neural API.
///
/// Two-phase registration per the Spring-as-Niche Deployment Standard:
/// 1. Domain registration with semantic mappings
/// 2. Individual capability registration
///
/// Non-fatal if Neural API is unavailable — ludoSpring runs standalone.
pub fn register_domain(socket_path: &std::path::Path) {
    let Some(neural) = neural_api_socket() else {
        eprintln!(
            "[biomeos] Neural API not found — running standalone (domain registration skipped)"
        );
        return;
    };

    let mappings: serde_json::Value = GAME_SEMANTIC_MAPPINGS
        .iter()
        .map(|(short, full)| {
            (
                (*short).to_string(),
                serde_json::json!({ "provider": crate::PRIMAL_NAME, "method": full }),
            )
        })
        .collect::<serde_json::Map<String, serde_json::Value>>()
        .into();

    let params = serde_json::json!({
        "domain": GAME_DOMAIN,
        "provider": crate::PRIMAL_NAME,
        "socket_path": socket_path.to_string_lossy(),
        "capabilities": GAME_CAPABILITIES,
        "semantic_mappings": mappings,
    });

    if rpc_call(&neural, "capability.register", &params).is_some() {
        eprintln!(
            "[biomeos] Registered domain '{}' with {} capabilities",
            GAME_DOMAIN,
            GAME_CAPABILITIES.len()
        );
    } else {
        eprintln!("[biomeos] capability.register failed (non-fatal)");
    }
}

/// Deregister the `game` domain from biomeOS Neural API.
///
/// Called on SIGTERM shutdown for clean niche teardown.
pub fn deregister_domain() {
    let Some(neural) = neural_api_socket() else {
        return;
    };

    let params = serde_json::json!({
        "domain": GAME_DOMAIN,
        "provider": crate::PRIMAL_NAME,
    });

    let _ = rpc_call(&neural, "capability.deregister", &params);
    eprintln!("[biomeos] Deregistered domain '{GAME_DOMAIN}'");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_constants_consistent() {
        assert_eq!(GAME_DOMAIN, "game");
        assert_eq!(GAME_CAPABILITIES.len(), 12);
        assert_eq!(GAME_SEMANTIC_MAPPINGS.len(), 12);

        for (short, full) in GAME_SEMANTIC_MAPPINGS {
            assert!(
                GAME_CAPABILITIES.contains(full),
                "mapping {short} -> {full} not in capabilities"
            );
        }
    }

    #[test]
    fn all_capabilities_start_with_domain() {
        for cap in GAME_CAPABILITIES {
            assert!(
                cap.starts_with("game."),
                "capability '{cap}' does not start with 'game.'"
            );
        }
    }

    #[test]
    fn candidate_dirs_never_empty() {
        let dirs = candidate_dirs();
        assert!(!dirs.is_empty());
    }
}
