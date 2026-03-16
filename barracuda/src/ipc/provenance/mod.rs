// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provenance trio integration for ludoSpring IPC.
//!
//! Wires game session lifecycle to the provenance trio via capability-based
//! discovery through [`NeuralBridge`]. Uses `capability.call` routing:
//!
//! - `dag.session.create` / `dag.event.append` / `dag.dehydration.trigger` → rhizoCrypt
//! - `session.commit` → loamSpine
//! - `braid.create` → sweetGrass
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

/// Structured dehydration summary from rhizoCrypt.
///
/// Follows the `provenance-trio-types` `DehydrationSummary` wire format,
/// extracted as a typed struct rather than raw JSON.
#[derive(Debug, Clone)]
pub struct DehydrationSummary {
    /// Merkle root of the dehydrated DAG.
    pub merkle_root: String,
    /// Current frontier vertices (DAG tips).
    pub frontier: Vec<String>,
    /// Total vertex count in the dehydrated session.
    pub vertex_count: u64,
    /// The raw response for forwarding to loamSpine.
    pub raw: serde_json::Value,
}

/// Progression stage of the trio pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrioStage {
    /// Neural API not available.
    Unavailable,
    /// Dehydration succeeded, commit pending.
    Dehydrated,
    /// Commit succeeded, attribution pending.
    Committed,
    /// Full pipeline completed.
    Complete,
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
/// Creates a session via `dag.session.create`. Returns the session ID or
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
        .capability_call("dag", "session.create", &args)
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
/// Appends a vertex via `dag.event.append`. Returns the vertex ID or a note
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
        .capability_call("dag", "event.append", &args)
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

/// Complete a game session through the full provenance trio pipeline.
///
/// 1. `dag.dehydration.trigger` → rhizoCrypt: compute Merkle root and frontier
/// 2. `contribution.record_dehydration` → sweetGrass: record who created what
/// 3. `session.commit` → loamSpine: anchor to permanent spine
/// 4. `braid.create` → sweetGrass: create attribution braid
///
/// # Errors
///
/// Returns an error string only on non-recoverable failures. Partial
/// completion returns `Ok` with a `stage` field describing progress.
pub fn complete_game_session(session_id: &str) -> Result<serde_json::Value, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(stage_result(TrioStage::Unavailable, session_id, None));
    };

    // Step 1: Dehydrate the DAG — compute Merkle root and frontier
    let dehydrate_args = serde_json::json!({ "session_id": session_id });
    let Ok(dehydration_raw) = bridge.capability_call("dag", "dehydration.trigger", &dehydrate_args)
    else {
        return Ok(stage_result(TrioStage::Unavailable, session_id, None));
    };

    let summary = parse_dehydration(&dehydration_raw);

    // Step 2: Record dehydration with sweetGrass (who created what)
    let primal_did = format!("did:key:{}", crate::PRIMAL_NAME);
    let agents = vec![serde_json::json!({
        "did": primal_did,
        "role": "author",
        "contribution": 1.0
    })];
    let operations = vec![serde_json::json!({
        "type": "game_session",
        "vertex_count": summary.vertex_count,
    })];
    let _ = record_dehydration(session_id, &summary.merkle_root, &agents, &operations);

    // Step 3: Commit to loamSpine — anchor the dehydration summary
    let commit_args = serde_json::json!({
        "summary": summary.raw,
        "content_hash": summary.merkle_root,
        "vertex_count": summary.vertex_count,
        "frontier": summary.frontier,
    });
    let Ok(commit_result) = bridge.capability_call("session", "commit", &commit_args) else {
        return Ok(stage_result(
            TrioStage::Dehydrated,
            session_id,
            Some(&summary),
        ));
    };

    let commit_id = commit_result
        .get("commit_id")
        .or_else(|| commit_result.get("entry_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Step 4: Create attribution braid via sweetGrass
    let braid_args = serde_json::json!({
        "commit_ref": commit_id,
        "agents": agents,
    });
    let braid_result = bridge.capability_call("braid", "create", &braid_args);

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
        "stage": "complete",
        "session_id": session_id,
        "merkle_root": summary.merkle_root,
        "vertex_count": summary.vertex_count,
        "frontier_size": summary.frontier.len(),
        "commit_id": commit_id,
        "braid_id": braid_id,
    }))
}

/// Parse a raw dehydration JSON response into a typed summary.
fn parse_dehydration(raw: &serde_json::Value) -> DehydrationSummary {
    let merkle_root = raw
        .get("merkle_root")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let frontier = raw
        .get("frontier")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let vertex_count = raw
        .get("vertex_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);

    DehydrationSummary {
        merkle_root,
        frontier,
        vertex_count,
        raw: raw.clone(),
    }
}

/// Build a stage-aware JSON result for partial completions.
fn stage_result(
    stage: TrioStage,
    session_id: &str,
    summary: Option<&DehydrationSummary>,
) -> serde_json::Value {
    let stage_str = match stage {
        TrioStage::Unavailable => "unavailable",
        TrioStage::Dehydrated => "dehydrated",
        TrioStage::Committed => "committed",
        TrioStage::Complete => "complete",
    };

    let mut result = serde_json::json!({
        "stage": stage_str,
        "session_id": session_id,
    });

    if let Some(s) = summary {
        result["merkle_root"] = serde_json::json!(s.merkle_root);
        result["vertex_count"] = serde_json::json!(s.vertex_count);
        result["frontier_size"] = serde_json::json!(s.frontier.len());
    }

    result
}

fn unavailable_result() -> ProvenanceResult {
    ProvenanceResult {
        id: format!("local-{}", uuid::Uuid::now_v7()),
        available: false,
        data: serde_json::json!({ "provenance": "unavailable" }),
    }
}
