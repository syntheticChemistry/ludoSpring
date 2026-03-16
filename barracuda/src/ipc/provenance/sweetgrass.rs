// SPDX-License-Identifier: AGPL-3.0-or-later
//! sweetGrass attribution — creative provenance and contribution tracking.
//!
//! Routes attribution capability calls through [`NeuralBridge`]:
//!
//! - `contribution.record_dehydration` — who created what during session end
//! - `braid.query` / `braid.commit` — attribution braid lifecycle
//! - `provenance.graph` — derivation history (NPC design evolution)
//! - `attribution.chain` — contribution shares for collaborative campaigns

use crate::ipc::neural_bridge::NeuralBridge;

use super::ProvenanceResult;
use super::unavailable_result;

/// Record a dehydration contribution (who created what during session end).
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn record_dehydration(
    session_id: &str,
    merkle_root: &str,
    agents: &[serde_json::Value],
    operations: &[serde_json::Value],
) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({
        "session_id": session_id,
        "merkle_root": merkle_root,
        "agents": agents,
        "operations": operations,
    });

    bridge
        .capability_call("contribution", "record_dehydration", &args)
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

/// Query braids by activity type (e.g. find all "npc_creation" braids).
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn query_braids(
    activity_type: Option<&str>,
    agent: Option<&str>,
) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({
        "activity_type": activity_type,
        "agent": agent,
    });

    bridge.capability_call("braid", "query", &args).map_or_else(
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

/// Anchor a braid to a loamSpine spine for permanence.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn commit_braid(braid_id: &str, spine_id: &str) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({
        "braid_id": braid_id,
        "spine_id": spine_id,
        "committer": format!("did:key:{}", crate::PRIMAL_NAME),
    });

    bridge
        .capability_call("braid", "commit", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: braid_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}

/// Get derivation lineage for a braid (who designed/modified NPCs).
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn provenance_lineage(braid_id: &str) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({ "braid_id": braid_id });

    bridge
        .capability_call("provenance", "graph", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: braid_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}

/// Get attribution shares for a braid (collaborative campaign contributions).
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn provenance_attribution(braid_id: &str) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({ "braid_id": braid_id });

    bridge
        .capability_call("attribution", "chain", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: braid_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}
