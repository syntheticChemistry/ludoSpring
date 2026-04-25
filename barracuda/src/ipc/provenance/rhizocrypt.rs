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

use crate::ipc::envelope::IpcError;
use crate::ipc::neural_bridge::NeuralBridge;

use super::ProvenanceResult;
use super::unavailable_result;

/// `dag.vertex.query` operation name.
pub const OP_VERTEX_QUERY: &str = "vertex.query";

/// `dag.vertex.children` operation name.
pub const OP_VERTEX_CHILDREN: &str = "vertex.children";

/// `dag.frontier.get` operation name.
pub const OP_FRONTIER_GET: &str = "frontier.get";

/// `dag.merkle.root` operation name.
pub const OP_MERKLE_ROOT: &str = "merkle.root";

/// `dag.merkle.proof` operation name.
pub const OP_MERKLE_PROOF: &str = "merkle.proof";

/// `dag.event.append_batch` operation name.
pub const OP_EVENT_APPEND_BATCH: &str = "event.append_batch";

/// `dag.session.list` operation name.
pub const OP_SESSION_LIST: &str = "session.list";

/// JSON-RPC args for `dag.vertex.query`.
pub(crate) fn query_vertices_params(
    session_id: &str,
    event_type: Option<&str>,
    agent: Option<&str>,
    limit: Option<u32>,
) -> serde_json::Value {
    serde_json::json!({
        "session_id": session_id,
        "type": event_type,
        "agent": agent,
        "limit": limit.unwrap_or(crate::tolerances::DEFAULT_VERTEX_QUERY_LIMIT),
    })
}

/// JSON-RPC args for `dag.vertex.children`.
pub(crate) fn vertex_children_params(session_id: &str, vertex_id: &str) -> serde_json::Value {
    serde_json::json!({
        "session_id": session_id,
        "vertex_id": vertex_id,
    })
}

/// JSON-RPC args for `dag.frontier.get` / `dag.merkle.root` (session-scoped).
pub(crate) fn session_only_params(session_id: &str) -> serde_json::Value {
    serde_json::json!({ "session_id": session_id })
}

/// JSON-RPC args for `dag.merkle.proof`.
pub(crate) fn merkle_proof_params(session_id: &str, vertex_id: &str) -> serde_json::Value {
    serde_json::json!({
        "session_id": session_id,
        "vertex_id": vertex_id,
    })
}

/// Per-event requests for `dag.event.append_batch`.
pub(crate) fn append_batch_request_items(
    session_id: &str,
    events: &[serde_json::Value],
) -> Vec<serde_json::Value> {
    events
        .iter()
        .map(|e| serde_json::json!({ "session_id": session_id, "event": e }))
        .collect()
}

/// JSON-RPC args for `dag.event.append_batch`.
pub(crate) fn append_batch_params(
    session_id: &str,
    events: &[serde_json::Value],
) -> serde_json::Value {
    let requests = append_batch_request_items(session_id, events);
    serde_json::json!({ "requests": requests })
}

/// Wraps a session-scoped DAG response (result id = session id).
pub(crate) fn provenance_result_session_scoped(
    session_id: &str,
    result: serde_json::Value,
) -> ProvenanceResult {
    ProvenanceResult {
        id: session_id.to_string(),
        available: true,
        data: result,
    }
}

/// Wraps a vertex-scoped DAG response (result id = vertex id).
pub(crate) fn provenance_result_vertex_scoped(
    vertex_id: &str,
    result: serde_json::Value,
) -> ProvenanceResult {
    ProvenanceResult {
        id: vertex_id.to_string(),
        available: true,
        data: result,
    }
}

/// Wraps `session.list` response (no stable id in wire format).
pub(crate) const fn provenance_result_list(result: serde_json::Value) -> ProvenanceResult {
    ProvenanceResult {
        id: String::new(),
        available: true,
        data: result,
    }
}

/// Query DAG vertices by type and/or agent — the core of NPC memory retrieval.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn query_vertices(
    session_id: &str,
    event_type: Option<&str>,
    agent: Option<&str>,
    limit: Option<u32>,
) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = query_vertices_params(session_id, event_type, agent, limit);

    bridge
        .capability_call(super::CAP_DAG, OP_VERTEX_QUERY, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_session_scoped(session_id, result)),
        )
}

/// Get children of a vertex — follow conversation threads.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn vertex_children(session_id: &str, vertex_id: &str) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = vertex_children_params(session_id, vertex_id);

    bridge
        .capability_call(super::CAP_DAG, OP_VERTEX_CHILDREN, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_vertex_scoped(vertex_id, result)),
        )
}

/// Get DAG frontier (tips) — current state for save/load.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn get_frontier(session_id: &str) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = session_only_params(session_id);

    bridge
        .capability_call(super::CAP_DAG, OP_FRONTIER_GET, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_session_scoped(session_id, result)),
        )
}

/// Get Merkle root for anti-cheat session integrity verification.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn merkle_root(session_id: &str) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = session_only_params(session_id);

    bridge
        .capability_call(super::CAP_DAG, OP_MERKLE_ROOT, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_session_scoped(session_id, result)),
        )
}

/// Generate Merkle inclusion proof for a specific vertex.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn merkle_proof(session_id: &str, vertex_id: &str) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = merkle_proof_params(session_id, vertex_id);

    bridge
        .capability_call(super::CAP_DAG, OP_MERKLE_PROOF, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_vertex_scoped(vertex_id, result)),
        )
}

/// Batch-append events for high-frequency game actions (combat rounds).
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn append_batch(
    session_id: &str,
    events: &[serde_json::Value],
) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = append_batch_params(session_id, events);

    bridge
        .capability_call(super::CAP_DAG, OP_EVENT_APPEND_BATCH, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_session_scoped(session_id, result)),
        )
}

/// List past sessions for campaign continuity.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn list_sessions() -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    bridge
        .capability_call(super::CAP_DAG, OP_SESSION_LIST, &serde_json::json!({}))
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_list(result)),
        )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn query_vertices_degrades() {
        let r = query_vertices("sess-1", Some("move"), None, None).unwrap();
        assert!(!r.available);
    }

    #[test]
    fn vertex_children_degrades() {
        let r = vertex_children("sess-1", "v-001").unwrap();
        assert!(!r.available);
    }

    #[test]
    fn get_frontier_degrades() {
        let r = get_frontier("sess-1").unwrap();
        assert!(!r.available);
    }

    #[test]
    fn merkle_root_degrades() {
        let r = merkle_root("sess-1").unwrap();
        assert!(!r.available);
    }

    #[test]
    fn merkle_proof_degrades() {
        let r = merkle_proof("sess-1", "v-001").unwrap();
        assert!(!r.available);
    }

    #[test]
    fn append_batch_degrades() {
        let events = vec![serde_json::json!({"type": "attack"})];
        let r = append_batch("sess-1", &events).unwrap();
        assert!(!r.available);
    }

    #[test]
    fn list_sessions_degrades() {
        let r = list_sessions().unwrap();
        assert!(!r.available);
    }

    #[test]
    fn dag_ops_match_strings() {
        assert_eq!(super::super::CAP_DAG, "dag");
        assert_eq!(OP_VERTEX_QUERY, "vertex.query");
        assert_eq!(OP_SESSION_LIST, "session.list");
    }

    #[test]
    fn query_vertices_params_applies_default_limit() {
        let v = query_vertices_params("s", Some("t"), None, None);
        assert_eq!(
            v["limit"],
            serde_json::json!(crate::tolerances::DEFAULT_VERTEX_QUERY_LIMIT)
        );
        assert_eq!(v["session_id"], "s");
        assert_eq!(v["type"], "t");
        assert!(v["agent"].is_null());
    }

    #[test]
    fn query_vertices_params_custom_limit() {
        let v = query_vertices_params("s", None, Some("a"), Some(7));
        assert_eq!(v["limit"], 7);
        assert_eq!(v["agent"], "a");
    }

    #[test]
    fn append_batch_request_items_and_params() {
        let ev = vec![serde_json::json!({"x": 1}), serde_json::json!({"y": 2})];
        let items = append_batch_request_items("sid", &ev);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["session_id"], "sid");
        assert_eq!(items[0]["event"], ev[0]);
        let p = append_batch_params("sid", &ev);
        assert_eq!(p["requests"], serde_json::json!(items));
    }

    #[test]
    fn session_only_and_merkle_proof_params() {
        let s = session_only_params("sess");
        assert_eq!(s["session_id"], "sess");
        let m = merkle_proof_params("sess", "v1");
        assert_eq!(m["vertex_id"], "v1");
    }

    #[test]
    fn provenance_wrappers_serde_roundtrip() {
        let p = provenance_result_session_scoped("s", serde_json::json!({"a": 1}));
        let json = serde_json::to_string(&p).unwrap();
        let back: ProvenanceResult = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
        let p2 = provenance_result_list(serde_json::json!([]));
        assert!(p2.id.is_empty());
    }

    #[test]
    fn provenance_result_vertex_scoped_sets_id_and_data() {
        let p = provenance_result_vertex_scoped("vx", serde_json::json!({"k": 2}));
        assert_eq!(p.id, "vx");
        assert!(p.available);
        assert_eq!(p.data["k"], 2);
    }

    #[test]
    fn vertex_children_params_shape() {
        let v = vertex_children_params("sess", "vert");
        assert_eq!(v["session_id"], "sess");
        assert_eq!(v["vertex_id"], "vert");
    }

    #[test]
    fn provenance_result_list_preserves_payload() {
        let p = provenance_result_list(serde_json::json!({"sessions": [1]}));
        assert!(p.id.is_empty());
        assert_eq!(p.data["sessions"][0], 1);
    }
}
