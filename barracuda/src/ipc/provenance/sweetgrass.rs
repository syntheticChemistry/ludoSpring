// SPDX-License-Identifier: AGPL-3.0-or-later
//! sweetGrass attribution — creative provenance and contribution tracking.
//!
//! Routes attribution capability calls through [`NeuralBridge`]:
//!
//! - `contribution.record_dehydration` — who created what during session end
//! - `braid.query` / `braid.commit` — attribution braid lifecycle
//! - `provenance.graph` — derivation history (NPC design evolution)
//! - `attribution.chain` — contribution shares for collaborative campaigns

use crate::ipc::envelope::IpcError;
use crate::ipc::neural_bridge::NeuralBridge;

use super::ProvenanceResult;
use super::unavailable_result;

/// JSON-RPC capability namespace for contribution records.
pub const CAP_CONTRIBUTION: &str = "contribution";

/// JSON-RPC capability namespace for provenance graph queries.
pub const CAP_PROVENANCE: &str = "provenance";

/// JSON-RPC capability namespace for attribution chains.
pub const CAP_ATTRIBUTION: &str = "attribution";

/// `contribution.record_dehydration` operation name.
pub const OP_RECORD_DEHYDRATION: &str = "record_dehydration";

/// `braid.query` operation name.
pub const OP_BRAID_QUERY: &str = "query";

/// `braid.commit` operation name.
pub const OP_BRAID_COMMIT: &str = "commit";

/// `provenance.graph` operation name.
pub const OP_PROVENANCE_GRAPH: &str = "graph";

/// `attribution.chain` operation name.
pub const OP_ATTRIBUTION_CHAIN: &str = "chain";

/// JSON-RPC args for `contribution.record_dehydration`.
pub(crate) fn record_dehydration_params(
    session_id: &str,
    merkle_root: &str,
    agents: &[serde_json::Value],
    operations: &[serde_json::Value],
) -> serde_json::Value {
    serde_json::json!({
        "session_id": session_id,
        "merkle_root": merkle_root,
        "agents": agents,
        "operations": operations,
    })
}

/// JSON-RPC args for `braid.query`.
pub(crate) fn query_braids_params(
    activity_type: Option<&str>,
    agent: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "activity_type": activity_type,
        "agent": agent,
    })
}

/// JSON-RPC args for `braid.commit`.
pub(crate) fn commit_braid_params(braid_id: &str, spine_id: &str) -> serde_json::Value {
    serde_json::json!({
        "braid_id": braid_id,
        "spine_id": spine_id,
        "committer": format!("did:key:{}", crate::PRIMAL_NAME),
    })
}

/// JSON-RPC args for `provenance.graph` / `attribution.chain` (braid-scoped).
pub(crate) fn braid_id_only_params(braid_id: &str) -> serde_json::Value {
    serde_json::json!({ "braid_id": braid_id })
}

/// Wraps a session-scoped sweetGrass response.
pub(crate) fn provenance_result_session_id(
    session_id: &str,
    result: serde_json::Value,
) -> ProvenanceResult {
    ProvenanceResult {
        id: session_id.to_string(),
        available: true,
        data: result,
    }
}

/// Wraps a braid query response (no stable id in wire format).
pub(crate) const fn provenance_result_list(result: serde_json::Value) -> ProvenanceResult {
    ProvenanceResult {
        id: String::new(),
        available: true,
        data: result,
    }
}

/// Wraps a braid-id-scoped response.
pub(crate) fn provenance_result_braid_id(
    braid_id: &str,
    result: serde_json::Value,
) -> ProvenanceResult {
    ProvenanceResult {
        id: braid_id.to_string(),
        available: true,
        data: result,
    }
}

/// Record a dehydration contribution (who created what during session end).
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn record_dehydration(
    session_id: &str,
    merkle_root: &str,
    agents: &[serde_json::Value],
    operations: &[serde_json::Value],
) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = record_dehydration_params(session_id, merkle_root, agents, operations);

    bridge
        .capability_call(CAP_CONTRIBUTION, OP_RECORD_DEHYDRATION, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_session_id(session_id, result)),
        )
}

/// Query braids by activity type (e.g. find all "npc_creation" braids).
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn query_braids(
    activity_type: Option<&str>,
    agent: Option<&str>,
) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = query_braids_params(activity_type, agent);

    bridge
        .capability_call(super::CAP_BRAID, OP_BRAID_QUERY, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_list(result)),
        )
}

/// Anchor a braid to a loamSpine spine for permanence.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn commit_braid(braid_id: &str, spine_id: &str) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = commit_braid_params(braid_id, spine_id);

    bridge
        .capability_call(super::CAP_BRAID, OP_BRAID_COMMIT, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_braid_id(braid_id, result)),
        )
}

/// Get derivation lineage for a braid (who designed/modified NPCs).
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn provenance_lineage(braid_id: &str) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = braid_id_only_params(braid_id);

    bridge
        .capability_call(CAP_PROVENANCE, OP_PROVENANCE_GRAPH, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_braid_id(braid_id, result)),
        )
}

/// Get attribution shares for a braid (collaborative campaign contributions).
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn provenance_attribution(braid_id: &str) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = braid_id_only_params(braid_id);

    bridge
        .capability_call(CAP_ATTRIBUTION, OP_ATTRIBUTION_CHAIN, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_braid_id(braid_id, result)),
        )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn record_dehydration_degrades() {
        let agents = vec![serde_json::json!({"did": "did:key:test", "role": "author"})];
        let ops = vec![serde_json::json!({"type": "game_session"})];
        let r = record_dehydration("sess-1", "abc123", &agents, &ops).unwrap();
        assert!(!r.available);
    }

    #[test]
    fn query_braids_degrades() {
        let r = query_braids(Some("npc_creation"), None).unwrap();
        assert!(!r.available);
    }

    #[test]
    fn commit_braid_degrades() {
        let r = commit_braid("braid-1", "spine-1").unwrap();
        assert!(!r.available);
    }

    #[test]
    fn provenance_lineage_degrades() {
        let r = provenance_lineage("braid-1").unwrap();
        assert!(!r.available);
    }

    #[test]
    fn provenance_attribution_degrades() {
        let r = provenance_attribution("braid-1").unwrap();
        assert!(!r.available);
    }

    #[test]
    fn sweetgrass_capability_constants() {
        assert_eq!(CAP_CONTRIBUTION, "contribution");
        assert_eq!(super::super::CAP_BRAID, "braid");
        assert_eq!(CAP_PROVENANCE, "provenance");
        assert_eq!(CAP_ATTRIBUTION, "attribution");
    }

    #[test]
    fn record_dehydration_params_shape() {
        let ag = vec![serde_json::json!({"a": 1})];
        let op = vec![serde_json::json!({"b": 2})];
        let v = record_dehydration_params("sid", "root", &ag, &op);
        assert_eq!(v["session_id"], "sid");
        assert_eq!(v["merkle_root"], "root");
        assert_eq!(v["agents"], serde_json::json!(ag));
        assert_eq!(v["operations"], serde_json::json!(op));
    }

    #[test]
    fn query_braids_params_optional_fields() {
        let v = query_braids_params(Some("npc"), Some("agent1"));
        assert_eq!(v["activity_type"], "npc");
        assert_eq!(v["agent"], "agent1");
        let v2 = query_braids_params(None, None);
        assert!(v2["activity_type"].is_null());
    }

    #[test]
    fn commit_braid_params_includes_committer_did() {
        let v = commit_braid_params("b1", "s1");
        assert_eq!(v["braid_id"], "b1");
        assert_eq!(v["spine_id"], "s1");
        assert_eq!(v["committer"], format!("did:key:{}", crate::PRIMAL_NAME));
    }

    #[test]
    fn braid_id_only_params_shape() {
        let v = braid_id_only_params("bid");
        assert_eq!(v["braid_id"], "bid");
    }

    #[test]
    fn provenance_result_wrappers_roundtrip() {
        let p = provenance_result_braid_id("b", serde_json::json!({}));
        let json = serde_json::to_string(&p).unwrap();
        let back: ProvenanceResult = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn map_session_id_response_matches_session() {
        let p = provenance_result_session_id("sess", serde_json::json!({"ok": true}));
        assert_eq!(p.id, "sess");
        assert!(p.available);
    }

    #[test]
    fn provenance_result_list_empty_id_with_data() {
        let p = provenance_result_list(serde_json::json!({"braids": []}));
        assert!(p.id.is_empty());
        assert!(p.available);
        assert_eq!(p.data["braids"], serde_json::json!([]));
    }
}
