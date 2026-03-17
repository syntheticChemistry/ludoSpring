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

use crate::ipc::neural_bridge::NeuralBridge;

use super::ProvenanceResult;
use super::unavailable_result;

/// Mint a certificate (NPC personality, ruleset, character sheet, world lore).
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn mint_certificate(
    cert_type: &str,
    owner: &str,
    payload: &serde_json::Value,
) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({
        "owner": owner,
        "type": cert_type,
        "metadata": {
            "domain": "game",
            "primal": crate::PRIMAL_NAME,
        },
        "payload": payload,
    });

    bridge
        .capability_call("certificate", "mint", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                let cert_id = result
                    .get("cert_id")
                    .or_else(|| result.get("certificate_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                Ok(ProvenanceResult {
                    id: cert_id,
                    available: true,
                    data: result,
                })
            },
        )
}

/// Retrieve a certificate by ID.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn get_certificate(cert_id: &str) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({ "cert_id": cert_id });

    bridge
        .capability_call("certificate", "get", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: cert_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}

/// Verify a certificate's authenticity and integrity.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn verify_certificate(cert_id: &str) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({ "cert_id": cert_id });

    bridge
        .capability_call("certificate", "verify", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: cert_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}

/// Get the full lifecycle history of a certificate (NPC evolution across sessions).
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn certificate_lifecycle(cert_id: &str) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({ "cert_id": cert_id });

    bridge
        .capability_call("certificate", "lifecycle", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: cert_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}

/// Create a loamSpine spine for persistent progression (character, campaign).
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn create_spine(owner: &str, metadata: &serde_json::Value) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({
        "owner": owner,
        "metadata": metadata,
    });

    bridge
        .capability_call("spine", "create", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                let spine_id = result
                    .get("spine_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                Ok(ProvenanceResult {
                    id: spine_id,
                    available: true,
                    data: result,
                })
            },
        )
}

/// Append an entry to a loamSpine spine.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn append_spine_entry(
    spine_id: &str,
    entry_type: &str,
    payload: &serde_json::Value,
) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({
        "spine_id": spine_id,
        "entry_type": entry_type,
        "payload": payload,
    });

    bridge
        .capability_call("entry", "append", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                let entry_id = result
                    .get("entry_hash")
                    .or_else(|| result.get("entry_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                Ok(ProvenanceResult {
                    id: entry_id,
                    available: true,
                    data: result,
                })
            },
        )
}

/// Loan a certificate to another agent (shared NPCs between campaigns).
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn loan_certificate(
    cert_id: &str,
    borrower: &str,
    terms: &serde_json::Value,
) -> Result<ProvenanceResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable_result());
    };

    let args = serde_json::json!({
        "cert_id": cert_id,
        "borrower": borrower,
        "terms": terms,
    });

    bridge
        .capability_call("certificate", "loan", &args)
        .map_or_else(
            |_| Ok(unavailable_result()),
            |result| {
                Ok(ProvenanceResult {
                    id: cert_id.to_string(),
                    available: true,
                    data: result,
                })
            },
        )
}

#[cfg(test)]
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
}
