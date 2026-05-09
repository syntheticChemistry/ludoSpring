// SPDX-License-Identifier: AGPL-3.0-or-later
//! Domain-agnostic fraud detection — same graph analysis catches fraud
//! in gaming, science, and medical domains.

use std::collections::{HashMap, HashSet};

// =============================================================================
// Domain vocabulary and generic operations
// =============================================================================

/// Generic operation types — domain-agnostic (`GenericOp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenericOp {
    CreateObject,
    TransferObject,
    TransformObject,
    ConsumeObject,
    AuditObject,
    GrantAccess,
    RevokeAccess,
}

/// Maps `GenericOp` to domain-specific names.
#[derive(Debug, Clone)]
pub struct DomainVocabulary {
    pub domain_name: String,
    pub op_labels: HashMap<GenericOp, String>,
    pub fraud_labels: HashMap<GenericFraudType, String>,
}

/// Generic event in the provenance DAG (`GenericEvent`).
#[derive(Debug, Clone)]
pub struct GenericEvent {
    pub op: GenericOp,
    pub actor: String,
    pub target: String,
    pub tick: u64,
    pub metadata: HashMap<String, String>,
}

/// Generic fraud types — structural patterns that indicate fraud (`GenericFraudType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenericFraudType {
    OrphanObject,
    DuplicateIdentity,
    UnauthorizedAction,
    ScopeViolation,
    BrokenChain,
}

/// Fraud report with generic type and domain-specific labeling (`GenericFraudReport`).
#[derive(Debug, Clone)]
pub struct GenericFraudReport {
    pub fraud_type: GenericFraudType,
    pub description: String,
    pub domain_label: String,
}

// =============================================================================
// Domain vocabularies
// =============================================================================

/// Gaming domain vocabulary.
#[must_use]
pub fn gaming_vocabulary() -> DomainVocabulary {
    let mut op_labels = HashMap::new();
    op_labels.insert(GenericOp::CreateObject, "ItemSpawn".to_string());
    op_labels.insert(GenericOp::TransferObject, "ItemTrade".to_string());
    op_labels.insert(GenericOp::TransformObject, "ItemModify".to_string());
    op_labels.insert(GenericOp::ConsumeObject, "ItemConsume/Fire".to_string());
    op_labels.insert(GenericOp::AuditObject, "ItemInspect".to_string());
    op_labels.insert(GenericOp::GrantAccess, "LootPickup".to_string());
    op_labels.insert(GenericOp::RevokeAccess, "ItemDrop".to_string());

    let mut fraud_labels = HashMap::new();
    fraud_labels.insert(GenericFraudType::OrphanObject, "OrphanItem".to_string());
    fraud_labels.insert(
        GenericFraudType::DuplicateIdentity,
        "DuplicateItem".to_string(),
    );
    fraud_labels.insert(
        GenericFraudType::UnauthorizedAction,
        "UnauthorizedModify".to_string(),
    );
    fraud_labels.insert(
        GenericFraudType::ScopeViolation,
        "ScopeViolation".to_string(),
    );
    fraud_labels.insert(
        GenericFraudType::BrokenChain,
        "BrokenTradeChain".to_string(),
    );

    DomainVocabulary {
        domain_name: "gaming".to_string(),
        op_labels,
        fraud_labels,
    }
}

/// Science domain vocabulary.
#[must_use]
pub fn science_vocabulary() -> DomainVocabulary {
    let mut op_labels = HashMap::new();
    op_labels.insert(GenericOp::CreateObject, "SampleCollect".to_string());
    op_labels.insert(GenericOp::TransferObject, "CustodyTransfer".to_string());
    op_labels.insert(GenericOp::TransformObject, "ProcessingStep".to_string());
    op_labels.insert(GenericOp::ConsumeObject, "SampleSequenced".to_string());
    op_labels.insert(GenericOp::AuditObject, "SampleInspect".to_string());
    op_labels.insert(GenericOp::GrantAccess, "LabAccess".to_string());
    op_labels.insert(GenericOp::RevokeAccess, "SampleDisposal".to_string());

    let mut fraud_labels = HashMap::new();
    fraud_labels.insert(GenericFraudType::OrphanObject, "PhantomSample".to_string());
    fraud_labels.insert(
        GenericFraudType::DuplicateIdentity,
        "DuplicateSample".to_string(),
    );
    fraud_labels.insert(
        GenericFraudType::UnauthorizedAction,
        "UnauthorizedProcessing".to_string(),
    );
    fraud_labels.insert(
        GenericFraudType::ScopeViolation,
        "ScopeViolation".to_string(),
    );
    fraud_labels.insert(
        GenericFraudType::BrokenChain,
        "BrokenCustodyChain".to_string(),
    );

    DomainVocabulary {
        domain_name: "science".to_string(),
        op_labels,
        fraud_labels,
    }
}

/// Medical domain vocabulary.
#[must_use]
pub fn medical_vocabulary() -> DomainVocabulary {
    let mut op_labels = HashMap::new();
    op_labels.insert(GenericOp::CreateObject, "RecordCreation".to_string());
    op_labels.insert(GenericOp::TransferObject, "RecordReferral".to_string());
    op_labels.insert(GenericOp::TransformObject, "RecordUpdate".to_string());
    op_labels.insert(GenericOp::ConsumeObject, "RecordArchive".to_string());
    op_labels.insert(GenericOp::AuditObject, "RecordAudit".to_string());
    op_labels.insert(GenericOp::GrantAccess, "ConsentGrant".to_string());
    op_labels.insert(GenericOp::RevokeAccess, "ConsentRevoke".to_string());

    let mut fraud_labels = HashMap::new();
    fraud_labels.insert(GenericFraudType::OrphanObject, "PhantomAccess".to_string());
    fraud_labels.insert(
        GenericFraudType::DuplicateIdentity,
        "DuplicateRecord".to_string(),
    );
    fraud_labels.insert(
        GenericFraudType::UnauthorizedAction,
        "UnauthorizedUpdate".to_string(),
    );
    fraud_labels.insert(
        GenericFraudType::ScopeViolation,
        "ScopeViolation".to_string(),
    );
    fraud_labels.insert(
        GenericFraudType::BrokenChain,
        "BrokenReferralChain".to_string(),
    );

    DomainVocabulary {
        domain_name: "medical".to_string(),
        op_labels,
        fraud_labels,
    }
}

// =============================================================================
// Generic fraud detector
// =============================================================================

/// Domain-agnostic fraud detector (`GenericFraudDetector`).
pub struct GenericFraudDetector {
    pub events: Vec<GenericEvent>,
    pub vocabulary: DomainVocabulary,
}

impl GenericFraudDetector {
    #[must_use]
    #[expect(clippy::missing_const_for_fn, reason = "mutates self")]
    pub fn new(vocabulary: DomainVocabulary) -> Self {
        Self {
            events: Vec::new(),
            vocabulary,
        }
    }

    pub fn add_event(&mut self, event: GenericEvent) {
        self.events.push(event);
    }

    /// Run all five generic fraud checks.
    #[must_use]
    #[expect(
        clippy::too_many_lines,
        reason = "fraud detection requires sequential rule checks"
    )]
    pub fn detect(&self) -> Vec<GenericFraudReport> {
        let mut reports = Vec::new();

        // OrphanObject: TransformObject/ConsumeObject with no prior CreateObject for target
        for ev in &self.events {
            if matches!(ev.op, GenericOp::TransformObject | GenericOp::ConsumeObject) {
                let has_create = self.events.iter().any(|e| {
                    e.tick < ev.tick && e.op == GenericOp::CreateObject && e.target == ev.target
                });
                if !has_create {
                    reports.push(GenericFraudReport {
                        fraud_type: GenericFraudType::OrphanObject,
                        description: format!(
                            "{} on {} with no prior CreateObject",
                            op_name(ev.op),
                            ev.target
                        ),
                        domain_label: String::new(),
                    });
                }
            }
        }

        // DuplicateIdentity: two CreateObject events with same target
        let create_targets: Vec<&str> = self
            .events
            .iter()
            .filter(|e| e.op == GenericOp::CreateObject)
            .map(|e| e.target.as_str())
            .collect();
        let mut seen = HashSet::new();
        let mut reported = HashSet::new();
        for t in &create_targets {
            if !seen.insert(*t) && reported.insert(*t) {
                reports.push(GenericFraudReport {
                    fraud_type: GenericFraudType::DuplicateIdentity,
                    description: format!("Duplicate CreateObject for target {t}"),
                    domain_label: String::new(),
                });
            }
        }

        // UnauthorizedAction: TransformObject by actor who never had GrantAccess/TransferObject
        // for that target. Actor must be creator, transferee, or have GrantAccess.
        for ev in &self.events {
            if ev.op == GenericOp::TransformObject {
                let was_creator = self.events.iter().any(|e| {
                    e.tick < ev.tick
                        && e.op == GenericOp::CreateObject
                        && e.target == ev.target
                        && e.actor == ev.actor
                });
                let had_grant = self.events.iter().any(|e| {
                    e.tick < ev.tick
                        && e.op == GenericOp::GrantAccess
                        && e.target == ev.target
                        && e.actor == ev.actor
                });
                let was_transferee = self.events.iter().any(|e| {
                    e.tick < ev.tick
                        && e.op == GenericOp::TransferObject
                        && e.target == ev.target
                        && e.actor == ev.actor
                });
                if !was_creator && !had_grant && !was_transferee {
                    reports.push(GenericFraudReport {
                        fraud_type: GenericFraudType::UnauthorizedAction,
                        description: format!(
                            "TransformObject by {} on {} without prior access",
                            ev.actor, ev.target
                        ),
                        domain_label: String::new(),
                    });
                }
            }
        }

        // ScopeViolation: ConsumeObject when actor's metadata "scope" doesn't include target
        for ev in &self.events {
            if ev.op == GenericOp::ConsumeObject {
                if let Some(scope) = ev.metadata.get("scope") {
                    if !scope.split(',').any(|s| s.trim() == ev.target) {
                        reports.push(GenericFraudReport {
                            fraud_type: GenericFraudType::ScopeViolation,
                            description: format!(
                                "ConsumeObject on {} outside actor scope",
                                ev.target
                            ),
                            domain_label: String::new(),
                        });
                    }
                }
            }
        }

        // BrokenChain: TransferObject where actor (sender) is not current holder.
        // Convention: TransferObject actor is the sender (current holder giving away).
        for ev in &self.events {
            if ev.op == GenericOp::TransferObject {
                let is_holder = self.events.iter().any(|e| {
                    e.tick < ev.tick
                        && e.target == ev.target
                        && matches!(e.op, GenericOp::CreateObject | GenericOp::TransferObject)
                        && e.actor == ev.actor
                });
                if !is_holder {
                    reports.push(GenericFraudReport {
                        fraud_type: GenericFraudType::BrokenChain,
                        description: format!(
                            "TransferObject by {} who does not hold {}",
                            ev.actor, ev.target
                        ),
                        domain_label: String::new(),
                    });
                }
            }
        }

        reports
    }

    /// Relabel a report using the detector's vocabulary (`GenericFraudReport`).
    #[must_use]
    pub fn relabel_report(&self, report: &GenericFraudReport) -> GenericFraudReport {
        relabel_report(report, &self.vocabulary)
    }
}

/// Apply `DomainVocabulary` to a report's description and domain label.
#[must_use]
pub fn relabel_report(
    report: &GenericFraudReport,
    vocabulary: &DomainVocabulary,
) -> GenericFraudReport {
    let domain_label = vocabulary
        .fraud_labels
        .get(&report.fraud_type)
        .map_or_else(|| format!("{:?}", report.fraud_type), Clone::clone);

    GenericFraudReport {
        fraud_type: report.fraud_type,
        description: report.description.clone(),
        domain_label,
    }
}

const fn op_name(op: GenericOp) -> &'static str {
    match op {
        GenericOp::CreateObject => "CreateObject",
        GenericOp::TransferObject => "TransferObject",
        GenericOp::TransformObject => "TransformObject",
        GenericOp::ConsumeObject => "ConsumeObject",
        GenericOp::AuditObject => "AuditObject",
        GenericOp::GrantAccess => "GrantAccess",
        GenericOp::RevokeAccess => "RevokeAccess",
    }
}

// =============================================================================
// Cross-domain structural similarity
// =============================================================================

/// Compute Jaccard similarity of detected fraud types between two domains (`GenericFraudReport`).
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
pub fn compute_structural_similarity(
    reports_a: &[GenericFraudReport],
    reports_b: &[GenericFraudReport],
) -> f64 {
    let types_a: HashSet<GenericFraudType> = reports_a.iter().map(|r| r.fraud_type).collect();
    let types_b: HashSet<GenericFraudType> = reports_b.iter().map(|r| r.fraud_type).collect();

    if types_a.is_empty() && types_b.is_empty() {
        return 1.0;
    }
    if types_a.is_empty() || types_b.is_empty() {
        return 0.0;
    }

    let intersection = types_a.intersection(&types_b).count();
    let union = types_a.union(&types_b).count();
    intersection as f64 / union as f64
}
