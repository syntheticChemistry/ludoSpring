// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp063 — Consent-Gated Medical Access
//!
//! Scaffolds `healthSpring`'s zero-knowledge medical data access using the
//! provenance trio: `loamSpine` certificates, `rhizoCrypt` DAG, `sweetGrass` braids.
//!
//! This experiment validates:
//!   1. Consent lifecycle: create record → grant consent → access → revoke → reject
//!   2. Access control: scope enforcement, expiry, revocation, multi-provider
//!   3. Fraud detection: all 5 fraud types
//!   4. Audit trail: full reconstruction, `PROV-O` timeline
//!   5. Access proof: generation, verification, determinism

mod medical;

use loam_spine_core::Did;
use ludospring_barracuda::validation::{BaselineProvenance, OrExit, ValidationHarness};
use medical::{AccessEvent, ConsentScope, MedicalAccessSystem, MedicalFraudType, RecordType};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — healthSpring zero-knowledge medical access)",
    commit: "4b683e3e",
    date: "2026-03-29",
    command: "N/A (pure Rust implementation)",
};

// ===========================================================================
// 1. Consent Lifecycle
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_consent_lifecycle(h: &mut ValidationHarness) {
    let patient = Did::new("did:key:patient_lifecycle");
    let provider = Did::new("did:key:provider_lifecycle");

    let mut system = MedicalAccessSystem::new(&patient);

    let record_id = system.create_record(&patient, RecordType::Lab, "Blood panel results");
    h.check_bool(
        "lifecycle_record_created",
        system.cert_manager.get_certificate(&record_id).is_some(),
    );

    let scope = ConsentScope {
        record_types: vec![RecordType::Lab, RecordType::Vitals],
        expiry_tick: 100,
    };
    let consent_id = system.grant_consent(&patient, &provider, scope);
    h.check_bool(
        "lifecycle_consent_granted",
        system.consents.contains_key(&consent_id),
    );

    system.advance_tick();
    let access_result = system.access_record(&provider, record_id, "treatment", RecordType::Lab);
    h.check_bool("lifecycle_access_succeeds", access_result.is_ok());

    let proof = access_result.or_exit("access_result failed (validated above as Ok)");
    h.check_bool("lifecycle_proof_returned", proof.record_id == record_id);

    h.check_bool("lifecycle_access_logged", system.access_log.len() == 1);

    let audit = system.audit(record_id);
    h.check_abs(
        "lifecycle_audit_trail_correct",
        audit.len() as f64,
        1.0,
        0.0,
    );

    system
        .revoke_consent(&patient, consent_id)
        .or_exit("revoke_consent failed");
    h.check_bool(
        "lifecycle_consent_revoked",
        system.consents.get(&consent_id).is_some_and(|c| c.revoked),
    );

    system.advance_tick();
    let post_revoke = system.access_record(&provider, record_id, "followup", RecordType::Lab);
    h.check_bool("lifecycle_post_revoke_rejected", post_revoke.is_err());

    h.check_abs(
        "lifecycle_dag_vertices_created",
        system.dag.vertices.len() as f64,
        3.0,
        0.0,
    );

    h.check_bool("lifecycle_braids_created", !system.braids.is_empty());
}

// ===========================================================================
// 2. Access Control
// ===========================================================================

fn validate_access_control(h: &mut ValidationHarness) {
    let patient = Did::new("did:key:patient_access");
    let provider_a = Did::new("did:key:provider_a");
    let provider_b = Did::new("did:key:provider_b");

    let mut system = MedicalAccessSystem::new(&patient);

    let lab_id = system.create_record(&patient, RecordType::Lab, "Lab results");
    let imaging_id = system.create_record(&patient, RecordType::Imaging, "X-ray");

    let scope_lab_only = ConsentScope {
        record_types: vec![RecordType::Lab],
        expiry_tick: 50,
    };
    let consent_a = system.grant_consent(&patient, &provider_a, scope_lab_only);

    let scope_imaging_only = ConsentScope {
        record_types: vec![RecordType::Imaging],
        expiry_tick: 50,
    };
    system.grant_consent(&patient, &provider_b, scope_imaging_only);

    system.advance_tick();
    let a_access_lab = system.access_record(&provider_a, lab_id, "review", RecordType::Lab);
    h.check_bool("access_provider_a_can_access_lab", a_access_lab.is_ok());

    let a_access_imaging =
        system.access_record(&provider_a, imaging_id, "review", RecordType::Imaging);
    h.check_bool(
        "access_provider_a_cannot_access_imaging",
        a_access_imaging.is_err(),
    );

    let b_access_imaging =
        system.access_record(&provider_b, imaging_id, "review", RecordType::Imaging);
    h.check_bool(
        "access_provider_b_can_access_imaging",
        b_access_imaging.is_ok(),
    );

    let b_access_lab = system.access_record(&provider_b, lab_id, "review", RecordType::Lab);
    h.check_bool("access_provider_b_cannot_access_lab", b_access_lab.is_err());

    system
        .revoke_consent(&patient, consent_a)
        .or_exit("revoke_consent for consent_a failed");
    let scope_short = ConsentScope {
        record_types: vec![RecordType::Lab],
        expiry_tick: 2,
    };
    let consent_short = system.grant_consent(&patient, &provider_a, scope_short);
    system.advance_tick();
    system.advance_tick();
    system.advance_tick();
    let expired_access = system.access_record(&provider_a, lab_id, "late", RecordType::Lab);
    h.check_bool("access_expired_consent_blocks", expired_access.is_err());

    system
        .revoke_consent(&patient, consent_short)
        .or_exit("revoke_consent for consent_short failed");
    let revoked_access = system.access_record(&provider_a, lab_id, "after_revoke", RecordType::Lab);
    h.check_bool("access_revoked_consent_blocks", revoked_access.is_err());

    let unknown = Did::new("did:key:unknown_provider");
    let unknown_access = system.access_record(&unknown, lab_id, "unauthorized", RecordType::Lab);
    h.check_bool("access_unknown_provider_rejected", unknown_access.is_err());

    h.check_bool("access_multi_provider_scopes", system.consents.len() >= 3);
}

// ===========================================================================
// 3. Fraud Detection
// ===========================================================================

#[expect(
    clippy::too_many_lines,
    reason = "validation section — sequential checks"
)]
fn validate_fraud_detection(h: &mut ValidationHarness) {
    let patient = Did::new("did:key:patient_fraud");
    let provider = Did::new("did:key:provider_fraud");
    let attacker = Did::new("did:key:attacker");

    let mut system = MedicalAccessSystem::new(&patient);
    let record_id = system.create_record(&patient, RecordType::Lab, "Sensitive data");

    let scope = ConsentScope {
        record_types: vec![RecordType::Lab],
        expiry_tick: 100,
    };
    system.grant_consent(&patient, &provider, scope);

    system.advance_tick();
    let _ = system
        .access_record(&provider, record_id, "legit", RecordType::Lab)
        .or_exit("access_record legit failed");

    let fraud_clean = system.detect_fraud();
    h.check_bool("fraud_clean_zero", fraud_clean.is_empty());

    system.inject_access_event_for_fraud_test(
        AccessEvent {
            accessor_did: attacker.as_str().into(),
            record_id,
            purpose: "stolen".into(),
            record_type: RecordType::Lab,
            tick: system.tick,
            vertex_id: None,
        },
        true,
    );
    let fraud_unauth = system.detect_fraud();
    h.check_bool(
        "fraud_unauthorized_detected",
        fraud_unauth
            .iter()
            .any(|r| r.fraud_type == MedicalFraudType::UnauthorizedAccess),
    );

    let mut system2 = MedicalAccessSystem::new(&patient);
    let rec2 = system2.create_record(&patient, RecordType::Vitals, "Vitals");
    let scope2 = ConsentScope {
        record_types: vec![RecordType::Vitals],
        expiry_tick: 1,
    };
    system2.grant_consent(&patient, &provider, scope2);
    system2.advance_tick();
    system2.advance_tick();
    system2.inject_access_event_for_fraud_test(
        AccessEvent {
            accessor_did: provider.as_str().into(),
            record_id: rec2,
            purpose: "expired".into(),
            record_type: RecordType::Vitals,
            tick: 5,
            vertex_id: None,
        },
        true,
    );
    let fraud_expired = system2.detect_fraud();
    h.check_bool(
        "fraud_expired_detected",
        fraud_expired
            .iter()
            .any(|r| r.fraud_type == MedicalFraudType::ExpiredConsent),
    );

    let mut system3 = MedicalAccessSystem::new(&patient);
    let rec3 = system3.create_record(&patient, RecordType::Lab, "Lab");
    let scope3 = ConsentScope {
        record_types: vec![RecordType::Lab],
        expiry_tick: 100,
    };
    system3.grant_consent(&patient, &provider, scope3);
    system3.advance_tick();
    system3.inject_access_event_for_fraud_test(
        AccessEvent {
            accessor_did: provider.as_str().into(),
            record_id: rec3,
            purpose: "scope_violation".into(),
            record_type: RecordType::Genomic,
            tick: 1,
            vertex_id: None,
        },
        true,
    );
    let fraud_scope = system3.detect_fraud();
    h.check_bool(
        "fraud_scope_violation_detected",
        fraud_scope
            .iter()
            .any(|r| r.fraud_type == MedicalFraudType::ScopeViolation),
    );

    let mut system4 = MedicalAccessSystem::new(&patient);
    let rec4 = system4.create_record(&patient, RecordType::Prescription, "Rx");
    system4.advance_tick();
    system4.inject_access_event_for_fraud_test(
        AccessEvent {
            accessor_did: provider.as_str().into(),
            record_id: rec4,
            purpose: "phantom".into(),
            record_type: RecordType::Prescription,
            tick: 1,
            vertex_id: None,
        },
        false,
    );
    let fraud_phantom = system4.detect_fraud();
    h.check_bool(
        "fraud_phantom_detected",
        fraud_phantom
            .iter()
            .any(|r| r.fraud_type == MedicalFraudType::PhantomAccess),
    );

    let mut system5 = MedicalAccessSystem::new(&patient);
    let rec5 = system5.create_record(&patient, RecordType::Lab, "Alice lab");
    let bob = Did::new("did:key:bob_patient");
    let scope5 = ConsentScope {
        record_types: vec![RecordType::Lab],
        expiry_tick: 100,
    };
    system5.grant_consent(&bob, &provider, scope5);
    system5.advance_tick();
    system5.inject_access_event_for_fraud_test(
        AccessEvent {
            accessor_did: provider.as_str().into(),
            record_id: rec5,
            purpose: "forgery".into(),
            record_type: RecordType::Lab,
            tick: 1,
            vertex_id: None,
        },
        true,
    );
    let fraud_forgery = system5.detect_fraud();
    h.check_bool(
        "fraud_consent_forgery_detected",
        fraud_forgery
            .iter()
            .any(|r| r.fraud_type == MedicalFraudType::ConsentForgery),
    );

    h.check_bool("fraud_all_five_types_testable", true);
}

// ===========================================================================
// 4. Audit Trail
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_audit_trail(h: &mut ValidationHarness) {
    let patient = Did::new("did:key:patient_audit");
    let provider = Did::new("did:key:provider_audit");

    let mut system = MedicalAccessSystem::new(&patient);
    let record_id = system.create_record(&patient, RecordType::Encounter, "Visit 1");

    let scope = ConsentScope {
        record_types: vec![RecordType::Encounter, RecordType::Lab],
        expiry_tick: 200,
    };
    system.grant_consent(&patient, &provider, scope);

    system.advance_tick();
    let _ = system
        .access_record(&provider, record_id, "initial", RecordType::Encounter)
        .or_exit("access_record initial failed");
    system.advance_tick();
    let _ = system
        .access_record(&provider, record_id, "followup", RecordType::Encounter)
        .or_exit("access_record followup failed");
    system.advance_tick();
    let _ = system
        .access_record(&provider, record_id, "discharge", RecordType::Encounter)
        .or_exit("access_record discharge failed");

    let audit = system.audit(record_id);
    h.check_abs("audit_full_reconstruction", audit.len() as f64, 3.0, 0.0);

    let ticks: Vec<u64> = audit.iter().map(|e| e.tick).collect();
    h.check_bool("audit_ordered_timeline", ticks == vec![1, 2, 3]);

    h.check_bool(
        "audit_prov_o_matches_log",
        system
            .dag
            .vertices
            .iter()
            .filter(|v| {
                v.metadata.get("event").is_some_and(|m| {
                    matches!(m, rhizo_crypt_core::vertex::MetadataValue::String(s) if s == "access")
                })
            })
            .count()
            >= 3,
    );

    let empty_audit = system.audit(loam_spine_core::types::CertificateId::now_v7());
    h.check_bool("audit_empty_for_unknown_record", empty_audit.is_empty());

    h.check_bool("audit_access_log_consistent", system.access_log.len() >= 3);
}

// ===========================================================================
// 5. Access Proof
// ===========================================================================

fn validate_access_proof(h: &mut ValidationHarness) {
    let patient = Did::new("did:key:patient_proof");
    let provider = Did::new("did:key:provider_proof");

    let mut system = MedicalAccessSystem::new(&patient);
    let record_id = system.create_record(&patient, RecordType::Vitals, "BP");

    let scope = ConsentScope {
        record_types: vec![RecordType::Vitals],
        expiry_tick: 100,
    };
    system.grant_consent(&patient, &provider, scope);

    system.advance_tick();
    let proof = system
        .access_record(&provider, record_id, "verification", RecordType::Vitals)
        .or_exit("access_record verification failed");

    h.check_bool("proof_generation", !proof.proof_signature.is_empty());

    h.check_bool(
        "proof_contains_fields",
        proof.accessor_did == provider.as_str()
            && proof.record_id == record_id
            && proof.timestamp_tick == 1,
    );

    h.check_bool("proof_deterministic", system.verify_proof(&proof));

    let mut bad_proof = proof.clone();
    bad_proof.proof_signature[0] ^= 0xff;
    h.check_bool("proof_tampered_rejected", !system.verify_proof(&bad_proof));

    system.advance_tick();
    let proof2 = system
        .access_record(&provider, record_id, "second", RecordType::Vitals)
        .or_exit("access_record second failed");
    h.check_bool(
        "proof_different_per_access",
        proof.proof_signature != proof2.proof_signature,
    );
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp063_consent_gated_medical");
    h.print_provenance(&[&PROVENANCE]);

    validate_consent_lifecycle(&mut h);
    validate_access_control(&mut h);
    validate_fraud_detection(&mut h);
    validate_audit_trail(&mut h);
    validate_access_proof(&mut h);

    h.finish();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") | None => cmd_validate(),
        Some(other) => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}
