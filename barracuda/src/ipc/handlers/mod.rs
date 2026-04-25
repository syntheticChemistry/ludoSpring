// SPDX-License-Identifier: AGPL-3.0-or-later
//! Method dispatch and handler implementations.
//!
//! Each handler deserializes parameters, calls into the library, and
//! returns a serialized result. No handler has side-effects beyond its
//! return value — ludoSpring is a pure-function primal.

mod delegation;
mod gpu;
mod lifecycle;
mod mcp;
mod neural;
mod science;

use tracing::info;

use super::envelope::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use super::{
    METHOD_ACCESSIBILITY, METHOD_ANALYZE_UI, METHOD_BEGIN_SESSION, METHOD_COMPLETE_SESSION,
    METHOD_DIFFICULTY_ADJUSTMENT, METHOD_ENGAGEMENT, METHOD_EVALUATE_FLOW, METHOD_FITTS_COST,
    METHOD_GENERATE_NOISE, METHOD_MINT_CERTIFICATE, METHOD_NARRATE_ACTION, METHOD_NPC_DIALOGUE,
    METHOD_POLL_TELEMETRY, METHOD_PUSH_SCENE, METHOD_QUERY_VERTICES, METHOD_RECORD_ACTION,
    METHOD_STORAGE_GET, METHOD_STORAGE_PUT, METHOD_TOOLS_CALL, METHOD_TOOLS_LIST,
    METHOD_VOICE_CHECK, METHOD_WFC_STEP,
};

pub(super) type HandlerResult = Result<serde_json::Value, JsonRpcError>;

/// Dispatch a JSON-RPC request to the appropriate handler.
///
/// Returns a serialized JSON-RPC response (success or error).
/// Emits structured metrics for Neural API Pathway Learner (passive).
///
/// Follows the two-tier dispatch pattern from `SPRING_COMPOSITION_PATTERNS`
/// §4: lifecycle/infrastructure first, then domain science. Method names
/// are normalized (§1) before matching to handle prefixed calls from
/// biomeOS or peer springs.
#[must_use]
pub fn dispatch(req: &JsonRpcRequest) -> String {
    let start = std::time::Instant::now();
    let method = super::envelope::normalize_method(&req.method);

    let result = dispatch_lifecycle(&method, req)
        .or_else(|| dispatch_infrastructure(&method, req))
        .or_else(|| dispatch_science(&method, req))
        .unwrap_or_else(|| Err(JsonRpcError::method_not_found(&req.id, &req.method)));

    #[cfg(feature = "ipc")]
    {
        let latency_us = start.elapsed().as_micros();
        let success = result.is_ok();
        info!(
            primal = crate::PRIMAL_NAME,
            op = %req.method,
            latency_us = latency_us,
            ok = success,
            "dispatch"
        );
    }
    #[cfg(not(feature = "ipc"))]
    let _ = start;

    match result {
        Ok(value) => serialize_response(&JsonRpcResponse::ok(&req.id, value)),
        Err(err) => serialize_error(&err),
    }
}

/// Tier 1: lifecycle and health probes.
fn dispatch_lifecycle(method: &str, req: &JsonRpcRequest) -> Option<HandlerResult> {
    Some(match method {
        "health.check" | "lifecycle.health" | "health" => lifecycle::handle_health(req),
        "health.liveness" => lifecycle::handle_liveness(req),
        "health.readiness" => lifecycle::handle_readiness(req),
        "lifecycle.status" => lifecycle::handle_lifecycle_status(req),
        "lifecycle.composition" => lifecycle::handle_composition(req),
        "lifecycle.register" => neural::handle_lifecycle_register(req),
        "capability.list" => lifecycle::handle_capability_list(req),
        "capability.deregister" => neural::handle_capability_deregister(req),
        "capability.discover" => neural::handle_capability_discover(req),
        _ => return None,
    })
}

/// Tier 2: infrastructure — MCP, Neural API delegation, capability routing.
fn dispatch_infrastructure(method: &str, req: &JsonRpcRequest) -> Option<HandlerResult> {
    Some(match method {
        "capability.call" => neural::handle_capability_call(req),
        "visualization.render"
        | "visualization.render.stream"
        | "visualization.render.scene"
        | "visualization.render.dashboard"
        | "visualization.export"
        | "visualization.validate"
        | "interaction.subscribe"
        | "interaction.poll" => neural::handle_visualization_delegation(req),
        METHOD_TOOLS_LIST => mcp::handle_tools_list(req),
        METHOD_TOOLS_CALL => mcp::handle_tools_call(req),
        _ => return None,
    })
}

/// Tier 3: domain science, delegation, and GPU dispatch.
fn dispatch_science(method: &str, req: &JsonRpcRequest) -> Option<HandlerResult> {
    Some(match method {
        METHOD_EVALUATE_FLOW => science::handle_evaluate_flow(req),
        METHOD_FITTS_COST => science::handle_fitts_cost(req),
        METHOD_ENGAGEMENT => science::handle_engagement(req),
        METHOD_GENERATE_NOISE => science::handle_generate_noise(req),
        METHOD_ANALYZE_UI => science::handle_analyze_ui(req),
        METHOD_ACCESSIBILITY => science::handle_accessibility(req),
        METHOD_WFC_STEP => science::handle_wfc_step(req),
        METHOD_DIFFICULTY_ADJUSTMENT => science::handle_difficulty_adjustment(req),
        METHOD_BEGIN_SESSION => delegation::handle_begin_session(req),
        METHOD_RECORD_ACTION => delegation::handle_record_action(req),
        METHOD_COMPLETE_SESSION => delegation::handle_complete_session(req),
        METHOD_POLL_TELEMETRY => delegation::handle_poll_telemetry(req),
        METHOD_NPC_DIALOGUE => delegation::handle_npc_dialogue(req),
        METHOD_NARRATE_ACTION => delegation::handle_narrate_action(req),
        METHOD_VOICE_CHECK => delegation::handle_voice_check(req),
        METHOD_PUSH_SCENE => delegation::handle_push_scene(req),
        METHOD_QUERY_VERTICES => delegation::handle_query_vertices(req),
        METHOD_MINT_CERTIFICATE => delegation::handle_mint_certificate(req),
        METHOD_STORAGE_PUT => delegation::handle_storage_put(req),
        METHOD_STORAGE_GET => delegation::handle_storage_get(req),
        "game.gpu.fog_of_war" => gpu::handle_gpu_fog_of_war(req),
        "game.gpu.tile_lighting" => gpu::handle_gpu_tile_lighting(req),
        "game.gpu.pathfind" => gpu::handle_gpu_pathfind(req),
        "game.gpu.perlin_terrain" => gpu::handle_gpu_perlin_terrain(req),
        "game.gpu.batch_raycast" => gpu::handle_gpu_batch_raycast(req),
        _ => return None,
    })
}

fn serialize_response(resp: &JsonRpcResponse) -> String {
    serde_json::to_string(&resp).unwrap_or_else(|e| {
        format!(
            r#"{{"jsonrpc":"2.0","error":{{"code":-32603,"message":"serialize: {e}"}},"id":null}}"#
        )
    })
}

fn serialize_error(err: &JsonRpcError) -> String {
    serde_json::to_string(&err).unwrap_or_else(|e| {
        format!(
            r#"{{"jsonrpc":"2.0","error":{{"code":-32603,"message":"serialize: {e}"}},"id":null}}"#
        )
    })
}

pub(super) fn parse_params<T: serde::de::DeserializeOwned>(
    req: &JsonRpcRequest,
) -> Result<T, JsonRpcError> {
    let params = req
        .params
        .as_ref()
        .ok_or_else(|| JsonRpcError::invalid_params(&req.id, "missing params"))?;
    serde_json::from_value(params.clone())
        .map_err(|e| JsonRpcError::invalid_params(&req.id, &e.to_string()))
}

pub(super) fn to_json(id: &serde_json::Value, val: impl serde::Serialize) -> HandlerResult {
    serde_json::to_value(val).map_err(|e| JsonRpcError::internal(id, &e.to_string()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::ipc::envelope::JsonRpcRequest;
    use crate::ipc::{
        METHOD_BEGIN_SESSION, METHOD_COMPLETE_SESSION, METHOD_MINT_CERTIFICATE,
        METHOD_NARRATE_ACTION, METHOD_NPC_DIALOGUE, METHOD_PUSH_SCENE, METHOD_QUERY_VERTICES,
        METHOD_RECORD_ACTION, METHOD_STORAGE_GET, METHOD_STORAGE_PUT, METHOD_TOOLS_CALL,
        METHOD_TOOLS_LIST, METHOD_VOICE_CHECK,
    };

    fn make_request(method: &str, params: serde_json::Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params: Some(params),
            id: serde_json::json!(1),
        }
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
        ] {
            assert!(
                names.contains(&expected),
                "missing tool {expected} in {names:?}"
            );
        }
        assert_eq!(tools.len(), 13);
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

    fn result_json(resp: &str) -> serde_json::Value {
        let v: serde_json::Value = serde_json::from_str(resp).expect("response json");
        v["result"].clone()
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
        // Without a live petalTongue, pushed is false and error is reported
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
}
