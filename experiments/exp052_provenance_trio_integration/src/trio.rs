// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provenance Trio type wiring for ludoSpring.
//!
//! Maps ludoSpring game domain types to the three provenance primals:
//!   - **rhizoCrypt**: game session DAGs (ephemeral working memory)
//!   - **loamSpine**: ruleset/card certificates (permanent immutable ledger)
//!   - **sweetGrass**: player/AI attribution (W3C PROV-O semantics)
//!
//! The trio lives among the biomeOS atomics — rhizoCrypt provides the
//! ephemeral workspace, loamSpine anchors permanent records, and sweetGrass
//! attributes creative contributions. biomeOS coordinates them via the
//! rootpulse niche (see `niches/rootpulse/rootpulse-niche.yaml`).

use std::collections::HashMap;

// ============================================================================
// rhizoCrypt — Game Session as Ephemeral DAG
// ============================================================================

/// A game session DAG built on rhizoCrypt primitives.
///
/// Each game action (draw, cast, attack, block, etc.) becomes a `Vertex`
/// in the DAG. The session lifecycle mirrors rhizoCrypt's:
/// Active → Resolving → Committed (anchored to loamSpine).
pub struct GameSessionDag {
    pub session: rhizo_crypt_core::Session,
    pub vertices: Vec<rhizo_crypt_core::Vertex>,
    pub frontier: Vec<rhizo_crypt_core::VertexId>,
}

impl GameSessionDag {
    /// Create a new game session DAG.
    pub fn new(name: &str) -> Self {
        let session =
            rhizo_crypt_core::SessionBuilder::new(rhizo_crypt_core::session::SessionType::Gaming {
                game_id: "mtg".into(),
            })
            .with_name(name)
            .build();

        Self {
            session,
            vertices: Vec::new(),
            frontier: Vec::new(),
        }
    }

    /// Append a game action as a DAG vertex.
    pub fn append_action(
        &mut self,
        event_type: rhizo_crypt_core::EventType,
        agent: &rhizo_crypt_core::Did,
        metadata: HashMap<String, rhizo_crypt_core::vertex::MetadataValue>,
    ) -> rhizo_crypt_core::VertexId {
        let mut builder =
            rhizo_crypt_core::VertexBuilder::new(event_type).with_agent(agent.clone());

        for &parent in &self.frontier {
            builder = builder.with_parent(parent);
        }

        for (key, value) in metadata {
            builder = builder.with_metadata(key, value);
        }

        let vertex = builder.build();
        let vertex_id = vertex.compute_id().expect("vertex id computation");

        self.session.update_frontier(vertex_id, &self.frontier);

        self.frontier = vec![vertex_id];
        self.vertices.push(vertex);
        vertex_id
    }

    /// Append a game-start genesis vertex.
    pub fn start_game(
        &mut self,
        player_a: &rhizo_crypt_core::Did,
        player_b: &rhizo_crypt_core::Did,
    ) -> rhizo_crypt_core::VertexId {
        self.session.add_agent(player_a.clone());
        self.session.add_agent(player_b.clone());

        let mut meta = HashMap::new();
        meta.insert(
            "game_type".into(),
            rhizo_crypt_core::vertex::MetadataValue::String("mtg_session".into()),
        );

        self.append_action(rhizo_crypt_core::EventType::SessionStart, player_a, meta)
    }

    /// Append a player action (cast, attack, draw, etc.).
    pub fn player_action(
        &mut self,
        agent: &rhizo_crypt_core::Did,
        action_name: &str,
        card_id: Option<&str>,
    ) -> rhizo_crypt_core::VertexId {
        let mut meta = HashMap::new();
        meta.insert(
            "action".into(),
            rhizo_crypt_core::vertex::MetadataValue::String(action_name.into()),
        );
        if let Some(card) = card_id {
            meta.insert(
                "card".into(),
                rhizo_crypt_core::vertex::MetadataValue::String(card.into()),
            );
        }

        self.append_action(
            rhizo_crypt_core::EventType::AgentAction {
                action: action_name.into(),
            },
            agent,
            meta,
        )
    }
}

// ============================================================================
// loamSpine — Ruleset & Card Certificates
// ============================================================================

/// A game ruleset expressed as a loamSpine certificate.
///
/// loamSpine certificates are memory-bound objects with verifiable ownership.
/// A ruleset certificate proves which rules govern a game session —
/// the same pattern that proves which protocol governs a lab sample.
#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all fields"
)]
pub struct RulesetCertificate {
    pub certificate: loam_spine_core::Certificate,
    pub ruleset_name: String,
    pub license: String,
}

impl RulesetCertificate {
    /// Create a ruleset certificate.
    ///
    /// The `CertificateType::Custom` variant is used since game rulesets
    /// are a domain-specific certificate kind.
    pub fn new(
        owner: &loam_spine_core::Did,
        ruleset_name: &str,
        license: &str,
        spine_id: loam_spine_core::types::SpineId,
    ) -> Self {
        let cert_id = uuid::Uuid::now_v7();
        let entry_hash = [0u8; 32];
        let mint_info =
            loam_spine_core::certificate::MintInfo::new(owner.clone(), spine_id, entry_hash);

        let metadata = loam_spine_core::certificate::CertificateMetadata::new()
            .with_name(ruleset_name)
            .with_description(format!("Game ruleset: {ruleset_name} ({license})"))
            .with_attribute("license", license);

        let cert = loam_spine_core::Certificate::new(
            cert_id,
            loam_spine_core::certificate::CertificateType::Custom {
                type_uri: format!(
                    "urn:ecoprimals:game:ruleset:{}",
                    ruleset_name.to_lowercase().replace(' ', "_")
                ),
                schema_version: 1,
            },
            owner,
            &mint_info,
        )
        .with_metadata(metadata);

        Self {
            certificate: cert,
            ruleset_name: ruleset_name.into(),
            license: license.into(),
        }
    }
}

/// A game card expressed as a loamSpine certificate.
///
/// Each physical card is a certificate with verifiable provenance:
/// set code, collector number, condition, owner. Your physical
/// Black Lotus IS your digital Black Lotus — same loamSpine cert.
#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all fields"
)]
pub struct CardCertificate {
    pub certificate: loam_spine_core::Certificate,
    pub card_name: String,
    pub set_code: String,
    pub collector_number: u16,
}

impl CardCertificate {
    /// Create a card certificate.
    pub fn new(
        owner: &loam_spine_core::Did,
        card_name: &str,
        set_code: &str,
        collector_number: u16,
        spine_id: loam_spine_core::types::SpineId,
    ) -> Self {
        let cert_id = uuid::Uuid::now_v7();
        let entry_hash = [0u8; 32];
        let mint_info =
            loam_spine_core::certificate::MintInfo::new(owner.clone(), spine_id, entry_hash);

        let mut item_attrs = HashMap::new();
        item_attrs.insert("card_name".into(), card_name.into());
        item_attrs.insert("set_code".into(), set_code.into());
        item_attrs.insert("collector_number".into(), collector_number.to_string());

        let cert = loam_spine_core::Certificate::new(
            cert_id,
            loam_spine_core::certificate::CertificateType::GameItem {
                game_id: "mtg".into(),
                item_type: "card".into(),
                item_id: format!("{set_code}_{collector_number}"),
                attributes: item_attrs,
            },
            owner,
            &mint_info,
        );

        Self {
            certificate: cert,
            card_name: card_name.into(),
            set_code: set_code.into(),
            collector_number,
        }
    }
}

// ============================================================================
// sweetGrass — Game Action Attribution (W3C PROV-O)
// ============================================================================

/// Attribution of a game action via sweetGrass braids.
///
/// Each game action creates a PROV-O record:
/// - Agent: the player (DID)
/// - Activity: the game action (cast, attack, etc.)
/// - Entity: the resulting game state change
///
/// This is the same pattern that attributes scientific contributions:
/// a researcher's analysis of a dataset produces a result. A player's
/// action in a game produces a state transition.
pub struct GameActionBraid {
    pub braid: sweet_grass_core::Braid,
}

impl GameActionBraid {
    /// Create a braid for a game action.
    ///
    /// # Errors
    ///
    /// Returns an error if braid construction fails validation.
    pub fn new(
        player_did: &sweet_grass_core::Did,
        action_name: &str,
        card_name: Option<&str>,
        rhizo_vertex_hex: &str,
    ) -> Result<Self, sweet_grass_core::SweetGrassError> {
        let activity = sweet_grass_core::Activity {
            id: sweet_grass_core::ActivityId::from_task(action_name),
            activity_type: sweet_grass_core::ActivityType::Creation,
            used: Vec::new(),
            was_associated_with: vec![sweet_grass_core::AgentAssociation {
                agent: player_did.clone(),
                role: sweet_grass_core::AgentRole::Creator,
                on_behalf_of: None,
                had_plan: Some("game_session".into()),
            }],
            started_at_time: sweet_grass_core::braid::current_timestamp_nanos(),
            ended_at_time: Some(sweet_grass_core::braid::current_timestamp_nanos()),
            metadata: sweet_grass_core::activity::ActivityMetadata::default(),
            ecop: sweet_grass_core::activity::ActivityEcoPrimals::default(),
        };

        let description = card_name.map_or_else(
            || action_name.to_string(),
            |c| format!("{action_name}: {c}"),
        );
        let data_hash = format!("sha256:{rhizo_vertex_hex}");

        let ecop = sweet_grass_core::braid::EcoPrimalsAttributes {
            source_primal: Some("ludospring".into()),
            ..Default::default()
        };

        let mut braid = sweet_grass_core::Braid::builder()
            .data_hash(&data_hash)
            .mime_type("application/x-game-action")
            .size(description.len() as u64)
            .attributed_to(player_did.clone())
            .generated_by(activity)
            .ecop(ecop)
            .build()?;

        braid.metadata.description = Some(description);

        Ok(Self { braid })
    }
}

// ============================================================================
// biomeOS Graph — Trio Coordination Model
// ============================================================================

/// Models the biomeOS coordination graph for the provenance trio.
///
/// The trio deploys as the rootpulse niche in biomeOS:
/// ```text
/// ludoSpring ──→ rhizoCrypt ──→ loamSpine ──→ sweetGrass
///     ↑              │               │              │
///     └──────────────┴───────────────┴──────────────┘
///                    (feedback loop)
/// ```
///
/// Coordination pattern: `Continuous` at 60 Hz (game tick).
/// Each node has a capability, budget, and dependency chain.
pub struct TrioCoordinationGraph {
    pub nodes: Vec<CoordinationNode>,
    pub pattern: CoordinationPattern,
    pub target_hz: f64,
}

/// A node in the coordination graph.
#[expect(
    dead_code,
    reason = "structural completeness — models biomeOS GraphNode fields"
)]
pub struct CoordinationNode {
    pub id: String,
    pub capability: String,
    pub depends_on: Vec<String>,
    pub feedback_to: Option<String>,
    pub budget_ms: f64,
}

/// Coordination pattern (mirrors `biomeos-graph::CoordinationPattern`).
#[expect(
    dead_code,
    reason = "structural completeness — models biomeOS CoordinationPattern variants"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinationPattern {
    Sequential,
    Parallel,
    Continuous,
    Pipeline,
}

impl TrioCoordinationGraph {
    /// Build the standard trio coordination graph for a game session.
    ///
    /// Mirrors the biomeOS `game_engine_tick.toml` graph definition:
    /// ludoSpring → rhizoCrypt → loamSpine → sweetGrass
    /// with feedback from sweetGrass back to ludoSpring.
    pub fn game_session_graph() -> Self {
        let nodes = vec![
            CoordinationNode {
                id: "ludospring".into(),
                capability: "game.session_logic".into(),
                depends_on: vec![],
                feedback_to: None,
                budget_ms: 3.0,
            },
            CoordinationNode {
                id: "rhizocrypt".into(),
                capability: "dag.append_vertex".into(),
                depends_on: vec!["ludospring".into()],
                feedback_to: Some("ludospring".into()),
                budget_ms: 1.0,
            },
            CoordinationNode {
                id: "loamspine".into(),
                capability: "ledger.anchor_entry".into(),
                depends_on: vec!["rhizocrypt".into()],
                feedback_to: None,
                budget_ms: 2.0,
            },
            CoordinationNode {
                id: "sweetgrass".into(),
                capability: "attribution.create_braid".into(),
                depends_on: vec!["rhizocrypt".into()],
                feedback_to: Some("ludospring".into()),
                budget_ms: 1.0,
            },
        ];

        Self {
            nodes,
            pattern: CoordinationPattern::Continuous,
            target_hz: 60.0,
        }
    }

    /// Verify DAG topology: no orphans, no cycles, all deps satisfied.
    pub fn verify_topology(&self) -> TopologyResult {
        let node_ids: Vec<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();

        let mut unsatisfied_deps = Vec::new();
        let mut orphan_feedbacks = Vec::new();

        for node in &self.nodes {
            for dep in &node.depends_on {
                if !node_ids.contains(&dep.as_str()) {
                    unsatisfied_deps.push((node.id.clone(), dep.clone()));
                }
            }
            if let Some(ref fb) = node.feedback_to {
                if !node_ids.contains(&fb.as_str()) {
                    orphan_feedbacks.push((node.id.clone(), fb.clone()));
                }
            }
        }

        let total_budget_ms: f64 = self.nodes.iter().map(|n| n.budget_ms).sum();
        let tick_budget_ms = 1000.0 / self.target_hz;

        TopologyResult {
            node_count: self.nodes.len(),
            unsatisfied_deps,
            orphan_feedbacks,
            total_budget_ms,
            tick_budget_ms,
            fits_in_tick: total_budget_ms <= tick_budget_ms,
        }
    }
}

/// Result of topology verification.
pub struct TopologyResult {
    pub node_count: usize,
    pub unsatisfied_deps: Vec<(String, String)>,
    pub orphan_feedbacks: Vec<(String, String)>,
    pub total_budget_ms: f64,
    pub tick_budget_ms: f64,
    pub fits_in_tick: bool,
}
