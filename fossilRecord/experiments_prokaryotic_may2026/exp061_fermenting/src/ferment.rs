// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fermenting system — memory-bound digital objects with provenance.
//!
//! A "ferment" is a digital object whose value accumulates through use,
//! like a culture that transforms raw materials into something richer.
//! The provenance DAG is the culture. The loamSpine certificate is the
//! vessel. The history cannot be forged — you cannot un-ferment.
//!
//! Ferment = loamSpine Certificate (ownership + identity)
//!         + rhizoCrypt DAG (action history)
//!         + sweetGrass Braids (semantic memory / provenance)
//!         + Cosmetic Schema (visual representation)

use std::collections::HashMap;

use loam_spine_core::Did;
use loam_spine_core::certificate::{CertificateMetadata, CertificateType, LoanTerms};
use loam_spine_core::entry::SpineConfig;
use loam_spine_core::manager::CertificateManager;
use loam_spine_core::spine::Spine;
use loam_spine_core::types::CertificateId;
use ludospring_barracuda::validation::OrExit;

// ============================================================================
// Cosmetic Schema
// ============================================================================

/// Visual rarity tier for a fermenting object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    /// String representation for certificate metadata.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Common => "common",
            Self::Uncommon => "uncommon",
            Self::Rare => "rare",
            Self::Epic => "epic",
            Self::Legendary => "legendary",
        }
    }
}

/// Visual appearance of a fermenting object.
#[derive(Debug, Clone)]
pub struct CosmeticSchema {
    pub rarity: Rarity,
    pub skin: String,
    pub color: String,
    pub material: String,
    pub wear_level: f64,
}

impl CosmeticSchema {
    /// Pack cosmetic data into certificate metadata attributes.
    pub fn to_attributes(&self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        attrs.insert("rarity".into(), self.rarity.as_str().into());
        attrs.insert("skin".into(), self.skin.clone());
        attrs.insert("color".into(), self.color.clone());
        attrs.insert("material".into(), self.material.clone());
        attrs.insert("wear_level".into(), format!("{:.2}", self.wear_level));
        attrs
    }

    /// Unpack cosmetic data from certificate metadata attributes.
    pub fn from_attributes(attrs: &HashMap<String, String>) -> Option<Self> {
        let rarity = match attrs.get("rarity")?.as_str() {
            "common" => Rarity::Common,
            "uncommon" => Rarity::Uncommon,
            "rare" => Rarity::Rare,
            "epic" => Rarity::Epic,
            "legendary" => Rarity::Legendary,
            _ => return None,
        };
        Some(Self {
            rarity,
            skin: attrs.get("skin")?.clone(),
            color: attrs.get("color")?.clone(),
            material: attrs.get("material")?.clone(),
            wear_level: attrs.get("wear_level")?.parse().ok()?,
        })
    }
}

// ============================================================================
// Fermenting Object — the unified type
// ============================================================================

/// A fermenting object: a memory-bound digital item with full provenance.
///
/// The certificate IS the persistent identity. The DAG records every action.
/// The braids provide semantic attribution. The cosmetics are visual metadata.
#[expect(
    dead_code,
    reason = "domain model completeness — fields used for inspection/queries"
)]
pub struct FermentingObject {
    pub cert_id: CertificateId,
    pub item_name: String,
    pub item_type: String,
    pub cosmetics: CosmeticSchema,
    pub event_count: u64,
}

/// An event in the life of a fermenting object.
#[derive(Debug, Clone)]
#[expect(
    dead_code,
    reason = "domain model completeness — fields available for timeline queries"
)]
pub struct FermentEvent {
    pub event_type: FermentEventType,
    pub actor_did: String,
    pub description: String,
    pub tick: u64,
}

/// Types of events that can happen to a fermenting object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FermentEventType {
    Mint,
    Inspect,
    Trade,
    Loan,
    LoanReturn,
    Consume,
    #[expect(
        dead_code,
        reason = "domain model completeness — variant for future cosmetic updates"
    )]
    CosmeticChange,
    Achievement,
}

impl FermentEventType {
    #[expect(
        dead_code,
        reason = "domain model completeness — used for serialization/display"
    )]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Mint => "mint",
            Self::Inspect => "inspect",
            Self::Trade => "trade",
            Self::Loan => "loan",
            Self::LoanReturn => "loan_return",
            Self::Consume => "consume",
            Self::CosmeticChange => "cosmetic_change",
            Self::Achievement => "achievement",
        }
    }
}

// ============================================================================
// Fermenting System — orchestrates trio integration
// ============================================================================

/// The fermenting system: certificate manager + DAG + attribution.
///
/// This is the glue that ties loamSpine certificates, rhizoCrypt DAG
/// vertices, and sweetGrass braids into a single coherent lifecycle.
pub struct FermentingSystem {
    pub cert_manager: CertificateManager,
    pub dag: FermentDag,
    pub braids: Vec<sweet_grass_core::Braid>,
    pub objects: HashMap<CertificateId, FermentingObject>,
    pub events: Vec<(CertificateId, FermentEvent)>,
    pub tick: u64,
}

/// rhizoCrypt DAG for fermenting object history.
pub struct FermentDag {
    pub session: rhizo_crypt_core::Session,
    pub vertices: Vec<rhizo_crypt_core::Vertex>,
    pub frontier: Vec<rhizo_crypt_core::VertexId>,
}

impl FermentDag {
    fn new() -> Self {
        let session =
            rhizo_crypt_core::SessionBuilder::new(rhizo_crypt_core::session::SessionType::Gaming {
                game_id: "fermenting".into(),
            })
            .with_name("Fermenting Object History")
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
    ) -> rhizo_crypt_core::VertexId {
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
        let vertex_id = vertex.compute_id().or_exit("vertex id computation");
        self.session.update_frontier(vertex_id, &self.frontier);
        self.frontier = vec![vertex_id];
        self.vertices.push(vertex);
        vertex_id
    }
}

impl FermentingSystem {
    /// Create a new fermenting system.
    pub fn new(owner: &Did) -> Self {
        let spine = Spine::new(
            owner.clone(),
            Some("Fermenting".into()),
            SpineConfig::default(),
        )
        .or_exit("spine creation");
        let cert_manager = CertificateManager::new(spine);

        Self {
            cert_manager,
            dag: FermentDag::new(),
            braids: Vec::new(),
            objects: HashMap::new(),
            events: Vec::new(),
            tick: 0,
        }
    }

    /// Advance the system tick.
    pub const fn advance_tick(&mut self) {
        self.tick += 1;
    }

    /// Mint a new fermenting object.
    ///
    /// Creates a loamSpine certificate, a rhizoCrypt DAG vertex, and a
    /// sweetGrass attribution braid. Returns the certificate ID.
    pub fn mint(
        &mut self,
        owner: &Did,
        item_name: &str,
        item_type: &str,
        cosmetics: CosmeticSchema,
    ) -> CertificateId {
        let mut item_attrs = cosmetics.to_attributes();
        item_attrs.insert("item_type".into(), item_type.into());

        let metadata = CertificateMetadata::new()
            .with_name(item_name)
            .with_description(format!("Fermenting object: {item_name} ({item_type})"));

        let cert_type = CertificateType::GameItem {
            game_id: "fermenting".into(),
            item_type: item_type.into(),
            item_id: format!("{}_{}", item_type, uuid::Uuid::now_v7()),
            attributes: item_attrs,
        };

        let (cert, _entry_hash) = self
            .cert_manager
            .mint(cert_type, owner, metadata)
            .or_exit("certificate minting");

        let cert_id = cert.id;

        let rhizo_did = rhizo_crypt_core::Did::new(owner.as_str());
        self.dag.session.add_agent(rhizo_did.clone());

        let mut meta = HashMap::new();
        meta.insert(
            "event".into(),
            rhizo_crypt_core::vertex::MetadataValue::String("mint".into()),
        );
        meta.insert(
            "item_name".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(item_name.into()),
        );
        meta.insert(
            "cert_id".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(cert_id.to_string()),
        );
        let vertex_id = self.dag.append("ferment_mint", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(owner.as_str());
        if let Ok(braid) = create_ferment_braid(
            &sweet_did,
            "mint",
            &format!("Minted: {item_name}"),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        self.objects.insert(
            cert_id,
            FermentingObject {
                cert_id,
                item_name: item_name.into(),
                item_type: item_type.into(),
                cosmetics,
                event_count: 1,
            },
        );

        self.events.push((
            cert_id,
            FermentEvent {
                event_type: FermentEventType::Mint,
                actor_did: owner.as_str().into(),
                description: format!("Minted {item_name}"),
                tick: self.tick,
            },
        ));

        cert_id
    }

    /// Record an inspection event (viewing the object's history/details).
    pub fn inspect(&mut self, cert_id: CertificateId, viewer: &Did) {
        let rhizo_did = rhizo_crypt_core::Did::new(viewer.as_str());
        let mut meta = HashMap::new();
        meta.insert(
            "event".into(),
            rhizo_crypt_core::vertex::MetadataValue::String("inspect".into()),
        );
        meta.insert(
            "cert_id".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(cert_id.to_string()),
        );
        let vertex_id = self.dag.append("ferment_inspect", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(viewer.as_str());
        if let Ok(braid) = create_ferment_braid(
            &sweet_did,
            "inspect",
            &format!("Inspected cert {cert_id}"),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        if let Some(obj) = self.objects.get_mut(&cert_id) {
            obj.event_count += 1;
        }

        self.events.push((
            cert_id,
            FermentEvent {
                event_type: FermentEventType::Inspect,
                actor_did: viewer.as_str().into(),
                description: format!("Inspected by {}", viewer.as_str()),
                tick: self.tick,
            },
        ));
    }

    /// Trade a fermenting object: transfer ownership with provenance.
    pub fn trade(
        &mut self,
        cert_id: CertificateId,
        from: &Did,
        to: &Did,
    ) -> Result<(), loam_spine_core::error::LoamSpineError> {
        self.cert_manager.transfer(cert_id, from, to)?;

        let rhizo_did = rhizo_crypt_core::Did::new(from.as_str());
        let to_rhizo = rhizo_crypt_core::Did::new(to.as_str());
        self.dag.session.add_agent(to_rhizo);

        let mut meta = HashMap::new();
        meta.insert(
            "event".into(),
            rhizo_crypt_core::vertex::MetadataValue::String("trade".into()),
        );
        meta.insert(
            "from".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(from.as_str().into()),
        );
        meta.insert(
            "to".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(to.as_str().into()),
        );
        meta.insert(
            "cert_id".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(cert_id.to_string()),
        );
        let vertex_id = self.dag.append("ferment_trade", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(from.as_str());
        if let Ok(braid) = create_ferment_braid(
            &sweet_did,
            "trade",
            &format!("Traded cert {cert_id} to {}", to.as_str()),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        if let Some(obj) = self.objects.get_mut(&cert_id) {
            obj.event_count += 1;
        }

        self.events.push((
            cert_id,
            FermentEvent {
                event_type: FermentEventType::Trade,
                actor_did: from.as_str().into(),
                description: format!("Traded to {}", to.as_str()),
                tick: self.tick,
            },
        ));

        Ok(())
    }

    /// Loan a fermenting object to another player.
    pub fn loan(
        &mut self,
        cert_id: CertificateId,
        owner: &Did,
        borrower: &Did,
        terms: LoanTerms,
    ) -> Result<(), loam_spine_core::error::LoamSpineError> {
        self.cert_manager.loan(cert_id, owner, borrower, terms)?;

        let rhizo_did = rhizo_crypt_core::Did::new(owner.as_str());
        let borrower_rhizo = rhizo_crypt_core::Did::new(borrower.as_str());
        self.dag.session.add_agent(borrower_rhizo);

        let mut meta = HashMap::new();
        meta.insert(
            "event".into(),
            rhizo_crypt_core::vertex::MetadataValue::String("loan".into()),
        );
        meta.insert(
            "lender".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(owner.as_str().into()),
        );
        meta.insert(
            "borrower".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(borrower.as_str().into()),
        );
        let vertex_id = self.dag.append("ferment_loan", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(owner.as_str());
        if let Ok(braid) = create_ferment_braid(
            &sweet_did,
            "loan",
            &format!("Loaned cert {cert_id} to {}", borrower.as_str()),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        if let Some(obj) = self.objects.get_mut(&cert_id) {
            obj.event_count += 1;
        }

        self.events.push((
            cert_id,
            FermentEvent {
                event_type: FermentEventType::Loan,
                actor_did: owner.as_str().into(),
                description: format!("Loaned to {}", borrower.as_str()),
                tick: self.tick,
            },
        ));

        Ok(())
    }

    /// Return a loaned fermenting object.
    pub fn return_loan(
        &mut self,
        cert_id: CertificateId,
        borrower: &Did,
    ) -> Result<(), loam_spine_core::error::LoamSpineError> {
        self.cert_manager.return_loan(cert_id, borrower)?;

        let rhizo_did = rhizo_crypt_core::Did::new(borrower.as_str());
        let mut meta = HashMap::new();
        meta.insert(
            "event".into(),
            rhizo_crypt_core::vertex::MetadataValue::String("loan_return".into()),
        );
        meta.insert(
            "cert_id".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(cert_id.to_string()),
        );
        let vertex_id = self.dag.append("ferment_return", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(borrower.as_str());
        if let Ok(braid) = create_ferment_braid(
            &sweet_did,
            "loan_return",
            &format!("Returned cert {cert_id}"),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        if let Some(obj) = self.objects.get_mut(&cert_id) {
            obj.event_count += 1;
        }

        self.events.push((
            cert_id,
            FermentEvent {
                event_type: FermentEventType::LoanReturn,
                actor_did: borrower.as_str().into(),
                description: "Returned from loan".into(),
                tick: self.tick,
            },
        ));

        Ok(())
    }

    /// Record an achievement for a fermenting object (e.g. "killed 100 enemies").
    pub fn record_achievement(&mut self, cert_id: CertificateId, actor: &Did, achievement: &str) {
        let rhizo_did = rhizo_crypt_core::Did::new(actor.as_str());
        let mut meta = HashMap::new();
        meta.insert(
            "event".into(),
            rhizo_crypt_core::vertex::MetadataValue::String("achievement".into()),
        );
        meta.insert(
            "achievement".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(achievement.into()),
        );
        meta.insert(
            "cert_id".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(cert_id.to_string()),
        );
        let vertex_id = self.dag.append("ferment_achievement", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(actor.as_str());
        if let Ok(braid) = create_ferment_braid(
            &sweet_did,
            "achievement",
            &format!("Achievement: {achievement}"),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        if let Some(obj) = self.objects.get_mut(&cert_id) {
            obj.event_count += 1;
        }

        self.events.push((
            cert_id,
            FermentEvent {
                event_type: FermentEventType::Achievement,
                actor_did: actor.as_str().into(),
                description: format!("Achievement: {achievement}"),
                tick: self.tick,
            },
        ));
    }

    /// Consume a fermenting object (irreversible — the object is spent).
    pub fn consume(&mut self, cert_id: CertificateId, actor: &Did) {
        let rhizo_did = rhizo_crypt_core::Did::new(actor.as_str());
        let mut meta = HashMap::new();
        meta.insert(
            "event".into(),
            rhizo_crypt_core::vertex::MetadataValue::String("consume".into()),
        );
        meta.insert(
            "cert_id".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(cert_id.to_string()),
        );
        let vertex_id = self.dag.append("ferment_consume", &rhizo_did, meta);

        let sweet_did = sweet_grass_core::Did::new(actor.as_str());
        if let Ok(braid) = create_ferment_braid(
            &sweet_did,
            "consume",
            &format!("Consumed cert {cert_id}"),
            &vertex_id.to_hex(),
        ) {
            self.braids.push(braid);
        }

        if let Some(obj) = self.objects.get_mut(&cert_id) {
            obj.event_count += 1;
        }

        self.events.push((
            cert_id,
            FermentEvent {
                event_type: FermentEventType::Consume,
                actor_did: actor.as_str().into(),
                description: "Consumed (irreversible)".into(),
                tick: self.tick,
            },
        ));
    }

    /// Get the event timeline for a specific fermenting object.
    pub fn object_timeline(&self, cert_id: CertificateId) -> Vec<&FermentEvent> {
        self.events
            .iter()
            .filter(|(id, _)| *id == cert_id)
            .map(|(_, event)| event)
            .collect()
    }

    /// Get all objects owned by a specific DID.
    pub fn objects_owned_by(&self, owner: &Did) -> Vec<&FermentingObject> {
        self.objects
            .values()
            .filter(|obj| {
                self.cert_manager
                    .get_certificate(&obj.cert_id)
                    .is_some_and(|cert| &cert.owner == owner)
            })
            .collect()
    }
}

// ============================================================================
// Trading Protocol — offer/accept/reject flow
// ============================================================================

/// A trade offer between two players.
#[derive(Debug, Clone)]
pub struct TradeOffer {
    pub offer_id: uuid::Uuid,
    pub from: String,
    pub to: String,
    pub offered_cert: CertificateId,
    pub requested_cert: Option<CertificateId>,
    pub state: TradeState,
}

/// State of a trade offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradeState {
    Pending,
    Accepted,
    Rejected,
    Cancelled,
    Completed,
}

/// Trading protocol — manages offers and atomic swaps.
pub struct TradingProtocol {
    pub offers: Vec<TradeOffer>,
}

impl TradingProtocol {
    pub const fn new() -> Self {
        Self { offers: Vec::new() }
    }

    /// Create a new trade offer (one-sided: I give you my item).
    pub fn offer(&mut self, from: &str, to: &str, offered_cert: CertificateId) -> uuid::Uuid {
        let offer_id = uuid::Uuid::now_v7();
        self.offers.push(TradeOffer {
            offer_id,
            from: from.into(),
            to: to.into(),
            offered_cert,
            requested_cert: None,
            state: TradeState::Pending,
        });
        offer_id
    }

    /// Create an atomic swap offer (I give you X, you give me Y).
    pub fn offer_swap(
        &mut self,
        from: &str,
        to: &str,
        offered_cert: CertificateId,
        requested_cert: CertificateId,
    ) -> uuid::Uuid {
        let offer_id = uuid::Uuid::now_v7();
        self.offers.push(TradeOffer {
            offer_id,
            from: from.into(),
            to: to.into(),
            offered_cert,
            requested_cert: Some(requested_cert),
            state: TradeState::Pending,
        });
        offer_id
    }

    /// Accept a trade offer.
    pub fn accept(&mut self, offer_id: uuid::Uuid) -> bool {
        if let Some(offer) = self.offers.iter_mut().find(|o| o.offer_id == offer_id) {
            if offer.state == TradeState::Pending {
                offer.state = TradeState::Accepted;
                return true;
            }
        }
        false
    }

    /// Reject a trade offer.
    pub fn reject(&mut self, offer_id: uuid::Uuid) -> bool {
        if let Some(offer) = self.offers.iter_mut().find(|o| o.offer_id == offer_id) {
            if offer.state == TradeState::Pending {
                offer.state = TradeState::Rejected;
                return true;
            }
        }
        false
    }

    /// Cancel a trade offer (by the offerer).
    pub fn cancel(&mut self, offer_id: uuid::Uuid) -> bool {
        if let Some(offer) = self.offers.iter_mut().find(|o| o.offer_id == offer_id) {
            if offer.state == TradeState::Pending {
                offer.state = TradeState::Cancelled;
                return true;
            }
        }
        false
    }

    /// Execute an accepted offer via the fermenting system (atomic swap).
    ///
    /// For a simple trade: transfers `offered_cert` from -> to.
    /// For a swap: transfers both certs atomically (both or neither).
    pub fn execute(
        &mut self,
        offer_id: uuid::Uuid,
        system: &mut FermentingSystem,
    ) -> Result<(), String> {
        let offer = self
            .offers
            .iter()
            .find(|o| o.offer_id == offer_id)
            .ok_or("offer not found")?
            .clone();

        if offer.state != TradeState::Accepted {
            return Err("offer not accepted".into());
        }

        let from_did = Did::new(offer.from.as_str());
        let to_did = Did::new(offer.to.as_str());

        system
            .trade(offer.offered_cert, &from_did, &to_did)
            .map_err(|e| format!("forward trade failed: {e}"))?;

        if let Some(requested_cert) = offer.requested_cert {
            if let Err(err) = system.trade(requested_cert, &to_did, &from_did) {
                // Rollback: reverse the first trade
                let _ = system.trade(offer.offered_cert, &to_did, &from_did);
                return Err(format!("reverse trade failed (rolled back): {err}"));
            }
        }

        if let Some(o) = self.offers.iter_mut().find(|o| o.offer_id == offer_id) {
            o.state = TradeState::Completed;
        }

        Ok(())
    }

    /// Count offers by state.
    pub fn count_by_state(&self, state: &TradeState) -> usize {
        self.offers.iter().filter(|o| &o.state == state).count()
    }
}

// ============================================================================
// sweetGrass Braid Helper
// ============================================================================

/// Create a PROV-O attribution braid for a fermenting event.
fn create_ferment_braid(
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
            had_plan: Some("fermenting_system".into()),
        }],
        started_at_time: sweet_grass_core::braid::current_timestamp_nanos(),
        ended_at_time: Some(sweet_grass_core::braid::current_timestamp_nanos()),
        metadata: sweet_grass_core::activity::ActivityMetadata::default(),
        ecop: sweet_grass_core::activity::ActivityEcoPrimals::default(),
    };

    let data_hash = format!("sha256:{rhizo_vertex_hex}");
    let ecop = sweet_grass_core::braid::EcoPrimalsAttributes {
        source_primal: Some("ludospring".into()),
        ..Default::default()
    };

    let mut braid = sweet_grass_core::Braid::builder()
        .data_hash(&data_hash)
        .mime_type("application/x-ferment-event")
        .size(description.len() as u64)
        .attributed_to(actor.clone())
        .generated_by(activity)
        .ecop(ecop)
        .build()?;

    braid.metadata.description = Some(description.into());
    Ok(braid)
}
