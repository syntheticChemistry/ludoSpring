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
use super::methods;
use super::{
    METHOD_ACCESSIBILITY, METHOD_ANALYZE_UI, METHOD_BEGIN_SESSION, METHOD_COMPLETE_SESSION,
    METHOD_DIFFICULTY_ADJUSTMENT, METHOD_ENGAGEMENT, METHOD_EVALUATE_FLOW, METHOD_FITTS_COST,
    METHOD_GAME_TICK, METHOD_GENERATE_NOISE, METHOD_MINT_CERTIFICATE, METHOD_NARRATE_ACTION,
    METHOD_NPC_DIALOGUE, METHOD_POLL_INTERACTION, METHOD_POLL_TELEMETRY, METHOD_PUSH_SCENE,
    METHOD_QUERY_VERTICES, METHOD_RECORD_ACTION, METHOD_STORAGE_GET, METHOD_STORAGE_PUT,
    METHOD_SUBSCRIBE_INTERACTION, METHOD_TOOLS_CALL, METHOD_TOOLS_LIST, METHOD_VOICE_CHECK,
    METHOD_WFC_STEP,
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
        methods::health::CHECK | methods::lifecycle::HEALTH | "health" => {
            lifecycle::handle_health(req)
        }
        methods::health::LIVENESS => lifecycle::handle_liveness(req),
        "health.readiness" => lifecycle::handle_readiness(req),
        methods::lifecycle::STATUS => lifecycle::handle_lifecycle_status(req),
        methods::lifecycle::COMPOSITION => lifecycle::handle_composition(req),
        methods::lifecycle::REGISTER => neural::handle_lifecycle_register(req),
        methods::capability::LIST => lifecycle::handle_capability_list(req),
        methods::capability::DEREGISTER => neural::handle_capability_deregister(req),
        methods::capability::DISCOVER => neural::handle_capability_discover(req),
        _ => return None,
    })
}

/// Tier 2: infrastructure — MCP, Neural API delegation, capability routing.
fn dispatch_infrastructure(method: &str, req: &JsonRpcRequest) -> Option<HandlerResult> {
    Some(match method {
        methods::capability::CALL => neural::handle_capability_call(req),
        methods::visualization::RENDER
        | methods::visualization::RENDER_STREAM
        | methods::visualization::RENDER_SCENE
        | methods::visualization::RENDER_DASHBOARD
        | methods::visualization::EXPORT
        | methods::visualization::VALIDATE
        | methods::interaction::SUBSCRIBE
        | methods::interaction::POLL => neural::handle_visualization_delegation(req),
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
        METHOD_GAME_TICK => delegation::handle_game_tick(req),
        METHOD_SUBSCRIBE_INTERACTION => delegation::handle_subscribe_interaction(req),
        METHOD_POLL_INTERACTION => delegation::handle_poll_interaction(req),
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
#[path = "tests.rs"]
mod tests;
