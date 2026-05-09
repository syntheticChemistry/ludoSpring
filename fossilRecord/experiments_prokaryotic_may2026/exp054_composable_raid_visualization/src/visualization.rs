// SPDX-License-Identifier: AGPL-3.0-or-later
//! Visualization payloads for petalTongue.
//!
//! Builds [`DataBinding`] JSON payloads that petalTongue renders via
//! `visualization.render` / `visualization.render.dashboard`.
//! These are protocol-level constructs — no petalTongue dependency needed.

#![forbid(unsafe_code)]

use crate::protocol::{
    DashboardRenderRequest, DataBinding, JsonRpcRequest, StreamOperation, StreamUpdateRequest,
    VisualizationRenderRequest,
};
use crate::simulation::RaidSnapshot;

/// Zone adjacency heatmap — shows which zones connect to which.
#[must_use]
pub fn zone_topology_heatmap(zones: &[&str], adjacency: &[(usize, usize)]) -> DataBinding {
    let n = zones.len();
    let mut values = vec![0.0; n * n];
    for &(a, b) in adjacency {
        values[a * n + b] = 1.0;
        values[b * n + a] = 1.0;
    }
    // Diagonal = self
    for i in 0..n {
        values[i * n + i] = 0.5;
    }
    DataBinding::Heatmap {
        id: "zone_topology".into(),
        label: "Zone Adjacency Matrix".into(),
        x_labels: zones.iter().map(|z| (*z).into()).collect(),
        y_labels: zones.iter().map(|z| (*z).into()).collect(),
        values,
        unit: "connected".into(),
    }
}

/// Player health gauges — one per player.
#[must_use]
pub fn player_health_gauge(player: &str, health: f64) -> DataBinding {
    DataBinding::Gauge {
        id: format!("health_{player}"),
        label: format!("{player} Health"),
        value: health,
        min: 0.0,
        max: 100.0,
        unit: "HP".into(),
        normal_range: [50.0, 100.0],
        warning_range: [15.0, 50.0],
    }
}

/// Action timeline — actions per tick for each player.
#[must_use]
pub fn action_timeline(player: &str, ticks: &[f64], counts: &[f64]) -> DataBinding {
    DataBinding::TimeSeries {
        id: format!("actions_{player}"),
        label: format!("{player} Actions/Tick"),
        x_label: "Tick (ms)".into(),
        y_label: "Actions".into(),
        unit: "count".into(),
        x_values: ticks.to_vec(),
        y_values: counts.to_vec(),
    }
}

/// Fraud detection bar chart — violations by category.
#[must_use]
pub fn fraud_report_bar(categories: &[&str], counts: &[f64]) -> DataBinding {
    DataBinding::Bar {
        id: "fraud_report".into(),
        label: "Fraud Detections by Type".into(),
        categories: categories.iter().map(|c| (*c).into()).collect(),
        values: counts.to_vec(),
        unit: "violations".into(),
    }
}

/// Inventory bar chart — item counts by category.
#[must_use]
pub fn inventory_bar(player: &str, categories: &[&str], counts: &[f64]) -> DataBinding {
    DataBinding::Bar {
        id: format!("inventory_{player}"),
        label: format!("{player} Inventory"),
        categories: categories.iter().map(|c| (*c).into()).collect(),
        values: counts.to_vec(),
        unit: "items".into(),
    }
}

// ============================================================================
// Full dashboard from a raid snapshot
// ============================================================================

/// Build a complete petalTongue dashboard from a 2-player raid snapshot.
#[must_use]
pub fn build_raid_dashboard(snapshot: &RaidSnapshot) -> DashboardRenderRequest {
    let mut bindings = Vec::new();

    // Zone topology
    bindings.push(zone_topology_heatmap(
        &snapshot.zone_names,
        &snapshot.zone_adjacency,
    ));

    // Player health gauges
    for (name, health) in &snapshot.player_health {
        bindings.push(player_health_gauge(name, *health));
    }

    // Action timelines
    for (name, ticks, counts) in &snapshot.action_timelines {
        bindings.push(action_timeline(name, ticks, counts));
    }

    // Fraud report
    bindings.push(fraud_report_bar(
        &snapshot.fraud_categories,
        &snapshot.fraud_counts,
    ));

    // Inventory
    for (name, cats, counts) in &snapshot.inventories {
        bindings.push(inventory_bar(name, cats, counts));
    }

    DashboardRenderRequest {
        session_id: format!("raid-{}", snapshot.raid_id),
        title: format!("Raid: {} — Live Dashboard", snapshot.map_name),
        bindings,
        domain: Some("ecology".into()),
        max_columns: Some(3),
    }
}

// ============================================================================
// JSON-RPC message builders
// ============================================================================

/// Build a `visualization.render` JSON-RPC request from a dashboard.
#[must_use]
pub fn render_request(dashboard: &DashboardRenderRequest, id: u64) -> JsonRpcRequest {
    let render_req = VisualizationRenderRequest {
        session_id: dashboard.session_id.clone(),
        title: dashboard.title.clone(),
        bindings: dashboard.bindings.clone(),
        domain: dashboard.domain.clone(),
    };
    JsonRpcRequest::new(
        "visualization.render",
        serde_json::to_value(&render_req).unwrap_or_default(),
        id,
    )
}

/// Build a streaming gauge update for a player's health.
#[must_use]
pub fn health_stream_update(session_id: &str, player: &str, health: f64) -> StreamUpdateRequest {
    StreamUpdateRequest {
        session_id: session_id.into(),
        binding_id: format!("health_{player}"),
        operation: StreamOperation::SetValue { value: health },
    }
}

/// Build a streaming timeline append for new actions.
#[must_use]
pub fn action_stream_append(
    session_id: &str,
    player: &str,
    new_ticks: &[f64],
    new_counts: &[f64],
) -> StreamUpdateRequest {
    StreamUpdateRequest {
        session_id: session_id.into(),
        binding_id: format!("actions_{player}"),
        operation: StreamOperation::Append {
            x_values: new_ticks.to_vec(),
            y_values: new_counts.to_vec(),
        },
    }
}
