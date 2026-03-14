// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp065 — Cross-Domain Fraud Detection
//!
//! Proves that fraud detection is domain-agnostic — the same graph analysis
//! catches fraud in gaming, science, and medical domains.

mod unified;

use std::collections::HashMap;

use ludospring_barracuda::validation::ValidationResult;
use unified::{
    DomainVocabulary, GenericEvent, GenericFraudDetector, GenericFraudReport, GenericFraudType,
    GenericOp, compute_structural_similarity, gaming_vocabulary, medical_vocabulary,
    relabel_report, science_vocabulary,
};

const EXP: &str = "exp065_cross_domain_fraud";

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

// =============================================================================
// 1. Vocabulary mapping validation
// =============================================================================

fn validate_vocabulary_mapping() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let all_ops = [
        GenericOp::CreateObject,
        GenericOp::TransferObject,
        GenericOp::TransformObject,
        GenericOp::ConsumeObject,
        GenericOp::AuditObject,
        GenericOp::GrantAccess,
        GenericOp::RevokeAccess,
    ];

    let gaming = gaming_vocabulary();
    for op in all_ops {
        results.push(ValidationResult::check(
            EXP,
            &format!("gaming_covers_{op:?}"),
            bool_f64(gaming.op_labels.contains_key(&op)),
            1.0,
            0.0,
        ));
    }

    results.push(ValidationResult::check(
        EXP,
        "gaming_create_maps_to_item_spawn",
        bool_f64(gaming.op_labels.get(&GenericOp::CreateObject) == Some(&"ItemSpawn".to_string())),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "gaming_transfer_maps_to_item_trade",
        bool_f64(
            gaming.op_labels.get(&GenericOp::TransferObject) == Some(&"ItemTrade".to_string()),
        ),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "gaming_consume_maps_to_item_consume_fire",
        bool_f64(
            gaming.op_labels.get(&GenericOp::ConsumeObject)
                == Some(&"ItemConsume/Fire".to_string()),
        ),
        1.0,
        0.0,
    ));

    let science = science_vocabulary();
    for op in all_ops {
        results.push(ValidationResult::check(
            EXP,
            &format!("science_covers_{op:?}"),
            bool_f64(science.op_labels.contains_key(&op)),
            1.0,
            0.0,
        ));
    }
    results.push(ValidationResult::check(
        EXP,
        "science_create_maps_to_sample_collect",
        bool_f64(
            science.op_labels.get(&GenericOp::CreateObject) == Some(&"SampleCollect".to_string()),
        ),
        1.0,
        0.0,
    ));

    let medical = medical_vocabulary();
    for op in all_ops {
        results.push(ValidationResult::check(
            EXP,
            &format!("medical_covers_{op:?}"),
            bool_f64(medical.op_labels.contains_key(&op)),
            1.0,
            0.0,
        ));
    }
    results.push(ValidationResult::check(
        EXP,
        "medical_create_maps_to_record_creation",
        bool_f64(
            medical.op_labels.get(&GenericOp::CreateObject) == Some(&"RecordCreation".to_string()),
        ),
        1.0,
        0.0,
    ));

    for v in [&gaming, &science, &medical] {
        for op in all_ops {
            let label = v.op_labels.get(&op).map_or("", |s| s.as_str());
            results.push(ValidationResult::check(
                EXP,
                &format!("{}_no_empty_{op:?}", v.domain_name),
                bool_f64(!label.is_empty()),
                1.0,
                0.0,
            ));
        }
    }

    results
}

// =============================================================================
// 2. Generic fraud detection validation
// =============================================================================

fn event(op: GenericOp, actor: &str, target: &str, tick: u64) -> GenericEvent {
    GenericEvent {
        op,
        actor: actor.to_string(),
        target: target.to_string(),
        tick,
        metadata: HashMap::new(),
    }
}

fn event_with_meta(
    op: GenericOp,
    actor: &str,
    target: &str,
    tick: u64,
    scope: &str,
) -> GenericEvent {
    let mut metadata = HashMap::new();
    metadata.insert("scope".to_string(), scope.to_string());
    GenericEvent {
        op,
        actor: actor.to_string(),
        target: target.to_string(),
        tick,
        metadata,
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "validation section — sequential checks"
)]
#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_generic_fraud_detection() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let vocab = gaming_vocabulary();

    // OrphanObject: TransformObject with no prior CreateObject
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::TransformObject, "alice", "item_x", 10));
    let reports = det.detect();
    let has_orphan = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::OrphanObject);
    results.push(ValidationResult::check(
        EXP,
        "orphan_object_detected",
        bool_f64(has_orphan),
        1.0,
        0.0,
    ));

    // OrphanObject: ConsumeObject with no prior CreateObject
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::ConsumeObject, "bob", "item_y", 5));
    let reports = det.detect();
    let has_orphan = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::OrphanObject);
    results.push(ValidationResult::check(
        EXP,
        "orphan_consume_detected",
        bool_f64(has_orphan),
        1.0,
        0.0,
    ));

    // DuplicateIdentity: two CreateObject for same target
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::CreateObject, "alice", "item_z", 1));
    det.add_event(event(GenericOp::CreateObject, "bob", "item_z", 2));
    let reports = det.detect();
    let has_dup = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::DuplicateIdentity);
    results.push(ValidationResult::check(
        EXP,
        "duplicate_identity_detected",
        bool_f64(has_dup),
        1.0,
        0.0,
    ));

    // UnauthorizedAction: TransformObject by actor who never had access
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::CreateObject, "alice", "item_a", 1));
    det.add_event(event(GenericOp::TransformObject, "bob", "item_a", 2)); // bob never had access
    let reports = det.detect();
    let has_unauth = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::UnauthorizedAction);
    results.push(ValidationResult::check(
        EXP,
        "unauthorized_action_detected",
        bool_f64(has_unauth),
        1.0,
        0.0,
    ));

    // ScopeViolation: ConsumeObject when scope doesn't include target
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::CreateObject, "alice", "item_b", 1));
    det.add_event(event_with_meta(
        GenericOp::ConsumeObject,
        "alice",
        "item_c",
        2,
        "item_b,item_d",
    )); // scope has item_b, item_d but not item_c
    let reports = det.detect();
    let has_scope = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::ScopeViolation);
    results.push(ValidationResult::check(
        EXP,
        "scope_violation_detected",
        bool_f64(has_scope),
        1.0,
        0.0,
    ));

    // BrokenChain: TransferObject by non-holder
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::CreateObject, "alice", "item_e", 1));
    det.add_event(event(GenericOp::TransferObject, "bob", "item_e", 2)); // bob doesn't hold item_e
    let reports = det.detect();
    let has_broken = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::BrokenChain);
    results.push(ValidationResult::check(
        EXP,
        "broken_chain_detected",
        bool_f64(has_broken),
        1.0,
        0.0,
    ));

    // Clean sequence: CreateObject -> TransferObject (valid) -> TransformObject (authorized)
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::CreateObject, "alice", "item_f", 1));
    det.add_event(event(GenericOp::GrantAccess, "bob", "item_f", 2));
    det.add_event(event(GenericOp::TransformObject, "bob", "item_f", 3));
    let reports = det.detect();
    results.push(ValidationResult::check(
        EXP,
        "clean_sequence_zero_fraud",
        reports.len() as f64,
        0.0,
        0.0,
    ));

    // Clean: CreateObject -> TransferObject by creator (valid)
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::CreateObject, "alice", "item_g", 1));
    det.add_event(event(GenericOp::TransferObject, "alice", "item_g", 2));
    let reports = det.detect();
    let has_broken = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::BrokenChain);
    results.push(ValidationResult::check(
        EXP,
        "valid_transfer_no_broken_chain",
        bool_f64(!has_broken),
        1.0,
        0.0,
    ));

    // Clean: CreateObject -> ConsumeObject with scope including target
    let mut det = GenericFraudDetector::new(vocab);
    det.add_event(event(GenericOp::CreateObject, "alice", "item_h", 1));
    det.add_event(event_with_meta(
        GenericOp::ConsumeObject,
        "alice",
        "item_h",
        2,
        "item_h",
    ));
    let reports = det.detect();
    let has_scope = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::ScopeViolation);
    results.push(ValidationResult::check(
        EXP,
        "valid_consume_in_scope",
        bool_f64(!has_scope),
        1.0,
        0.0,
    ));

    results
}

// =============================================================================
// 3. Cross-domain equivalence validation
// =============================================================================

/// Build the same DAG shape (same ops, same structure) for a given vocabulary.
fn build_same_dag(vocab: &DomainVocabulary) -> GenericFraudDetector {
    let mut det = GenericFraudDetector::new(vocab.clone());
    // DAG with one fraud: OrphanObject (TransformObject before CreateObject)
    det.add_event(event(GenericOp::TransformObject, "actor1", "obj_x", 1));
    det.add_event(event(GenericOp::CreateObject, "actor2", "obj_x", 2)); // too late
    det.add_event(event(GenericOp::CreateObject, "actor1", "obj_y", 3));
    det.add_event(event(GenericOp::CreateObject, "actor1", "obj_y", 4)); // DuplicateIdentity
    det.add_event(event(GenericOp::TransferObject, "actor2", "obj_y", 5)); // BrokenChain - actor2 doesn't hold
    det
}

#[expect(
    clippy::too_many_lines,
    reason = "validation section — sequential checks"
)]
fn validate_cross_domain_equivalence() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let gaming_det = build_same_dag(&gaming_vocabulary());
    let science_det = build_same_dag(&science_vocabulary());
    let medical_det = build_same_dag(&medical_vocabulary());

    let gaming_reports = gaming_det.detect();
    let science_reports = science_det.detect();
    let medical_reports = medical_det.detect();

    let gaming_types: std::collections::HashSet<_> =
        gaming_reports.iter().map(|r| r.fraud_type).collect();
    let science_types: std::collections::HashSet<_> =
        science_reports.iter().map(|r| r.fraud_type).collect();
    let medical_types: std::collections::HashSet<_> =
        medical_reports.iter().map(|r| r.fraud_type).collect();

    results.push(ValidationResult::check(
        EXP,
        "gaming_detects_orphan",
        bool_f64(gaming_types.contains(&GenericFraudType::OrphanObject)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "science_detects_orphan",
        bool_f64(science_types.contains(&GenericFraudType::OrphanObject)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "medical_detects_orphan",
        bool_f64(medical_types.contains(&GenericFraudType::OrphanObject)),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "gaming_detects_duplicate",
        bool_f64(gaming_types.contains(&GenericFraudType::DuplicateIdentity)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "science_detects_duplicate",
        bool_f64(science_types.contains(&GenericFraudType::DuplicateIdentity)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "medical_detects_duplicate",
        bool_f64(medical_types.contains(&GenericFraudType::DuplicateIdentity)),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "gaming_detects_broken_chain",
        bool_f64(gaming_types.contains(&GenericFraudType::BrokenChain)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "science_detects_broken_chain",
        bool_f64(science_types.contains(&GenericFraudType::BrokenChain)),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "medical_detects_broken_chain",
        bool_f64(medical_types.contains(&GenericFraudType::BrokenChain)),
        1.0,
        0.0,
    ));

    let sim_gaming_science = compute_structural_similarity(&gaming_reports, &science_reports);
    let sim_gaming_medical = compute_structural_similarity(&gaming_reports, &medical_reports);
    let sim_science_medical = compute_structural_similarity(&science_reports, &medical_reports);

    // Similarity >= 0.8: use expected=1.0, tolerance=0.2 so [0.8, 1.2] passes
    results.push(ValidationResult::check(
        EXP,
        "similarity_gaming_science_gt_80",
        sim_gaming_science,
        1.0,
        0.2,
    ));
    results.push(ValidationResult::check(
        EXP,
        "similarity_gaming_medical_gt_80",
        sim_gaming_medical,
        1.0,
        0.2,
    ));
    results.push(ValidationResult::check(
        EXP,
        "similarity_science_medical_gt_80",
        sim_science_medical,
        1.0,
        0.2,
    ));

    results
}

// =============================================================================
// 4. Unification table validation
// =============================================================================

fn validate_unification_table() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let gaming = gaming_vocabulary();
    let science = science_vocabulary();
    let medical = medical_vocabulary();

    let base_report = GenericFraudReport {
        fraud_type: GenericFraudType::OrphanObject,
        description: "test".to_string(),
        domain_label: String::new(),
    };

    let gaming_det = GenericFraudDetector::new(gaming.clone());
    let relabeled_gaming = gaming_det.relabel_report(&base_report);
    let relabeled_science = relabel_report(&base_report, &science);
    let relabeled_medical = relabel_report(&base_report, &medical);

    results.push(ValidationResult::check(
        EXP,
        "orphan_maps_to_orphan_item_gaming",
        bool_f64(relabeled_gaming.domain_label == "OrphanItem"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "orphan_maps_to_phantom_sample_science",
        bool_f64(relabeled_science.domain_label == "PhantomSample"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "orphan_maps_to_phantom_access_medical",
        bool_f64(relabeled_medical.domain_label == "PhantomAccess"),
        1.0,
        0.0,
    ));

    let dup_report = GenericFraudReport {
        fraud_type: GenericFraudType::DuplicateIdentity,
        description: "test".to_string(),
        domain_label: String::new(),
    };
    results.push(ValidationResult::check(
        EXP,
        "duplicate_maps_to_duplicate_item_gaming",
        bool_f64(relabel_report(&dup_report, &gaming).domain_label == "DuplicateItem"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "duplicate_maps_to_duplicate_sample_science",
        bool_f64(relabel_report(&dup_report, &science).domain_label == "DuplicateSample"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "duplicate_maps_to_duplicate_record_medical",
        bool_f64(relabel_report(&dup_report, &medical).domain_label == "DuplicateRecord"),
        1.0,
        0.0,
    ));

    results
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    let mut all_results = Vec::new();
    all_results.extend(validate_vocabulary_mapping());
    all_results.extend(validate_generic_fraud_detection());
    all_results.extend(validate_cross_domain_equivalence());
    all_results.extend(validate_unification_table());

    let total = all_results.len();
    let passed = all_results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    println!("\n=== {EXP} ===");
    println!("{passed}/{total} checks passed");

    if failed > 0 {
        for r in &all_results {
            if !r.passed {
                println!("  FAIL: {}", r.description);
            }
        }
        std::process::exit(1);
    }
}
