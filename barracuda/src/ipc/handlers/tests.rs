// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration tests for JSON-RPC handler dispatch.
//!
//! Extracted from `mod.rs` to keep the dispatch module under 200 lines.

use super::*;
use crate::ipc::envelope::JsonRpcRequest;
use crate::ipc::{
    METHOD_BEGIN_SESSION, METHOD_COMPLETE_SESSION, METHOD_MINT_CERTIFICATE, METHOD_NARRATE_ACTION,
    METHOD_NPC_DIALOGUE, METHOD_PUSH_SCENE, METHOD_QUERY_VERTICES, METHOD_RECORD_ACTION,
    METHOD_STORAGE_GET, METHOD_STORAGE_PUT, METHOD_TOOLS_CALL, METHOD_TOOLS_LIST,
    METHOD_VOICE_CHECK,
};

fn make_request(method: &str, params: serde_json::Value) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: method.into(),
        params: Some(params),
        id: serde_json::json!(1),
    }
}

fn result_json(resp: &str) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_str(resp).expect("response json");
    v["result"].clone()
}

#[test]
fn evaluate_flow_returns_state() {
    let req = make_request(
        "game.evaluate_flow",
        serde_json::json!({"challenge": 0.5, "skill": 0.5}),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("flow"), "expected flow state in: {resp}");
}

#[test]
fn fitts_cost_returns_time() {
    let req = make_request(
        "game.fitts_cost",
        serde_json::json!({"distance": 100.0, "target_width": 20.0}),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("movement_time_ms"));
}

#[test]
fn fitts_cost_hick_reaction_time() {
    let req = make_request(
        "game.fitts_cost",
        serde_json::json!({"method": "hick_reaction_time", "n": 7}),
    );
    let resp = dispatch(&req);
    assert!(
        resp.contains("reaction_time_ms"),
        "expected Hick result: {resp}"
    );
}

#[test]
fn fitts_cost_steering_time() {
    let req = make_request(
        "game.fitts_cost",
        serde_json::json!({"method": "steering_time", "distance": 100.0, "width": 20.0}),
    );
    let resp = dispatch(&req);
    assert!(
        resp.contains("steering_time_ms"),
        "expected steering result: {resp}"
    );
}

#[test]
fn engagement_returns_composite() {
    let req = make_request(
        "game.engagement",
        serde_json::json!({
            "session_duration_s": 300.0,
            "action_count": 100,
            "exploration_breadth": 5,
            "challenge_seeking": 3,
            "retry_count": 10,
            "deliberate_pauses": 5
        }),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("composite"));
}

#[test]
fn noise_returns_value() {
    let req = make_request(
        "game.generate_noise",
        serde_json::json!({"x": 1.0, "y": 2.0}),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("value"));
}

#[test]
fn unknown_method_returns_error() {
    let req = make_request("game.nonexistent", serde_json::json!({}));
    let resp = dispatch(&req);
    assert!(resp.contains("-32601"));
}

#[test]
fn dispatch_normalizes_prefixed_method() {
    let req = make_request(
        "ludospring.game.evaluate_flow",
        serde_json::json!({"challenge": 0.5, "skill": 0.5}),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("flow"), "normalized dispatch: {resp}");
}

#[test]
fn dispatch_normalizes_double_prefixed_method() {
    let req = make_request(
        "biomeos.ludospring.game.fitts_cost",
        serde_json::json!({"distance": 100.0, "target_width": 20.0}),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("movement_time_ms"), "double-prefix: {resp}");
}

#[test]
fn lifecycle_register_returns_status() {
    let req = make_request(
        "lifecycle.register",
        serde_json::json!({"name": "x", "domain": "game"}),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("registered"));
}

#[test]
fn capability_call_routes_to_evaluate_flow() {
    let req = make_request(
        "capability.call",
        serde_json::json!({
            "capability": "game",
            "operation": "evaluate_flow",
            "args": {"challenge": 0.5, "skill": 0.5}
        }),
    );
    let resp = dispatch(&req);
    let result = result_json(&resp);
    assert_eq!(result["state"], "flow");
}

#[test]
fn visualization_render_delegation_degraded_without_peer() {
    let req = make_request(
        "visualization.render",
        serde_json::json!({"session_id": "s", "title": "t", "data": {}}),
    );
    let resp = dispatch(&req);
    let result = result_json(&resp);
    assert_eq!(result["degraded"], true);
    assert_eq!(result["delegated"], false);
}

#[test]
fn capability_call_propagates_inner_method_not_found() {
    let req = make_request(
        "capability.call",
        serde_json::json!({
            "capability": "game",
            "operation": "nonexistent_flow_method",
            "args": {}
        }),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("-32601"), "nested error: {resp}");
}

#[test]
fn capability_call_requires_capability_field() {
    let req = make_request(
        "capability.call",
        serde_json::json!({
            "operation": "evaluate_flow",
            "args": {"challenge": 0.5, "skill": 0.5}
        }),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("-32602"), "missing capability: {resp}");
}

#[test]
fn tools_list_returns_tools() {
    let req = make_request(METHOD_TOOLS_LIST, serde_json::json!({}));
    let resp = dispatch(&req);
    let result = result_json(&resp);
    let tools = result.as_array().expect("tools list array");
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    for expected in [
        "game.evaluate_flow",
        "game.fitts_cost",
        "game.engagement",
        "game.generate_noise",
        "game.analyze_ui",
        "game.accessibility",
        "game.wfc_step",
        "game.difficulty_adjustment",
        "game.begin_session",
        "game.complete_session",
        "game.npc_dialogue",
        "game.narrate_action",
        "game.push_scene",
        "game.record_action",
        "game.voice_check",
    ] {
        assert!(
            names.contains(&expected),
            "missing tool {expected} in {names:?}"
        );
    }
    assert_eq!(tools.len(), 15);
    let first = &tools[0];
    assert!(first.get("description").is_some());
    assert!(first.get("input_schema").is_some());
}

#[test]
fn tools_call_dispatches_to_evaluate_flow() {
    let req = make_request(
        METHOD_TOOLS_CALL,
        serde_json::json!({
            "name": "game.evaluate_flow",
            "arguments": {"challenge": 0.5, "skill": 0.5}
        }),
    );
    let resp = dispatch(&req);
    let result = result_json(&resp);
    assert_eq!(result["state"], "flow");
}

#[test]
fn tools_call_unknown_tool_returns_error() {
    let req = make_request(
        METHOD_TOOLS_CALL,
        serde_json::json!({
            "name": "game.unknown_mcp_tool",
            "arguments": {}
        }),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("-32601"), "expected method not found: {resp}");
}

#[test]
fn invalid_params_returns_error() {
    let req = make_request("game.evaluate_flow", serde_json::json!({"wrong": true}));
    let resp = dispatch(&req);
    assert!(resp.contains("-32602"));
}

#[test]
fn accessibility_returns_score() {
    let req = make_request(
        "game.accessibility",
        serde_json::json!({
            "audio_cues": true,
            "descriptions": true,
            "braille": false,
            "haptic": false,
            "color_independent": true,
            "scalable_text": true
        }),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("score"));
}

#[test]
fn analyze_ui_returns_report() {
    let req = make_request(
        "game.analyze_ui",
        serde_json::json!({
            "elements": [{
                "name": "health",
                "bounds": [0.0, 0.9, 0.1, 0.1],
                "data_values": 1,
                "pixel_area": 100.0,
                "data_ink_area": 80.0,
                "critical": true
            }]
        }),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("data_ink_ratio"));
}

#[test]
fn difficulty_adjustment_returns_recommendation() {
    let req = make_request(
        "game.difficulty_adjustment",
        serde_json::json!({
            "outcomes": [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
        }),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("adjustment"));
    assert!(resp.contains("estimated_skill"));
    assert!(resp.contains("trend"));
    assert!(resp.contains("reason"));
}

#[test]
fn wfc_step_returns_state() {
    let req = make_request(
        "game.wfc_step",
        serde_json::json!({
            "width": 4,
            "height": 4,
            "n_tiles": 3,
            "collapse": [1, 1, 0]
        }),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("fully_collapsed"));
}

#[test]
fn lifecycle_status_returns_name_and_capabilities() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: "lifecycle.status".into(),
        params: None,
        id: serde_json::json!(1),
    };
    let resp = dispatch(&req);
    assert!(
        resp.contains("ludospring"),
        "expected primal name in: {resp}"
    );
    assert!(
        resp.contains("game.evaluate_flow"),
        "expected capabilities in: {resp}"
    );
    assert!(
        resp.contains("\"domain\":\"game\""),
        "expected domain in: {resp}"
    );
}

#[test]
fn health_includes_name_field() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: "health.check".into(),
        params: None,
        id: serde_json::json!(1),
    };
    let resp = dispatch(&req);
    assert!(
        resp.contains("\"name\":\"ludospring\""),
        "expected name field in: {resp}"
    );
    assert!(
        resp.contains("game.begin_session"),
        "expected provenance caps in: {resp}"
    );
}

#[test]
fn capability_list_includes_domain_and_dependencies() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: "capability.list".into(),
        params: None,
        id: serde_json::json!(1),
    };
    let resp = dispatch(&req);
    assert!(
        resp.contains("\"domain\":\"game\""),
        "expected domain in: {resp}"
    );
    assert!(
        resp.contains("operation_dependencies"),
        "expected deps in: {resp}"
    );
    assert!(resp.contains("cost_estimates"), "expected costs in: {resp}");
}

#[test]
fn poll_telemetry_includes_domain() {
    let req = make_request("game.poll_telemetry", serde_json::json!({}));
    let resp = dispatch(&req);
    assert!(
        resp.contains("\"domain\":\"game\""),
        "expected domain in: {resp}"
    );
    assert!(
        resp.contains("frame_budget_ms"),
        "expected budget in: {resp}"
    );
}

#[test]
fn health_liveness_returns_alive_per_semantic_standard() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: "health.liveness".into(),
        params: None,
        id: serde_json::json!(1),
    };
    let resp = dispatch(&req);
    let result = result_json(&resp);
    assert_eq!(result["status"], "alive");
}

#[test]
fn health_readiness_returns_subsystems() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: "health.readiness".into(),
        params: None,
        id: serde_json::json!(1),
    };
    let resp = dispatch(&req);
    let result = result_json(&resp);
    assert_eq!(result["ready"], true);
    assert_eq!(result["subsystems"]["science_dispatch"], true);
    assert_eq!(
        result["subsystems"]["provenance_trio"],
        crate::ipc::provenance::has_active_session()
    );
    assert_eq!(result["subsystems"]["gpu_compute"], cfg!(feature = "gpu"));
}

#[test]
fn begin_session_returns_session_payload_without_trio() {
    let req = make_request(
        METHOD_BEGIN_SESSION,
        serde_json::json!({ "session_name": "unit-test-session" }),
    );
    let result = result_json(&dispatch(&req));
    assert!(result["session_id"].as_str().is_some());
    assert_eq!(result["provenance"], "unavailable");
}

#[test]
fn record_action_returns_vertex_payload_without_trio() {
    let req = make_request(
        METHOD_RECORD_ACTION,
        serde_json::json!({
            "session_id": "sess-local",
            "action": { "type": "attack", "target": "goblin" }
        }),
    );
    let result = result_json(&dispatch(&req));
    assert!(result["vertex_id"].as_str().is_some());
    assert_eq!(result["provenance"], "unavailable");
}

#[test]
fn complete_session_returns_stage_without_trio() {
    let req = make_request(
        METHOD_COMPLETE_SESSION,
        serde_json::json!({ "session_id": "sess-local" }),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["stage"], "unavailable");
    assert_eq!(result["session_id"], "sess-local");
}

#[test]
fn npc_dialogue_delegates_to_squirrel() {
    let req = make_request(
        METHOD_NPC_DIALOGUE,
        serde_json::json!({
            "npc_name": "Merchant",
            "personality_prompt": "You sell potions.",
            "player_input": "How much?"
        }),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["available"], false);
    assert_eq!(result["text"], "");
}

#[test]
fn narrate_action_delegates_to_squirrel() {
    let req = make_request(
        METHOD_NARRATE_ACTION,
        serde_json::json!({
            "action": "casts fireball",
            "context": "boss room"
        }),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["available"], false);
    assert_eq!(result["text"], "");
}

#[test]
fn voice_check_delegates_to_squirrel() {
    let req = make_request(
        METHOD_VOICE_CHECK,
        serde_json::json!({
            "voice_name": "Empathy",
            "voice_personality": "Warm and cautious.",
            "game_state": "ally low HP"
        }),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["available"], false);
    assert_eq!(result["voice"], "Empathy");
}

#[test]
fn push_scene_reports_honest_status() {
    let req = make_request(
        METHOD_PUSH_SCENE,
        serde_json::json!({
            "session_id": "pt-sess",
            "channel": "DialogueTree",
            "scene": { "root": "n1" }
        }),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["session_id"], "pt-sess");
    assert_eq!(result["channel"], "DialogueTree");
    assert_eq!(result["pushed"], false);
    assert!(result["error"].is_string());
}

#[test]
fn query_vertices_delegates_to_provenance() {
    let req = make_request(
        METHOD_QUERY_VERTICES,
        serde_json::json!({
            "session_id": "q-sess",
            "event_type": "move",
            "agent": "player1",
            "limit": 5
        }),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["available"], false);
}

#[test]
fn mint_certificate_delegates_to_provenance() {
    let req = make_request(
        METHOD_MINT_CERTIFICATE,
        serde_json::json!({
            "cert_type": "NpcPersonality",
            "owner": "npc-1",
            "payload": { "traits": ["kind"] }
        }),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["available"], false);
    assert!(result["cert_id"].as_str().is_some());
}

#[test]
fn storage_put_delegates_to_nestgate() {
    let req = make_request(
        METHOD_STORAGE_PUT,
        serde_json::json!({
            "key": "k1",
            "data": { "v": 1 }
        }),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["available"], false);
}

#[test]
fn storage_get_delegates_to_nestgate() {
    let req = make_request(METHOD_STORAGE_GET, serde_json::json!({ "key": "k1" }));
    let result = result_json(&dispatch(&req));
    assert_eq!(result["available"], false);
}

#[test]
fn lifecycle_composition_returns_report() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: "lifecycle.composition".into(),
        params: None,
        id: serde_json::json!(1),
    };
    let resp = dispatch(&req);
    let result = result_json(&resp);
    assert_eq!(result["spring"], "ludospring");
    assert_eq!(result["composition_model"], "pure");
    assert!(result["dependencies"].is_array());
    assert!(result["live_count"].is_number());
    assert!(result["complete"].is_boolean());
}

#[test]
fn gpu_batch_raycast_degrades_without_toadstool() {
    let req = make_request(
        "game.gpu.batch_raycast",
        serde_json::json!({
            "grid_w": 8,
            "grid_h": 8,
            "origins_x": [1.5],
            "origins_y": [1.5],
            "angles": [0.0]
        }),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["available"], false);
    assert_eq!(result["fallback"], "cpu");
}

#[test]
fn gpu_batch_raycast_rejects_mismatched_arrays() {
    let req = make_request(
        "game.gpu.batch_raycast",
        serde_json::json!({
            "grid_w": 4,
            "grid_h": 4,
            "origins_x": [1.0, 2.0],
            "origins_y": [1.0],
            "angles": [0.0]
        }),
    );
    let resp = dispatch(&req);
    assert!(resp.contains("-32602"), "expected invalid params: {resp}");
}

#[test]
fn gpu_fog_of_war_degrades_without_toadstool() {
    let req = make_request(
        "game.gpu.fog_of_war",
        serde_json::json!({
            "grid_w": 4,
            "grid_h": 4,
            "viewer_x": 2.0,
            "viewer_y": 2.0,
            "sight_radius": 3
        }),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["available"], false);
    assert_eq!(result["fallback"], "cpu");
    assert_eq!(
        result["reason"],
        "compute dispatch unavailable — CPU fallback active"
    );
}

#[test]
fn game_tick_returns_composite_state() {
    let req = make_request(
        "game.tick",
        serde_json::json!({
            "session_id": "tick-1",
            "channel": "combat",
            "scene": {"type": "grid", "entities": []},
            "action": {"type": "move", "x": 5}
        }),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["session_id"], "tick-1");
    assert!(result.get("scene_pushed").is_some());
    assert!(result.get("scene_degraded").is_some());
    assert!(result.get("interaction_events").is_some());
    assert!(result.get("engagement").is_some());
    assert!(result.get("frame_budget_ms").is_some());
}

#[test]
fn game_tick_minimal_without_action() {
    let req = make_request(
        "game.tick",
        serde_json::json!({
            "session_id": "tick-2",
            "scene": {"type": "dialogue"}
        }),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["session_id"], "tick-2");
    assert_eq!(result["action_recorded"], false);
}

#[test]
fn subscribe_interaction_degrades_without_peer() {
    let req = make_request(
        "game.subscribe_interaction",
        serde_json::json!({"session_id": "s1"}),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["session_id"], "s1");
    assert!(result.get("subscribed").is_some());
    assert!(result.get("degraded").is_some());
}

#[test]
fn poll_interaction_degrades_without_peer() {
    let req = make_request(
        "game.poll_interaction",
        serde_json::json!({"session_id": "s1"}),
    );
    let result = result_json(&dispatch(&req));
    assert_eq!(result["session_id"], "s1");
    assert!(result.get("events").is_some());
    assert!(result.get("degraded").is_some());
}

#[test]
fn push_scene_reports_degraded_field() {
    let req = make_request(
        "game.push_scene",
        serde_json::json!({
            "session_id": "s1",
            "channel": "test",
            "scene": {"type": "grid"}
        }),
    );
    let result = result_json(&dispatch(&req));
    assert!(result.get("degraded").is_some());
}
