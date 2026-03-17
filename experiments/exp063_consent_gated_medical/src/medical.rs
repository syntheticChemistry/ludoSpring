// SPDX-License-Identifier: AGPL-3.0-or-later
//! Consent-gated medical data access — `healthSpring` zero-knowledge provenance.
//!
//! Scaffolds `healthSpring`'s zero-knowledge medical data access using the
//! provenance trio: `loamSpine` certificates, `rhizoCrypt` DAG, `sweetGrass` braids.
//!
//! Medical records are consent-gated: providers can only access patient data
//! when the patient has granted explicit, scoped consent. All access is
//! logged to the DAG and produces verifiable access proofs.

use std::collections::HashMap;

use loam_spine_core::Did;
use loam_spine_core::certificate::{CertificateMetadata, CertificateType};
use loam_spine_core::entry::SpineConfig;
use loam_spine_core::manager::CertificateManager;
use loam_spine_core::spine::Spine;
use loam_spine_core::types::CertificateId;

// ============================================================================
// Domain Model
// ============================================================================

/// Medical record type taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordType {
    Lab,
    Imaging,
    Prescription,
    Vitals,
    Encounter,
    Genomic,
}

impl RecordType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lab => "lab",
            Self::Imaging => "imaging",
            Self::Prescription => "prescription",
            Self::Vitals => "vitals",
            Self::Encounter => "encounter",
            Self::Genomic => "genomic",
        }
    }
}

/// Scope of consent: which record types and until when.
#[derive(Debug, Clone)]
pub struct ConsentScope {
    pub record_types: Vec<RecordType>,
    pub expiry_tick: u64,
}

/// A single access event (provider accessed a record).
#[derive(Debug, Clone)]
pub struct AccessEvent {
    pub accessor_did: String,
    pub record_id: CertificateId,
    pub purpose: String,
    pub record_type: RecordType,
    pub tick: u64,
    /// Set when created via `access_record`; `None` when injected for fraud testing.
    pub vertex_id: Option<rhizo_crypt_core::VertexId>,
}

/// Verifiable proof of access (deterministic signature model).
#[derive(Debug, Clone)]
pub struct AccessProof {
    pub accessor_did: String,
    pub record_id: CertificateId,
    pub timestamp_tick: u64,
    pub proof_signature: Vec<u8>,
}

/// Types of medical data fraud.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MedicalFraudType {
    UnauthorizedAccess,
    ExpiredConsent,
    ScopeViolation,
    PhantomAccess,
    ConsentForgery,
}

/// Fraud report from detection.
#[derive(Debug, Clone)]
pub struct MedicalFraudReport {
    pub fraud_type: MedicalFraudType,
    #[expect(
        dead_code,
        reason = "domain model completeness — used for audit display"
    )]
    pub description: String,
    #[expect(
        dead_code,
        reason = "domain model completeness — used for audit display"
    )]
    pub record_id: Option<CertificateId>,
}

/// Consent record: patient granted provider access with scope.
#[derive(Debug, Clone)]
pub struct ConsentRecord {
    pub patient_did: String,
    pub provider_did: String,
    pub scope: ConsentScope,
    pub revoked: bool,
}

// ============================================================================
// Medical DAG — rhizoCrypt session + vertices + frontier
// ============================================================================

/// `rhizoCrypt` DAG for medical access provenance.
pub struct MedicalDag {
    pub session: rhizo_crypt_core::Session,
    pub vertices: Vec<rhizo_crypt_core::Vertex>,
    pub frontier: Vec<rhizo_crypt_core::VertexId>,
}

impl MedicalDag {
    fn new() -> Self {
        let session =
            rhizo_crypt_core::SessionBuilder::new(rhizo_crypt_core::SessionType::Gaming {
                game_id: "medical_access".into(),
            })
            .with_name("Medical Access Provenance")
            .build();

        Self {
            session,
            vertices: Vec::new(),
            frontier: Vec::new(),
        }
    }

    fn append(
        &mut self,
        action: &str,
        agent: &rhizo_crypt_core::Did,
        metadata: HashMap<String, rhizo_crypt_core::vertex::MetadataValue>,
    ) -> rhizo_crypt_core::VertexId {
        let mut builder =
            rhizo_crypt_core::VertexBuilder::new(rhizo_crypt_core::EventType::AgentAction {
                action: action.into(),
            })
            .with_agent(agent.clone());

        for &parent in &self.frontier {
            builder = builder.with_parent(parent);
        }
        for (key, value) in metadata {
            builder = builder.with_metadata(key, value);
        }

        let vertex = builder.build();
        let Ok(vertex_id) = vertex.compute_id() else {
            eprintln!("FATAL: vertex id computation failed");
            std::process::exit(1);
        };
        self.session.update_frontier(vertex_id, &self.frontier);
        self.frontier = vec![vertex_id];
        self.vertices.push(vertex);
        vertex_id
    }
}

// ============================================================================
// Access Proof — deterministic signature (`BearDog` model)
// ============================================================================

const PROOF_KEY: [u8; 32] = [0xec; 32]; // Fixed pattern for deterministic proof

fn compute_proof_signature(accessor_did: &str, record_id: &CertificateId, tick: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(accessor_did.as_bytes());
    bytes.extend_from_slice(record_id.to_string().as_bytes());
    bytes.extend_from_slice(&tick.to_le_bytes());
    let hash = blake3::hash(&bytes);
    let hash_bytes = hash.as_bytes();
    let mut sig = vec![0u8; 32];
    for (i, (a, b)) in hash_bytes.iter().zip(PROOF_KEY.iter()).enumerate() {
        sig[i] = a ^ b;
    }
    sig
}

// ============================================================================
// `sweetGrass` Braid Helper
// ============================================================================

fn create_medical_braid(
    actor: &sweet_grass_core::Did,
    event_type: &str,
    description: &str,
    rhizo_vertex_hex: &str,
) -> Result<sweet_grass_core::Braid, sweet_grass_core::SweetGrassError> {
    let activity = sweet_grass_core::Activity {
        id: sweet_grass_core::ActivityId::from_task(event_type),
        activity_type: sweet_grass_core::ActivityType::Creation,
        used: Vec::new(),
        was_associated_with: vec![sweet_grass_core::AgentAssociation {
            agent: actor.clone(),
            role: sweet_grass_core::AgentRole::Creator,
            on_behalf_of: None,
            had_plan: Some("medical_access_system".into()),
        }],
        started_at_time: sweet_grass_core::braid::current_timestamp_nanos(),
        ended_at_time: Some(sweet_grass_core::braid::current_timestamp_nanos()),
        metadata: sweet_grass_core::activity::ActivityMetadata::default(),
        ecop: sweet_grass_core::activity::ActivityEcoPrimals::default(),
    };

    let data_hash = format!("sha256:{rhizo_vertex_hex}");
    let ecop = sweet_grass_core::braid::EcoPrimalsAttributes {
        source_primal: Some("healthspring".into()),
        ..Default::default()
    };

    let mut braid = sweet_grass_core::Braid::builder()
        .data_hash(&data_hash)
        .mime_type("application/x-medical-access")
        .size(description.len() as u64)
        .attributed_to(actor.clone())
        .generated_by(activity)
        .ecop(ecop)
        .build()?;

    braid.metadata.description = Some(description.into());
    Ok(braid)
}

// ============================================================================
// Medical Access System
// ============================================================================

/// Consent-gated medical access system.
pub struct MedicalAccessSystem {
    pub cert_manager: CertificateManager,
    pub dag: MedicalDag,
    pub braids: Vec<sweet_grass_core::Braid>,
    pub consents: HashMap<CertificateId, ConsentRecord>,
    pub access_log: Vec<AccessEvent>,
    pub tick: u64,
}

impl MedicalAccessSystem {
    /// Create a new medical access system.
    pub fn new(owner: &Did) -> Self {
        let Ok(spine) = Spine::new(
            owner.clone(),
            Some("MedicalAccess".into()),
            SpineConfig::default(),
        ) else {
            eprintln!("FATAL: spine creation failed");
            std::process::exit(1);
        };
        let cert_manager = CertificateManager::new(spine);

        Self {
            cert_manager,
            dag: MedicalDag::new(),
            braids: Vec::new(),
            consents: HashMap::new(),
            access_log: Vec::new(),
            tick: 0,
        }
    }

    /// Advance the system tick.
    #[expect(
        clippy::missing_const_for_fn,
        reason = "mutates self — cannot be const"
    )]
    pub fn advance_tick(&mut self) {
        self.tick += 1;
    }

    /// Create a patient medical record.
    pub fn create_record(
        &mut self,
        patient: &Did,
        record_type: RecordType,
        description: &str,
    ) -> CertificateId {
        let metadata = CertificateMetadata::new()
            .with_name(description)
            .with_description(format!("Medical record: {description}"))
            .with_attribute("record_type", record_type.as_str());

        let cert_type = CertificateType::Custom {
            type_uri: "ecoPrimals:medical_record".into(),
            schema_version: 1,
        };

        let Ok((cert, _entry_hash)) = self
            .cert_manager
            .mint(cert_type, patient, metadata)
        else {
            eprintln!("FATAL: certificate minting failed");
            std::process::exit(1);
        };

        let cert_id = cert.id;

        let rhizo_did = rhizo_crypt_core::Did::new(patient.as_str());
        self.dag.session.add_agent(rhizo_did.clone());

        let mut meta = HashMap::new();
        meta.insert(
            "event".into(),
            rhizo_crypt_core::vertex::MetadataValue::String("record_create".into()),
        );
        meta.insert(
            "record_id".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(cert_id.to_string()),
        );
        meta.insert(
            "record_type".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(record_type.as_str().into()),
        );
        self.dag.append("medical_record_create", &rhizo_did, meta);

        cert_id
    }

    /// Grant consent from patient to provider.
    pub fn grant_consent(
        &mut self,
        patient: &Did,
        provider: &Did,
        scope: ConsentScope,
    ) -> CertificateId {
        let metadata = CertificateMetadata::new()
            .with_name("Consent")
            .with_description(format!(
                "Consent: {} -> {} until tick {}",
                patient.as_str(),
                provider.as_str(),
                scope.expiry_tick
            ));

        let cert_type = CertificateType::Custom {
            type_uri: "ecoPrimals:consent".into(),
            schema_version: 1,
        };

        let Ok((cert, _entry_hash)) = self
            .cert_manager
            .mint(cert_type, patient, metadata)
        else {
            eprintln!("FATAL: certificate minting failed");
            std::process::exit(1);
        };

        let consent_id = cert.id;

        self.consents.insert(
            consent_id,
            ConsentRecord {
                patient_did: patient.as_str().into(),
                provider_did: provider.as_str().into(),
                scope,
                revoked: false,
            },
        );

        let rhizo_did = rhizo_crypt_core::Did::new(patient.as_str());
        let provider_rhizo = rhizo_crypt_core::Did::new(provider.as_str());
        self.dag.session.add_agent(provider_rhizo);

        let mut meta = HashMap::new();
        meta.insert(
            "event".into(),
            rhizo_crypt_core::vertex::MetadataValue::String("consent_grant".into()),
        );
        meta.insert(
            "consent_id".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(consent_id.to_string()),
        );
        self.dag.append("medical_consent_grant", &rhizo_did, meta);

        consent_id
    }

    /// Access a record (validates consent, logs, returns proof).
    pub fn access_record(
        &mut self,
        provider: &Did,
        record_id: CertificateId,
        purpose: &str,
        record_type: RecordType,
    ) -> Result<AccessProof, String> {
        let record_cert = self
            .cert_manager
            .get_certificate(&record_id)
            .ok_or("record not found")?;

        let patient_did = record_cert.owner.as_str();

        let _ = self
            .consents
            .values()
            .find(|c| {
                c.patient_did == patient_did
                    && c.provider_did == provider.as_str()
                    && c.scope.record_types.contains(&record_type)
                    && !c.revoked
                    && c.scope.expiry_tick >= self.tick
            })
            .ok_or("no valid consent")?;

        let rhizo_did = rhizo_crypt_core::Did::new(provider.as_str());
        self.dag.session.add_agent(rhizo_did.clone());

        let mut meta = HashMap::new();
        meta.insert(
            "event".into(),
            rhizo_crypt_core::vertex::MetadataValue::String("access".into()),
        );
        meta.insert(
            "record_id".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(record_id.to_string()),
        );
        meta.insert(
            "purpose".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(purpose.into()),
        );
        meta.insert(
            "record_type".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(record_type.as_str().into()),
        );
        let vertex_id = self.dag.append("medical_access", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(provider.as_str());
        if let Ok(braid) = create_medical_braid(
            &sweet_did,
            "medical_access",
            &format!("Access record {record_id} for {purpose}"),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        let event = AccessEvent {
            accessor_did: provider.as_str().into(),
            record_id,
            purpose: purpose.into(),
            record_type,
            tick: self.tick,
            vertex_id: Some(vertex_id),
        };
        self.access_log.push(event);

        let proof_signature = compute_proof_signature(provider.as_str(), &record_id, self.tick);

        Ok(AccessProof {
            accessor_did: provider.as_str().into(),
            record_id,
            timestamp_tick: self.tick,
            proof_signature,
        })
    }

    /// Revoke consent.
    pub fn revoke_consent(
        &mut self,
        patient: &Did,
        consent_id: CertificateId,
    ) -> Result<(), String> {
        let record = self
            .consents
            .get_mut(&consent_id)
            .ok_or("consent not found")?;

        if record.patient_did != patient.as_str() {
            return Err("not consent owner".into());
        }

        record.revoked = true;
        Ok(())
    }

    /// Audit: all access events for a record.
    pub fn audit(&self, record_id: CertificateId) -> Vec<&AccessEvent> {
        self.access_log
            .iter()
            .filter(|e| e.record_id == record_id)
            .collect()
    }

    /// Inject access event for fraud detection testing (bypasses consent checks).
    #[doc(hidden)]
    pub fn inject_access_event_for_fraud_test(
        &mut self,
        event: AccessEvent,
        include_dag_vertex: bool,
    ) {
        if include_dag_vertex {
            let rhizo_did = rhizo_crypt_core::Did::new(&event.accessor_did);
            self.dag.session.add_agent(rhizo_did.clone());

            let mut meta = HashMap::new();
            meta.insert(
                "event".into(),
                rhizo_crypt_core::vertex::MetadataValue::String("access".into()),
            );
            meta.insert(
                "record_id".into(),
                rhizo_crypt_core::vertex::MetadataValue::String(event.record_id.to_string()),
            );
            meta.insert(
                "purpose".into(),
                rhizo_crypt_core::vertex::MetadataValue::String(event.purpose.clone()),
            );
            meta.insert(
                "record_type".into(),
                rhizo_crypt_core::vertex::MetadataValue::String(event.record_type.as_str().into()),
            );
            let vertex_id = self.dag.append("medical_access", &rhizo_did, meta);

            let mut ev = event;
            ev.vertex_id = Some(vertex_id);
            self.access_log.push(ev);
        } else {
            let mut ev = event;
            ev.vertex_id = None;
            self.access_log.push(ev);
        }
    }

    /// Detect all fraud types.
    pub fn detect_fraud(&self) -> Vec<MedicalFraudReport> {
        let mut reports = Vec::new();

        for event in &self.access_log {
            let record_owner = self
                .cert_manager
                .get_certificate(&event.record_id)
                .map(|c| c.owner.as_str().to_string());

            let record_owner = match &record_owner {
                Some(o) => o.clone(),
                None => continue,
            };

            // UnauthorizedAccess: no matching consent for provider+record_type
            let has_consent = self.consents.values().any(|c| {
                c.patient_did == record_owner
                    && c.provider_did == event.accessor_did
                    && c.scope.record_types.contains(&event.record_type)
                    && !c.revoked
            });
            if !has_consent {
                reports.push(MedicalFraudReport {
                    fraud_type: MedicalFraudType::UnauthorizedAccess,
                    description: format!(
                        "Access by {} to record {} without valid consent",
                        event.accessor_did, event.record_id
                    ),
                    record_id: Some(event.record_id),
                });
            }

            // ExpiredConsent: access tick > consent expiry
            if let Some(consent) = self.consents.values().find(|c| {
                c.patient_did == record_owner
                    && c.provider_did == event.accessor_did
                    && c.scope.record_types.contains(&event.record_type)
            }) {
                if event.tick > consent.scope.expiry_tick {
                    reports.push(MedicalFraudReport {
                        fraud_type: MedicalFraudType::ExpiredConsent,
                        description: format!(
                            "Access at tick {} exceeded consent expiry {}",
                            event.tick, consent.scope.expiry_tick
                        ),
                        record_id: Some(event.record_id),
                    });
                }
            }

            // ScopeViolation: record_type not in consent scope
            if let Some(consent) = self
                .consents
                .values()
                .find(|c| c.patient_did == record_owner && c.provider_did == event.accessor_did)
            {
                if !consent.scope.record_types.contains(&event.record_type) {
                    reports.push(MedicalFraudReport {
                        fraud_type: MedicalFraudType::ScopeViolation,
                        description: format!(
                            "Record type {:?} not in consent scope",
                            event.record_type
                        ),
                        record_id: Some(event.record_id),
                    });
                }
            }

            // PhantomAccess: access event has no DAG vertex
            if event.vertex_id.is_none() {
                reports.push(MedicalFraudReport {
                    fraud_type: MedicalFraudType::PhantomAccess,
                    description: format!(
                        "Access event for record {} has no DAG vertex",
                        event.record_id
                    ),
                    record_id: Some(event.record_id),
                });
            }

            // ConsentForgery: consent patient doesn't match record owner
            if let Some(consent) = self.consents.values().find(|c| {
                c.provider_did == event.accessor_did
                    && c.scope.record_types.contains(&event.record_type)
                    && c.patient_did != record_owner
            }) {
                reports.push(MedicalFraudReport {
                    fraud_type: MedicalFraudType::ConsentForgery,
                    description: format!(
                        "Consent patient {} does not match record owner {}",
                        consent.patient_did, record_owner
                    ),
                    record_id: Some(event.record_id),
                });
            }
        }

        reports
    }

    /// Verify an access proof (deterministic check).
    #[expect(
        clippy::unused_self,
        reason = "method belongs to API for consistency with other systems"
    )]
    pub fn verify_proof(&self, proof: &AccessProof) -> bool {
        let expected =
            compute_proof_signature(&proof.accessor_did, &proof.record_id, proof.timestamp_tick);
        proof.proof_signature == expected
    }
}
