// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provenance trio integration for ludoSpring IPC.
//!
//! Wires game session lifecycle to the provenance trio via capability-based
//! discovery through [`NeuralBridge`]. Uses `capability.call` routing:
//!
//! - `dag.create_session` / `dag.append_event` / `dag.dehydrate` → rhizoCrypt
//! - `commit.session` → loamSpine
//! - `provenance.create_braid` → sweetGrass
//!
//! No hardcoded primal names — capability routing is delegated to biomeOS.
//! Graceful degradation: if the trio is unavailable, handlers return success
//! with `"provenance": "unavailable"`.
//!
//! # Module structure
//!
//! - [`rhizocrypt`] — DAG queries (vertex query, Merkle proofs, batch append)
//! - [`loamspine`] — certificate operations (mint, verify, lifecycle, spines)
//! - [`sweetgrass`] — attribution (braids, lineage, dehydration records)

use super::neural_bridge::NeuralBridge;

pub mod loamspine;
pub mod rhizocrypt;
pub mod sweetgrass;

pub use loamspine::*;
pub use rhizocrypt::*;
pub use sweetgrass::*;

/// Result of a provenance operation; includes availability status.
#[derive(Debug)]
pub struct ProvenanceResult {
    /// Session ID, vertex ID, or braid ID from the trio.
    pub id: String,
    /// Whether the trio was available and the operation succeeded.
    pub available: bool,
    /// Additional data (e.g. `merkle_root`, `commit_id`, `braid_id`).
    pub data: serde_json::Value,
}

/// Whether the Neural API is discoverable (trio might be reachable).
///
/// Used by `game.poll_telemetry` to report streaming vs idle status.
#[must_use]
pub fn has_active_session() -> bool {
    crate::niche::resolve_neural_api_socket().is_some()
}

/// Begin a game session in the provenance trio.
///
/// Creates a session via `dag.create_session`. Returns the session ID or
/// a fallback note if the trio is unavailable.
///
/// # Errors
///
/// Returns an error string only on non-recoverable failures.
pub fn begin_game_session(session_name: &str) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({
        "metadata": { "type": "game_session", "name": session_name },
        "session_type": { "Gaming": { "game_id": crate::PRIMAL_NAME } },
        "description": session_name,
    });

    bridge
        .capability_call("dag", "create_session", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                let session_id = result
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                Ok(ProvenanceResult {
                    id: session_id.clone(),
                    available: true,
                    data: serde_json::json!({ "session_id": session_id }),
                })
            },
        )
}

/// Record a game action in the provenance trio.
///
/// Appends a vertex via `dag.append_event`. Returns the vertex ID or a note
/// if unavailable.
///
/// # Errors
///
/// Returns an error string only on non-recoverable failures.
pub fn record_game_action(
    session_id: &str,
    action: &serde_json::Value,
) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({
        "session_id": session_id,
        "event": action,
    });

    bridge
        .capability_call("dag", "append_event", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                let vertex_id = result
                    .get("vertex_id")
                    .or_else(|| result.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                Ok(ProvenanceResult {
                    id: vertex_id.clone(),
                    available: true,
                    data: serde_json::json!({ "vertex_id": vertex_id }),
                })
            },
        )
}

/// Complete a game session: dehydrate, commit, and attribute.
///
/// 1. `dag.dehydrate` — compute Merkle root and frontier
/// 2. `commit.session` — anchor to loamSpine
/// 3. `provenance.create_braid` — attribute via sweetGrass
///
/// # Errors
///
/// Returns an error string only on non-recoverable failures. Partial
/// completion (e.g. dehydration succeeded but commit failed) returns `Ok`
/// with a status field describing how far the pipeline progressed.
pub fn complete_game_session(session_id: &str) -> Result<serde_json::Value, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(serde_json::json!({
            "provenance": "unavailable",
            "session_id": session_id,
        }));
    };

    let dehydrate_args = serde_json::json!({ "session_id": session_id });
    let Ok(dehydration) = bridge.capability_call("dag", "dehydrate", &dehydrate_args) else {
        return Ok(serde_json::json!({
            "provenance": "unavailable",
            "session_id": session_id,
        }));
    };

    let merkle_root = dehydration
        .get("merkle_root")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let commit_args = serde_json::json!({
        "summary": dehydration,
        "content_hash": merkle_root,
    });
    let Ok(commit_result) = bridge.capability_call("commit", "session", &commit_args) else {
        return Ok(serde_json::json!({
            "provenance": "partial",
            "session_id": session_id,
            "dehydrated": true,
            "committed": false,
        }));
    };

    let commit_id = commit_result
        .get("commit_id")
        .or_else(|| commit_result.get("entry_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let braid_args = serde_json::json!({
        "commit_ref": commit_id,
        "agents": [{
            "did": format!("did:key:{}", crate::PRIMAL_NAME),
            "role": "author",
            "contribution": 1.0
        }],
    });
    let braid_result = bridge.capability_call("provenance", "create_braid", &braid_args);

    let braid_id = braid_result
        .ok()
        .and_then(|r| {
            r.get("braid_id")
                .or_else(|| r.get("id"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .unwrap_or_default();

    Ok(serde_json::json!({
        "provenance": "complete",
        "session_id": session_id,
        "merkle_root": merkle_root,
        "commit_id": commit_id,
        "braid_id": braid_id,
    }))
}

fn unavailable_result() -> ProvenanceResult {
    ProvenanceResult {
        id: format!("local-{}", uuid::Uuid::now_v7()),
        available: false,
        data: serde_json::json!({ "provenance": "unavailable" }),
    }
}
