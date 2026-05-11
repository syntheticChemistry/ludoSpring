// SPDX-License-Identifier: AGPL-3.0-or-later
//! skunkBat audit integration — typed client for cross-primal audit logging.
//!
//! Emits audit events to skunkBat via the Neural API (`security.audit_log`)
//! for forwarding to rhizoCrypt DAG + sweetGrass braid. Events cover game
//! session lifecycle, certification results, and composition state changes.
//!
//! Graceful degradation: returns `AuditResult { delivered: false, .. }` when
//! skunkBat is not reachable through the Neural API.

use super::envelope::IpcError;
use super::neural_bridge::NeuralBridge;

/// Result of a skunkBat audit operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditResult {
    /// Whether the audit event was delivered to skunkBat.
    pub delivered: bool,
    /// Sequence number assigned by skunkBat (if delivered).
    pub seq: Option<u64>,
    /// Additional response data from skunkBat.
    pub data: serde_json::Value,
}

/// Emit a generic audit event to skunkBat via `security.audit_log`.
///
/// # Errors
///
/// Returns [`IpcError`] only on non-recoverable failures (transport, parse).
/// Unreachable skunkBat is *not* an error — returns `AuditResult { delivered: false }`.
pub fn audit_log(
    event_type: &str,
    source: &str,
    payload: &serde_json::Value,
) -> Result<AuditResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — audit logging unavailable"));
    };

    let args = serde_json::json!({
        "event_type": event_type,
        "source": source,
        "payload": payload,
    });

    bridge
        .capability_call("security", "audit_log", &args)
        .map_or_else(
            |_| Ok(unavailable("skunkBat security.audit_log unavailable")),
            |result| Ok(delivered(result)),
        )
}

/// Emit a game session lifecycle audit event.
///
/// Covers `session.begin`, `session.complete`, and `session.error`.
///
/// # Errors
///
/// Returns [`IpcError`] only on non-recoverable failures.
pub fn audit_session(
    event_type: &str,
    session_id: &str,
    details: &serde_json::Value,
) -> Result<AuditResult, IpcError> {
    audit_log(
        event_type,
        crate::niche::NICHE_NAME,
        &serde_json::json!({
            "session_id": session_id,
            "details": details,
        }),
    )
}

/// Emit a certification result audit event.
///
/// Called after `ludospring certify` completes. Records the tier, pass/fail
/// counts, and certification level for provenance tracking.
///
/// # Errors
///
/// Returns [`IpcError`] only on non-recoverable failures.
pub fn audit_certification(
    tier: u8,
    passed: u32,
    failed: u32,
    skipped: u32,
) -> Result<AuditResult, IpcError> {
    audit_log(
        "certification",
        crate::niche::NICHE_NAME,
        &serde_json::json!({
            "tier": tier,
            "passed": passed,
            "failed": failed,
            "skipped": skipped,
        }),
    )
}

/// Emit a validation scenario result audit event.
///
/// Called after `ludospring validate` runs a scenario. Records scenario id,
/// track, and outcome for downstream provenance.
///
/// # Errors
///
/// Returns [`IpcError`] only on non-recoverable failures.
pub fn audit_validation(
    scenario_id: &str,
    track: &str,
    passed: u32,
    failed: u32,
) -> Result<AuditResult, IpcError> {
    audit_log(
        "validation",
        crate::niche::NICHE_NAME,
        &serde_json::json!({
            "scenario_id": scenario_id,
            "track": track,
            "passed": passed,
            "failed": failed,
        }),
    )
}

/// Query the skunkBat audit trail (most recent events).
///
/// Uses `security.audit_log` with cursor parameters per skunkBat's query API.
///
/// # Errors
///
/// Returns [`IpcError`] only on non-recoverable failures.
pub fn query_audit_trail(since_seq: u64, limit: u32) -> Result<AuditResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — audit query unavailable"));
    };

    let args = serde_json::json!({
        "since_seq": since_seq,
        "limit": limit,
    });

    bridge
        .capability_call("security", "audit_log", &args)
        .map_or_else(
            |_| Ok(unavailable("skunkBat security.audit_log unavailable")),
            |result| Ok(delivered(result)),
        )
}

fn delivered(result: serde_json::Value) -> AuditResult {
    let seq = result
        .get("latest_seq")
        .or_else(|| result.get("seq"))
        .and_then(serde_json::Value::as_u64);
    AuditResult {
        delivered: true,
        seq,
        data: result,
    }
}

fn unavailable(reason: &str) -> AuditResult {
    AuditResult {
        delivered: false,
        seq: None,
        data: serde_json::json!({ "reason": reason }),
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_result_structure() {
        let r = unavailable("test");
        assert!(!r.delivered);
        assert!(r.seq.is_none());
        assert_eq!(r.data["reason"], "test");
    }

    #[test]
    fn delivered_extracts_latest_seq() {
        let r = delivered(serde_json::json!({
            "latest_seq": 42,
            "count": 1,
            "events": []
        }));
        assert!(r.delivered);
        assert_eq!(r.seq, Some(42));
    }

    #[test]
    fn delivered_extracts_seq_fallback() {
        let r = delivered(serde_json::json!({ "seq": 7 }));
        assert!(r.delivered);
        assert_eq!(r.seq, Some(7));
    }

    #[test]
    fn delivered_no_seq_when_missing() {
        let r = delivered(serde_json::json!({ "status": "ok" }));
        assert!(r.delivered);
        assert!(r.seq.is_none());
    }

    #[test]
    fn audit_log_without_neural_api() {
        let result = audit_log("test.event", "ludospring", &serde_json::json!({})).unwrap();
        assert!(!result.delivered);
    }

    #[test]
    fn audit_session_without_neural_api() {
        let result = audit_session(
            "session.begin",
            "sess-001",
            &serde_json::json!({"players": 1}),
        )
        .unwrap();
        assert!(!result.delivered);
    }

    #[test]
    fn audit_certification_without_neural_api() {
        let result = audit_certification(3, 54, 0, 0).unwrap();
        assert!(!result.delivered);
    }

    #[test]
    fn audit_validation_without_neural_api() {
        let result = audit_validation("s_interaction_laws", "science", 12, 0).unwrap();
        assert!(!result.delivered);
    }

    #[test]
    fn query_audit_trail_without_neural_api() {
        let result = query_audit_trail(0, 100).unwrap();
        assert!(!result.delivered);
    }

    #[test]
    fn audit_result_serde_round_trip() {
        let original = AuditResult {
            delivered: true,
            seq: Some(42),
            data: serde_json::json!({ "events": [] }),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let back: AuditResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.delivered, original.delivered);
        assert_eq!(back.seq, original.seq);
        assert_eq!(back.data, original.data);
    }
}
