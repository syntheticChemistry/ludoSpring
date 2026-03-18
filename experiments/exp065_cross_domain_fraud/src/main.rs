// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp065 — Cross-Domain Fraud Detection
//!
//! Proves that fraud detection is domain-agnostic — the same graph analysis
//! catches fraud in gaming, science, and medical domains.

mod unified;

use std::collections::HashMap;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use unified::{
    DomainVocabulary, GenericEvent, GenericFraudDetector, GenericFraudReport, GenericFraudType,
    GenericOp, compute_structural_similarity, gaming_vocabulary, medical_vocabulary,
    relabel_report, science_vocabulary,
};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — domain-agnostic fraud detection)",
    commit: "N/A",
    date: "N/A",
    command: "N/A (pure Rust implementation)",
};

// =============================================================================
// 1. Vocabulary mapping validation
// =============================================================================

fn validate_vocabulary_mapping(h: &mut ValidationHarness) {
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
        h.check_bool(
            &format!("gaming_covers_{op:?}"),
            gaming.op_labels.contains_key(&op),
        );
    }

    h.check_bool(
        "gaming_create_maps_to_item_spawn",
        gaming.op_labels.get(&GenericOp::CreateObject) == Some(&"ItemSpawn".to_string()),
    );
    h.check_bool(
        "gaming_transfer_maps_to_item_trade",
        gaming.op_labels.get(&GenericOp::TransferObject) == Some(&"ItemTrade".to_string()),
    );
    h.check_bool(
        "gaming_consume_maps_to_item_consume_fire",
        gaming.op_labels.get(&GenericOp::ConsumeObject) == Some(&"ItemConsume/Fire".to_string()),
    );

    let science = science_vocabulary();
    for op in all_ops {
        h.check_bool(
            &format!("science_covers_{op:?}"),
            science.op_labels.contains_key(&op),
        );
    }
    h.check_bool(
        "science_create_maps_to_sample_collect",
        science.op_labels.get(&GenericOp::CreateObject) == Some(&"SampleCollect".to_string()),
    );

    let medical = medical_vocabulary();
    for op in all_ops {
        h.check_bool(
            &format!("medical_covers_{op:?}"),
            medical.op_labels.contains_key(&op),
        );
    }
    h.check_bool(
        "medical_create_maps_to_record_creation",
        medical.op_labels.get(&GenericOp::CreateObject) == Some(&"RecordCreation".to_string()),
    );

    for v in [&gaming, &science, &medical] {
        for op in all_ops {
            let label = v.op_labels.get(&op).map_or("", |s| s.as_str());
            h.check_bool(
                &format!("{}_no_empty_{op:?}", v.domain_name),
                !label.is_empty(),
            );
        }
    }
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
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_generic_fraud_detection(h: &mut ValidationHarness) {
    let vocab = gaming_vocabulary();

    // OrphanObject: TransformObject with no prior CreateObject
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::TransformObject, "alice", "item_x", 10));
    let reports = det.detect();
    let has_orphan = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::OrphanObject);
    h.check_bool("orphan_object_detected", has_orphan);

    // OrphanObject: ConsumeObject with no prior CreateObject
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::ConsumeObject, "bob", "item_y", 5));
    let reports = det.detect();
    let has_orphan = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::OrphanObject);
    h.check_bool("orphan_consume_detected", has_orphan);

    // DuplicateIdentity: two CreateObject for same target
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::CreateObject, "alice", "item_z", 1));
    det.add_event(event(GenericOp::CreateObject, "bob", "item_z", 2));
    let reports = det.detect();
    let has_dup = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::DuplicateIdentity);
    h.check_bool("duplicate_identity_detected", has_dup);

    // UnauthorizedAction: TransformObject by actor who never had access
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::CreateObject, "alice", "item_a", 1));
    det.add_event(event(GenericOp::TransformObject, "bob", "item_a", 2));
    let reports = det.detect();
    let has_unauth = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::UnauthorizedAction);
    h.check_bool("unauthorized_action_detected", has_unauth);

    // ScopeViolation: ConsumeObject when scope doesn't include target
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::CreateObject, "alice", "item_b", 1));
    det.add_event(event_with_meta(
        GenericOp::ConsumeObject,
        "alice",
        "item_c",
        2,
        "item_b,item_d",
    ));
    let reports = det.detect();
    let has_scope = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::ScopeViolation);
    h.check_bool("scope_violation_detected", has_scope);

    // BrokenChain: TransferObject by non-holder
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::CreateObject, "alice", "item_e", 1));
    det.add_event(event(GenericOp::TransferObject, "bob", "item_e", 2));
    let reports = det.detect();
    let has_broken = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::BrokenChain);
    h.check_bool("broken_chain_detected", has_broken);

    // Clean sequence: CreateObject -> TransferObject (valid) -> TransformObject (authorized)
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::CreateObject, "alice", "item_f", 1));
    det.add_event(event(GenericOp::GrantAccess, "bob", "item_f", 2));
    det.add_event(event(GenericOp::TransformObject, "bob", "item_f", 3));
    let reports = det.detect();
    h.check_abs("clean_sequence_zero_fraud", reports.len() as f64, 0.0, 0.0);

    // Clean: CreateObject -> TransferObject by creator (valid)
    let mut det = GenericFraudDetector::new(vocab.clone());
    det.add_event(event(GenericOp::CreateObject, "alice", "item_g", 1));
    det.add_event(event(GenericOp::TransferObject, "alice", "item_g", 2));
    let reports = det.detect();
    let has_broken = reports
        .iter()
        .any(|r| r.fraud_type == GenericFraudType::BrokenChain);
    h.check_bool("valid_transfer_no_broken_chain", !has_broken);

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
    h.check_bool("valid_consume_in_scope", !has_scope);
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

fn validate_cross_domain_equivalence(h: &mut ValidationHarness) {
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

    h.check_bool(
        "gaming_detects_orphan",
        gaming_types.contains(&GenericFraudType::OrphanObject),
    );
    h.check_bool(
        "science_detects_orphan",
        science_types.contains(&GenericFraudType::OrphanObject),
    );
    h.check_bool(
        "medical_detects_orphan",
        medical_types.contains(&GenericFraudType::OrphanObject),
    );

    h.check_bool(
        "gaming_detects_duplicate",
        gaming_types.contains(&GenericFraudType::DuplicateIdentity),
    );
    h.check_bool(
        "science_detects_duplicate",
        science_types.contains(&GenericFraudType::DuplicateIdentity),
    );
    h.check_bool(
        "medical_detects_duplicate",
        medical_types.contains(&GenericFraudType::DuplicateIdentity),
    );

    h.check_bool(
        "gaming_detects_broken_chain",
        gaming_types.contains(&GenericFraudType::BrokenChain),
    );
    h.check_bool(
        "science_detects_broken_chain",
        science_types.contains(&GenericFraudType::BrokenChain),
    );
    h.check_bool(
        "medical_detects_broken_chain",
        medical_types.contains(&GenericFraudType::BrokenChain),
    );

    let sim_gaming_science = compute_structural_similarity(&gaming_reports, &science_reports);
    let sim_gaming_medical = compute_structural_similarity(&gaming_reports, &medical_reports);
    let sim_science_medical = compute_structural_similarity(&science_reports, &medical_reports);

    h.check_abs(
        "similarity_gaming_science_gt_80",
        sim_gaming_science,
        1.0,
        0.2,
    );
    h.check_abs(
        "similarity_gaming_medical_gt_80",
        sim_gaming_medical,
        1.0,
        0.2,
    );
    h.check_abs(
        "similarity_science_medical_gt_80",
        sim_science_medical,
        1.0,
        0.2,
    );
}

// =============================================================================
// 4. Unification table validation
// =============================================================================

fn validate_unification_table(h: &mut ValidationHarness) {
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

    h.check_bool(
        "orphan_maps_to_orphan_item_gaming",
        relabeled_gaming.domain_label == "OrphanItem",
    );
    h.check_bool(
        "orphan_maps_to_phantom_sample_science",
        relabeled_science.domain_label == "PhantomSample",
    );
    h.check_bool(
        "orphan_maps_to_phantom_access_medical",
        relabeled_medical.domain_label == "PhantomAccess",
    );

    let dup_report = GenericFraudReport {
        fraud_type: GenericFraudType::DuplicateIdentity,
        description: "test".to_string(),
        domain_label: String::new(),
    };
    h.check_bool(
        "duplicate_maps_to_duplicate_item_gaming",
        relabel_report(&dup_report, &gaming).domain_label == "DuplicateItem",
    );
    h.check_bool(
        "duplicate_maps_to_duplicate_sample_science",
        relabel_report(&dup_report, &science).domain_label == "DuplicateSample",
    );
    h.check_bool(
        "duplicate_maps_to_duplicate_record_medical",
        relabel_report(&dup_report, &medical).domain_label == "DuplicateRecord",
    );
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    let mut h = ValidationHarness::new("exp065_cross_domain_fraud");
    h.print_provenance(&[&PROVENANCE]);

    validate_vocabulary_mapping(&mut h);
    validate_generic_fraud_detection(&mut h);
    validate_cross_domain_equivalence(&mut h);
    validate_unification_table(&mut h);

    h.finish();
}
