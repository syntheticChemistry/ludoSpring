// SPDX-License-Identifier: AGPL-3.0-or-later
//! loamSpine certificate operations — permanent records for game entities.
//!
//! Routes certificate and spine capability calls through [`NeuralBridge`]:
//!
//! - `certificate.mint` / `certificate.get` / `certificate.verify` — NPC
//!   personality, ruleset, character sheet, and world lore certificates
//! - `certificate.lifecycle` — full history of certificate evolution
//! - `certificate.loan` — share NPCs between campaigns
//! - `spine.create` / `entry.append` — persistent progression spines

use crate::ipc::envelope::IpcError;
use crate::ipc::neural_bridge::NeuralBridge;

use super::ProvenanceResult;
use super::unavailable_result;

/// JSON-RPC capability namespace for certificate operations.
pub const CAP_CERTIFICATE: &str = "certificate";

/// JSON-RPC capability namespace for loamSpine spine creation.
pub const CAP_SPINE: &str = "spine";

/// JSON-RPC capability namespace for spine entry append.
pub const CAP_ENTRY: &str = "entry";

/// `certificate.mint` operation name.
pub const OP_CERT_MINT: &str = "mint";

/// `certificate.get` operation name.
pub const OP_CERT_GET: &str = "get";

/// `certificate.verify` operation name.
pub const OP_CERT_VERIFY: &str = "verify";

/// `certificate.lifecycle` operation name.
pub const OP_CERT_LIFECYCLE: &str = "lifecycle";

/// `certificate.loan` operation name.
pub const OP_CERT_LOAN: &str = "loan";

/// `spine.create` operation name.
pub const OP_SPINE_CREATE: &str = "create";

/// `entry.append` operation name.
pub const OP_ENTRY_APPEND: &str = "append";

/// JSON-RPC args for `certificate.mint`.
pub(crate) fn mint_certificate_params(
    cert_type: &str,
    owner: &str,
    payload: &serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "owner": owner,
        "type": cert_type,
        "metadata": {
            "domain": "game",
            "primal": crate::PRIMAL_NAME,
        },
        "payload": payload,
    })
}

/// Maps a successful `certificate.mint` response to [`ProvenanceResult`].
pub(crate) fn map_mint_certificate_response(result: serde_json::Value) -> ProvenanceResult {
    let cert_id = result
        .get("cert_id")
        .or_else(|| result.get("certificate_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    ProvenanceResult {
        id: cert_id,
        available: true,
        data: result,
    }
}

/// JSON-RPC args for certificate operations keyed by `cert_id` only.
pub(crate) fn certificate_by_id_params(cert_id: &str) -> serde_json::Value {
    serde_json::json!({ "cert_id": cert_id })
}

/// Wraps a certificate response that uses the request `cert_id` as result id.
pub(crate) fn provenance_result_with_cert_id(
    cert_id: &str,
    result: serde_json::Value,
) -> ProvenanceResult {
    ProvenanceResult {
        id: cert_id.to_string(),
        available: true,
        data: result,
    }
}

/// JSON-RPC args for `spine.create`.
pub(crate) fn create_spine_params(owner: &str, metadata: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "owner": owner,
        "metadata": metadata,
    })
}

/// Maps a successful `spine.create` response to [`ProvenanceResult`].
pub(crate) fn map_create_spine_response(result: serde_json::Value) -> ProvenanceResult {
    let spine_id = result
        .get("spine_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    ProvenanceResult {
        id: spine_id,
        available: true,
        data: result,
    }
}

/// JSON-RPC args for `entry.append`.
pub(crate) fn append_spine_entry_params(
    spine_id: &str,
    entry_type: &str,
    payload: &serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "spine_id": spine_id,
        "entry_type": entry_type,
        "payload": payload,
    })
}

/// Maps a successful `entry.append` response to [`ProvenanceResult`].
pub(crate) fn map_append_spine_entry_response(result: serde_json::Value) -> ProvenanceResult {
    let entry_id = result
        .get("entry_hash")
        .or_else(|| result.get("entry_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    ProvenanceResult {
        id: entry_id,
        available: true,
        data: result,
    }
}

/// JSON-RPC args for `certificate.loan`.
pub(crate) fn loan_certificate_params(
    cert_id: &str,
    borrower: &str,
    terms: &serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "cert_id": cert_id,
        "borrower": borrower,
        "terms": terms,
    })
}

/// Mint a certificate (NPC personality, ruleset, character sheet, world lore).
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn mint_certificate(
    cert_type: &str,
    owner: &str,
    payload: &serde_json::Value,
) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = mint_certificate_params(cert_type, owner, payload);

    bridge
        .capability_call(CAP_CERTIFICATE, OP_CERT_MINT, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(map_mint_certificate_response(result)),
        )
}

/// Retrieve a certificate by ID.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn get_certificate(cert_id: &str) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = certificate_by_id_params(cert_id);

    bridge
        .capability_call(CAP_CERTIFICATE, OP_CERT_GET, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_with_cert_id(cert_id, result)),
        )
}

/// Verify a certificate's authenticity and integrity.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn verify_certificate(cert_id: &str) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = certificate_by_id_params(cert_id);

    bridge
        .capability_call(CAP_CERTIFICATE, OP_CERT_VERIFY, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_with_cert_id(cert_id, result)),
        )
}

/// Get the full lifecycle history of a certificate (NPC evolution across sessions).
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn certificate_lifecycle(cert_id: &str) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = certificate_by_id_params(cert_id);

    bridge
        .capability_call(CAP_CERTIFICATE, OP_CERT_LIFECYCLE, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_with_cert_id(cert_id, result)),
        )
}

/// Create a loamSpine spine for persistent progression (character, campaign).
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn create_spine(owner: &str, metadata: &serde_json::Value) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = create_spine_params(owner, metadata);

    bridge
        .capability_call(CAP_SPINE, OP_SPINE_CREATE, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(map_create_spine_response(result)),
        )
}

/// Append an entry to a loamSpine spine.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn append_spine_entry(
    spine_id: &str,
    entry_type: &str,
    payload: &serde_json::Value,
) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = append_spine_entry_params(spine_id, entry_type, payload);

    bridge
        .capability_call(CAP_ENTRY, OP_ENTRY_APPEND, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(map_append_spine_entry_response(result)),
        )
}

/// Loan a certificate to another agent (shared NPCs between campaigns).
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn loan_certificate(
    cert_id: &str,
    borrower: &str,
    terms: &serde_json::Value,
) -> Result<ProvenanceResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = loan_certificate_params(cert_id, borrower, terms);

    bridge
        .capability_call(CAP_CERTIFICATE, OP_CERT_LOAN, &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| Ok(provenance_result_with_cert_id(cert_id, result)),
        )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn mint_certificate_degrades() {
        let payload = serde_json::json!({"name": "Test NPC"});
        let r = mint_certificate("npc_personality", "did:key:test", &payload).unwrap();
        assert!(!r.available);
    }

    #[test]
    fn get_certificate_degrades() {
        let r = get_certificate("cert-001").unwrap();
        assert!(!r.available);
    }

    #[test]
    fn verify_certificate_degrades() {
        let r = verify_certificate("cert-001").unwrap();
        assert!(!r.available);
    }

    #[test]
    fn certificate_lifecycle_degrades() {
        let r = certificate_lifecycle("cert-001").unwrap();
        assert!(!r.available);
    }

    #[test]
    fn create_spine_degrades() {
        let meta = serde_json::json!({"type": "character"});
        let r = create_spine("did:key:test", &meta).unwrap();
        assert!(!r.available);
    }

    #[test]
    fn append_spine_entry_degrades() {
        let payload = serde_json::json!({"level": 5});
        let r = append_spine_entry("spine-1", "level_up", &payload).unwrap();
        assert!(!r.available);
    }

    #[test]
    fn loan_certificate_degrades() {
        let terms = serde_json::json!({"duration_secs": 3600});
        let r = loan_certificate("cert-001", "did:key:borrower", &terms).unwrap();
        assert!(!r.available);
    }

    #[test]
    fn capability_names_match_expected_strings() {
        assert_eq!(CAP_CERTIFICATE, "certificate");
        assert_eq!(CAP_SPINE, "spine");
        assert_eq!(CAP_ENTRY, "entry");
    }

    #[test]
    fn mint_certificate_params_includes_owner_type_payload_and_metadata() {
        let payload = serde_json::json!({"k": "v"});
        let v = mint_certificate_params("npc", "did:o:1", &payload);
        assert_eq!(v["owner"], "did:o:1");
        assert_eq!(v["type"], "npc");
        assert_eq!(v["payload"], payload);
        assert_eq!(v["metadata"]["domain"], "game");
        assert_eq!(v["metadata"]["primal"], crate::PRIMAL_NAME);
    }

    #[test]
    fn map_mint_certificate_response_prefers_cert_id_then_certificate_id() {
        let r = map_mint_certificate_response(serde_json::json!({"cert_id": "a"}));
        assert_eq!(r.id, "a");
        assert!(r.available);
        let r2 = map_mint_certificate_response(serde_json::json!({"certificate_id": "b"}));
        assert_eq!(r2.id, "b");
    }

    #[test]
    fn certificate_by_id_params_shape() {
        let v = certificate_by_id_params("cid");
        assert_eq!(v["cert_id"], "cid");
    }

    #[test]
    fn provenance_result_with_cert_id_roundtrip_serde() {
        let p = provenance_result_with_cert_id("x", serde_json::json!({"ok": true}));
        let json = serde_json::to_string(&p).unwrap();
        let back: ProvenanceResult = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn create_spine_params_shape() {
        let m = serde_json::json!({"t": 1});
        let v = create_spine_params("owner1", &m);
        assert_eq!(v["owner"], "owner1");
        assert_eq!(v["metadata"], m);
    }

    #[test]
    fn map_create_spine_response_uses_spine_id() {
        let r = map_create_spine_response(serde_json::json!({"spine_id": "s1", "extra": 2}));
        assert_eq!(r.id, "s1");
        assert_eq!(r.data["extra"], 2);
    }

    #[test]
    fn append_spine_entry_params_shape() {
        let pl = serde_json::json!({"p": 1});
        let v = append_spine_entry_params("sid", "etype", &pl);
        assert_eq!(v["spine_id"], "sid");
        assert_eq!(v["entry_type"], "etype");
        assert_eq!(v["payload"], pl);
    }

    #[test]
    fn map_append_spine_entry_response_prefers_entry_hash() {
        let r = map_append_spine_entry_response(serde_json::json!({"entry_hash": "h1"}));
        assert_eq!(r.id, "h1");
        let r2 = map_append_spine_entry_response(serde_json::json!({"entry_id": "e2"}));
        assert_eq!(r2.id, "e2");
    }

    #[test]
    fn loan_certificate_params_shape() {
        let terms = serde_json::json!({"t": 1});
        let v = loan_certificate_params("c", "borrow", &terms);
        assert_eq!(v["cert_id"], "c");
        assert_eq!(v["borrower"], "borrow");
        assert_eq!(v["terms"], terms);
    }
}
