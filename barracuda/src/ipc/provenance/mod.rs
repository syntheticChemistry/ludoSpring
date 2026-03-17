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

// ── Resilient IPC (healthSpring V32 circuit breaker pattern) ─────────

use std::sync::atomic::{AtomicU64, Ordering};

/// Cooldown period after a circuit opens (5 seconds, per healthSpring V32).
const CIRCUIT_COOLDOWN_MS: u64 = 5_000;

/// Maximum retry count with exponential backoff.
const MAX_RETRIES: u32 = 2;

/// Base delay between retries (doubles each attempt).
const BASE_RETRY_DELAY_MS: u64 = 50;

/// Timestamp (epoch ms) when the circuit last opened. 0 = circuit closed.
static CIRCUIT_OPEN_SINCE: AtomicU64 = AtomicU64::new(0);

/// Check whether the circuit breaker allows a call.
fn circuit_allows() -> bool {
    let opened = CIRCUIT_OPEN_SINCE.load(Ordering::Relaxed);
    if opened == 0 {
        return true;
    }
    let now = epoch_ms();
    if now.saturating_sub(opened) >= CIRCUIT_COOLDOWN_MS {
        CIRCUIT_OPEN_SINCE.store(0, Ordering::Relaxed);
        return true;
    }
    false
}

/// Trip the circuit breaker open.
fn trip_circuit() {
    CIRCUIT_OPEN_SINCE.store(epoch_ms(), Ordering::Relaxed);
}

/// Reset the circuit breaker (call succeeded).
fn reset_circuit() {
    CIRCUIT_OPEN_SINCE.store(0, Ordering::Relaxed);
}

fn epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Execute a provenance trio call with circuit breaker and exponential backoff.
///
/// If the circuit is open, returns `None` immediately (graceful degradation).
/// On failure, retries up to `MAX_RETRIES` times with exponential backoff,
/// then trips the circuit.
fn resilient_trio_call<F>(f: F) -> Option<serde_json::Value>
where
    F: Fn(&NeuralBridge) -> Result<serde_json::Value, super::envelope::IpcError>,
{
    if !circuit_allows() {
        return None;
    }

    let Ok(bridge) = NeuralBridge::discover() else {
        trip_circuit();
        return None;
    };

    for attempt in 0..=MAX_RETRIES {
        match f(&bridge) {
            Ok(value) => {
                reset_circuit();
                return Some(value);
            }
            Err(_) if attempt < MAX_RETRIES => {
                let delay = BASE_RETRY_DELAY_MS * (1 << attempt);
                std::thread::sleep(std::time::Duration::from_millis(delay));
            }
            Err(_) => {
                trip_circuit();
                return None;
            }
        }
    }

    None
}

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
    let args = serde_json::json!({
        "metadata": { "type": "game_session", "name": session_name },
        "session_type": { "Gaming": { "game_id": crate::PRIMAL_NAME } },
        "description": session_name,
    });

    let Some(result) = resilient_trio_call(|bridge| {
        bridge.capability_call("dag", "session.create", &args)
    }) else {
        return Ok(unavailable_result());
    };

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
    let args = serde_json::json!({
        "session_id": session_id,
        "event": action,
    });

    let Some(result) = resilient_trio_call(|bridge| {
        bridge.capability_call("dag", "event.append", &args)
    }) else {
        return Ok(unavailable_result());
    };

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
    // Step 1: Dehydrate the DAG — compute Merkle root and frontier
    let dehydrate_args = serde_json::json!({ "session_id": session_id });
    let Some(dehydration_raw) = resilient_trio_call(|bridge| {
        bridge.capability_call("dag", "dehydration.trigger", &dehydrate_args)
    }) else {
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
    let Some(commit_result) = resilient_trio_call(|bridge| {
        bridge.capability_call("session", "commit", &commit_args)
    }) else {
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
    let braid_id = resilient_trio_call(|bridge| {
        bridge.capability_call("braid", "create", &braid_args)
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_session_degrades_without_neural_api() {
        let result = begin_game_session("test-session").unwrap();
        assert!(!result.available);
        assert!(result.id.starts_with("local-"));
        assert_eq!(result.data["provenance"], "unavailable");
    }

    #[test]
    fn record_action_degrades_without_neural_api() {
        let action = serde_json::json!({"type": "move", "x": 5, "y": 3});
        let result = record_game_action("sess-1", &action).unwrap();
        assert!(!result.available);
    }

    #[test]
    fn complete_session_degrades_without_neural_api() {
        let result = complete_game_session("sess-1").unwrap();
        assert_eq!(result["stage"], "unavailable");
        assert_eq!(result["session_id"], "sess-1");
    }

    #[test]
    fn has_active_session_false_without_neural_api() {
        assert!(!has_active_session());
    }

    #[test]
    fn circuit_breaker_initially_closed() {
        reset_circuit();
        assert!(circuit_allows());
    }

    #[test]
    fn circuit_breaker_trips_and_blocks() {
        trip_circuit();
        assert!(!circuit_allows());
        reset_circuit();
    }

    #[test]
    fn resilient_trio_call_degrades_without_neural_api() {
        reset_circuit();
        let result = resilient_trio_call(|bridge| {
            bridge.capability_call("dag", "session.create", &serde_json::json!({}))
        });
        assert!(result.is_none());
    }

    #[test]
    fn trio_stage_equality() {
        assert_eq!(TrioStage::Unavailable, TrioStage::Unavailable);
        assert_ne!(TrioStage::Dehydrated, TrioStage::Complete);
    }

    #[test]
    fn parse_dehydration_handles_empty_response() {
        let raw = serde_json::json!({});
        let summary = parse_dehydration(&raw);
        assert!(summary.merkle_root.is_empty());
        assert!(summary.frontier.is_empty());
        assert_eq!(summary.vertex_count, 0);
    }

    #[test]
    fn parse_dehydration_extracts_fields() {
        let raw = serde_json::json!({
            "merkle_root": "abc123",
            "frontier": ["v1", "v2"],
            "vertex_count": 42
        });
        let summary = parse_dehydration(&raw);
        assert_eq!(summary.merkle_root, "abc123");
        assert_eq!(summary.frontier, vec!["v1", "v2"]);
        assert_eq!(summary.vertex_count, 42);
    }

    #[test]
    fn unavailable_result_has_uuid() {
        let r1 = unavailable_result();
        let r2 = unavailable_result();
        assert_ne!(r1.id, r2.id);
        assert!(r1.id.starts_with("local-"));
    }

    #[test]
    fn stage_result_formats_correctly() {
        let r = stage_result(TrioStage::Dehydrated, "sess-1", None);
        assert_eq!(r["stage"], "dehydrated");
        assert_eq!(r["session_id"], "sess-1");
    }
}
