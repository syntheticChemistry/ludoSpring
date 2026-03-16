// SPDX-License-Identifier: AGPL-3.0-or-later
//! rhizoCrypt DAG queries — NPC memory, session history, Merkle proofs.
//!
//! Routes DAG capability calls through [`NeuralBridge`]:
//!
//! - `dag.vertex.query` — retrieve vertices by type/agent (NPC memory)
//! - `dag.vertex.children` — follow conversation threads
//! - `dag.frontier.get` — current DAG tips for save/load
//! - `dag.merkle.root` / `dag.merkle.proof` — anti-cheat integrity
//! - `dag.event.append_batch` — high-frequency combat actions
//! - `dag.session.list` — campaign continuity

use crate::ipc::neural_bridge::NeuralBridge;

use super::ProvenanceResult;
use super::unavailable_result;

/// Query DAG vertices by type and/or agent — the core of NPC memory retrieval.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn query_vertices(
    session_id: &str,
    event_type: Option<&str>,
    agent: Option<&str>,
    limit: Option<u32>,
) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({
        "session_id": session_id,
        "type": event_type,
        "agent": agent,
        "limit": limit.unwrap_or(crate::tolerances::DEFAULT_VERTEX_QUERY_LIMIT),
    });

    bridge
        .capability_call("dag", "vertex.query", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: session_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}

/// Get children of a vertex — follow conversation threads.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn vertex_children(session_id: &str, vertex_id: &str) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({
        "session_id": session_id,
        "vertex_id": vertex_id,
    });

    bridge
        .capability_call("dag", "vertex.children", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: vertex_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}

/// Get DAG frontier (tips) — current state for save/load.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn get_frontier(session_id: &str) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({ "session_id": session_id });

    bridge
        .capability_call("dag", "frontier.get", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: session_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}

/// Get Merkle root for anti-cheat session integrity verification.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn merkle_root(session_id: &str) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({ "session_id": session_id });

    bridge
        .capability_call("dag", "merkle.root", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: session_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}

/// Generate Merkle inclusion proof for a specific vertex.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn merkle_proof(session_id: &str, vertex_id: &str) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({
        "session_id": session_id,
        "vertex_id": vertex_id,
    });

    bridge
        .capability_call("dag", "merkle.proof", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: vertex_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}

/// Batch-append events for high-frequency game actions (combat rounds).
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn append_batch(
    session_id: &str,
    events: &[serde_json::Value],
) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let requests: Vec<serde_json::Value> = events
        .iter()
        .map(|e| serde_json::json!({ "session_id": session_id, "event": e }))
        .collect();

    let args = serde_json::json!({ "requests": requests });

    bridge
        .capability_call("dag", "event.append_batch", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: session_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}

/// List past sessions for campaign continuity.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn list_sessions() -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    bridge
        .capability_call("dag", "session.list", &serde_json::json!({}))
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: String::new(),
                    available: true,
                    data: result,
                })
            },
        )
}
