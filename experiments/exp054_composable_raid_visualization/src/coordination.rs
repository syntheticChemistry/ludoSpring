// SPDX-License-Identifier: AGPL-3.0-or-later
//! biomeOS + songbird coordination for multi-player raids.
//!
//! Models the deployment graph, songbird registration/discovery, and
//! biomeOS lifecycle management for a 2-player extraction raid.
//! All types are protocol-level — no chimeric deps on infrastructure primals.

#![forbid(unsafe_code)]

use std::fmt::Write;

use crate::protocol::{
    CapabilityRegisterRequest, DeploymentGraphDef, GraphNode, JsonRpcRequest,
    LifecycleRegisterRequest, SongbirdDiscoverResponse, SongbirdProvider, SongbirdRegisterRequest,
    TickConfig,
};

// ============================================================================
// Deployment graph — Continuous coordination for extraction raid
// ============================================================================

/// Build the biomeOS deployment graph for a 2-player extraction raid.
///
/// Topology:
/// ```text
/// [input_p1] ──┐
///              ├──► [raid_server] ──► [fraud_detect] ──► [viz_push]
/// [input_p2] ──┘         │
///                        └──feedback──► [raid_server]
/// ```
///
/// Coordination: `Continuous` at 20 Hz (raid server tick rate).
#[must_use]
pub fn build_raid_deployment_graph() -> DeploymentGraphDef {
    DeploymentGraphDef {
        id: "extraction_raid_2p".into(),
        name: "2-Player Extraction Raid".into(),
        version: "1.0.0".into(),
        coordination: "continuous".into(),
        tick: Some(TickConfig {
            target_hz: 20.0,
            max_accumulator_ms: 100.0,
            budget_warning_ms: Some(10.0),
        }),
        nodes: vec![
            GraphNode {
                id: "input_p1".into(),
                name: "Player 1 Input".into(),
                depends_on: vec![],
                capability: Some("game.player_input".into()),
                feedback_to: None,
                budget_ms: Some(2.0),
            },
            GraphNode {
                id: "input_p2".into(),
                name: "Player 2 Input".into(),
                depends_on: vec![],
                capability: Some("game.player_input".into()),
                feedback_to: None,
                budget_ms: Some(2.0),
            },
            GraphNode {
                id: "raid_server".into(),
                name: "Raid Server (Authority)".into(),
                depends_on: vec!["input_p1".into(), "input_p2".into()],
                capability: Some("game.raid_authority".into()),
                feedback_to: Some("raid_server".into()),
                budget_ms: Some(15.0),
            },
            GraphNode {
                id: "fraud_detect".into(),
                name: "Fraud Detection".into(),
                depends_on: vec!["raid_server".into()],
                capability: Some("game.fraud_analysis".into()),
                feedback_to: None,
                budget_ms: Some(5.0),
            },
            GraphNode {
                id: "viz_push".into(),
                name: "Visualization Push (petalTongue)".into(),
                depends_on: vec!["fraud_detect".into()],
                capability: Some("visualization.render".into()),
                feedback_to: None,
                budget_ms: Some(5.0),
            },
        ],
    }
}

/// Validate graph topology: acyclic deps, all refs resolve, feedback edges valid.
#[must_use]
pub fn validate_graph_topology(graph: &DeploymentGraphDef) -> Vec<String> {
    let mut errors = Vec::new();
    let node_ids: Vec<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();

    for node in &graph.nodes {
        for dep in &node.depends_on {
            if !node_ids.contains(&dep.as_str()) {
                errors.push(format!(
                    "node '{}' depends on '{}' which doesn't exist",
                    node.id, dep
                ));
            }
        }
        if let Some(fb) = &node.feedback_to {
            if !node_ids.contains(&fb.as_str()) {
                errors.push(format!(
                    "node '{}' feedback_to '{}' which doesn't exist",
                    node.id, fb
                ));
            }
        }
    }

    // Check no cycles (excluding feedback edges)
    let mut visited = vec![false; graph.nodes.len()];
    let mut stack = vec![false; graph.nodes.len()];
    for (i, _) in graph.nodes.iter().enumerate() {
        if !visited[i] && has_cycle(i, &graph.nodes, &node_ids, &mut visited, &mut stack) {
            errors.push("dependency cycle detected (excluding feedback)".into());
            break;
        }
    }

    errors
}

fn has_cycle(
    idx: usize,
    nodes: &[GraphNode],
    ids: &[&str],
    visited: &mut [bool],
    stack: &mut [bool],
) -> bool {
    visited[idx] = true;
    stack[idx] = true;

    for dep in &nodes[idx].depends_on {
        if let Some(dep_idx) = ids.iter().position(|&id| id == dep.as_str()) {
            if !visited[dep_idx] {
                if has_cycle(dep_idx, nodes, ids, visited, stack) {
                    return true;
                }
            } else if stack[dep_idx] {
                return true;
            }
        }
    }
    stack[idx] = false;
    false
}

/// Compute topological execution order (dependency-first).
#[must_use]
pub fn execution_order(graph: &DeploymentGraphDef) -> Vec<String> {
    let node_ids: Vec<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
    let mut in_degree: Vec<usize> = graph.nodes.iter().map(|n| n.depends_on.len()).collect();

    let mut queue: Vec<usize> = in_degree
        .iter()
        .enumerate()
        .filter(|(_, &d)| d == 0)
        .map(|(i, _)| i)
        .collect();

    let mut order = Vec::new();
    while let Some(idx) = queue.pop() {
        order.push(graph.nodes[idx].id.clone());
        for (i, node) in graph.nodes.iter().enumerate() {
            if node.depends_on.iter().any(|d| d.as_str() == node_ids[idx]) {
                in_degree[i] -= 1;
                if in_degree[i] == 0 {
                    queue.push(i);
                }
            }
        }
    }
    order
}

// ============================================================================
// songbird registration — player agent discovery
// ============================================================================

/// Build songbird registration for a player agent.
#[must_use]
pub fn player_register(player_id: &str, socket_path: &str) -> SongbirdRegisterRequest {
    SongbirdRegisterRequest {
        primal_id: format!("ludospring-{player_id}"),
        capabilities: vec![
            "game.player_input".into(),
            "game.inventory".into(),
            "game.movement".into(),
        ],
        endpoint: socket_path.into(),
    }
}

/// Build songbird registration for the raid server.
#[must_use]
pub fn raid_server_register(socket_path: &str) -> SongbirdRegisterRequest {
    SongbirdRegisterRequest {
        primal_id: format!("ludospring-raid-server-{}", super::family_id()),
        capabilities: vec![
            "game.raid_authority".into(),
            "game.fraud_analysis".into(),
            "game.tick_logic".into(),
        ],
        endpoint: socket_path.into(),
    }
}

/// Build songbird registration for a visualization primal.
///
/// The primal ID is provided by the caller — ludoSpring discovers
/// visualization primals by capability, never by hardcoded name.
#[must_use]
pub fn viz_register(primal_id: &str, socket_path: &str) -> SongbirdRegisterRequest {
    SongbirdRegisterRequest {
        primal_id: primal_id.into(),
        capabilities: vec![
            "visualization.render".into(),
            "visualization.render.stream".into(),
            "visualization.render.dashboard".into(),
        ],
        endpoint: socket_path.into(),
    }
}

/// Simulate songbird discovery response for `game.player_input`.
///
/// Socket paths are derived from the platform's temp directory to avoid
/// hardcoded `/tmp` references.  In production, songbird resolves these
/// at runtime via XDG-compliant discovery.
#[must_use]
pub fn discover_players() -> SongbirdDiscoverResponse {
    let base = std::env::temp_dir().join("biomeos");
    let fid = super::family_id();
    let p1_id = format!("ludospring-player1-{fid}");
    let p2_id = format!("ludospring-player2-{fid}");
    SongbirdDiscoverResponse {
        providers: vec![
            SongbirdProvider {
                primal_id: p1_id.clone(),
                virtual_endpoint: format!("/primal/{p1_id}"),
                native_endpoint: base
                    .join(format!("ludospring-player1-{fid}.sock"))
                    .to_string_lossy()
                    .into_owned(),
                capabilities: vec!["game.player_input".into()],
            },
            SongbirdProvider {
                primal_id: p2_id.clone(),
                virtual_endpoint: format!("/primal/{p2_id}"),
                native_endpoint: base
                    .join(format!("ludospring-player2-{fid}.sock"))
                    .to_string_lossy()
                    .into_owned(),
                capabilities: vec!["game.player_input".into()],
            },
        ],
    }
}

// ============================================================================
// biomeOS lifecycle messages
// ============================================================================

/// Build lifecycle.register JSON-RPC for a player agent.
#[must_use]
pub fn lifecycle_register(name: &str, socket: &str, pid: u32) -> JsonRpcRequest {
    let params = LifecycleRegisterRequest {
        name: name.into(),
        socket_path: socket.into(),
        pid,
        deployment_node: Some("extraction_raid_2p".into()),
    };
    JsonRpcRequest::new(
        "lifecycle.register",
        serde_json::to_value(&params).unwrap_or_default(),
        1,
    )
}

/// Build capability.register JSON-RPC for a capability.
#[must_use]
pub fn capability_register(capability: &str, primal: &str, socket: &str) -> JsonRpcRequest {
    let params = CapabilityRegisterRequest {
        capability: capability.into(),
        primal: primal.into(),
        socket: socket.into(),
    };
    JsonRpcRequest::new(
        "capability.register",
        serde_json::to_value(&params).unwrap_or_default(),
        2,
    )
}

/// Serialize a deployment graph to TOML for biomeOS consumption.
#[must_use]
pub fn graph_to_toml(graph: &DeploymentGraphDef) -> String {
    let mut toml = String::new();
    toml.push_str("[graph]\n");
    let _ = writeln!(toml, "id = \"{}\"", graph.id);
    let _ = writeln!(toml, "name = \"{}\"", graph.name);
    let _ = writeln!(toml, "version = \"{}\"", graph.version);
    let _ = writeln!(toml, "coordination = \"{}\"", graph.coordination);

    if let Some(tick) = &graph.tick {
        toml.push_str("\n[graph.tick]\n");
        let _ = writeln!(toml, "target_hz = {}", tick.target_hz);
        let _ = writeln!(toml, "max_accumulator_ms = {}", tick.max_accumulator_ms);
        if let Some(bw) = tick.budget_warning_ms {
            let _ = writeln!(toml, "budget_warning_ms = {bw}");
        }
    }

    for node in &graph.nodes {
        toml.push_str("\n[[graph.nodes]]\n");
        let _ = writeln!(toml, "id = \"{}\"", node.id);
        let _ = writeln!(toml, "name = \"{}\"", node.name);
        if !node.depends_on.is_empty() {
            let deps: Vec<String> = node.depends_on.iter().map(|d| format!("\"{d}\"")).collect();
            let _ = writeln!(toml, "depends_on = [{}]", deps.join(", "));
        }
        if let Some(cap) = &node.capability {
            let _ = writeln!(toml, "capability = \"{cap}\"");
        }
        if let Some(fb) = &node.feedback_to {
            let _ = writeln!(toml, "feedback_to = \"{fb}\"");
        }
        if let Some(budget) = node.budget_ms {
            let _ = writeln!(toml, "budget_ms = {budget}");
        }
    }

    toml
}
