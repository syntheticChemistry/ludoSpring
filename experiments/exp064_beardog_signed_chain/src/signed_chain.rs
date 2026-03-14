// SPDX-License-Identifier: AGPL-3.0-or-later
//! BearDog-signed provenance chain — cryptographic binding for trio operations.
//!
//! Every operation on a Novel Ferment Transcript produces a signed artifact:
//! - Vertex signatures (`rhizoCrypt` DAG)
//! - Certificate signatures (`loamSpine`)
//! - Braid signatures (`sweetGrass`)
//!
//! `BearDog`'s Ed25519 signatures bind the entire chain cryptographically.
//! This module models the signing protocol without a live `BearDog` instance,
//! validating the wire format and verification logic.

use std::collections::HashMap;

use loam_spine_core::Did;
use loam_spine_core::certificate::{CertificateMetadata, CertificateType};
use loam_spine_core::entry::SpineConfig;
use loam_spine_core::manager::CertificateManager;
use loam_spine_core::spine::Spine;
use loam_spine_core::types::CertificateId;

/// Ed25519 key pair (simplified model for validation — real signing via `BearDog` IPC).
#[derive(Debug, Clone)]
pub struct Ed25519KeyPair {
    pub public_key: [u8; 32],
    secret_seed: [u8; 32],
}

impl Ed25519KeyPair {
    /// Generate a deterministic key pair from a seed string.
    pub fn from_seed(seed: &str) -> Self {
        let mut secret_seed = [0u8; 32];
        let bytes = seed.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            secret_seed[i % 32] ^= b;
        }
        let mut public_key = [0u8; 32];
        for (i, &b) in secret_seed.iter().enumerate() {
            public_key[i] = b ^ 0xFF;
        }
        Self {
            public_key,
            secret_seed,
        }
    }

    /// Sign a message (deterministic HMAC-like model for validation).
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let mut sig = [0u8; 64];
        for (i, &b) in message.iter().enumerate().take(32) {
            sig[i] = b ^ self.secret_seed[i % 32];
        }
        for (i, &b) in self.secret_seed.iter().enumerate() {
            sig[32 + i] = b ^ message.first().map_or(0, |&x| x);
        }
        sig
    }

    /// Verify a signature against a message and public key.
    pub fn verify(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
        let mut secret_seed = [0u8; 32];
        for (i, &b) in public_key.iter().enumerate() {
            secret_seed[i] = b ^ 0xFF;
        }
        let mut expected = [0u8; 64];
        for (i, &b) in message.iter().enumerate().take(32) {
            expected[i] = b ^ secret_seed[i % 32];
        }
        for (i, &b) in secret_seed.iter().enumerate() {
            expected[32 + i] = b ^ message.first().map_or(0, |&x| x);
        }
        *signature == expected
    }
}

/// A signed `rhizoCrypt` DAG vertex.
#[derive(Debug, Clone)]
#[expect(
    dead_code,
    reason = "domain model completeness — fields used for chain inspection"
)]
pub struct SignedVertex {
    pub vertex_id: rhizo_crypt_core::VertexId,
    pub event_type: String,
    pub agent: String,
    pub content_hash: Vec<u8>,
    pub signature: [u8; 64],
    pub signer_public_key: [u8; 32],
}

/// A signed `loamSpine` certificate operation.
#[derive(Debug, Clone)]
pub struct SignedCertOperation {
    pub cert_id: CertificateId,
    pub operation: String,
    pub content_hash: Vec<u8>,
    pub signature: [u8; 64],
    pub signer_public_key: [u8; 32],
}

/// A signed `sweetGrass` braid.
#[derive(Debug, Clone)]
pub struct SignedBraid {
    pub braid_description: String,
    pub content_hash: Vec<u8>,
    pub signature: [u8; 64],
    pub signer_public_key: [u8; 32],
}

/// A complete signed provenance chain.
pub struct SignedProvenanceChain {
    pub key_pair: Ed25519KeyPair,
    pub cert_manager: CertificateManager,
    pub dag: ProvDag,
    pub signed_vertices: Vec<SignedVertex>,
    pub signed_certs: Vec<SignedCertOperation>,
    pub signed_braids: Vec<SignedBraid>,
}

/// `rhizoCrypt` DAG wrapper (same pattern as exp061).
pub struct ProvDag {
    pub session: rhizo_crypt_core::Session,
    pub vertices: Vec<rhizo_crypt_core::Vertex>,
    pub frontier: Vec<rhizo_crypt_core::VertexId>,
}

impl ProvDag {
    fn new(session_type: &str) -> Self {
        let session =
            rhizo_crypt_core::SessionBuilder::new(rhizo_crypt_core::session::SessionType::Gaming {
                game_id: session_type.into(),
            })
            .with_name("Signed Provenance Chain")
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
        agent: &rhizo_crypt_core::Did,
        metadata: HashMap<String, rhizo_crypt_core::vertex::MetadataValue>,
    ) -> (rhizo_crypt_core::VertexId, Vec<u8>) {
        let mut builder =
            rhizo_crypt_core::VertexBuilder::new(rhizo_crypt_core::EventType::AgentAction {
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
        let vertex_id = vertex.compute_id().expect("vertex id");
        self.session.update_frontier(vertex_id, &self.frontier);

        let content_hash = vertex_id.as_bytes().to_vec();

        self.frontier = vec![vertex_id];
        self.vertices.push(vertex);
        (vertex_id, content_hash)
    }
}

impl SignedProvenanceChain {
    /// Create a new signed provenance chain.
    pub fn new(owner: &Did, key_seed: &str) -> Self {
        let spine = Spine::new(
            owner.clone(),
            Some("SignedChain".into()),
            SpineConfig::default(),
        )
        .expect("spine creation");
        let cert_manager = CertificateManager::new(spine);
        let key_pair = Ed25519KeyPair::from_seed(key_seed);

        Self {
            key_pair,
            cert_manager,
            dag: ProvDag::new("signed_chain"),
            signed_vertices: Vec::new(),
            signed_certs: Vec::new(),
            signed_braids: Vec::new(),
        }
    }

    /// Sign and append a DAG vertex.
    pub fn sign_and_append_vertex(
        &mut self,
        event_type: &str,
        agent_did: &str,
    ) -> rhizo_crypt_core::VertexId {
        let agent = rhizo_crypt_core::Did::new(agent_did);
        self.dag.session.add_agent(agent.clone());

        let mut meta = HashMap::new();
        meta.insert(
            "event".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(event_type.into()),
        );

        let (vertex_id, content_hash) = self.dag.append(event_type, &agent, meta);
        let signature = self.key_pair.sign(&content_hash);

        self.signed_vertices.push(SignedVertex {
            vertex_id,
            event_type: event_type.into(),
            agent: agent_did.into(),
            content_hash,
            signature,
            signer_public_key: self.key_pair.public_key,
        });

        vertex_id
    }

    /// Sign and mint a certificate.
    pub fn sign_and_mint_cert(
        &mut self,
        owner: &Did,
        name: &str,
        item_type: &str,
    ) -> CertificateId {
        let metadata = CertificateMetadata::new()
            .with_name(name)
            .with_description(format!("Signed cert: {name}"));

        let cert_type = CertificateType::GameItem {
            game_id: "signed_chain".into(),
            item_type: item_type.into(),
            item_id: format!("signed_{}", uuid::Uuid::now_v7()),
            attributes: HashMap::new(),
        };

        let (cert, _) = self
            .cert_manager
            .mint(cert_type, owner, metadata)
            .expect("certificate minting");

        let cert_id = cert.id;
        let content_hash = cert_id.to_string().into_bytes();
        let signature = self.key_pair.sign(&content_hash);

        self.signed_certs.push(SignedCertOperation {
            cert_id,
            operation: "mint".into(),
            content_hash,
            signature,
            signer_public_key: self.key_pair.public_key,
        });

        cert_id
    }

    /// Sign a certificate transfer.
    pub fn sign_and_transfer(
        &mut self,
        cert_id: CertificateId,
        from: &Did,
        to: &Did,
    ) -> Result<(), loam_spine_core::error::LoamSpineError> {
        self.cert_manager.transfer(cert_id, from, to)?;

        let content = format!("transfer:{}:{}:{}", cert_id, from.as_str(), to.as_str());
        let content_hash = content.into_bytes();
        let signature = self.key_pair.sign(&content_hash);

        self.signed_certs.push(SignedCertOperation {
            cert_id,
            operation: "transfer".into(),
            content_hash,
            signature,
            signer_public_key: self.key_pair.public_key,
        });

        Ok(())
    }

    /// Sign and create a braid.
    pub fn sign_and_create_braid(&mut self, description: &str, actor_did: &str) {
        let content_hash = description.as_bytes().to_vec();
        let signature = self.key_pair.sign(&content_hash);

        let data_hash = format!(
            "sha256:{}",
            hex::encode(&self.key_pair.sign(&content_hash)[..16])
        );

        self.signed_braids.push(SignedBraid {
            braid_description: description.into(),
            content_hash,
            signature,
            signer_public_key: self.key_pair.public_key,
        });

        let sweet_did = sweet_grass_core::Did::new(actor_did);
        let activity = sweet_grass_core::Activity {
            id: sweet_grass_core::ActivityId::from_task("signed_event"),
            activity_type: sweet_grass_core::ActivityType::Creation,
            used: Vec::new(),
            was_associated_with: vec![sweet_grass_core::AgentAssociation {
                agent: sweet_did.clone(),
                role: sweet_grass_core::AgentRole::Creator,
                on_behalf_of: None,
                had_plan: Some("signed_chain".into()),
            }],
            started_at_time: sweet_grass_core::braid::current_timestamp_nanos(),
            ended_at_time: Some(sweet_grass_core::braid::current_timestamp_nanos()),
            metadata: sweet_grass_core::activity::ActivityMetadata::default(),
            ecop: sweet_grass_core::activity::ActivityEcoPrimals::default(),
        };
        let _ = sweet_grass_core::Braid::builder()
            .data_hash(&data_hash)
            .mime_type("application/x-signed-event")
            .size(description.len() as u64)
            .attributed_to(sweet_did)
            .generated_by(activity)
            .build();
    }

    /// Verify the entire chain — every signature must match its content.
    pub fn verify_chain(&self) -> ChainVerification {
        let mut result = ChainVerification {
            total_items: 0,
            verified: 0,
            tampered: Vec::new(),
        };

        for sv in &self.signed_vertices {
            result.total_items += 1;
            if Ed25519KeyPair::verify(&sv.signer_public_key, &sv.content_hash, &sv.signature) {
                result.verified += 1;
            } else {
                let event_type = &sv.event_type;
                result.tampered.push(format!("vertex:{event_type}"));
            }
        }

        for sc in &self.signed_certs {
            result.total_items += 1;
            if Ed25519KeyPair::verify(&sc.signer_public_key, &sc.content_hash, &sc.signature) {
                result.verified += 1;
            } else {
                let operation = &sc.operation;
                let cert_id = sc.cert_id;
                result.tampered.push(format!("cert:{operation}:{cert_id}"));
            }
        }

        for sb in &self.signed_braids {
            result.total_items += 1;
            if Ed25519KeyPair::verify(&sb.signer_public_key, &sb.content_hash, &sb.signature) {
                result.verified += 1;
            } else {
                let braid_description = &sb.braid_description;
                result.tampered.push(format!("braid:{braid_description}"));
            }
        }

        result
    }
}

/// Result of chain verification.
pub struct ChainVerification {
    pub total_items: usize,
    pub verified: usize,
    pub tampered: Vec<String>,
}

impl ChainVerification {
    pub const fn is_clean(&self) -> bool {
        self.tampered.is_empty()
    }
}

/// hex encoding helper (no external dependency).
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().fold(String::new(), |mut s, b| {
            use std::fmt::Write;
            let _ = write!(s, "{b:02x}");
            s
        })
    }
}

// ============================================================================
// BearDog IPC Wire Format
// ============================================================================

/// JSON-RPC request for `crypto.sign_ed25519`.
#[expect(
    dead_code,
    reason = "wire format completeness — types used at runtime over IPC"
)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BearDogSignRequest {
    pub message: String,
    pub key_id: String,
}

/// JSON-RPC response for `crypto.sign_ed25519`.
#[expect(
    dead_code,
    reason = "wire format completeness — types used at runtime over IPC"
)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BearDogSignResponse {
    pub signature: String,
    pub public_key: String,
}

/// JSON-RPC request for `crypto.verify_ed25519`.
#[expect(
    dead_code,
    reason = "wire format completeness — types used at runtime over IPC"
)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BearDogVerifyRequest {
    pub message: String,
    pub signature: String,
    pub public_key: String,
}

/// JSON-RPC response for `crypto.verify_ed25519`.
#[expect(
    dead_code,
    reason = "wire format completeness — types used at runtime over IPC"
)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BearDogVerifyResponse {
    pub valid: bool,
}

/// JSON-RPC request for `crypto.blake3_hash`.
#[expect(
    dead_code,
    reason = "wire format completeness — types used at runtime over IPC"
)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BearDogHashRequest {
    pub data: String,
}

/// JSON-RPC response for `crypto.blake3_hash`.
#[expect(
    dead_code,
    reason = "wire format completeness — types used at runtime over IPC"
)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BearDogHashResponse {
    pub hash: String,
}

/// Build the IPC sequence for signing a vertex.
pub fn sign_vertex_ipc(vertex_content: &str, key_id: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "crypto.sign_ed25519",
        "params": {
            "message": vertex_content,
            "key_id": key_id,
        },
        "id": uuid::Uuid::now_v7().to_string(),
    })
}

/// Build the IPC sequence for verifying a signature.
pub fn verify_signature_ipc(message: &str, signature: &str, public_key: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "crypto.verify_ed25519",
        "params": {
            "message": message,
            "signature": signature,
            "public_key": public_key,
        },
        "id": uuid::Uuid::now_v7().to_string(),
    })
}

/// Build the IPC sequence for hashing content.
pub fn hash_content_ipc(data: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "crypto.blake3_hash",
        "params": {
            "data": data,
        },
        "id": uuid::Uuid::now_v7().to_string(),
    })
}
