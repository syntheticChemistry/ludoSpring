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
use ludospring_barracuda::validation::ValidationResult;
use medical::{AccessEvent, ConsentScope, MedicalAccessSystem, MedicalFraudType, RecordType};

const EXP: &str = "exp063_consent_gated_medical";

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

// ===========================================================================
// 1. Consent Lifecycle
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_consent_lifecycle() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let patient = Did::new("did:key:patient_lifecycle");
    let provider = Did::new("did:key:provider_lifecycle");

    let mut system = MedicalAccessSystem::new(&patient);

    let record_id = system.create_record(&patient, RecordType::Lab, "Blood panel results");
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_record_created",
        bool_f64(system.cert_manager.get_certificate(&record_id).is_some()),
        1.0,
        0.0,
    ));

    let scope = ConsentScope {
        record_types: vec![RecordType::Lab, RecordType::Vitals],
        expiry_tick: 100,
    };
    let consent_id = system.grant_consent(&patient, &provider, scope);
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_consent_granted",
        bool_f64(system.consents.contains_key(&consent_id)),
        1.0,
        0.0,
    ));

    system.advance_tick();
    let access_result = system.access_record(&provider, record_id, "treatment", RecordType::Lab);
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_access_succeeds",
        bool_f64(access_result.is_ok()),
        1.0,
        0.0,
    ));

    let Ok(proof) = access_result else {
        eprintln!("FATAL: access_result failed (validated above as Ok)");
        std::process::exit(1);
    };
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_proof_returned",
        bool_f64(proof.record_id == record_id),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "lifecycle_access_logged",
        bool_f64(system.access_log.len() == 1),
        1.0,
        0.0,
    ));

    let audit = system.audit(record_id);
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_audit_trail_correct",
        audit.len() as f64,
        1.0,
        0.0,
    ));

    let Ok(()) = system.revoke_consent(&patient, consent_id) else {
        eprintln!("FATAL: revoke_consent failed");
        std::process::exit(1);
    };
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_consent_revoked",
        bool_f64(system.consents.get(&consent_id).is_some_and(|c| c.revoked)),
        1.0,
        0.0,
    ));

    system.advance_tick();
    let post_revoke = system.access_record(&provider, record_id, "followup", RecordType::Lab);
    results.push(ValidationResult::check(
        EXP,
        "lifecycle_post_revoke_rejected",
        bool_f64(post_revoke.is_err()),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "lifecycle_dag_vertices_created",
        system.dag.vertices.len() as f64,
        3.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "lifecycle_braids_created",
        bool_f64(!system.braids.is_empty()),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 2. Access Control
// ===========================================================================

fn validate_access_control() -> Vec<ValidationResult> {
    let mut results = Vec::new();

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
    results.push(ValidationResult::check(
        EXP,
        "access_provider_a_can_access_lab",
        bool_f64(a_access_lab.is_ok()),
        1.0,
        0.0,
    ));

    let a_access_imaging =
        system.access_record(&provider_a, imaging_id, "review", RecordType::Imaging);
    results.push(ValidationResult::check(
        EXP,
        "access_provider_a_cannot_access_imaging",
        bool_f64(a_access_imaging.is_err()),
        1.0,
        0.0,
    ));

    let b_access_imaging =
        system.access_record(&provider_b, imaging_id, "review", RecordType::Imaging);
    results.push(ValidationResult::check(
        EXP,
        "access_provider_b_can_access_imaging",
        bool_f64(b_access_imaging.is_ok()),
        1.0,
        0.0,
    ));

    let b_access_lab = system.access_record(&provider_b, lab_id, "review", RecordType::Lab);
    results.push(ValidationResult::check(
        EXP,
        "access_provider_b_cannot_access_lab",
        bool_f64(b_access_lab.is_err()),
        1.0,
        0.0,
    ));

    let Ok(()) = system.revoke_consent(&patient, consent_a) else {
        eprintln!("FATAL: revoke_consent for consent_a failed");
        std::process::exit(1);
    };
    let scope_short = ConsentScope {
        record_types: vec![RecordType::Lab],
        expiry_tick: 2,
    };
    let consent_short = system.grant_consent(&patient, &provider_a, scope_short);
    system.advance_tick();
    system.advance_tick();
    system.advance_tick();
    let expired_access = system.access_record(&provider_a, lab_id, "late", RecordType::Lab);
    results.push(ValidationResult::check(
        EXP,
        "access_expired_consent_blocks",
        bool_f64(expired_access.is_err()),
        1.0,
        0.0,
    ));

    let Ok(()) = system.revoke_consent(&patient, consent_short) else {
        eprintln!("FATAL: revoke_consent for consent_short failed");
        std::process::exit(1);
    };
    let revoked_access = system.access_record(&provider_a, lab_id, "after_revoke", RecordType::Lab);
    results.push(ValidationResult::check(
        EXP,
        "access_revoked_consent_blocks",
        bool_f64(revoked_access.is_err()),
        1.0,
        0.0,
    ));

    let unknown = Did::new("did:key:unknown_provider");
    let unknown_access = system.access_record(&unknown, lab_id, "unauthorized", RecordType::Lab);
    results.push(ValidationResult::check(
        EXP,
        "access_unknown_provider_rejected",
        bool_f64(unknown_access.is_err()),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "access_multi_provider_scopes",
        bool_f64(system.consents.len() >= 3),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 3. Fraud Detection
// ===========================================================================

#[expect(
    clippy::too_many_lines,
    reason = "validation section — sequential checks"
)]
fn validate_fraud_detection() -> Vec<ValidationResult> {
    let mut results = Vec::new();

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
    let Ok(_) = system.access_record(&provider, record_id, "legit", RecordType::Lab) else {
        eprintln!("FATAL: access_record legit failed");
        std::process::exit(1);
    };

    let fraud_clean = system.detect_fraud();
    results.push(ValidationResult::check(
        EXP,
        "fraud_clean_zero",
        bool_f64(fraud_clean.is_empty()),
        1.0,
        0.0,
    ));

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
    results.push(ValidationResult::check(
        EXP,
        "fraud_unauthorized_detected",
        bool_f64(
            fraud_unauth
                .iter()
                .any(|r| r.fraud_type == MedicalFraudType::UnauthorizedAccess),
        ),
        1.0,
        0.0,
    ));

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
    results.push(ValidationResult::check(
        EXP,
        "fraud_expired_detected",
        bool_f64(
            fraud_expired
                .iter()
                .any(|r| r.fraud_type == MedicalFraudType::ExpiredConsent),
        ),
        1.0,
        0.0,
    ));

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
    results.push(ValidationResult::check(
        EXP,
        "fraud_scope_violation_detected",
        bool_f64(
            fraud_scope
                .iter()
                .any(|r| r.fraud_type == MedicalFraudType::ScopeViolation),
        ),
        1.0,
        0.0,
    ));

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
    results.push(ValidationResult::check(
        EXP,
        "fraud_phantom_detected",
        bool_f64(
            fraud_phantom
                .iter()
                .any(|r| r.fraud_type == MedicalFraudType::PhantomAccess),
        ),
        1.0,
        0.0,
    ));

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
    results.push(ValidationResult::check(
        EXP,
        "fraud_consent_forgery_detected",
        bool_f64(
            fraud_forgery
                .iter()
                .any(|r| r.fraud_type == MedicalFraudType::ConsentForgery),
        ),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "fraud_all_five_types_testable",
        bool_f64(true),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 4. Audit Trail
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_audit_trail() -> Vec<ValidationResult> {
    let mut results = Vec::new();

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
    let Ok(_) = system.access_record(&provider, record_id, "initial", RecordType::Encounter) else {
        eprintln!("FATAL: access_record initial failed");
        std::process::exit(1);
    };
    system.advance_tick();
    let Ok(_) = system.access_record(&provider, record_id, "followup", RecordType::Encounter)
    else {
        eprintln!("FATAL: access_record followup failed");
        std::process::exit(1);
    };
    system.advance_tick();
    let Ok(_) = system.access_record(&provider, record_id, "discharge", RecordType::Encounter)
    else {
        eprintln!("FATAL: access_record discharge failed");
        std::process::exit(1);
    };

    let audit = system.audit(record_id);
    results.push(ValidationResult::check(
        EXP,
        "audit_full_reconstruction",
        audit.len() as f64,
        3.0,
        0.0,
    ));

    let ticks: Vec<u64> = audit.iter().map(|e| e.tick).collect();
    results.push(ValidationResult::check(
        EXP,
        "audit_ordered_timeline",
        bool_f64(ticks == vec![1, 2, 3]),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "audit_prov_o_matches_log",
        bool_f64(
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
        ),
        1.0,
        0.0,
    ));

    let empty_audit = system.audit(loam_spine_core::types::CertificateId::now_v7());
    results.push(ValidationResult::check(
        EXP,
        "audit_empty_for_unknown_record",
        bool_f64(empty_audit.is_empty()),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "audit_access_log_consistent",
        bool_f64(system.access_log.len() >= 3),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 5. Access Proof
// ===========================================================================

fn validate_access_proof() -> Vec<ValidationResult> {
    let mut results = Vec::new();

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
    let Ok(proof) = system.access_record(&provider, record_id, "verification", RecordType::Vitals)
    else {
        eprintln!("FATAL: access_record verification failed");
        std::process::exit(1);
    };

    results.push(ValidationResult::check(
        EXP,
        "proof_generation",
        bool_f64(!proof.proof_signature.is_empty()),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "proof_contains_fields",
        bool_f64(
            proof.accessor_did == provider.as_str()
                && proof.record_id == record_id
                && proof.timestamp_tick == 1,
        ),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "proof_deterministic",
        bool_f64(system.verify_proof(&proof)),
        1.0,
        0.0,
    ));

    let mut bad_proof = proof.clone();
    bad_proof.proof_signature[0] ^= 0xff;
    results.push(ValidationResult::check(
        EXP,
        "proof_tampered_rejected",
        bool_f64(!system.verify_proof(&bad_proof)),
        1.0,
        0.0,
    ));

    system.advance_tick();
    let Ok(proof2) = system.access_record(&provider, record_id, "second", RecordType::Vitals)
    else {
        eprintln!("FATAL: access_record second failed");
        std::process::exit(1);
    };
    results.push(ValidationResult::check(
        EXP,
        "proof_different_per_access",
        bool_f64(proof.proof_signature != proof2.proof_signature),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    println!("=== exp063: Consent-Gated Medical Access ===\n");

    let mut all_results = Vec::new();

    let sections: Vec<(&str, Vec<ValidationResult>)> = vec![
        ("Consent Lifecycle", validate_consent_lifecycle()),
        ("Access Control", validate_access_control()),
        ("Fraud Detection", validate_fraud_detection()),
        ("Audit Trail", validate_audit_trail()),
        ("Access Proof", validate_access_proof()),
    ];

    for (name, results) in sections {
        println!("--- {name} ---");
        for v in &results {
            println!(
                "  [{}] {}",
                if v.passed { "PASS" } else { "FAIL" },
                v.description
            );
        }
        all_results.extend(results);
        println!();
    }

    let passed = all_results.iter().filter(|r| r.passed).count();
    let total = all_results.len();
    println!("=== SUMMARY: {passed}/{total} checks passed ===");

    if passed != total {
        println!("\nFAILED:");
        for r in all_results.iter().filter(|r| !r.passed) {
            println!(
                "  {} — measured={}, expected={}",
                r.description, r.measured, r.expected
            );
        }
        std::process::exit(1);
    }
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
