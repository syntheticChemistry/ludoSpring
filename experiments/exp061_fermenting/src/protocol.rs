// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC wire types for composable fermenting deployment.
//!
//! These mirror the wire format that would be used to communicate with
//! the provenance trio (loamSpine, rhizoCrypt, sweetGrass) when deployed
//! as separate binaries via the `provenance_node_atomic.toml` graph.
//!
//! The fermenting system becomes a client that sends JSON-RPC calls over
//! Unix sockets to each primal, discovered via songbird.
//!
//! Architecture:
//!
//! ```text
//!  ┌──────────────────┐
//!  │  fermenting app  │
//!  │   (exp061)       │
//!  └──┬───┬───┬───────┘
//!     │   │   │
//!     │   │   │  certificate.*         ┌────────────┐
//!     │   │   └───────────────────────►│  loamSpine │
//!     │   │                            │  (port 8301)│
//!     │   │  dag.*                     └────────────┘
//!     │   └───────────────────────────►┌────────────┐
//!     │                                │ rhizoCrypt │
//!     │                                │ (port 9400)│
//!     │  provenance.*                  └────────────┘
//!     └───────────────────────────────►┌────────────┐
//!                                      │ sweetGrass │
//!                                      │ (port 8302)│
//!                                      └────────────┘
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// JSON-RPC 2.0 envelope
// ============================================================================

/// JSON-RPC 2.0 request.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
}

impl JsonRpcRequest {
    #[must_use]
    pub fn new(method: &str, params: serde_json::Value, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params,
            id,
        }
    }
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

/// JSON-RPC 2.0 error.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

// ============================================================================
// loamSpine — certificate wire types
// ============================================================================

/// `certificate.mint` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct CertMintRequest {
    pub cert_type: String,
    pub owner_did: String,
    pub metadata: CertMetadataWire,
    pub item_attributes: std::collections::HashMap<String, String>,
}

/// Certificate metadata wire format.
#[derive(Debug, Serialize, Deserialize)]
pub struct CertMetadataWire {
    pub name: String,
    pub description: String,
}

/// `certificate.transfer` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct CertTransferRequest {
    pub cert_id: String,
    pub from_did: String,
    pub to_did: String,
}

/// `certificate.loan` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct CertLoanRequest {
    pub cert_id: String,
    pub lender_did: String,
    pub borrower_did: String,
    pub duration_secs: Option<u64>,
    pub auto_return: bool,
}

/// `certificate.return_loan` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct CertReturnRequest {
    pub cert_id: String,
    pub borrower_did: String,
}

/// `certificate.trade_offer` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct CertTradeOfferRequest {
    pub offered_cert: String,
    pub requested_cert: Option<String>,
    pub from_did: String,
    pub to_did: String,
}

/// `certificate.trade_accept` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct CertTradeAcceptRequest {
    pub offer_id: String,
    pub accepted_by_did: String,
}

// ============================================================================
// rhizoCrypt — DAG wire types
// ============================================================================

/// `dag.create_session` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct DagCreateSessionRequest {
    pub session_type: String,
    pub game_id: String,
    pub name: String,
}

/// `dag.append_vertex` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct DagAppendVertexRequest {
    pub session_id: String,
    pub event_type: String,
    pub agent_did: String,
    pub metadata: std::collections::HashMap<String, String>,
    pub parents: Vec<String>,
}

// ============================================================================
// sweetGrass — attribution wire types
// ============================================================================

/// `provenance.create_braid` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBraidRequest {
    pub data_hash: String,
    pub mime_type: String,
    pub attributed_to: String,
    pub activity_type: String,
    pub description: String,
    pub derived_from: Vec<String>,
}

/// `provenance.object_event` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectEventRequest {
    pub object_id: String,
    pub event_type: String,
    pub description: String,
    pub actor_did: String,
    pub metadata: std::collections::HashMap<String, String>,
}

/// `provenance.get_timeline` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTimelineRequest {
    pub object_id: String,
}

// ============================================================================
// Composable fermenting flow — builds the full sequence of IPC calls
// ============================================================================

/// Build the full sequence of JSON-RPC calls for a fermenting mint operation.
///
/// In a composable deployment, minting a fermenting object requires three
/// IPC calls: one to each primal in the trio.
pub fn mint_ipc_sequence(
    owner_did: &str,
    item_name: &str,
    item_type: &str,
    rarity: &str,
) -> Vec<JsonRpcRequest> {
    let mut calls = Vec::new();

    let mut attrs = std::collections::HashMap::new();
    attrs.insert("rarity".into(), rarity.into());
    attrs.insert("item_type".into(), item_type.into());

    calls.push(JsonRpcRequest::new(
        "certificate.mint",
        serde_json::to_value(CertMintRequest {
            cert_type: "game_item".into(),
            owner_did: owner_did.into(),
            metadata: CertMetadataWire {
                name: item_name.into(),
                description: format!("Fermenting object: {item_name}"),
            },
            item_attributes: attrs,
        })
        .expect("serialization"),
        1,
    ));

    let mut dag_meta = std::collections::HashMap::new();
    dag_meta.insert("event".into(), "mint".into());
    dag_meta.insert("item_name".into(), item_name.into());

    calls.push(JsonRpcRequest::new(
        "dag.append_vertex",
        serde_json::to_value(DagAppendVertexRequest {
            session_id: "active_session".into(),
            event_type: "ferment_mint".into(),
            agent_did: owner_did.into(),
            metadata: dag_meta,
            parents: vec![],
        })
        .expect("serialization"),
        2,
    ));

    calls.push(JsonRpcRequest::new(
        "provenance.object_event",
        serde_json::to_value(ObjectEventRequest {
            object_id: "pending_cert_id".into(),
            event_type: "mint".into(),
            description: format!("Minted: {item_name}"),
            actor_did: owner_did.into(),
            metadata: std::collections::HashMap::new(),
        })
        .expect("serialization"),
        3,
    ));

    calls
}

/// Build the JSON-RPC call for a trade operation.
pub fn trade_ipc_sequence(cert_id: &str, from_did: &str, to_did: &str) -> Vec<JsonRpcRequest> {
    let mut calls = Vec::new();

    calls.push(JsonRpcRequest::new(
        "certificate.transfer",
        serde_json::to_value(CertTransferRequest {
            cert_id: cert_id.into(),
            from_did: from_did.into(),
            to_did: to_did.into(),
        })
        .expect("serialization"),
        1,
    ));

    let mut dag_meta = std::collections::HashMap::new();
    dag_meta.insert("event".into(), "trade".into());
    dag_meta.insert("from".into(), from_did.into());
    dag_meta.insert("to".into(), to_did.into());

    calls.push(JsonRpcRequest::new(
        "dag.append_vertex",
        serde_json::to_value(DagAppendVertexRequest {
            session_id: "active_session".into(),
            event_type: "ferment_trade".into(),
            agent_did: from_did.into(),
            metadata: dag_meta,
            parents: vec![],
        })
        .expect("serialization"),
        2,
    ));

    calls.push(JsonRpcRequest::new(
        "provenance.object_event",
        serde_json::to_value(ObjectEventRequest {
            object_id: cert_id.into(),
            event_type: "trade".into(),
            description: format!("Traded to {to_did}"),
            actor_did: from_did.into(),
            metadata: std::collections::HashMap::new(),
        })
        .expect("serialization"),
        3,
    ));

    calls
}

/// Build the JSON-RPC calls for the full deployment graph health check.
pub fn deployment_health_sequence() -> Vec<JsonRpcRequest> {
    vec![
        JsonRpcRequest::new(
            "health.check",
            serde_json::json!({"primal": "loamspine"}),
            1,
        ),
        JsonRpcRequest::new("health", serde_json::json!({"primal": "rhizocrypt"}), 2),
        JsonRpcRequest::new(
            "health.check",
            serde_json::json!({"primal": "sweetgrass"}),
            3,
        ),
    ]
}
