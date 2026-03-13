// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC wire types for inter-primal IPC.
//!
//! These are local definitions matching the wire format of petalTongue,
//! songbird, and biomeOS. We do NOT import those primals as dependencies —
//! they are composable binaries we communicate with at runtime over Unix
//! sockets using JSON-RPC 2.0.
//!
//! # Architecture
//!
//! ```text
//!                   ┌─────────────┐
//!                   │   biomeOS   │
//!                   │   NUCLEUS   │
//!                   └──┬──────┬───┘
//!         lifecycle.   │      │   graph.execute
//!         register     │      │
//!     ┌────────────────┘      └────────────────┐
//!     │                                        │
//!  ┌──▼───────┐   ipc.discover    ┌───────────▼──┐
//!  │ player_1 │◄─────────────────►│   songbird   │
//!  │ (ludo)   │   ipc.register    │  (discovery)  │
//!  └──┬───────┘                   └───────────┬──┘
//!     │                                       │
//!     │  visualization.render                 │
//!     │                            ┌──────────┘
//!  ┌──▼──────────┐                │
//!  │ petalTongue │◄───────────────┘
//!  │   (UI)      │  visualization.render
//!  └─────────────┘
//! ```

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ============================================================================
// JSON-RPC 2.0 envelope
// ============================================================================

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

#[derive(Debug, Serialize, Deserialize)]
#[expect(dead_code, reason = "wire format completeness")]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

// ============================================================================
// petalTongue — visualization wire types
// ============================================================================

/// Matches `petal-tongue-core::DataBinding` serde format.
/// Tagged with `channel_type` (lowercase variant names).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "channel_type")]
pub enum DataBinding {
    #[serde(rename = "timeseries")]
    TimeSeries {
        id: String,
        label: String,
        x_label: String,
        y_label: String,
        unit: String,
        x_values: Vec<f64>,
        y_values: Vec<f64>,
    },
    #[serde(rename = "heatmap")]
    Heatmap {
        id: String,
        label: String,
        x_labels: Vec<String>,
        y_labels: Vec<String>,
        values: Vec<f64>,
        unit: String,
    },
    #[serde(rename = "gauge")]
    Gauge {
        id: String,
        label: String,
        value: f64,
        min: f64,
        max: f64,
        unit: String,
        normal_range: [f64; 2],
        warning_range: [f64; 2],
    },
    #[serde(rename = "bar")]
    Bar {
        id: String,
        label: String,
        categories: Vec<String>,
        values: Vec<f64>,
        unit: String,
    },
}

/// `visualization.render` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct VisualizationRenderRequest {
    pub session_id: String,
    pub title: String,
    pub bindings: Vec<DataBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

/// `visualization.render.stream` operation.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamOperation {
    #[serde(rename = "append")]
    Append {
        x_values: Vec<f64>,
        y_values: Vec<f64>,
    },
    #[serde(rename = "set_value")]
    SetValue { value: f64 },
    #[serde(rename = "replace")]
    Replace { binding: DataBinding },
}

/// `visualization.render.stream` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct StreamUpdateRequest {
    pub session_id: String,
    pub binding_id: String,
    pub operation: StreamOperation,
}

/// `visualization.render.dashboard` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardRenderRequest {
    pub session_id: String,
    pub title: String,
    pub bindings: Vec<DataBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_columns: Option<usize>,
}

// ============================================================================
// songbird — discovery wire types
// ============================================================================

/// `ipc.register` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct SongbirdRegisterRequest {
    pub primal_id: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
}

/// `ipc.register` response.
#[derive(Debug, Serialize, Deserialize)]
#[expect(dead_code, reason = "wire format completeness")]
pub struct SongbirdRegisterResponse {
    pub virtual_endpoint: String,
    pub registered_at: String,
}

/// `ipc.discover` request params.
#[derive(Debug, Serialize, Deserialize)]
#[expect(dead_code, reason = "wire format completeness")]
pub struct SongbirdDiscoverRequest {
    pub capability: String,
}

/// Provider entry from discovery.
#[derive(Debug, Serialize, Deserialize)]
pub struct SongbirdProvider {
    pub primal_id: String,
    pub virtual_endpoint: String,
    pub native_endpoint: String,
    pub capabilities: Vec<String>,
}

/// `ipc.discover` response.
#[derive(Debug, Serialize, Deserialize)]
pub struct SongbirdDiscoverResponse {
    pub providers: Vec<SongbirdProvider>,
}

// ============================================================================
// biomeOS — lifecycle + graph wire types
// ============================================================================

/// `lifecycle.register` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct LifecycleRegisterRequest {
    pub name: String,
    pub socket_path: String,
    pub pid: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_node: Option<String>,
}

/// `capability.register` request params.
#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityRegisterRequest {
    pub capability: String,
    pub primal: String,
    pub socket: String,
}

/// `DeploymentGraph` node (matches biomeos-graph TOML wire format).
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_ms: Option<f64>,
}

/// Continuous tick configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct TickConfig {
    pub target_hz: f64,
    pub max_accumulator_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_warning_ms: Option<f64>,
}

/// Full deployment graph definition.
#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentGraphDef {
    pub id: String,
    pub name: String,
    pub version: String,
    pub coordination: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick: Option<TickConfig>,
    pub nodes: Vec<GraphNode>,
}
