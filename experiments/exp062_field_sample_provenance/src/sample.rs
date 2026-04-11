// SPDX-License-Identifier: AGPL-3.0-or-later
//! Field sample provenance — wetSpring's sample chain-of-custody using the provenance trio.
//!
//! A sample is a biological specimen tracked from collection through analysis and publication.
//! The loamSpine certificate is the sample identity. The rhizoCrypt DAG records every custody
//! and processing step. The sweetGrass braids provide semantic provenance.

use std::collections::{HashMap, HashSet};

use loam_spine_core::Did;
use loam_spine_core::certificate::{CertificateMetadata, CertificateType};
use loam_spine_core::entry::SpineConfig;
use loam_spine_core::manager::CertificateManager;
use loam_spine_core::spine::Spine;
use loam_spine_core::types::CertificateId;
use ludospring_barracuda::validation::OrExit;

use rhizo_crypt_core::session::SessionType;
use rhizo_crypt_core::vertex::MetadataValue;
use rhizo_crypt_core::{
    Did as RhizoDid, EventType, Session, SessionBuilder, Vertex, VertexBuilder, VertexId,
};

use sweet_grass_core::activity::{ActivityEcoPrimals, ActivityMetadata};
use sweet_grass_core::braid::{EcoPrimalsAttributes, current_timestamp_nanos};
use sweet_grass_core::{Activity, ActivityId, ActivityType, AgentAssociation, AgentRole, Braid};

// ============================================================================
// Domain Model
// ============================================================================

/// Type of biological sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(dead_code, reason = "domain model completeness")]
pub enum SampleType {
    Soil,
    Water,
    Swab,
    Tissue,
    Blood,
    Isolate,
}

impl SampleType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Soil => "soil",
            Self::Water => "water",
            Self::Swab => "swab",
            Self::Tissue => "tissue",
            Self::Blood => "blood",
            Self::Isolate => "isolate",
        }
    }
}

/// Condition of a sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(dead_code, reason = "domain model completeness")]
pub enum SampleCondition {
    Fresh,
    Refrigerated,
    Frozen,
    Degraded,
    Destroyed,
}

impl SampleCondition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Refrigerated => "refrigerated",
            Self::Frozen => "frozen",
            Self::Degraded => "degraded",
            Self::Destroyed => "destroyed",
        }
    }
}

/// Processing step in the sample pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(dead_code, reason = "domain model completeness")]
pub enum ProcessingStep {
    DnaExtraction,
    PcrAmplification,
    Sequencing,
    BioinformaticAnalysis,
    QualityControl,
}

impl ProcessingStep {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DnaExtraction => "dna_extraction",
            Self::PcrAmplification => "pcr_amplification",
            Self::Sequencing => "sequencing",
            Self::BioinformaticAnalysis => "bioinformatic_analysis",
            Self::QualityControl => "quality_control",
        }
    }
}

/// Custody transfer record.
#[derive(Debug, Clone)]
#[expect(dead_code, reason = "domain model completeness")]
pub struct CustodyTransfer {
    pub from_did: String,
    pub to_did: String,
    pub location: String,
    pub condition: SampleCondition,
    pub temperature_c: Option<f64>,
    pub tick: u64,
}

/// Sample event type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SampleEventType {
    Collect,
    Transport,
    Store,
    Extract,
    Amplify,
    Sequence,
    Analyze,
    Publish,
    CustodyTransfer,
    QualityControl,
}

impl SampleEventType {
    #[expect(
        dead_code,
        reason = "domain model completeness — used for serialization/display"
    )]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Collect => "collect",
            Self::Transport => "transport",
            Self::Store => "store",
            Self::Extract => "extract",
            Self::Amplify => "amplify",
            Self::Sequence => "sequence",
            Self::Analyze => "analyze",
            Self::Publish => "publish",
            Self::CustodyTransfer => "custody_transfer",
            Self::QualityControl => "quality_control",
        }
    }
}

/// Sample event.
#[derive(Debug, Clone)]
#[expect(
    dead_code,
    reason = "domain model completeness — tick used for timeline ordering"
)]
pub struct SampleEvent {
    pub event_type: SampleEventType,
    pub actor_did: String,
    pub description: String,
    pub tick: u64,
}

// ============================================================================
// Sample DAG
// ============================================================================

/// rhizoCrypt DAG for sample history.
pub struct SampleDag {
    pub session: Session,
    pub vertices: Vec<Vertex>,
    pub frontier: Vec<VertexId>,
}

impl SampleDag {
    fn new() -> Self {
        let session = SessionBuilder::new(SessionType::Gaming {
            game_id: "field_sample".into(),
        })
        .with_name("Field Sample History")
        .build();

        Self {
            session,
            vertices: Vec::new(),
            frontier: Vec::new(),
        }
    }

    fn append(
        &mut self,
        event_type: &str,
        agent: &RhizoDid,
        metadata: HashMap<String, MetadataValue>,
    ) -> VertexId {
        let mut builder = VertexBuilder::new(EventType::AgentAction {
            action: event_type.into(),
        })
        .with_agent(agent.clone());

        for &parent in &self.frontier {
            builder = builder.with_parent(parent);
        }
        for (key, value) in metadata {
            builder = builder.with_metadata(key, value);
        }

        let vertex = builder.build();
        let vertex_id = vertex.compute_id().or_exit("vertex id computation");
        self.session.update_frontier(vertex_id, &self.frontier);
        self.frontier = vec![vertex_id];
        self.vertices.push(vertex);
        vertex_id
    }
}

// ============================================================================
// Sample System
// ============================================================================

/// Sample system: certificate manager + DAG + braids + events.
pub struct SampleSystem {
    pub cert_manager: CertificateManager,
    pub dag: SampleDag,
    pub braids: Vec<Braid>,
    pub events: Vec<(CertificateId, SampleEvent)>,
    pub tick: u64,
    /// Current holder DID per `cert` (from `collect` + `custody_transfer` chain).
    current_holder: HashMap<CertificateId, String>,
    /// Last recorded condition per `cert` (for cold chain).
    last_condition: HashMap<CertificateId, SampleCondition>,
    /// Custody chain: (`cert_id`, `from`, `to`) for each transfer.
    custody_transfers: Vec<(CertificateId, String, String)>,
    /// Condition history for cold chain: (`cert_id`, `tick`, `condition`).
    condition_history: Vec<(CertificateId, u64, SampleCondition)>,
    /// Structural record of sample type per cert (set at collection time).
    collect_sample_types: HashMap<CertificateId, SampleType>,
}

impl SampleSystem {
    /// Create a new sample system.
    pub fn new(owner: &Did) -> Self {
        let spine = Spine::new(
            owner.clone(),
            Some("FieldSample".into()),
            SpineConfig::default(),
        )
        .or_exit("spine creation");
        let cert_manager = CertificateManager::new(spine);

        Self {
            cert_manager,
            dag: SampleDag::new(),
            braids: Vec::new(),
            events: Vec::new(),
            tick: 0,
            current_holder: HashMap::new(),
            last_condition: HashMap::new(),
            custody_transfers: Vec::new(),
            condition_history: Vec::new(),
            collect_sample_types: HashMap::new(),
        }
    }

    /// Advance the system `tick`.
    #[expect(
        clippy::missing_const_for_fn,
        reason = "mutates self — cannot be const"
    )]
    pub fn advance_tick(&mut self) {
        self.tick += 1;
    }

    /// Collect a new sample (creates `cert`, DAG vertex, braid, event).
    pub fn collect_sample(
        &mut self,
        collector: &Did,
        sample_type: SampleType,
        location: &str,
        accession: &str,
    ) -> CertificateId {
        let metadata = CertificateMetadata::new()
            .with_name(format!("Sample {accession}"))
            .with_description(format!("{sample_type:?} from {location}"))
            .with_attribute("sample_type", sample_type.as_str())
            .with_attribute("location", location)
            .with_attribute("accession", accession)
            .with_attribute("condition", SampleCondition::Fresh.as_str());

        let cert_type = CertificateType::Custom {
            type_uri: "ecoPrimals:sample".into(),
            schema_version: 1,
        };

        let (cert, _entry_hash) = self
            .cert_manager
            .mint(cert_type, collector, metadata)
            .or_exit("certificate minting");

        let cert_id = cert.id;

        let rhizo_did = RhizoDid::new(collector.as_str());
        self.dag.session.add_agent(rhizo_did.clone());

        let mut meta = HashMap::new();
        meta.insert("event".into(), MetadataValue::String("collect".into()));
        meta.insert(
            "sample_type".into(),
            MetadataValue::String(sample_type.as_str().into()),
        );
        meta.insert("location".into(), MetadataValue::String(location.into()));
        meta.insert("accession".into(), MetadataValue::String(accession.into()));
        meta.insert("cert_id".into(), MetadataValue::String(cert_id.to_string()));
        let vertex_id = self.dag.append("sample_collect", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(collector.as_str());
        if let Ok(braid) = create_sample_braid(
            &sweet_did,
            "collect",
            &format!("Collected {sample_type:?} at {location}"),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        self.collect_sample_types.insert(cert_id, sample_type);
        self.current_holder
            .insert(cert_id, collector.as_str().into());
        self.last_condition.insert(cert_id, SampleCondition::Fresh);
        self.condition_history
            .push((cert_id, self.tick, SampleCondition::Fresh));

        self.events.push((
            cert_id,
            SampleEvent {
                event_type: SampleEventType::Collect,
                actor_did: collector.as_str().into(),
                description: format!("Collected {sample_type:?} at {location}"),
                tick: self.tick,
            },
        ));

        cert_id
    }

    /// Transport sample from one location to another.
    pub fn transport(
        &mut self,
        cert_id: CertificateId,
        from: &Did,
        to: &Did,
        condition: SampleCondition,
        temp: Option<f64>,
    ) {
        let rhizo_did = RhizoDid::new(from.as_str());
        self.dag.session.add_agent(RhizoDid::new(to.as_str()));

        let mut meta = HashMap::new();
        meta.insert("event".into(), MetadataValue::String("transport".into()));
        meta.insert("from".into(), MetadataValue::String(from.as_str().into()));
        meta.insert("to".into(), MetadataValue::String(to.as_str().into()));
        meta.insert(
            "condition".into(),
            MetadataValue::String(condition.as_str().into()),
        );
        if let Some(t) = temp {
            meta.insert("temperature_c".into(), MetadataValue::Float(t));
        }
        meta.insert("cert_id".into(), MetadataValue::String(cert_id.to_string()));
        let vertex_id = self.dag.append("sample_transport", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(from.as_str());
        if let Ok(braid) = create_sample_braid(
            &sweet_did,
            "transport",
            &format!("Transported to {}", to.as_str()),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        self.current_holder.insert(cert_id, to.as_str().into());
        self.last_condition.insert(cert_id, condition);
        self.condition_history.push((cert_id, self.tick, condition));

        self.events.push((
            cert_id,
            SampleEvent {
                event_type: SampleEventType::Transport,
                actor_did: from.as_str().into(),
                description: format!("Transported to {}", to.as_str()),
                tick: self.tick,
            },
        ));
    }

    /// Store sample (`condition` + `temp`).
    pub fn store(
        &mut self,
        cert_id: CertificateId,
        actor: &Did,
        condition: SampleCondition,
        temp: Option<f64>,
    ) {
        let rhizo_did = RhizoDid::new(actor.as_str());
        let mut meta = HashMap::new();
        meta.insert("event".into(), MetadataValue::String("store".into()));
        meta.insert(
            "condition".into(),
            MetadataValue::String(condition.as_str().into()),
        );
        if let Some(t) = temp {
            meta.insert("temperature_c".into(), MetadataValue::Float(t));
        }
        meta.insert("cert_id".into(), MetadataValue::String(cert_id.to_string()));
        let vertex_id = self.dag.append("sample_store", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(actor.as_str());
        if let Ok(braid) = create_sample_braid(
            &sweet_did,
            "store",
            &format!("Stored at {}", condition.as_str()),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        self.last_condition.insert(cert_id, condition);
        self.condition_history.push((cert_id, self.tick, condition));

        self.events.push((
            cert_id,
            SampleEvent {
                event_type: SampleEventType::Store,
                actor_did: actor.as_str().into(),
                description: format!("Stored at {}", condition.as_str()),
                tick: self.tick,
            },
        ));
    }

    /// Process sample (extraction, amplification, sequencing, etc.).
    pub fn process(&mut self, cert_id: CertificateId, actor: &Did, step: ProcessingStep) {
        let rhizo_did = RhizoDid::new(actor.as_str());
        let mut meta = HashMap::new();
        meta.insert("event".into(), MetadataValue::String(step.as_str().into()));
        meta.insert("cert_id".into(), MetadataValue::String(cert_id.to_string()));
        let vertex_id = self
            .dag
            .append(&format!("sample_{}", step.as_str()), &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(actor.as_str());
        if let Ok(braid) = create_sample_braid(
            &sweet_did,
            step.as_str(),
            &format!("Processed: {}", step.as_str()),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        let event_type = match step {
            ProcessingStep::DnaExtraction => SampleEventType::Extract,
            ProcessingStep::PcrAmplification => SampleEventType::Amplify,
            ProcessingStep::Sequencing => SampleEventType::Sequence,
            ProcessingStep::BioinformaticAnalysis => SampleEventType::Analyze,
            ProcessingStep::QualityControl => SampleEventType::QualityControl,
        };

        self.events.push((
            cert_id,
            SampleEvent {
                event_type,
                actor_did: actor.as_str().into(),
                description: format!("Processed: {}", step.as_str()),
                tick: self.tick,
            },
        ));
    }

    /// Publish sample results (`DOI`).
    pub fn publish(&mut self, cert_id: CertificateId, actor: &Did, doi: &str) {
        let rhizo_did = RhizoDid::new(actor.as_str());
        let mut meta = HashMap::new();
        meta.insert("event".into(), MetadataValue::String("publish".into()));
        meta.insert("doi".into(), MetadataValue::String(doi.into()));
        meta.insert("cert_id".into(), MetadataValue::String(cert_id.to_string()));
        let vertex_id = self.dag.append("sample_publish", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(actor.as_str());
        if let Ok(braid) = create_sample_braid(
            &sweet_did,
            "publish",
            &format!("Published: {doi}"),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        self.events.push((
            cert_id,
            SampleEvent {
                event_type: SampleEventType::Publish,
                actor_did: actor.as_str().into(),
                description: format!("Published: {doi}"),
                tick: self.tick,
            },
        ));
    }

    /// Custody transfer (`from` -> `to`).
    pub fn custody_transfer(
        &mut self,
        cert_id: CertificateId,
        from: &Did,
        to: &Did,
        location: &str,
    ) {
        let rhizo_did = RhizoDid::new(from.as_str());
        self.dag.session.add_agent(RhizoDid::new(to.as_str()));

        let condition = self
            .last_condition
            .get(&cert_id)
            .copied()
            .unwrap_or(SampleCondition::Fresh);

        let mut meta = HashMap::new();
        meta.insert(
            "event".into(),
            MetadataValue::String("custody_transfer".into()),
        );
        meta.insert("from".into(), MetadataValue::String(from.as_str().into()));
        meta.insert("to".into(), MetadataValue::String(to.as_str().into()));
        meta.insert("location".into(), MetadataValue::String(location.into()));
        meta.insert(
            "condition".into(),
            MetadataValue::String(condition.as_str().into()),
        );
        meta.insert("cert_id".into(), MetadataValue::String(cert_id.to_string()));
        let vertex_id = self.dag.append("sample_custody", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(from.as_str());
        if let Ok(braid) = create_sample_braid(
            &sweet_did,
            "custody_transfer",
            &format!("Custody to {}", to.as_str()),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        self.current_holder.insert(cert_id, to.as_str().into());
        self.custody_transfers
            .push((cert_id, from.as_str().into(), to.as_str().into()));

        self.events.push((
            cert_id,
            SampleEvent {
                event_type: SampleEventType::CustodyTransfer,
                actor_did: from.as_str().into(),
                description: format!("Custody to {}", to.as_str()),
                tick: self.tick,
            },
        ));
    }

    /// Get sample timeline for a `cert`.
    pub fn sample_timeline(&self, cert_id: CertificateId) -> Vec<&SampleEvent> {
        self.events
            .iter()
            .filter(|(id, _)| *id == cert_id)
            .map(|(_, event)| event)
            .collect()
    }

    /// Get cert IDs held by a `DID` (current holder).
    pub fn samples_held_by(&self, did: &str) -> Vec<CertificateId> {
        self.current_holder
            .iter()
            .filter(|(_, holder)| *holder == did)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get cert attributes (`sample_type`, `location`, `accession`, `condition`).
    pub fn cert_attributes(&self, cert_id: CertificateId) -> Option<&HashMap<String, String>> {
        self.cert_manager
            .get_certificate(&cert_id)
            .map(|c| &c.metadata.attributes)
    }

    /// Get authorized actors for a `cert` (collector + anyone who received custody).
    fn authorized_actors_for_cert(&self, cert_id: CertificateId) -> HashSet<String> {
        let mut authorized = HashSet::new();
        for (cid, event) in &self.events {
            if *cid != cert_id {
                continue;
            }
            if event.event_type == SampleEventType::Collect {
                authorized.insert(event.actor_did.clone());
            }
        }
        for (cid, _from, to) in &self.custody_transfers {
            if *cid == cert_id {
                authorized.insert(to.clone());
            }
        }
        authorized
    }

    /// Inject a collect event for testing (e.g. mislabeled specimen fraud scenario).
    pub fn inject_collect_event_for_test(
        &mut self,
        cert_id: CertificateId,
        actor_did: &str,
        description: &str,
        tick: u64,
        actual_sample_type: SampleType,
    ) {
        self.events.push((
            cert_id,
            SampleEvent {
                event_type: SampleEventType::Collect,
                actor_did: actor_did.into(),
                description: description.into(),
                tick,
            },
        ));
        self.current_holder.insert(cert_id, actor_did.into());
        self.last_condition.insert(cert_id, SampleCondition::Fresh);
        self.condition_history
            .push((cert_id, tick, SampleCondition::Fresh));
        self.collect_sample_types
            .insert(cert_id, actual_sample_type);
    }

    /// Get the sample type recorded at collection time.
    pub fn collected_sample_type(&self, cert_id: CertificateId) -> Option<SampleType> {
        self.collect_sample_types.get(&cert_id).copied()
    }

    /// Get condition history for cold chain (ordered by `tick`).
    fn condition_history_for_cert(&self, cert_id: CertificateId) -> Vec<(u64, SampleCondition)> {
        self.condition_history
            .iter()
            .filter(|(cid, _, _)| *cid == cert_id)
            .map(|(_, tick, cond)| (*tick, *cond))
            .collect()
    }
}

// ============================================================================
// Fraud Detection
// ============================================================================

/// Fraud report.
#[derive(Debug, Clone)]
#[expect(
    dead_code,
    reason = "domain model completeness — fields used for reporting"
)]
pub struct FraudReport {
    pub fraud_type: SampleFraudType,
    pub description: String,
    pub cert_id: Option<CertificateId>,
}

/// Types of sample fraud.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SampleFraudType {
    PhantomSample,
    ContaminationGap,
    MislabeledSpecimen,
    BrokenColdChain,
    UnauthorizedAccess,
    DuplicateAccession,
}

/// Detect sample fraud by analyzing the DAG and certificates.
pub fn detect_sample_fraud(system: &SampleSystem) -> Vec<FraudReport> {
    let mut reports = Vec::new();
    let certs_with_collect = certs_with_collect_events(system);

    detect_phantom_samples(system, &certs_with_collect, &mut reports);
    detect_duplicate_accessions(system, &certs_with_collect, &mut reports);
    detect_broken_cold_chain(system, &certs_with_collect, &mut reports);
    detect_unauthorized_access(system, &mut reports);
    detect_mislabeled_specimens(system, &certs_with_collect, &mut reports);
    detect_contamination_gaps(system, &mut reports);

    reports
}

fn certs_with_collect_events(system: &SampleSystem) -> HashSet<CertificateId> {
    system
        .events
        .iter()
        .filter(|(_, e)| e.event_type == SampleEventType::Collect)
        .map(|(id, _)| *id)
        .collect()
}

fn detect_phantom_samples(
    system: &SampleSystem,
    certs_with_collect: &HashSet<CertificateId>,
    reports: &mut Vec<FraudReport>,
) {
    for cert in system.cert_manager.list_certificates() {
        if !certs_with_collect.contains(&cert.id) {
            reports.push(FraudReport {
                fraud_type: SampleFraudType::PhantomSample,
                description: "Certificate exists but no collect event".into(),
                cert_id: Some(cert.id),
            });
        }
    }
}

fn detect_duplicate_accessions(
    system: &SampleSystem,
    certs_with_collect: &HashSet<CertificateId>,
    reports: &mut Vec<FraudReport>,
) {
    let mut accession_to_certs: HashMap<String, Vec<CertificateId>> = HashMap::new();
    for &cert_id in certs_with_collect {
        if let Some(attrs) = system.cert_attributes(cert_id) {
            if let Some(acc) = attrs.get("accession") {
                accession_to_certs
                    .entry(acc.clone())
                    .or_default()
                    .push(cert_id);
            }
        }
    }
    for certs in accession_to_certs.values() {
        if certs.len() > 1 {
            for &cert_id in certs {
                reports.push(FraudReport {
                    fraud_type: SampleFraudType::DuplicateAccession,
                    description: format!("Duplicate accession across {} certs", certs.len()),
                    cert_id: Some(cert_id),
                });
            }
        }
    }
}

fn detect_broken_cold_chain(
    system: &SampleSystem,
    certs_with_collect: &HashSet<CertificateId>,
    reports: &mut Vec<FraudReport>,
) {
    for &cert_id in certs_with_collect {
        let history = system.condition_history_for_cert(cert_id);
        let mut prev = None;
        for (_tick, cond) in history {
            if let Some(p) = prev {
                if p == SampleCondition::Frozen && cond == SampleCondition::Fresh {
                    reports.push(FraudReport {
                        fraud_type: SampleFraudType::BrokenColdChain,
                        description: "Sample went from Frozen to Fresh without documented thaw"
                            .into(),
                        cert_id: Some(cert_id),
                    });
                    break;
                }
            }
            prev = Some(cond);
        }
    }
}

fn detect_unauthorized_access(system: &SampleSystem, reports: &mut Vec<FraudReport>) {
    for (cert_id, event) in &system.events {
        let is_processing = matches!(
            event.event_type,
            SampleEventType::Extract
                | SampleEventType::Amplify
                | SampleEventType::Sequence
                | SampleEventType::Analyze
                | SampleEventType::QualityControl
        );
        if is_processing {
            let authorized = system.authorized_actors_for_cert(*cert_id);
            if !authorized.contains(&event.actor_did) {
                reports.push(FraudReport {
                    fraud_type: SampleFraudType::UnauthorizedAccess,
                    description: format!(
                        "Actor {} processed sample without custody chain",
                        event.actor_did
                    ),
                    cert_id: Some(*cert_id),
                });
            }
        }
    }
}

/// Compares structurally recorded sample type against cert attribute.
fn detect_mislabeled_specimens(
    system: &SampleSystem,
    certs_with_collect: &HashSet<CertificateId>,
    reports: &mut Vec<FraudReport>,
) {
    for &cert_id in certs_with_collect {
        let cert_type_str = system
            .cert_attributes(cert_id)
            .and_then(|a| a.get("sample_type").map(String::as_str));
        let collected_type_str = system
            .collected_sample_type(cert_id)
            .map(SampleType::as_str);

        if let (Some(cert_st), Some(coll_st)) = (cert_type_str, collected_type_str) {
            if cert_st != coll_st {
                reports.push(FraudReport {
                    fraud_type: SampleFraudType::MislabeledSpecimen,
                    description: format!("Cert says {cert_st} but collection recorded {coll_st}"),
                    cert_id: Some(cert_id),
                });
            }
        }
    }
}

fn detect_contamination_gaps(system: &SampleSystem, reports: &mut Vec<FraudReport>) {
    let mut actor_sample_sequence: HashMap<String, Vec<(CertificateId, bool)>> = HashMap::new();
    for (cert_id, event) in &system.events {
        let is_processing = matches!(
            event.event_type,
            SampleEventType::Extract
                | SampleEventType::Amplify
                | SampleEventType::Sequence
                | SampleEventType::Analyze
        );
        let is_qc = event.event_type == SampleEventType::QualityControl;
        if is_processing || is_qc {
            actor_sample_sequence
                .entry(event.actor_did.clone())
                .or_default()
                .push((*cert_id, is_qc));
        }
    }
    for seq in actor_sample_sequence.values() {
        let mut last_sample = None;
        let mut had_qc_since = true;
        for &(cert_id, is_qc) in seq {
            if is_qc {
                had_qc_since = true;
            } else {
                if let Some(prev) = last_sample {
                    if prev != cert_id && !had_qc_since {
                        reports.push(FraudReport {
                            fraud_type: SampleFraudType::ContaminationGap,
                            description:
                                "Same actor processed different samples without QC between".into(),
                            cert_id: Some(cert_id),
                        });
                    }
                }
                last_sample = Some(cert_id);
                had_qc_since = false;
            }
        }
    }
}

// ============================================================================
// sweetGrass Braid Helper
// ============================================================================

/// Create a PROV-O attribution braid for a sample event.
fn create_sample_braid(
    actor: &sweet_grass_core::Did,
    event_type: &str,
    description: &str,
    rhizo_vertex_hex: &str,
) -> Result<Braid, sweet_grass_core::SweetGrassError> {
    let activity = Activity {
        id: ActivityId::from_task(event_type),
        activity_type: ActivityType::Creation,
        used: Vec::new(),
        was_associated_with: vec![AgentAssociation {
            agent: actor.clone(),
            role: AgentRole::Creator,
            on_behalf_of: None,
            had_plan: Some("wetspring".into()),
        }],
        started_at_time: current_timestamp_nanos(),
        ended_at_time: Some(current_timestamp_nanos()),
        metadata: ActivityMetadata::default(),
        ecop: ActivityEcoPrimals::default(),
    };

    let data_hash = format!("sha256:{rhizo_vertex_hex}");
    let ecop = EcoPrimalsAttributes {
        source_primal: Some("wetspring".into()),
        ..Default::default()
    };

    let mut braid = Braid::builder()
        .data_hash(&data_hash)
        .mime_type("application/x-sample-event")
        .size(description.len() as u64)
        .attributed_to(actor.clone())
        .generated_by(activity)
        .ecop(ecop)
        .build()?;

    braid.metadata.description = Some(description.into());
    Ok(braid)
}
